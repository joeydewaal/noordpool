pub mod event_type;
pub mod game;
pub mod game_event;
pub mod game_status;
pub mod home_away;
pub mod position;
pub mod user;
pub mod user_role;

pub use event_type::EventType;
pub use game::Game;
pub use game_event::GameEvent;
pub use game_status::GameStatus;
pub use home_away::HomeAway;
pub use position::Position;
use toasty::{Db, create, db::Builder};
pub use user::User;
pub use user_role::{Role, UserRole};

use crate::auth::password;

pub fn build_db() -> Builder {
    let mut builder = Db::builder();
    builder
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

pub async fn init_db(db: &mut Db) {
    let password = password::hash_password("Admin123").await.unwrap();

    let _ = create!(User {
        name: "admin",
        email: "admin@noordpool.be",
        password_hash: password,
        roles: [{ role: Role::Admin }]
    })
    .exec(db)
    .await;
}
