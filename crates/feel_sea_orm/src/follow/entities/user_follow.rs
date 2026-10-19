use chrono::{DateTime, Utc};
use feel_entity::follow::UserFollow;
use sea_orm::entity::prelude::*;

/// ORM entity for the `user_follow` table
/// (follower `user_id` -> followee `follow_user_id`).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_follow")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// Follower (references users.id)
    #[sea_orm(index)]
    pub user_id: i64,
    /// Followee (references users.id)
    #[sea_orm(index)]
    pub follow_user_id: i64,
    /// Whether the relation is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for UserFollow {
    fn from(value: Model) -> Self {
        UserFollow {
            id: value.id,
            user_id: value.user_id,
            follow_user_id: value.follow_user_id,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
