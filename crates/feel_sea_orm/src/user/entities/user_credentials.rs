use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    /// User unique identifier
    #[sea_orm(index)]
    pub user_uid: String,
    /// Credential type (e.g., password, email, phone)
    #[sea_orm(index)]
    pub credential_type: String,
    /// Credential name/identifier
    pub credential_name: String,
    /// Encrypted credential data
    #[sea_orm(select, insert, update)]
    pub encrypted_data: String,
    /// Encryption key identifier
    pub encryption_key: String,
    /// Whether the credential is enabled
    pub is_enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
