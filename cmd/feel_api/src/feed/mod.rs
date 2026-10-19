//! 关注流（Feed）接口：推拉结合的"我关注的人"时间线。
//!
//! 推：普通作者发布内容时扇出写入粉丝的 Redis 收件箱（见 `UserFeedDataBase::publish`）；
//! 拉：大 V 作者不推送，读取时按需拉取其最近内容并合并（见 `UserFeedDataBase::get_following_feed`）。

use std::collections::HashMap;

use crate::AppState;
use crate::auth::{AuthUser, auth_middleware};
use crate::model::content::content_item;
use crate::model::feed::{FollowingFeedRequest, FollowingFeedResponse};
use crate::model::label::UserBrief;
use crate::model::response::{ApiResponse, from_storage_error, ok};
use feel_entity::content::Content;
use poem::web::{Data, Json};
use poem::{EndpointExt, Route, get, handler};

/// 关注流路由：/api/v1/feed/...
pub fn router() -> Route {
    Route::new().at("/following", get(following_feed).around(auth_middleware))
}

// -- 内部辅助 --

/// 从请求扩展提取认证用户。
fn current_user(req: &poem::Request) -> AuthUser {
    req.extensions()
        .get::<AuthUser>()
        .expect("AuthUser not found — missing auth middleware")
        .clone()
}

/// 为内容列表内嵌作者摘要（同一作者只查询一次；用户缺失时以空名称占位）。
async fn enrich_items(
    state: &AppState,
    contents: Vec<Content>,
) -> Result<Vec<crate::model::content::ContentItem>, feel_storage::Error> {
    let mut cache: HashMap<i64, UserBrief> = HashMap::new();
    let mut out = Vec::with_capacity(contents.len());
    for content in contents {
        let author_id = content.base.user_id;
        if !cache.contains_key(&author_id) {
            let author = match state.user_database.get_user_by_id(author_id).await? {
                Some(user) => UserBrief {
                    user_id: user.id,
                    name: user.name,
                    avatar: user.avatar,
                },
                None => UserBrief {
                    user_id: author_id,
                    name: String::new(),
                    avatar: String::new(),
                },
            };
            cache.insert(author_id, author);
        }
        let author = cache[&author_id].clone();
        out.push(content_item(content, author));
    }
    Ok(out)
}

// -- GET /feed/following — 关注流（推拉结合） --

#[handler]
async fn following_feed(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<FollowingFeedRequest>,
) -> Json<ApiResponse<FollowingFeedResponse>> {
    // 1. 认证用户 → 收件箱主人
    let auth = current_user(req);
    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 2. 推拉合并获取关注流
    let page = match state
        .feed_database
        .get_following_feed(me.id, body.0.page, body.0.page_size)
        .await
    {
        Ok(page) => page,
        Err(e) => return from_storage_error(e),
    };

    // 3. 补齐作者摘要 → 统一卡片结构
    let items = match enrich_items(&state, page.items).await {
        Ok(items) => items,
        Err(e) => return from_storage_error(e),
    };

    ok(FollowingFeedResponse {
        items,
        page: page.page,
        page_size: page.page_size,
        has_more: page.has_more,
    })
}
