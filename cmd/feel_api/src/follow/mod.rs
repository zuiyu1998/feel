//! 用户关注系统 HTTP 接口：关注 / 取关 / 统计 / 关注与粉丝列表。
//!
//! 参数传递沿用项目约定：不使用 URL 路径与查询串，通过 `Json` 请求体携带参数。

use std::collections::HashMap;

use crate::AppState;
use crate::auth::{AuthUser, auth_middleware};
use crate::model::follow::{
    AddFollowRequest, FollowListRequest, FollowPageResponse, FollowResponse, FollowStatsRequest,
    FollowStatsResponse, FollowUserBrief, RemoveFollowRequest,
};
use crate::model::response::{ApiResponse, from_storage_error, ok};
use feel_entity::follow::UserFollow;
use feel_entity::user::UserBase;
use poem::web::{Data, Json};
use poem::{EndpointExt, Route, delete, get, handler, post};

/// 关注系统路由：/api/v1/follow/...
pub fn router() -> Route {
    Route::new()
        .at("/add", post(add_follow).around(auth_middleware))
        .at("/remove", delete(remove_follow).around(auth_middleware))
        .at("/stats", get(follow_stats))
        .at("/followers", get(list_followers))
        .at("/followings", get(list_followings))
}

// -- 内部辅助 --

/// 从请求扩展提取认证用户（需挂载 `auth_middleware` 的路由使用）。
fn current_user(req: &poem::Request) -> AuthUser {
    req.extensions()
        .get::<AuthUser>()
        .expect("AuthUser not found — missing auth middleware")
        .clone()
}

/// 按 uid 解析目标用户；uid 不存在时映射为 10004（RecordNotFound）。
async fn load_user_by_uid(state: &AppState, uid: &str) -> Result<UserBase, feel_storage::Error> {
    match state.user_database.get_user(uid).await {
        Ok(user) => Ok(user),
        Err(feel_storage::Error::Authentication(msg)) => {
            Err(feel_storage::Error::Db(sea_orm::DbErr::RecordNotFound(msg)))
        }
        Err(e) => Err(e),
    }
}

/// 可选认证：请求头带合法 Bearer token 时返回请求者 user_id（用于 `is_following`）。
async fn optional_viewer(state: &AppState, req: &poem::Request) -> Option<i64> {
    let token = req
        .headers()
        .get("authorization")?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")?;
    let claims = state.user_database.parse_token(token).ok()?;
    state
        .user_database
        .get_user(&claims.sub)
        .await
        .ok()
        .map(|user| user.id)
}

/// 列表方向：粉丝（对端为关注者）或关注（对端为被关注者）。
#[derive(Clone, Copy)]
enum FollowSide {
    Followers,
    Followings,
}

/// 为关注 / 粉丝关联行补齐对端用户摘要（同一用户只查询一次；用户缺失时跳过）。
async fn enrich_users(
    state: &AppState,
    items: Vec<UserFollow>,
    side: FollowSide,
) -> Result<Vec<FollowUserBrief>, feel_storage::Error> {
    let mut cache: HashMap<i64, Option<UserBase>> = HashMap::new();
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let partner_id = match side {
            FollowSide::Followers => item.user_id,
            FollowSide::Followings => item.follow_user_id,
        };
        if !cache.contains_key(&partner_id) {
            let user = state.user_database.get_user_by_id(partner_id).await?;
            cache.insert(partner_id, user);
        }
        if let Some(Some(user)) = cache.get(&partner_id) {
            out.push(FollowUserBrief {
                user_id: user.id,
                uid: user.uid.clone(),
                name: user.name.clone(),
                avatar: user.avatar.clone(),
            });
        }
    }
    Ok(out)
}

// -- POST /follow/add — 关注 --

#[handler]
async fn add_follow(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<AddFollowRequest>,
) -> Json<ApiResponse<FollowResponse>> {
    // 1. 认证用户 → 关注者
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 2. 解析被关注者（uid 业务键）
    let target = match load_user_by_uid(&state, &body.0.target_uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 3. 建立关注关系（幂等；禁止关注自己）
    let follow = match state.user_follow_database.follow(me.id, target.id).await {
        Ok(follow) => follow,
        Err(e) => return from_storage_error(e),
    };

    ok(follow.into())
}

// -- DELETE /follow/remove — 取关 --

#[handler]
async fn remove_follow(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<RemoveFollowRequest>,
) -> Json<ApiResponse<()>> {
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    let target = match load_user_by_uid(&state, &body.0.target_uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    if let Err(e) = state.user_follow_database.unfollow(me.id, target.id).await {
        return from_storage_error(e);
    }

    ok(())
}

// -- GET /follow/stats — 关注统计（可选认证，返回 is_following） --

#[handler]
async fn follow_stats(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<FollowStatsRequest>,
) -> Json<ApiResponse<FollowStatsResponse>> {
    let target = match load_user_by_uid(&state, &body.0.user_uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    let viewer = optional_viewer(&state, req).await;

    let stats = match state
        .user_follow_database
        .get_stats(target.id, viewer)
        .await
    {
        Ok(stats) => stats,
        Err(e) => return from_storage_error(e),
    };

    ok(stats.into())
}

// -- GET /follow/followers — 粉丝列表（分页） --

#[handler]
async fn list_followers(
    state: Data<&AppState>,
    body: Json<FollowListRequest>,
) -> Json<ApiResponse<FollowPageResponse>> {
    let target = match load_user_by_uid(&state, &body.0.user_uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    let page = match state
        .user_follow_database
        .list_followers(target.id, body.0.page, body.0.page_size)
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    let items = match enrich_users(&state, page.items, FollowSide::Followers).await {
        Ok(items) => items,
        Err(e) => return from_storage_error(e),
    };

    ok(FollowPageResponse {
        items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    })
}

// -- GET /follow/followings — 关注列表（分页） --

#[handler]
async fn list_followings(
    state: Data<&AppState>,
    body: Json<FollowListRequest>,
) -> Json<ApiResponse<FollowPageResponse>> {
    let target = match load_user_by_uid(&state, &body.0.user_uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    let page = match state
        .user_follow_database
        .list_followings(target.id, body.0.page, body.0.page_size)
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    let items = match enrich_users(&state, page.items, FollowSide::Followings).await {
        Ok(items) => items,
        Err(e) => return from_storage_error(e),
    };

    ok(FollowPageResponse {
        items,
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    })
}
