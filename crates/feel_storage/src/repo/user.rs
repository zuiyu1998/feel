use sea_orm::DatabaseConnection;

use crate::{Result, repo::UserRepo};
use feel_entity::prelude::*;

pub struct SeaOrmUserRepo {
    /// Sea-orm database connection
    pub conn: DatabaseConnection,
}

impl SeaOrmUserRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        SeaOrmUserRepo { conn }
    }
}

impl UserRepo for SeaOrmUserRepo {
    fn register(&self, _register: &UserRegister) -> Result<UserBase> {
        todo!()
    }

    fn unregister(&self, _user_id: u32) -> Result<UserBase> {
        todo!()
    }

    fn login(&self, _login: &UserLogin) -> Result<String> {
        todo!()
    }

    fn update(&self, _update: &UserUpdate) -> Result<UserBase> {
        todo!()
    }
}
