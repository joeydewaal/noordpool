use jiff::Timestamp;
use toasty::HasMany;
use uuid::Uuid;

use super::UserRole;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: Uuid,
    #[unique]
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub avatar_url: Option<String>,
    #[default(Timestamp::now())]
    pub created_at: Timestamp,
    #[has_many]
    pub roles: HasMany<UserRole>,
}
