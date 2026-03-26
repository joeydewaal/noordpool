pub mod event_type;
pub mod game;
pub mod game_event;
pub mod game_status;
pub mod home_away;
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
pub use position::Position;
use toasty::{Db, Executor, create, db::Builder, stmt::CreateMany};
pub use user::User;
pub use user_role::{Role, UserRole};

use crate::{auth::password, import::parse_voetbal_csv, models::team::Team};

pub fn build_db() -> Builder {
    let mut builder = Db::builder();
    builder
        .register::<Team>()
        .register::<User>()
        .register::<UserRole>()
        .register::<Role>()
        .register::<Position>()
        .register::<Game>()
        .register::<GameStatus>()
        .register::<HomeAway>()
        .register::<GameEvent>()
        .register::<EventType>();

    builder
}

pub async fn init_db(db: &mut Db) -> Result<(), Box<dyn Error>> {
    let password = password::hash_password("Admin123")
        .await
        .expect("Couldn't hash password");

    let res = create!(User {
        name: "admin",
        email: "admin@noordpool.be",
        password_hash: password,
        roles: [{ role: Role::Admin }]
    })
    .exec(db)
    .await;

    if res.is_err() {
        return Ok(());
    }

    let mut tx = db.transaction().await?;

    let (spelers, teams, matches) = parse_voetbal_csv("../data/voetbal.csv").unwrap();

    dbg!(&spelers, &teams, &matches);

    let noordpool = create!(Team {
        name: "De Noordpool"
    })
    .exec(&mut tx)
    .await?;

    // Create players one-by-one to capture shirt_number → User mapping
    let mut shirt_to_user: HashMap<i32, User> = HashMap::new();
    for p in &spelers {
        let user = create!(User {
                name: format!("{} {}", p.first_name, p.last_name),
                shirt_number: p.shirt_number,
                roles: [{ role: Role::Player }],
                team: noordpool.clone()
        })
        .exec(&mut tx)
        .await?;
        shirt_to_user.insert(p.shirt_number, user);
    }

    let create_teams: CreateMany<Team> = teams
        .into_iter()
        .map(|t| create!(Team { name: t.name }))
        .collect();
    create_teams.exec(&mut tx).await?;

    // Create games one-by-one to capture col_index → Game mapping
    let mut col_to_game: HashMap<usize, Game> = HashMap::new();
    for m in &matches {
        let game = create!(Game {
            opponent: m.opponent.clone(),
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

    // Create GameEvents for each goal scored
    for p in &spelers {
        if p.goals_per_match.is_empty() {
            continue;
        }
        let user = match shirt_to_user.get(&p.shirt_number) {
            Some(u) => u,
            None => continue,
        };
        for (col_index, goal_count) in &p.goals_per_match {
            let game = match col_to_game.get(col_index) {
                Some(g) => g,
                None => continue,
            };
            for _ in 0..*goal_count {
                create!(GameEvent {
                    game: game.clone(),
                    user: user.clone(),
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
