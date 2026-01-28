use feel_core::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserBase {
    /// Database primary key
    pub id: i64,
    /// Unique user identifier
    pub uid: String,
    /// User display name
    pub name: String,
    /// User avatar URL
    pub avatar: String,
    /// User slogan or bio
    pub slogan: String,
    /// Whether the user is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

pub struct UserRegister {}

pub struct UserLogin {}

pub struct UserUpdate {}
