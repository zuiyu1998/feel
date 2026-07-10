use chrono::{DateTime, Utc};
use feel_entity::user::UserBase;
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
    pub is_enable: bool,
    /// Soft delete flag
    pub is_delete: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for UserBase {
    fn from(value: Model) -> Self {
        UserBase {
            id: value.id,
            uid: value.uid,
            name: value.name,
            avatar: value.avatar,
            slogan: value.slogan,
            enabled: value.is_enable,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
