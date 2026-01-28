use crate::UserDataBase;
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
