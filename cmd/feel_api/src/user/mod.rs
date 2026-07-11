use crate::AppState;
use crate::model::response::{ApiResponse, ok};
use crate::model::user::{RegisterRequest, RegisterResponse};
use feel_entity::user::UserRegister;
use poem::web::{Data, Json, Path};
use poem::{Route, handler, post};

pub fn router() -> Route {
    Route::new()
        .at("/register", post(register))
        .at("/unregister/:user_id", post(unregister))
        .at("/login", post(login))
        .at("/logout", post(logout))
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
    let user_base = state.user_database.register(&user_register).await.unwrap();

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
    let user_base = state.user_database.unregister(user_id).await.unwrap();

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

// -- login / logout (stubs) --

#[handler]
async fn login() {}

#[handler]
async fn logout() {}
