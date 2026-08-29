use async_trait::async_trait;

use crate::{Result, repo::LabelRepo};
use feel_entity::prelude::*;

/// Maximum number of label associations a single user may have.
pub const MAX_USER_LABELS: u64 = 20;

pub struct CommonLabelDataBase {
    label_repo: Box<dyn LabelRepo>,
}

impl CommonLabelDataBase {
    pub fn new<T: LabelRepo>(label_repo: T) -> Self {
        CommonLabelDataBase {
            label_repo: Box::new(label_repo),
        }
    }
}

#[async_trait]
pub trait LabelDataBase: 'static + Send + Sync {
    /// 创建标签本体
    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase>;

    /// 更新标签本体(如修改备注、禁用/启用)
    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase>;

    /// 根据 ID 获取标签本体
    async fn get_label(&self, label_id: i64) -> Result<Option<LabelBase>>;

    /// 根据名称获取标签本体(名称全局唯一)
    async fn get_label_by_name(&self, name: &str) -> Result<Option<LabelBase>>;

    /// 获取某用户的全部标签关联(按创建时间排序)
    async fn get_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>>;

    /// 获取某标签下的全部用户关联
    async fn get_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>>;

    /// 统计某用户的关联数量(用于数量上限校验)
    async fn count_user_labels(&self, user_id: i64) -> Result<u64>;

    /// 为用户添加标签关联(同一用户对同一标签只能关联一次,单个用户上限 20 个)
    async fn add_label(&self, user_id: i64, label_id: i64) -> Result<UserLabel>;

    /// 解除用户标签关联(仅解除关联,不删除标签本体)
    async fn remove_label(&self, user_id: i64, label_id: i64) -> Result<()>;
}

#[async_trait]
impl LabelDataBase for CommonLabelDataBase {
    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase> {
        self.label_repo.create_label(create).await
    }

    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase> {
        self.label_repo.update_label(update).await
    }

    async fn get_label(&self, label_id: i64) -> Result<Option<LabelBase>> {
        self.label_repo.find_label_by_id(label_id).await
    }

    async fn get_label_by_name(&self, name: &str) -> Result<Option<LabelBase>> {
        self.label_repo.find_label_by_name(name).await
    }

    async fn get_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>> {
        self.label_repo.find_user_labels(user_id).await
    }

    async fn get_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>> {
        self.label_repo.find_users_by_label(label_id).await
    }

    async fn count_user_labels(&self, user_id: i64) -> Result<u64> {
        self.label_repo.count_user_labels(user_id).await
    }

    async fn add_label(&self, user_id: i64, label_id: i64) -> Result<UserLabel> {
        // 业务规则:单个用户的标签关联数量上限为 MAX_USER_LABELS 个
        let count = self.label_repo.count_user_labels(user_id).await?;
        if count >= MAX_USER_LABELS {
            return Err(crate::Error::Business(format!(
                "User {} already has {} labels, limit is {}",
                user_id, count, MAX_USER_LABELS
            )));
        }

        let create = UserLabelCreate { user_id, label_id };
        self.label_repo.create_user_label(&create).await
    }

    async fn remove_label(&self, user_id: i64, label_id: i64) -> Result<()> {
        self.label_repo
            .delete_user_label_by_ids(user_id, label_id)
            .await
    }
}
