//! 通用 API 响应封装。
//!
//! 提供统一的 JSON 响应格式：`ApiResponse<T>` 及其工厂函数 `ok()` / `err()`。

use poem::web::Json;
use serde::Serialize;

/// 泛型 API 响应结构体。
///
/// 所有接口返回统一的 JSON 结构：
/// ```json
/// { "code": 0, "message": "ok", "data": { ... } }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: T,
}

/// 构造成功响应（`code = 0`, `message = "ok"`）。
pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        code: 0,
        message: "ok".to_string(),
        data,
    })
}

/// 构造错误响应（自定义状态码和消息）。
pub fn err<T: Serialize>(code: i32, message: impl Into<String>, data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        code,
        message: message.into(),
        data,
    })
}
