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

/// User registration entity (combines UserBase and UserCredential)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserRegister {
    /// User display name (required)
    pub name: String,
    /// User avatar URL (required)
    pub avatar: String,
    /// Credential type (required, e.g., "password", "email", "phone")
    pub credential_type: String,
    /// Credential name/identifier (required, e.g., email address or phone number)
    pub credential_name: String,
    /// Credential data (required, will be encrypted)
    pub data: String,
}

pub struct UserLogin {}

pub struct UserUpdate {}

/// User credential entity for authentication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserCredential {
    /// Database primary key
    pub id: i64,
    /// User unique identifier
    pub user_uid: String,
    /// Credential type (e.g., password, email, phone)
    pub credential_type: String,
    /// Credential name/identifier
    pub credential_name: String,
    /// Encrypted credential data
    pub encrypted_data: String,
    /// Encryption key identifier
    pub encryption_key: String,
    /// Whether the credential is enabled
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}
