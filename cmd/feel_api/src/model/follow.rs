//! 关注系统 API 的请求/响应 DTO。

use feel_entity::follow::{FollowPage, FollowStats, UserFollow};
use serde::{Deserialize, Serialize};

// -- 请求 --

/// 关注请求体（POST /follow/add）
#[derive(Debug, Deserialize)]
pub struct AddFollowRequest {
    /// 被关注者的 uid（业务键）
    pub target_uid: String,
}

/// 取关请求体（DELETE /follow/remove）
#[derive(Debug, Deserialize)]
pub struct RemoveFollowRequest {
    /// 被取关者的 uid（业务键）
    pub target_uid: String,
}

/// 关注统计请求体（GET /follow/stats）
#[derive(Debug, Deserialize)]
pub struct FollowStatsRequest {
    /// 要查询的用户 uid（业务键）
    pub user_uid: String,
}

/// 关注 / 粉丝列表请求体（GET /follow/followers | /follow/followings）
#[derive(Debug, Deserialize)]
pub struct FollowListRequest {
    /// 要查询的用户 uid（业务键）
    pub user_uid: String,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

// -- 响应 --

/// 关注关系响应（添加关注后返回）。
#[derive(Debug, Serialize)]
pub struct FollowResponse {
    pub id: i64,
    pub user_id: i64,
    pub follow_user_id: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserFollow> for FollowResponse {
    fn from(value: UserFollow) -> Self {
        FollowResponse {
            id: value.id,
            user_id: value.user_id,
            follow_user_id: value.follow_user_id,
            enabled: value.enabled,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

/// 关注 / 粉丝列表项（内嵌对端用户摘要）。
#[derive(Debug, Serialize)]
pub struct FollowUserBrief {
    pub user_id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
}

/// 关注 / 粉丝分页响应。
#[derive(Debug, Serialize)]
pub struct FollowPageResponse {
    pub items: Vec<FollowUserBrief>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl FollowPageResponse {
    /// 由领域分页结果（分页元信息）与已组装的对端用户摘要构造响应。
    pub fn with_items(items: Vec<FollowUserBrief>, page: &FollowPage) -> Self {
        FollowPageResponse {
            items,
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        }
    }
}

/// 关注统计响应。
#[derive(Debug, Serialize)]
pub struct FollowStatsResponse {
    pub user_id: i64,
    pub follower_count: u64,
    pub following_count: u64,
    /// 请求者是否关注该用户（匿名访问或查看本人时为 null）
    pub is_following: Option<bool>,
}

impl From<FollowStats> for FollowStatsResponse {
    fn from(value: FollowStats) -> Self {
        FollowStatsResponse {
            user_id: value.user_id,
            follower_count: value.follower_count,
            following_count: value.following_count,
            is_following: value.is_following,
        }
    }
}
