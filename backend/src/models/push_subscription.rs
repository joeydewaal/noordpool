use jiff::Timestamp;
use serde::Serialize;
use toasty::BelongsTo;
use uuid::Uuid;

use crate::models::User;

/// A browser Web Push endpoint registered by an authenticated user.
/// One user may have many (phone, desktop) with different prefs.
#[derive(Debug, toasty::Model, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PushSubscription {
    #[key]
    #[auto]
    pub id: Uuid,

    #[index]
    #[serde(skip)]
    pub user_id: Uuid,

    #[belongs_to(key = user_id, references = id)]
    #[serde(skip_serializing_if = "BelongsTo::is_unloaded")]
    pub user: BelongsTo<User>,

    #[unique]
    pub endpoint: String,

    pub p256dh: String,
    pub auth: String,

    #[default(true)]
    pub notify_goal: bool,

    #[default(Timestamp::now())]
    pub created_at: Timestamp,
}
