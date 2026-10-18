//! 内容相关 HTTP 接口（文章 / 视频 …）。
//!
//! 参数传递沿用项目约定：不使用 URL 路径与查询串，通过 `Json` 请求体携带参数。

use crate::AppState;
use crate::auth::{AuthUser, auth_middleware};
use crate::model::content::{
    ContentInfoRequest, ContentItem, ContentPageResponse, CreateContentRequest, FeedContentRequest,
    ListContentRequest, RemoveContentRequest, UpdateContentRequest,
};
use crate::model::response::{ApiResponse, from_storage_error, ok};
use feel_entity::content::{ContentCreate, ContentUpdate};
use poem::web::{Data, Json};
use poem::{EndpointExt, Route, delete, get, handler, post, put};

/// 内容相关路由：/api/v1/content/...
pub fn router() -> Route {
    Route::new()
        .at("/create", post(create_content).around(auth_middleware))
        .at("/info", get(content_info))
        .at("/list", get(list_content))
        .at("/feed", get(content_feed))
        .at("/update", put(update_content).around(auth_middleware))
        .at("/remove", delete(remove_content).around(auth_middleware))
}

/// 从请求扩展提取认证用户（需挂载 `auth_middleware` 的路由使用）。
fn current_user(req: &poem::Request) -> AuthUser {
    req.extensions()
        .get::<AuthUser>()
        .expect("AuthUser not found — missing auth middleware")
        .clone()
}

// -- POST /content/create — 创建内容 --

#[handler]
async fn create_content(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<CreateContentRequest>,
) -> Json<ApiResponse<ContentItem>> {
    // 1. 认证用户 → 作者主键
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 2. DTO → 领域模型
    let create = ContentCreate {
        user_id: me.id,
        content_type: body.0.content_type,
        title: body.0.title,
        payload: body.0.payload,
    };

    // 3. 调用领域层（标题/载荷校验 + 基表与明细落库）
    let content = match state.content_database.create(&create).await {
        Ok(content) => content,
        Err(e) => return from_storage_error(e),
    };

    ok(content.into())
}

// -- GET /content/info — 查看内容详情 --

#[handler]
async fn content_info(
    state: Data<&AppState>,
    body: Json<ContentInfoRequest>,
) -> Json<ApiResponse<ContentItem>> {
    let content = match state.content_database.get(body.0.content_id).await {
        Ok(Some(content)) => content,
        Ok(None) => {
            return from_storage_error(feel_storage::Error::Db(sea_orm::DbErr::RecordNotFound(
                format!("Content {} not found", body.0.content_id),
            )));
        }
        Err(e) => return from_storage_error(e),
    };

    ok(content.into())
}

// -- GET /content/list — 某用户的内容列表（分页） --

#[handler]
async fn list_content(
    state: Data<&AppState>,
    body: Json<ListContentRequest>,
) -> Json<ApiResponse<ContentPageResponse>> {
    let page = match state
        .content_database
        .list_by_user(body.0.user_id, body.0.page, body.0.page_size)
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    ok(page.into())
}

// -- GET /content/feed — 全站内容流（分页，可按类型过滤） --

#[handler]
async fn content_feed(
    state: Data<&AppState>,
    body: Json<FeedContentRequest>,
) -> Json<ApiResponse<ContentPageResponse>> {
    let page = match state
        .content_database
        .list_feed(
            body.0.content_type.as_deref(),
            body.0.page,
            body.0.page_size,
        )
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    ok(page.into())
}

// -- PUT /content/update — 更新内容（仅作者） --

#[handler]
async fn update_content(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<UpdateContentRequest>,
) -> Json<ApiResponse<ContentItem>> {
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    let update = ContentUpdate {
        id: body.0.content_id,
        user_id: me.id,
        title: body.0.title,
        payload: body.0.payload,
    };

    let content = match state.content_database.update(&update).await {
        Ok(content) => content,
        Err(e) => return from_storage_error(e),
    };

    ok(content.into())
}

// -- DELETE /content/remove — 软删除（仅作者） --

#[handler]
async fn remove_content(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<RemoveContentRequest>,
) -> Json<ApiResponse<()>> {
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    if let Err(e) = state
        .content_database
        .remove(body.0.content_id, me.id)
        .await
    {
        return from_storage_error(e);
    }

    ok(())
}
