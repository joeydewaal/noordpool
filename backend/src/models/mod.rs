pub mod event_type;
pub mod game;
pub mod home_away;
pub mod match_event;
pub mod match_status;
pub mod player;
pub mod position;
pub mod user;
pub mod user_role;

pub use event_type::EventType;
pub use game::Game;
pub use home_away::HomeAway;
pub use match_event::MatchEvent;
pub use match_status::MatchStatus;
pub use player::Player;
pub use position::Position;
use toasty::{Db, db::Builder};
pub use user::User;
pub use user_role::{Role, UserRole};

pub fn build_db() -> Builder {
    let mut builder = Db::builder();
    builder.register::<User>();
    builder.register::<UserRole>();
    builder.register::<Role>();
    builder.register::<Player>();
    builder.register::<Position>();
    builder.register::<Game>();
    builder.register::<MatchStatus>();
    builder.register::<HomeAway>();
    builder.register::<MatchEvent>();
    builder.register::<EventType>();
    builder
}
