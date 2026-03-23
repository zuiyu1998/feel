mod user;

pub use user::*;

use crate::Result;
use async_trait::async_trait;
use feel_entity::prelude::*;

#[async_trait]
pub trait UserRepo: 'static + Send + Sync {
    ///用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;
    ///用户系统注销用户
    fn unregister(&self, user_id: u32) -> Result<UserBase>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<String>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;
}
