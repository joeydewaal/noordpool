pub mod event_type;
pub mod game;
pub mod game_event;
pub mod game_status;
pub mod home_away;
pub mod player;
pub mod position;
pub mod team;
pub mod user;
pub mod user_role;

use std::collections::HashMap;
use std::error::Error;

pub use event_type::EventType;
pub use game::Game;
pub use game_event::GameEvent;
pub use game_status::GameStatus;
pub use home_away::HomeAway;
pub use player::Player;
pub use position::Position;
use toasty::{Db, Executor, create};
pub use user::User;
pub use user_role::{Role, UserRole};

use crate::{
    auth::password,
    config::Config,
    import,
    models::{self, team::Team},
};

pub async fn build_db(config: &Config) -> Result<Db, Box<dyn Error>> {
    let mut db = Db::builder()
        .register::<Team>()
        .register::<User>()
        .register::<UserRole>()
        .register::<Role>()
        .register::<Position>()
        .register::<Player>()
        .register::<Game>()
        .register::<GameStatus>()
        .register::<HomeAway>()
        .register::<GameEvent>()
        .register::<EventType>()
        .connect(&config.database_url)
        .await?;

    if !cfg!(feature = "prod") {
        let _ = db.push_schema().await;
        models::init_db(&mut db).await?;
    }

    Ok(db)
}

pub async fn init_db(db: &mut Db) -> Result<(), Box<dyn Error>> {
    let mut tx = db.transaction().await?;

    let password = password::hash_password("Admin123")
        .await
        .expect("Couldn't hash password");

    let res = create!(User {
        first_name: "Admin",
        last_name: "",
        email: "admin@noordpool.be",
        password_hash: password,
        roles: [{ role: Role::Admin }]
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

    let mut col_to_game: HashMap<usize, Game> = HashMap::new();
    for m in matches {
        let game = create!(Game {
            opponent: m.opponent,
            location: "",
            date_time: jiff::Timestamp::UNIX_EPOCH,
            home_away: if m.is_home {
                HomeAway::Home
            } else {
                HomeAway::Away
            },
            status: GameStatus::Completed,
            home_score: m.home_score,
            away_score: m.away_score,
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
