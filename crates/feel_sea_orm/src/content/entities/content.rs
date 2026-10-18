use chrono::{DateTime, Utc};
use feel_entity::content::ContentBase;
use sea_orm::entity::prelude::*;

/// ORM entity for the `content` table (content base, shared fields).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// Author (references users.id)
    #[sea_orm(index)]
    pub user_id: i64,
    /// Content type ("article" | "video" | future)
    #[sea_orm(index)]
    pub content_type: String,
    /// Title
    pub title: String,
    /// Whether the content is visible/enabled
    pub enabled: bool,
    /// Soft delete flag
    pub is_delete: bool,
    /// View counter (popularity for feed ranking)
    #[sea_orm(index)]
    pub view_count: i64,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ContentBase {
    fn from(value: Model) -> Self {
        ContentBase {
            id: value.id,
            user_id: value.user_id,
            content_type: value.content_type,
            title: value.title,
            enabled: value.enabled,
            is_delete: value.is_delete,
            view_count: value.view_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
