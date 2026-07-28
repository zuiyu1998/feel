//! 用户 API 的请求/响应 DTO。
//!
//! 当前 API handler 直接使用 `feel_entity::user::models` 中的领域类型
//! （`UserRegister`、`UserBase`）。后续可按需在此定义 API 专用的
//! DTO，以解耦 HTTP 层与领域层。

use serde::{Deserialize, Serialize};

/// 注册用户请求体
///
/// 与 `feel_entity::user::models::UserRegister` 对应，
/// 后续可在此添加校验注解或字段调整。
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub avatar: String,
    pub credential_type: String,
    pub credential_name: String,
    pub data: String,
}

/// 注册用户响应体
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 登录请求体
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub credential_name: String,
    pub data: String,
}

/// 登录成功响应体
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

/// 用户信息响应体（用于 GET /user/info）
#[derive(Debug, Serialize)]
pub struct UserInfoResponse {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}
