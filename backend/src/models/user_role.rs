use super::User;

#[derive(Debug, toasty::Model)]
pub struct UserRole {
    #[key]
    #[auto]
    pub id: uuid::Uuid,
    #[index]
    pub user_id: uuid::Uuid,
    #[belongs_to(key = user_id, references = id)]
    pub user: toasty::BelongsTo<User>,
    pub role: String,
}
