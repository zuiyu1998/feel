//! JWT 认证中间件。
//!
//! 提供 [`auth_middleware`] 函数，配合 [`EndpointExt::around`] 使用，
//! 在需要认证的路由上拦截请求、校验 Bearer token，并将 [`AuthUser`]
//! 注入到请求扩展中，供后续 handler 通过 `req.extensions().get::<AuthUser>()` 提取。

use crate::AppState;
use poem::{Endpoint, IntoResponse, Request, Response, http::StatusCode};
use std::sync::Arc;

/// 从 JWT token 中解析出的认证用户信息。
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub uid: String,
}

/// JWT 认证中间件。
///
/// 1. 从 `Authorization: Bearer <token>` 请求头提取 token
/// 2. 调用 `UserDataBase::parse_token` 验证签名和过期
/// 3. 将解析出的 `AuthUser` 注入请求扩展
///
/// # 错误
///
/// 当 token 缺失、无效或已过期时，返回 HTTP 401 + 统一 JSON 错误体。
pub async fn auth_middleware<E: Endpoint>(
    inner: Arc<E>,
    mut req: Request,
) -> poem::Result<Response> {
    // 1. 从 request data 中获取 AppState
    let state = match req.data::<AppState>() {
        Some(s) => s,
        None => {
            tracing::error!("AppState not found in request data");
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };

    // 2. 提取 Bearer token
    let token = match req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(t) => t,
        None => {
            return Ok(auth_error_response(
                "Missing or invalid Authorization header",
            ));
        }
    };

    // 3. 解析并验证 JWT
    let claims = match state.user_database.parse_token(token) {
        Ok(c) => c,
        Err(e) => {
            return Ok(auth_error_response(&e.to_string()));
        }
    };

    // 4. 将 AuthUser 注入请求扩展
    req.extensions_mut().insert(AuthUser { uid: claims.sub });

    // 5. 调用内部 endpoint
    let resp = inner.call(req).await?;
    Ok(resp.into_response())
}

/// 构建统一的认证错误 JSON 响应（HTTP 401）。
fn auth_error_response(message: &str) -> Response {
    let body = serde_json::json!({
        "code": 10005,
        "message": message,
        "data": null,
    });
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .content_type("application/json")
        .body(body.to_string())
}
