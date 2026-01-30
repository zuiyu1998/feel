use crate::Result;
use feel_entity::prelude::*;

pub struct CommonUserDataBase;

impl CommonUserDataBase {
    pub fn new() -> Self {
        CommonUserDataBase
    }
}

impl UserDataBase for CommonUserDataBase {
    fn register(&self, _register: &UserRegister) -> crate::Result<UserBase> {
        todo!()
    }

    fn unregister(&self, _user_id: u32) -> crate::Result<UserBase> {
        todo!()
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

pub trait UserDataBase: 'static + Send + Sync {
    ///用户系统注册用户
    fn register(&self, register: &UserRegister) -> Result<UserBase>;
    ///用户系统注销用户
    fn unregister(&self, user_id: u32) -> Result<UserBase>;
    ///用户登录系统
    fn login(&self, login: &UserLogin) -> Result<String>;
    ///用户登出系统
    fn logout(&self, user_id: u32) -> Result<()>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;
}
