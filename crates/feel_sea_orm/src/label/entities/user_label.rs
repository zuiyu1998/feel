use chrono::{DateTime, Utc};
use feel_entity::label::UserLabel;
use sea_orm::entity::prelude::*;

/// ORM entity for the `user_label` table
/// (many-to-many association between users and labels).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_label")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// Owning user (references users.id)
    #[sea_orm(index)]
    pub user_id: i64,
    /// Associated label (references label.id)
    #[sea_orm(index)]
    pub label_id: i64,
    /// Whether the association is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for UserLabel {
    fn from(value: Model) -> Self {
        UserLabel {
            id: value.id,
            user_id: value.user_id,
            label_id: value.label_id,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
