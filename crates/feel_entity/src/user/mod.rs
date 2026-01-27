mod models;

pub use models::*;

use crate::Result;

///用户存储
pub trait UserRepo: 'static + Send + Sync {
    ///用户系统注册用户
    fn register(&self, register: &UserRegister) -> Result<User>;
    ///用户系统注销用户
    fn unregister(&self, user_id: u32) -> Result<User>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<User>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<User>;
}

pub trait UserDataBase: 'static + Send + Sync {}
