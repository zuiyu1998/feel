use feel_core::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户关注关系（`user_follow` 表）：`user_id` 关注 `follow_user_id`。
///
/// 关注者与被关注者均以 `UserBase.id`（内部主键）做关联，与 `user_label.user_id` 的约定一致。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserFollow {
    /// Database primary key
    pub id: i64,
    /// 关注者（引用 UserBase.id）
    pub user_id: i64,
    /// 被关注者（引用 UserBase.id）
    pub follow_user_id: i64,
    /// 关联是否启用
    pub enabled: bool,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// 建立关注关系请求（`UserFollowRepo::create_user_follow` 参数）。
/// `id`/`enabled`/时间戳由存储层管理。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserFollowCreate {
    /// 关注者
    pub user_id: i64,
    /// 被关注者
    pub follow_user_id: i64,
}

/// 关注统计。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowStats {
    /// 统计对象用户
    pub user_id: i64,
    /// 粉丝数（关注该用户的人数）
    pub follower_count: u64,
    /// 关注数（该用户关注的人数）
    pub following_count: u64,
    /// 请求者是否关注该用户（匿名访问或查看本人时为 None）
    pub is_following: Option<bool>,
}

/// 关注 / 粉丝分页结果（`items` 为 `user_follow` 关联行，对端用户摘要由 API 层补齐）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowPage {
    pub items: Vec<UserFollow>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
