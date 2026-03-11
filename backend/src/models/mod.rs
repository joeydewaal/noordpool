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
use toasty::{Db, db::Builder};
pub use user::User;
pub use user_role::{Role, UserRole};

pub fn build_db() -> Builder {
    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    builder.register::<Role>();
    builder.register::<Position>();
    builder.register::<Game>();
    builder.register::<GameStatus>();
    builder.register::<HomeAway>();
    builder.register::<GameEvent>();
    builder.register::<EventType>();
    builder
}
