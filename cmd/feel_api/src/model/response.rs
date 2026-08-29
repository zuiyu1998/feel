//! 通用 API 响应封装。
//!
//! 提供统一的 JSON 响应格式：`ApiResponse<T>` 及其工厂函数 `ok()` / `err()`。
//!
//! # 预定义状态码
//!
//! | 常量 | 值 | 说明 |
//! |------|-----|------|
//! | [`CODE_OK`] | 0 | 业务处理成功 |
//! | [`CODE_SERVER_ERROR`] | 10000 | 服务器内部错误 |
//! | [`CODE_DB_ERROR`] | 10003 | 数据库操作异常 |
//! | [`CODE_NOT_FOUND`] | 10004 | 请求的资源不存在 |
//! | [`CODE_REDIS_ERROR`] | 10001 | Redis 操作异常 |
//! | [`CODE_JSON_ERROR`] | 10002 | JSON 序列化/反序列化错误 |
//! | [`CODE_AUTH_ERROR`] | 10005 | 认证失败（凭据无效/token 错误） |
//! | [`CODE_BUSINESS_ERROR`] | 10006 | 业务规则校验失败（如标签数量超限） |

use feel_storage::error::Error as StorageError;
use poem::web::Json;
use serde::Serialize;

/// 业务处理成功。
pub const CODE_OK: i32 = 0;

/// 服务器内部错误。
pub const CODE_SERVER_ERROR: i32 = 10000;

/// 数据库操作异常。
pub const CODE_DB_ERROR: i32 = 10003;

/// 请求的资源不存在。
pub const CODE_NOT_FOUND: i32 = 10004;

/// Redis 操作异常。
pub const CODE_REDIS_ERROR: i32 = 10001;

/// JSON 序列化/反序列化错误。
pub const CODE_JSON_ERROR: i32 = 10002;

/// 认证失败（凭据无效/token 错误）。
pub const CODE_AUTH_ERROR: i32 = 10005;

/// 业务规则校验失败（如标签数量超限）。
pub const CODE_BUSINESS_ERROR: i32 = 10006;

/// 泛型 API 响应结构体。
///
/// 所有接口返回统一的 JSON 结构。`data` 字段使用 `Option<T>`，
/// 成功时携带数据（`Some`），错误时为 `null`（`None`）。
///
/// ```json
/// { "code": 0, "message": "ok", "data": { ... } }
/// { "code": 10000, "message": "...", "data": null }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

/// 构造成功响应（`code = CODE_OK`, `message = "ok"`）。
///
/// `data` 自动包装为 `Some(data)`，JSON 输出为对象。
pub fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        code: CODE_OK,
        message: "ok".to_string(),
        data: Some(data),
    })
}

/// 构造错误响应（自定义状态码和消息）。
///
/// `data` 置为 `None`，JSON 输出为 `null`。
/// 泛型 `T` 由调用处的返回类型推断。
pub fn err<T: Serialize>(code: i32, message: impl Into<String>) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        code,
        message: message.into(),
        data: None,
    })
}

/// 将 `feel_storage::Error` 转换为 API 错误响应。
///
/// 根据错误类型自动映射状态码：
/// - `Redis` → [`CODE_REDIS_ERROR`] (10001)
/// - `Json` → [`CODE_JSON_ERROR`] (10002)
/// - `Db(DbErr::RecordNotFound)` → [`CODE_NOT_FOUND`] (10004)
/// - `Db(_)` → [`CODE_DB_ERROR`] (10003)
/// - `Authentication` → [`CODE_AUTH_ERROR`] (10005)
/// - `Business` → [`CODE_BUSINESS_ERROR`] (10006)
///
/// 消息内容直接使用错误的 display 文本。
///
/// # 示例
///
/// ```rust,ignore
/// #[handler]
/// async fn get_user(state: Data<&AppState>, Path(id): Path<i64>) -> Json<ApiResponse<UserBase>> {
///     let user = state
///         .user_database
///         .unregister(id)
///         .await
///         .map_err(from_storage_error)?;  // Error 自动转换
///     ok(user.into())
/// }
/// ```
pub fn from_storage_error<T: Serialize>(e: StorageError) -> Json<ApiResponse<T>> {
    let code = match &e {
        StorageError::Redis(_) => CODE_REDIS_ERROR,
        StorageError::Json(_) => CODE_JSON_ERROR,
        StorageError::Db(db_err) => match db_err {
            sea_orm::DbErr::RecordNotFound(_) => CODE_NOT_FOUND,
            _ => CODE_DB_ERROR,
        },
        StorageError::Authentication(_) => CODE_AUTH_ERROR,
        StorageError::Business(_) => CODE_BUSINESS_ERROR,
    };
    err(code, e.to_string())
}
