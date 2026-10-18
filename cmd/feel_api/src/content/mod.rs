//! 内容相关 HTTP 接口（文章 / 视频 …）+ Feed 流。
//!
//! 参数传递沿用项目约定：不使用 URL 路径与查询串，通过 `Json` 请求体携带参数。

use std::collections::HashMap;

use crate::AppState;
use crate::auth::{AuthUser, auth_middleware};
use crate::model::content::{
    ContentInfoRequest, ContentItem, ContentPageResponse, CreateContentRequest, FeedContentRequest,
    ListContentRequest, RemoveContentRequest, UpdateContentRequest, content_item,
};
use crate::model::label::UserBrief;
use crate::model::response::{ApiResponse, from_storage_error, ok};
use feel_entity::content::{ContentCreate, ContentPage, ContentUpdate, FeedSort};
use feel_entity::user::UserBase;
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

// -- 内部辅助 --

/// 从请求扩展提取认证用户（需挂载 `auth_middleware` 的路由使用）。
fn current_user(req: &poem::Request) -> AuthUser {
    req.extensions()
        .get::<AuthUser>()
        .expect("AuthUser not found — missing auth middleware")
        .clone()
}

/// 将用户主体转换为作者摘要。
fn brief_of(user: &UserBase) -> UserBrief {
    UserBrief {
        user_id: user.id,
        name: user.name.clone(),
        avatar: user.avatar.clone(),
    }
}

/// 查询某用户的作者摘要（用户不存在时以空名称占位，保证 Feed 卡片可展示）。
async fn load_author(state: &AppState, user_id: i64) -> Result<UserBrief, feel_storage::Error> {
    Ok(match state.user_database.get_user_by_id(user_id).await? {
        Some(user) => brief_of(&user),
        None => UserBrief {
            user_id,
            name: String::new(),
            avatar: String::new(),
        },
    })
}

/// 为分页内容列表批量补齐作者摘要（同一作者只查询一次）。
async fn enrich_items(
    state: &AppState,
    page: &ContentPage,
) -> Result<Vec<ContentItem>, feel_storage::Error> {
    let mut cache: HashMap<i64, UserBrief> = HashMap::new();
    let mut items = Vec::with_capacity(page.items.len());
    for content in &page.items {
        let user_id = content.base.user_id;
        if !cache.contains_key(&user_id) {
            let author = load_author(state, user_id).await?;
            cache.insert(user_id, author);
        }
        let author = cache[&user_id].clone();
        items.push(content_item(content.clone(), author));
    }
    Ok(items)
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

    ok(content_item(content, brief_of(&me)))
}

// -- GET /content/info — 查看内容详情（记录浏览量） --

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

    let author = match load_author(&state, content.base.user_id).await {
        Ok(author) => author,
        Err(e) => return from_storage_error(e),
    };

    // 记录浏览量（best-effort：失败不影响详情返回，仅记录告警）
    let mut view_bumped = false;
    match state.content_database.record_view(content.base.id).await {
        Ok(()) => view_bumped = true,
        Err(e) => tracing::warn!("record view failed for content {}: {e}", body.0.content_id),
    }

    let mut item = content_item(content, author);
    if view_bumped {
        item.view_count += 1;
    }
    ok(item)
}

// -- GET /content/list — 某用户的Feed流列表（分页） --

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

    let items = match enrich_items(&state, &page).await {
        Ok(items) => items,
        Err(e) => return from_storage_error(e),
    };

    ok(ContentPageResponse::with_items(items, &page))
}

// -- GET /content/feed — 全站 Feed 流（分页，可按类型过滤、最新/热门排序） --

#[handler]
async fn content_feed(
    state: Data<&AppState>,
    body: Json<FeedContentRequest>,
) -> Json<ApiResponse<ContentPageResponse>> {
    // 1. 解析排序方式（默认最新）
    let sort = match body.0.sort.as_deref() {
        None => FeedSort::Latest,
        Some(raw) => match FeedSort::parse(raw) {
            Ok(sort) => sort,
            Err(msg) => return from_storage_error(feel_storage::Error::Business(msg)),
        },
    };

    // 2. 查询 feed 分页
    let page = match state
        .content_database
        .list_feed(
            body.0.content_type.as_deref(),
            sort,
            body.0.page,
            body.0.page_size,
        )
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    // 3. 补齐作者摘要 → Feed 卡片
    let items = match enrich_items(&state, &page).await {
        Ok(items) => items,
        Err(e) => return from_storage_error(e),
    };

    ok(ContentPageResponse::with_items(items, &page))
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

    ok(content_item(content, brief_of(&me)))
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
