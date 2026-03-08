use super::UserRole;

#[derive(Debug, toasty::Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: uuid::Uuid,
    #[unique]
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub created_at: jiff::Timestamp,
    #[has_many]
    pub roles: toasty::HasMany<UserRole>,
}
