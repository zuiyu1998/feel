use crate::{Result, repo::UserRepo, utils::get_passwod_hash};
use async_trait::async_trait;
use chrono::Local;
use feel_entity::prelude::*;
use feel_sea_orm::user::entities::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel,
    TransactionTrait,
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

    fn login(&self, _login: &UserLogin) -> Result<String> {
        todo!()
    }

    fn update(&self, _update: &UserUpdate) -> Result<UserBase> {
        todo!()
    }
}
