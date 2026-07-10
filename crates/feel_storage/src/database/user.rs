use crate::{Result, repo::UserRepo};
use async_trait::async_trait;
use feel_entity::prelude::*;

pub struct CommonUserDataBase {
    user_repo: Box<dyn UserRepo>,
}

impl CommonUserDataBase {
    pub fn new<T: UserRepo>(user_repo: T) -> Self {
        CommonUserDataBase {
            user_repo: Box::new(user_repo),
        }
    }
}

#[async_trait]
impl UserDataBase for CommonUserDataBase {
    async fn register(&self, register: &UserRegister) -> crate::Result<UserBase> {
        self.user_repo.register(register).await
    }

    async fn unregister(&self, user_id: i64) -> crate::Result<UserBase> {
        self.user_repo.unregister(user_id).await
    }

    fn login(&self, _login: &UserLogin) -> crate::Result<String> {
        todo!()
    }

    fn logout(&self, _user_id: u32) -> crate::Result<()> {
        todo!()
    }

    fn update(&self, _update: &UserUpdate) -> crate::Result<UserBase> {
        todo!()
    }
}

#[async_trait]
pub trait UserDataBase: 'static + Send + Sync {
    ///用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;
    ///用户系统注销用户
    async fn unregister(&self, user_id: i64) -> Result<UserBase>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<String>;
    ///用户登出系统
    fn logout(&self, user_id: u32) -> Result<()>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;
}
