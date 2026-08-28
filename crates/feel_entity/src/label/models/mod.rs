use feel_core::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Label/tag entity for categorizing or marking resources.
/// The label base is a public resource shared by multiple users.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelBase {
    /// Database primary key
    pub id: i64,
    /// Label name (globally unique identifier)
    pub name: String,
    /// Public description of the label
    pub description: String,
    /// Label remark shared by all users
    pub remark: String,
    /// Label influence, established at creation time and never changed afterwards
    pub influence: i64,
    /// Whether the label is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// User-label association entity.
/// Links a user and a label in a many-to-many relationship (user_id -> UserBase.id, label_id -> LabelBase.id).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLabel {
    /// Database primary key
    pub id: i64,
    /// Owning user (references UserBase.id)
    pub user_id: i64,
    /// Associated label (references LabelBase.id)
    pub label_id: i64,
    /// Whether the association is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// Label creation request (used by `LabelRepo::create_label`).
/// `id`/`enabled`/timestamps are managed by the storage layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelCreate {
    /// Label name (globally unique identifier)
    pub name: String,
    /// Public description of the label
    pub description: String,
    /// Label remark shared by all users
    pub remark: String,
    /// Label influence, established at creation time
    pub influence: i64,
}

/// Label update request (used by `LabelRepo::update_label`).
/// Carries the primary key plus the mutable fields to write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelUpdate {
    /// Database primary key of the label to update
    pub id: i64,
    /// Label name (globally unique identifier)
    pub name: String,
    /// Public description of the label
    pub description: String,
    /// Label remark shared by all users
    pub remark: String,
    /// Label influence
    pub influence: i64,
    /// Whether the label is enabled
    pub enabled: bool,
}

/// User-label association creation request (used by `LabelRepo::create_user_label`).
/// `id`/`enabled`/timestamps are managed by the storage layer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLabelCreate {
    /// Owning user (references UserBase.id)
    pub user_id: i64,
    /// Associated label (references LabelBase.id)
    pub label_id: i64,
}
