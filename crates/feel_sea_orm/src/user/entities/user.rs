use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// Unique user identifier
    #[sea_orm(unique, index)]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
