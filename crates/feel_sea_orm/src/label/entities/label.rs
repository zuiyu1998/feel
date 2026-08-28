use chrono::{DateTime, Utc};
use feel_entity::label::LabelBase;
use sea_orm::entity::prelude::*;

/// ORM entity for the `label` table (label base, a public resource shared by multiple users).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "label")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// Label name (globally unique identifier)
    #[sea_orm(unique, index)]
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

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for LabelBase {
    fn from(value: Model) -> Self {
        LabelBase {
            id: value.id,
            name: value.name,
            description: value.description,
            remark: value.remark,
            influence: value.influence,
            enabled: value.enabled,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
