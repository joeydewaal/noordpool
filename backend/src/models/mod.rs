pub mod event_type;
pub mod formation;
pub mod game;
pub mod game_event;
pub mod game_lineup;
pub mod game_lineup_slot;
pub mod player;
pub mod position;
pub mod push_subscription;
pub mod team;
pub mod user;
pub mod user_role;

use std::collections::HashMap;
use std::error::Error;

pub use event_type::EventType;
pub use formation::Formation;
pub use game::Game;
pub use game_event::GameEvent;
pub use game_lineup::GameLineup;
pub use game_lineup_slot::GameLineupSlot;
use jiff::Timestamp;
pub use player::Player;
pub use position::Position;
pub use push_subscription::PushSubscription;
pub use team::Team;
use toasty::{Db, create, db::Builder};
pub use user::User;
pub use user_role::Role;

use crate::{auth::password, config::Config, import};

pub fn build_db() -> Builder {
    let mut builder = Db::builder();
    builder.models(toasty::models!(
        Team,
        User,
        Role,
        Position,
        Player,
        Game,
        GameEvent,
        EventType,
        PushSubscription,
        GameLineup,
        GameLineupSlot
    ));
    builder
}

pub async fn create_db(config: &Config) -> Result<Db, Box<dyn Error>> {
    let mut db = build_db().connect(&config.database_url).await?;

    let _ = db.push_schema().await;

    // Seeding (admin user + demo data) is opt-in via env var so production
    // never auto-creates credentials. Local dev picks it up from the default
    // `.env`.
    if !cfg!(feature = "prod") {
        init_db(&mut db, config).await?;
    }

    Ok(db)
}

pub async fn init_db(db: &mut Db, config: &Config) -> Result<(), Box<dyn Error>> {
    let mut tx = db.transaction().await?;

    let password = password::hash_password(&config.admin_password)
        .await
        .expect("Couldn't hash password");

    let res = create!(User {
        first_name: "Admin",
        last_name: "",
        email: "admin@noordpool.be",
        password_hash: password,
        is_admin: true,
    })
    .exec(&mut tx)
    .await;

    if res.is_err() {
        return Ok(());
    }

    let spelers = import::PLAYERS;
    let teams = import::TEAMS;
    let matches = import::MATCHES;

    let noordpool = create!(Team {
        name: "De Noordpool"
    })
    .exec(&mut tx)
    .await?;

    let mut shirt_to_player: HashMap<i32, Player> = HashMap::new();

    let gastspeler = create!(Player {
        first_name: "Gastspeler",
        last_name: "",
        shirt_number: 20,
        position: Position::CentralMidfielder,
        active: false,
        team: noordpool.clone()
    })
    .exec(&mut tx)
    .await?;
    shirt_to_player.insert(20, gastspeler);

    for p in spelers {
        if !p.active {
            continue;
        }
        let player = create!(Player {
            first_name: p.first_name,
            last_name: p.last_name,
            shirt_number: p.shirt_number,
            position: p.position,
            team: noordpool.clone()
        })
        .exec(&mut tx)
        .await?;
        shirt_to_player.insert(p.shirt_number, player);
    }

    let mut create_teams = Team::create_many();
    for t in teams {
        create_teams = create_teams.item(create!(Team { name: t.name }));
    }
    create_teams.exec(&mut tx).await?;

    // Build name→Team lookup so we can resolve FKs for games.
    let all_teams = Team::all().exec(&mut tx).await?;
    let team_by_name: HashMap<&str, &Team> =
        all_teams.iter().map(|t| (t.name.as_str(), t)).collect();

    let mut col_to_game: HashMap<usize, Game> = HashMap::new();
    for m in matches {
        let opponent_team = team_by_name
            .get(m.opponent)
            .unwrap_or_else(|| panic!("seed opponent '{}' not found in teams", m.opponent));
        let (home_team_id, away_team_id) = if m.is_home {
            (noordpool.id, opponent_team.id)
        } else {
            (opponent_team.id, noordpool.id)
        };
        // Store only the opponent's goals as adjustments. Noordpool player
        // goals are tracked via events and computed by compute_scores.
        let game = create!(Game {
            home_team_id: home_team_id,
            away_team_id: away_team_id,
            location: "",
            date_time: m
                .date_time
                .parse::<Timestamp>()
                .ok()
                .unwrap_or(Timestamp::UNIX_EPOCH),
            home_score: if m.is_home { 0 } else { m.home_score },
            away_score: if m.is_home { m.away_score } else { 0 },
        })
        .exec(&mut tx)
        .await?;
        col_to_game.insert(m.col_index, game);
    }

    for p in spelers {
        if p.goals_per_match.is_empty() {
            continue;
        }
        let player = match shirt_to_player.get(&p.shirt_number) {
            Some(p) => p,
            None => continue,
        };
        for (col_index, goal_count) in p.goals_per_match {
            let game = match col_to_game.get(col_index) {
                Some(g) => g,
                None => continue,
            };
            for _ in 0..*goal_count {
                create!(GameEvent {
                    game: game.clone(),
                    player: player.clone(),
                    team_id: noordpool.id,
                    event_type: EventType::Goal,
                    minute: 0,
                })
                .exec(&mut tx)
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(())
}
