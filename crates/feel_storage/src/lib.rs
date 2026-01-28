pub mod error;

pub use error::Result;

use feel_entity::prelude::*;

pub trait UserStorage: 'static + Send + Sync {
    ///用户系统注册用户
    fn register(&self, register: &UserRegister) -> Result<User>;
    ///用户系统注销用户
    fn unregister(&self, user_id: u32) -> Result<User>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<String>;
    ///用户登出系统
    fn logout(&self, user_id: u32) -> Result<()>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<User>;
}

pub trait UserRepo: 'static + Send + Sync {
    ///用户系统注册用户
    fn register(&self, register: &UserRegister) -> Result<User>;
    ///用户系统注销用户
    fn unregister(&self, user_id: u32) -> Result<User>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<String>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<User>;
}
