use feel_core::chrono::NaiveDateTime;

pub struct UserRegister {}

pub struct UserLogin {}

pub struct UserUpdate {}

pub struct User {
    pub id: u32,
    pub uuid: String,
    pub nikename: String,
    pub avatar: String,
    pub slogan: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
}
