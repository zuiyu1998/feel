//! 标签相关 HTTP 接口。
//!
//! 参数传递不使用 URL 路径（Path）与查询串（Query）：所有接口均通过
//! `Json` 请求体携带参数。

use crate::AppState;
use crate::auth::{AuthUser, auth_middleware};
use crate::model::label::{
    AddLabelRequest, CreateLabelRequest, LabelBrief, LabelResponse, LabelUserItem,
    ListLabelUsersRequest, ListUserLabelsRequest, RemoveLabelRequest, UpdateLabelRequest,
    UserBrief, UserLabelItem, UserLabelResponse,
};
use crate::model::response::{ApiResponse, from_storage_error, ok};
use feel_entity::label::{LabelCreate, LabelUpdate};
use poem::web::{Data, Json};
use poem::{EndpointExt, Route, delete, get, handler, post, put};

/// 标签相关路由：/api/v1/labels/...
/// 各操作使用独立子路径区分：create / all / list / users / add / update / remove。
pub fn router() -> Route {
    Route::new()
        .at("/create", post(create_label).around(auth_middleware))
        .at("/all", get(list_all_labels))
        .at("/list", get(list_user_labels))
        .at("/users", get(list_label_users))
        .at("/add", post(add_label))
        .at("/update", put(update_label).around(auth_middleware))
        .at("/remove", delete(remove_label).around(auth_middleware))
}

// -- GET /labels/all — 获取所有标签 --

#[handler]
async fn list_all_labels(state: Data<&AppState>) -> Json<ApiResponse<Vec<LabelResponse>>> {
    // 1. 调用领域层（按创建时间排序）
    let labels = match state.label_database.get_all_labels().await {
        Ok(list) => list,
        Err(e) => return from_storage_error(e),
    };

    // 2. 领域模型 → DTO + 统一响应包装
    ok(labels.into_iter().map(Into::into).collect())
}

// -- POST /labels/create — 创建标签本体 --

#[handler]
async fn create_label(
    state: Data<&AppState>,
    body: Json<CreateLabelRequest>,
) -> Json<ApiResponse<LabelResponse>> {
    // 注：标签创建权限（角色/管理员）待后续补充，当前仅要求登录认证

    // 1. DTO → 领域模型转换
    let create = LabelCreate {
        name: body.0.name,
        description: body.0.description,
        remark: body.0.remark,
        influence: body.0.influence,
    };

    // 2. 调用领域层（名称唯一约束由存储层保证）
    let label = match state.label_database.create_label(&create).await {
        Ok(l) => l,
        Err(e) => return from_storage_error(e),
    };

    // 3. 领域模型 → DTO + 统一响应包装
    ok(label.into())
}

// -- GET /labels — 查看用户标签 --

#[handler]
async fn list_user_labels(
    state: Data<&AppState>,
    body: Json<ListUserLabelsRequest>,
) -> Json<ApiResponse<Vec<UserLabelItem>>> {
    // 1. 获取用户的标签关联（按创建时间排序）
    let user_labels = match state.label_database.get_user_labels(body.0.user_id).await {
        Ok(list) => list,
        Err(e) => return from_storage_error(e),
    };

    // 2. 逐个查询标签本体信息，组装列表
    let mut items = Vec::new();
    for ul in user_labels {
        let label = match state.label_database.get_label(ul.label_id).await {
            Ok(Some(l)) => l,
            Ok(None) => continue,
            Err(e) => return from_storage_error(e),
        };
        items.push(UserLabelItem {
            id: ul.id,
            label: LabelBrief {
                id: label.id,
                name: label.name,
                remark: label.remark,
            },
        });
    }

    ok(items)
}

// -- GET /labels/users — 查看标签下的用户 --

#[handler]
async fn list_label_users(
    state: Data<&AppState>,
    body: Json<ListLabelUsersRequest>,
) -> Json<ApiResponse<Vec<LabelUserItem>>> {
    // 1. 获取标签下的用户关联
    let user_labels = match state
        .label_database
        .get_users_by_label(body.0.label_id)
        .await
    {
        Ok(list) => list,
        Err(e) => return from_storage_error(e),
    };

    // 2. 按 user_id 查询用户信息，组装列表
    let mut items = Vec::new();
    for ul in user_labels {
        let user = match state.user_database.get_user_by_id(ul.user_id).await {
            Ok(Some(u)) => UserBrief {
                user_id: u.id,
                name: u.name,
                avatar: u.avatar,
            },
            Ok(None) => UserBrief {
                user_id: ul.user_id,
                name: String::new(),
                avatar: String::new(),
            },
            Err(e) => return from_storage_error(e),
        };
        items.push(LabelUserItem { user });
    }

    ok(items)
}

// -- POST /labels — 为用户添加标签 --

#[handler]
async fn add_label(
    state: Data<&AppState>,
    body: Json<AddLabelRequest>,
) -> Json<ApiResponse<UserLabelResponse>> {
    // 1. 将 user_uid（业务键）解析为 user_id（数据库主键）
    let user = match state.user_database.get_user(&body.0.user_uid).await {
        Ok(u) => u,
        Err(e) => return from_storage_error(e),
    };

    // 2. 调用领域层（内部校验数量上限 20）
    let user_label = match state
        .label_database
        .add_label(user.id, body.0.label_id)
        .await
    {
        Ok(ul) => ul,
        Err(e) => return from_storage_error(e),
    };

    // 3. 领域模型 → DTO + 统一响应包装
    ok(user_label.into())
}

// -- PUT /labels — 修改标签备注 --

#[handler]
async fn update_label(
    state: Data<&AppState>,
    body: Json<UpdateLabelRequest>,
) -> Json<ApiResponse<LabelResponse>> {
    // 注：标签维护权限（角色/管理员）待后续补充，当前仅要求登录认证

    // 1. 获取现有标签本体
    let label = match state.label_database.get_label(body.0.label_id).await {
        Ok(Some(l)) => l,
        Ok(None) => {
            return from_storage_error(feel_storage::Error::Db(sea_orm::DbErr::RecordNotFound(
                format!("Label {} not found", body.0.label_id),
            )));
        }
        Err(e) => return from_storage_error(e),
    };

    // 2. 构建 LabelUpdate（仅修改备注，其余字段保持原值）
    let update = LabelUpdate {
        id: label.id,
        name: label.name,
        description: label.description,
        remark: body.0.remark,
        influence: label.influence,
        enabled: label.enabled,
    };

    // 3. 调用领域层
    let updated = match state.label_database.update_label(&update).await {
        Ok(l) => l,
        Err(e) => return from_storage_error(e),
    };

    ok(updated.into())
}

// -- DELETE /labels — 解除关联 --

#[handler]
async fn remove_label(
    state: Data<&AppState>,
    req: &poem::Request,
    body: Json<RemoveLabelRequest>,
) -> Json<ApiResponse<()>> {
    // 1. 认证：仅能管理自己的关联
    let auth = req
        .extensions()
        .get::<AuthUser>()
        .expect("AuthUser not found — missing auth middleware");

    let me = match state.user_database.get_user(&auth.uid).await {
        Ok(u) => u,
        Err(e) => return from_storage_error(e),
    };
    if me.id != body.0.user_id {
        return from_storage_error(feel_storage::Error::Business(format!(
            "User {} cannot manage labels of user {}",
            me.id, body.0.user_id
        )));
    }

    // 2. 按 用户 + 标签 解除关联
    if let Err(e) = state
        .label_database
        .remove_label(body.0.user_id, body.0.label_id)
        .await
    {
        return from_storage_error(e);
    }

    ok(())
}
