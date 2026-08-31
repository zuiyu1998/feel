use crate::AppState;
use crate::model::response::{ApiResponse, from_storage_error, ok};
use crate::model::user::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse, UserInfoResponse,
};
use feel_entity::user::{UserLogin, UserRegister};
use poem::web::{Data, Json, Path};
use poem::{Route, get, handler, post};

pub fn router() -> Route {
    Route::new()
        .at("/register", post(register))
        .at("/unregister/:user_id", post(unregister))
        .at("/login", post(login))
        .at("/logout", post(logout))
        .at("/info/:uid", get(info))
}

// -- register --

#[handler]
async fn register(
    state: Data<&AppState>,
    body: Json<RegisterRequest>,
) -> Json<ApiResponse<RegisterResponse>> {
    // 1. DTO → 领域模型转换
    let user_register = UserRegister {
        name: body.0.name,
        avatar: body.0.avatar,
        credential_type: body.0.credential_type,
        credential_name: body.0.credential_name,
        data: body.0.data,
    };

    // 2. 调用领域层
    let user_base = match state.user_database.register(&user_register).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 3. 领域模型 → DTO 转换 + 统一响应包装
    let response = RegisterResponse {
        id: user_base.id,
        uid: user_base.uid,
        name: user_base.name,
        avatar: user_base.avatar,
        slogan: user_base.slogan,
        enabled: user_base.enabled,
        created_at: user_base.created_at.to_rfc3339(),
        updated_at: user_base.updated_at.to_rfc3339(),
    };

    ok(response)
}

// -- unregister --

#[handler]
async fn unregister(
    state: Data<&AppState>,
    Path(user_id): Path<i64>,
) -> Json<ApiResponse<RegisterResponse>> {
    // 1. 调用领域层
    let user_base = match state.user_database.unregister(user_id).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 2. 领域模型 → DTO 转换 + 统一响应包装
    let response = RegisterResponse {
        id: user_base.id,
        uid: user_base.uid,
        name: user_base.name,
        avatar: user_base.avatar,
        slogan: user_base.slogan,
        enabled: user_base.enabled,
        created_at: user_base.created_at.to_rfc3339(),
        updated_at: user_base.updated_at.to_rfc3339(),
    };

    ok(response)
}

// -- login --

#[handler]
async fn login(
    state: Data<&AppState>,
    body: Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // 1. DTO → 领域模型转换
    let user_login = UserLogin {
        credential_name: body.0.credential_name,
        data: body.0.data,
    };

    // 2. 调用领域层
    let login_result = match state.user_database.login(&user_login).await {
        Ok(result) => result,
        Err(e) => return from_storage_error(e),
    };

    // 3. 领域模型 → DTO 转换 + 统一响应包装
    let response = LoginResponse {
        token: login_result.token,
    };

    ok(response)
}

// -- logout (stub) --

#[handler]
async fn logout() {}

// -- info --

#[handler]
async fn info(
    state: Data<&AppState>,
    Path(uid): Path<String>,
) -> Json<ApiResponse<UserInfoResponse>> {
    // 1. Fetch user info by uid (from path parameter)
    let user_base = match state.user_database.get_user(&uid).await {
        Ok(u) => u,
        Err(e) => return from_storage_error(e),
    };

    // 2. Domain model → DTO + unified response
    let response = UserInfoResponse {
        id: user_base.id,
        uid: user_base.uid,
        name: user_base.name,
        avatar: user_base.avatar,
        slogan: user_base.slogan,
        enabled: user_base.enabled,
        created_at: user_base.created_at.to_rfc3339(),
        updated_at: user_base.updated_at.to_rfc3339(),
    };

    ok(response)
}
