//! 标签 API 的请求/响应 DTO。

use feel_entity::label::{LabelBase, UserLabel};
use serde::{Deserialize, Serialize};

// -- 请求 --

/// 查看用户标签请求体（GET /labels）
#[derive(Debug, Deserialize)]
pub struct ListUserLabelsRequest {
    pub user_id: i64,
}

/// 查看标签下的用户请求体（GET /labels/users）
#[derive(Debug, Deserialize)]
pub struct ListLabelUsersRequest {
    pub label_id: i64,
}

/// 添加标签关联请求体（POST /labels）
#[derive(Debug, Deserialize)]
pub struct AddLabelRequest {
    pub user_uid: String, // 归属用户（业务键,关联 UserBase.uid）
    pub label_id: i64,
}

/// 创建标签本体请求体（可选流程：标签不存在时先创建本体再关联）
#[derive(Debug, Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
}

/// 修改标签备注请求体（PUT /labels）
#[derive(Debug, Deserialize)]
pub struct UpdateLabelRequest {
    pub label_id: i64,
    pub remark: String,
}

/// 解除标签关联请求体（DELETE /labels）
#[derive(Debug, Deserialize)]
pub struct RemoveLabelRequest {
    pub user_id: i64,
    pub label_id: i64,
}

// -- 响应 --

/// 标签本体响应
#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<LabelBase> for LabelResponse {
    fn from(value: LabelBase) -> Self {
        LabelResponse {
            id: value.id,
            name: value.name,
            description: value.description,
            remark: value.remark,
            influence: value.influence,
            enabled: value.enabled,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

/// 用户标签关联响应（添加标签后返回）
#[derive(Debug, Serialize)]
pub struct UserLabelResponse {
    pub id: i64,
    pub user_id: i64,
    pub label_id: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserLabel> for UserLabelResponse {
    fn from(value: UserLabel) -> Self {
        UserLabelResponse {
            id: value.id,
            user_id: value.user_id,
            label_id: value.label_id,
            enabled: value.enabled,
            created_at: value.created_at.to_rfc3339(),
            updated_at: value.updated_at.to_rfc3339(),
        }
    }
}

/// 标签摘要（内嵌于"查看用户标签"列表项）
#[derive(Debug, Serialize)]
pub struct LabelBrief {
    pub id: i64,
    pub name: String,
    pub remark: String,
}

/// 查看用户标签列表项
#[derive(Debug, Serialize)]
pub struct UserLabelItem {
    pub id: i64,
    pub label: LabelBrief,
}

/// 用户摘要（内嵌于"查看标签下的用户"列表项）
#[derive(Debug, Serialize)]
pub struct UserBrief {
    pub user_id: i64,
    pub name: String,
    pub avatar: String,
}

/// 查看标签下的用户列表项
#[derive(Debug, Serialize)]
pub struct LabelUserItem {
    pub user: UserBrief,
}
