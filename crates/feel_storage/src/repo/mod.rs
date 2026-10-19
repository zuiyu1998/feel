mod content;
mod follow;
mod label;
mod user;

pub use content::*;
pub use follow::*;
pub use label::*;
pub use user::*;

use crate::Result;
use async_trait::async_trait;
use feel_entity::prelude::*;

#[async_trait]
pub trait UserRepo: 'static + Send + Sync {
    ///用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;
    ///用户系统注销用户
    async fn unregister(&self, user_id: i64) -> Result<UserBase>;
    ///用户登录系统
    async fn login(&self, login: &UserLogin) -> Result<UserBase>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;
    ///根据凭据名称查找用户
    async fn find_by_credential_name(&self, credential_name: &str) -> Result<Option<UserBase>>;
    ///根据用户 UID 查找用户
    async fn find_by_uid(&self, uid: &str) -> Result<Option<UserBase>>;
    ///根据用户 ID 查找用户
    async fn find_by_id(&self, user_id: i64) -> Result<Option<UserBase>>;
}

#[async_trait]
pub trait LabelRepo: 'static + Send + Sync {
    /// 按 ID 查找标签本体
    async fn find_label_by_id(&self, label_id: i64) -> Result<Option<LabelBase>>;

    /// 按名称查找标签本体(名称全局唯一)
    async fn find_label_by_name(&self, name: &str) -> Result<Option<LabelBase>>;

    /// 查询全部标签本体
    async fn find_all_labels(&self) -> Result<Vec<LabelBase>>;

    /// 创建标签本体
    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase>;

    /// 更新标签本体(如修改备注、禁用/启用)
    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase>;

    /// 查询某用户的全部标签关联(按创建时间排序)
    async fn find_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>>;

    /// 查询某标签下的全部用户关联
    async fn find_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>>;

    /// 统计某用户的关联数量(用于数量上限校验)
    async fn count_user_labels(&self, user_id: i64) -> Result<u64>;

    /// 创建用户标签关联(同一用户对同一标签只能关联一次)
    async fn create_user_label(&self, create: &UserLabelCreate) -> Result<UserLabel>;

    /// 按关联 ID 解除用户标签关联
    async fn delete_user_label(&self, user_label_id: i64) -> Result<()>;

    /// 按 用户 + 标签 解除关联
    async fn delete_user_label_by_ids(&self, user_id: i64, label_id: i64) -> Result<()>;
}

#[async_trait]
pub trait ContentRepo: 'static + Send + Sync {
    /// 创建内容（基表 + 类型明细，同一事务）
    async fn create(&self, create: &ContentCreate) -> Result<Content>;

    /// 按 ID 查找内容（含类型明细；明细缺失时返回 None）
    async fn find_by_id(&self, content_id: i64) -> Result<Option<Content>>;

    /// 分页查询某用户的公开内容（enabled 且未删除，按创建时间倒序）
    async fn find_by_user(&self, user_id: i64, offset: u64, limit: u64) -> Result<Vec<Content>>;

    /// 批量查询多个作者的最近公开内容（关注流拉模式合并用）
    async fn find_recent_by_users(
        &self,
        user_ids: &[i64],
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Content>>;

    /// 统计某用户的公开内容数量
    async fn count_by_user(&self, user_id: i64) -> Result<u64>;

    /// 分页查询全站内容流（enabled 且未删除，可按 content_type 过滤、按热度/时间排序）
    async fn find_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Content>>;

    /// 原子增加内容浏览量；返回更新行数（内容不存在/已删除时为 0）
    async fn increment_view_count(&self, content_id: i64) -> Result<u64>;

    /// 统计全站内容流数量
    async fn count_feed(&self, content_type: Option<&str>) -> Result<u64>;

    /// 更新内容（标题/载荷；类型明细 upsert）
    async fn update(&self, update: &ContentUpdate) -> Result<Content>;

    /// 软删除内容（仅作者）
    async fn soft_delete(&self, content_id: i64, user_id: i64) -> Result<()>;
}

#[async_trait]
pub trait UserFollowRepo: 'static + Send + Sync {
    /// 创建关注关系（follower -> followee）
    async fn create_user_follow(&self, create: &UserFollowCreate) -> Result<UserFollow>;

    /// 查询关注关系（follower 是否关注 followee）
    async fn find_user_follow(
        &self,
        user_id: i64,
        follow_user_id: i64,
    ) -> Result<Option<UserFollow>>;

    /// 解除关注关系
    async fn delete_user_follow(&self, user_id: i64, follow_user_id: i64) -> Result<()>;

    /// 分页查询某用户的关注列表（按创建时间倒序）
    async fn find_followings(
        &self,
        user_id: i64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<UserFollow>>;

    /// 分页查询某用户的粉丝列表（按创建时间倒序）
    async fn find_followers(
        &self,
        user_id: i64,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<UserFollow>>;

    /// 统计某用户的关注数
    async fn count_followings(&self, user_id: i64) -> Result<u64>;

    /// 统计某用户的粉丝数
    async fn count_followers(&self, user_id: i64) -> Result<u64>;

    /// 查询某用户的全部粉丝 id（推模式扇出使用）
    async fn find_follower_ids(&self, user_id: i64) -> Result<Vec<i64>>;

    /// 查询某用户的全部关注对象 id（读扩散判断大 V 使用）
    async fn find_following_ids(&self, user_id: i64) -> Result<Vec<i64>>;

    /// 批量统计多个用户的粉丝数（id -> 粉丝数）
    async fn count_followers_bulk(
        &self,
        user_ids: &[i64],
    ) -> Result<std::collections::HashMap<i64, u64>>;
}
