mod label;
mod user;

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
