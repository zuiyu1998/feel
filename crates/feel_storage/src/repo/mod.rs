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
    async fn unregister(&self, user_id: i64) -> Result<UserBase>;
    ///用户登录系统
    async fn login(&self, login: &UserLogin) -> Result<UserBase>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;
    ///根据凭据名称查找用户
    async fn find_by_credential_name(&self, credential_name: &str) -> Result<Option<UserBase>>;
    ///根据用户 UID 查找用户
    async fn find_by_uid(&self, uid: &str) -> Result<Option<UserBase>>;
}
