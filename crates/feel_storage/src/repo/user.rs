use crate::{Error, Result, repo::UserRepo, utils::get_passwod_hash};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use async_trait::async_trait;
use chrono::Local;
use feel_entity::prelude::*;
use feel_sea_orm::user::entities::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter, TransactionTrait,
};

typedflake::id!(UserId);

pub struct SeaOrmUserRepo {
    /// Sea-orm database connection
    pub conn: DatabaseConnection,
}

impl SeaOrmUserRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        SeaOrmUserRepo { conn }
    }
}

#[async_trait]
impl UserRepo for SeaOrmUserRepo {
    async fn register(&self, register: &UserRegister) -> Result<UserBase> {
        let begin = self.conn.begin().await?;

        let now = Local::now();

        let mut user_active_model: UserActiveModel = Default::default();
        user_active_model.name = Set(register.name.clone());
        user_active_model.avatar = Set(register.avatar.clone());
        user_active_model.uid = Set(UserId::generate().to_string());
        user_active_model.updated_at = Set(now.to_utc());
        user_active_model.created_at = Set(now.to_utc());
        let user: UserModel = user_active_model.insert(&begin).await?;

        let (salt, password) = get_passwod_hash(&register.data);

        let mut user_credentials_active_model: UserCredentialsActiveModel = Default::default();
        user_credentials_active_model.encryption_key = Set(salt);
        user_credentials_active_model.encrypted_data = Set(password);
        user_credentials_active_model.credential_type = Set(register.credential_type.to_string());
        user_credentials_active_model.credential_name = Set(register.credential_name.to_string());
        user_credentials_active_model.user_uid = Set(user.uid.clone());
        user_credentials_active_model.updated_at = Set(now.to_utc());
        user_credentials_active_model.created_at = Set(now.to_utc());

        let _user_credentials: UserCredentialsModel =
            user_credentials_active_model.insert(&begin).await?;

        begin.commit().await?;

        Ok(user.into())
    }

    async fn unregister(&self, user_id: i64) -> Result<UserBase> {
        let now = Local::now();

        let user = UserEntity::find_by_id(user_id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| sea_orm::DbErr::RecordNotFound(format!("User {} not found", user_id)))?;

        let mut user_active = user.into_active_model();
        user_active.is_delete = Set(true);
        user_active.updated_at = Set(now.to_utc());

        let updated_user: UserModel = user_active.update(&self.conn).await?;

        Ok(updated_user.into())
    }

    async fn login(&self, login: &UserLogin) -> Result<UserBase> {
        // 1. Find credential by credential_name
        let credential = UserCredentialsEntity::find()
            .filter(UserCredentialsColumn::CredentialName.eq(&login.credential_name))
            .one(&self.conn)
            .await?
            .ok_or_else(|| Error::Authentication("Credential not found".into()))?;

        // 2. Verify password with argon2
        let parsed_hash = PasswordHash::new(&credential.encrypted_data)
            .map_err(|e| Error::Authentication(format!("Invalid password hash: {}", e)))?;

        Argon2::default()
            .verify_password(login.data.as_bytes(), &parsed_hash)
            .map_err(|_| Error::Authentication("Invalid password".into()))?;

        // 3. Find user by user_uid and return user data (token generation is in CommonUserDataBase)
        UserEntity::find()
            .filter(UserColumn::Uid.eq(&credential.user_uid))
            .one(&self.conn)
            .await?
            .ok_or_else(|| Error::Authentication("User not found".into()))
            .map(|user| user.into())
    }

    fn update(&self, _update: &UserUpdate) -> Result<UserBase> {
        todo!()
    }

    async fn find_by_credential_name(&self, credential_name: &str) -> Result<Option<UserBase>> {
        let credential = UserCredentialsEntity::find()
            .filter(UserCredentialsColumn::CredentialName.eq(credential_name))
            .one(&self.conn)
            .await?;

        match credential {
            Some(cred) => {
                let user = UserEntity::find()
                    .filter(UserColumn::Uid.eq(cred.user_uid))
                    .one(&self.conn)
                    .await?;
                Ok(user.map(|u| u.into()))
            }
            None => Ok(None),
        }
    }

    async fn find_by_uid(&self, uid: &str) -> Result<Option<UserBase>> {
        let user = UserEntity::find()
            .filter(UserColumn::Uid.eq(uid))
            .one(&self.conn)
            .await?;
        Ok(user.map(|u| u.into()))
    }
}
