# Label API — 标签接口

## 概述

标签相关 HTTP 接口定义在 `cmd/feel_api/src/label/mod.rs` 中，通过 Poem 框架提供 RESTful 服务。

接口覆盖标签的两类对象操作：

| 对象 | 操作 |
| --- | --- |
| 标签本体 `LabelBase` | 创建、修改备注、查询 |
| 用户标签关联 `UserLabel` | 查看用户的标签、查看标签下的用户、添加关联、解除关联 |

设计依据：

- 特性设计：`docs/features/label.md`（核心交互）
- 数据访问层：`feel_storage::database::LabelDataBase` / `CommonLabelDataBase`（已实现，含数量上限校验）
- 持久化层：`feel_storage::repo::LabelRepo` / `SeaOrmLabelRepo`（已实现）

> **参数传递：** label 接口不使用 URL 路径（`Path`）与查询串（`Query`），所有参数均通过 `Json` 请求体传递；`user_id`/`label_id` 使用 `i64` 数据库主键（`docs/features/label.md` 核心交互中的 `{uid}` 为旧表述）。各操作使用独立子路径区分（`create` / `list` / `users` / `add` / `update` / `remove`）。

## 接口一览

| 方法 | 路径 | Handler | 描述 | 请求体 | 认证 |
| --- | --- | --- | --- | --- | --- |
| POST | `/api/v1/labels/create` | `create_label` | 创建标签本体 | `CreateLabelRequest` | ✅ Bearer |
| GET | `/api/v1/labels/list` | `list_user_labels` | 查看用户标签 | `ListUserLabelsRequest` | ❌ 公开 |
| GET | `/api/v1/labels/users` | `list_label_users` | 查看标签下的用户 | `ListLabelUsersRequest` | ❌ 公开 |
| POST | `/api/v1/labels/add` | `add_label` | 为用户添加标签 | `AddLabelRequest` | ✅ Bearer |
| PUT | `/api/v1/labels/update` | `update_label` | 修改标签备注 | `UpdateLabelRequest` | ✅ Bearer |
| DELETE | `/api/v1/labels/remove` | `remove_label` | 解除关联 | `RemoveLabelRequest` | ✅ Bearer |

---

## POST `/api/v1/labels/create` — 创建标签本体

### 当前状态

✅ 已实现。

### 设计意图

创建标签本体（`LabelBase`）。名称全局唯一，重复创建由唯一约束拒绝；`id`、`enabled`（默认 `true`）、时间戳由存储层管理。创建后的标签本体可供任意用户关联使用。

```
HTTP POST /api/v1/labels/create  (Authorization: Bearer <token>; Json<CreateLabelRequest>)
  → auth_middleware (JWT 验证)
  → create_label handler
    → CreateLabelRequest → LabelCreate（领域模型）
    → AppState.label_database.create_label(&LabelCreate)   [LabelDataBase]
      → CommonLabelDataBase.create_label()
        → SeaOrmLabelRepo.create_label()                  [LabelRepo — 写入 label 表]
    → LabelBase → LabelResponse（API DTO）
    → 用 ok() 包装为 ApiResponse<LabelResponse> 返回
```

### 请求类型 — `Json<CreateLabelRequest>`

定义在 `cmd/feel_api/src/model/label.rs`。

```rust
#[derive(Debug, Deserialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
}
```

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `name` | `String` | 标签名称（全局唯一，必填） |
| `description` | `String` | 公共描述 |
| `remark` | `String` | 标签备注（所有用户共享） |
| `influence` | `i64` | 标签影响力，创建时确立 |

**JSON 示例：**

```json
{
    "name": "Rust 开发者",
    "description": "使用 Rust 进行开发的人",
    "remark": "",
    "influence": 50
}
```

### 响应类型 — `ApiResponse<LabelResponse>`

```rust
#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: String,   // ISO 8601 字符串
    pub updated_at: String,   // ISO 8601 字符串
}
```

**JSON 示例：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 1,
        "name": "Rust 开发者",
        "description": "使用 Rust 进行开发的人",
        "remark": "",
        "influence": 50,
        "enabled": true,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T03:00:00Z"
    }
}
```

### 错误码映射

| 错误类型 | 业务码 | 说明 |
| --- | --- | --- |
| `StorageError::Db(_)` | `10003` | 数据库异常（含名称唯一约束冲突） |
| `StorageError::Authentication(_)` | `10005` | token 缺失或无效 |

### 下层调用链

```
create_label handler (已实现)
  ↓ LabelDataBase::create_label(&LabelCreate)
CommonLabelDataBase
  ↓ 委托
SeaOrmLabelRepo::create_label()
```

---

## GET `/api/v1/labels/list` — 查看用户标签

### 当前状态

✅ 已实现。

### 设计意图

任何用户（含未登录）可查看某用户的公开标签集合，这是"认识用户"的入口。返回该用户按创建时间排序的关联列表，每个关联内嵌标签本体信息（含备注）。

```
HTTP GET /api/v1/labels/list  (Json<ListUserLabelsRequest>)
  → list_user_labels handler
    → 从请求体取 user_id
    → AppState.label_database.get_user_labels(user_id)   [LabelDataBase]
      → CommonLabelDataBase.get_user_labels()
        → SeaOrmLabelRepo.find_user_labels(user_id)      [LabelRepo — 按创建时间排序]
    → 对每个 UserLabel 二次查询 get_label(label_id) 组装 LabelBrief
    → 用 ok() 包装为 ApiResponse<Vec<UserLabelItem>> 返回
```

### 请求类型 — `Json<ListUserLabelsRequest>`

```rust
#[derive(Debug, Deserialize)]
pub struct ListUserLabelsRequest {
    pub user_id: i64,       // 查看哪个用户的标签
}
```

**JSON 示例：**

```json
{
    "user_id": 1
}
```

### 响应类型 — `ApiResponse<Vec<UserLabelItem>>`

```rust
#[derive(Debug, Serialize)]
pub struct UserLabelItem {
    pub id: i64,                      // UserLabel 关联 ID
    pub label: LabelBrief,            // 内嵌标签本体信息
}

#[derive(Debug, Serialize)]
pub struct LabelBrief {
    pub id: i64,
    pub name: String,
    pub remark: String,               // 标签备注（所有用户共享）
}
```

**JSON 示例：**

```json
{
    "code": 0,
    "message": "ok",
    "data": [
        {
            "id": 10,
            "label": { "id": 1, "name": "Rust 开发者", "remark": "5 年 Rust 后端开发经验" }
        }
    ]
}
```

### 下层调用链

```
list_user_labels handler (已实现)
  ↓ LabelDataBase::get_user_labels(user_id)
CommonLabelDataBase
  ↓ 委托
SeaOrmLabelRepo::find_user_labels(user_id)
  ↓ 二次查询
LabelDataBase::get_label(label_id) → LabelBase → LabelBrief
```

---

## GET `/api/v1/labels/users` — 查看标签下的用户

### 当前状态

✅ 已实现。

### 设计意图

任何用户（含未登录）可查看某个标签下关联的所有用户，这是"按标签认识一群人"的入口（与"查看用户标签"互为反向）。

```
HTTP GET /api/v1/labels/users  (Json<ListLabelUsersRequest>)
  → list_label_users handler
    → 从请求体取 label_id
    → AppState.label_database.get_users_by_label(label_id)   [LabelDataBase]
      → SeaOrmLabelRepo.find_users_by_label(label_id)
    → 对每个 UserLabel 按 user_id 查询用户信息（name / avatar）组装 UserBrief
    → 用 ok() 包装为 ApiResponse<Vec<LabelUserItem>> 返回
```

### 请求类型 — `Json<ListLabelUsersRequest>`

```rust
#[derive(Debug, Deserialize)]
pub struct ListLabelUsersRequest {
    pub label_id: i64,      // 查看哪个标签下的用户
}
```

**JSON 示例：**

```json
{
    "label_id": 1
}
```

### 响应类型 — `ApiResponse<Vec<LabelUserItem>>`

```rust
#[derive(Debug, Serialize)]
pub struct LabelUserItem {
    pub user: UserBrief,              // 内嵌用户信息
}

#[derive(Debug, Serialize)]
pub struct UserBrief {
    pub user_id: i64,
    pub name: String,
    pub avatar: String,
}
```

**JSON 示例：**

```json
{
    "code": 0,
    "message": "ok",
    "data": [
        {
            "user": { "user_id": 1, "name": "张三", "avatar": "https://example.com/avatar.png" }
        }
    ]
}
```

> **用户信息组装：** `LabelDataBase` 返回的是关联记录（`UserLabel`），用户姓名/头像通过 `UserDataBase::get_user_by_id(user_id)`（新增方法，委托 `UserRepo::find_by_id`）二次查询后组装进 `UserBrief`。

### 下层调用链

```
list_label_users handler (已实现)
  ↓ LabelDataBase::get_users_by_label(label_id)
CommonLabelDataBase
  ↓ 委托
SeaOrmLabelRepo::find_users_by_label(label_id)
  ↓ 二次查询 UserDataBase::get_user_by_id(user_id)
用户信息 → UserBrief
```

---

## POST `/api/v1/labels/add` — 为用户添加标签

### 当前状态

✅ 已实现。

### 设计意图

已登录用户为画像添加一个标签（携带关联）。若标签本体不存在，可先按名称创建本体再关联。

```
HTTP POST /api/v1/labels/add  (Authorization: Bearer <token>; Json<AddLabelRequest>)
  → auth_middleware (JWT 验证,注入 AuthUser)
  → add_label handler
    → 校验请求体 user_id 为当前登录用户(仅能管理自己的关联)
    → AppState.label_database.add_label(user_id, label_id)   [LabelDataBase]
      → CommonLabelDataBase.add_label()
        ├─ 1. count_user_labels(user_id)          [数量上限校验,>= 20 返回 Business 错误]
        └─ 2. create_user_label(UserLabelCreate)  → UserLabel
    → 将 UserLabel 转换为 UserLabelResponse（API DTO）
    → 用 ok() 包装为 ApiResponse<UserLabelResponse> 返回
```

### 请求类型 — `Json<AddLabelRequest>`

定义在 `cmd/feel_api/src/model/label.rs`。

```rust
#[derive(Debug, Deserialize)]
pub struct AddLabelRequest {
    pub user_id: i64,       // 归属用户(需与登录用户一致)
    pub label_id: i64,      // 要关联的标签本体 ID
}
```

**JSON 示例：**

```json
{
    "user_id": 1,
    "label_id": 1
}
```

> 若标签本体不存在，可先调用 `POST /api/v1/labels/create` 创建标签本体，再添加关联。

### 响应类型 — `ApiResponse<UserLabelResponse>`

```rust
#[derive(Debug, Serialize)]
pub struct UserLabelResponse {
    pub id: i64,
    pub user_id: i64,
    pub label_id: i64,
    pub enabled: bool,
    pub created_at: String,   // ISO 8601 字符串
    pub updated_at: String,   // ISO 8601 字符串
}
```

**JSON 示例（成功）：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 10,
        "user_id": 1,
        "label_id": 1,
        "enabled": true,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T03:00:00Z"
    }
}
```

**JSON 示例（数量超限）：**

```json
{
    "code": 10006,
    "message": "Business error: User 1 already has 20 labels, limit is 20",
    "data": null
}
```

### 错误码映射

| 错误类型 | 业务码 | 说明 |
| --- | --- | --- |
| `StorageError::Business(_)` | `10006` | 标签关联数量达到上限 20 |
| `StorageError::Db(DbErr::RecordNotFound)` | `10004` | 标签本体不存在 |
| `StorageError::Db(_)` | `10003` | 数据库异常（含唯一约束冲突） |
| `StorageError::Authentication(_)` | `10005` | token 缺失或无效 |

### 下层调用链

```
add_label handler (已实现)
  ↓ LabelDataBase::add_label(user_id, label_id)
CommonLabelDataBase::add_label()
  ├─ count_user_labels(user_id)   → 超限返回 Error::Business
  └─ create_user_label(&UserLabelCreate)  → UserLabel
      ↓
SeaOrmLabelRepo::create_user_label()
```

---

## PUT `/api/v1/labels/update` — 修改标签备注

### 当前状态

✅ 已实现。

### 设计意图

更新标签本体的备注（备注位于 `LabelBase`，对所有用户共享），需具备标签维护权限。

```
HTTP PUT /api/v1/labels/update  (Authorization: Bearer <token>; Json<UpdateLabelRequest>)
  → auth_middleware (JWT 验证)
  → update_label handler
    → 按请求体 label_id 获取现有 LabelBase(get_label),将 remark 合并进 LabelUpdate
    → AppState.label_database.update_label(&LabelUpdate)   [LabelDataBase]
      → SeaOrmLabelRepo.update_label()
    → 返回更新后的 LabelResponse
```

### 请求类型 — `Json<UpdateLabelRequest>`

```rust
#[derive(Debug, Deserialize)]
pub struct UpdateLabelRequest {
    pub label_id: i64,      // 待修改的标签本体 ID
    pub remark: String,     // 新的标签备注
}
```

**JSON 示例：**

```json
{
    "label_id": 1,
    "remark": "跑过 5 个全马"
}
```

### 响应类型 — `ApiResponse<LabelResponse>`

```rust
#[derive(Debug, Serialize)]
pub struct LabelResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub remark: String,
    pub influence: i64,
    pub enabled: bool,
    pub created_at: String,   // ISO 8601 字符串
    pub updated_at: String,   // ISO 8601 字符串
}
```

**JSON 示例：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 1,
        "name": "Rust 开发者",
        "description": "使用 Rust 进行开发的人",
        "remark": "跑过 5 个全马",
        "influence": 100,
        "enabled": true,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T04:00:00Z"
    }
}
```

### 错误码映射

| 错误类型 | 业务码 | 说明 |
| --- | --- | --- |
| `StorageError::Db(DbErr::RecordNotFound)` | `10004` | 标签本体不存在 |
| `StorageError::Db(_)` | `10003` | 数据库异常（如名称唯一约束冲突） |
| `StorageError::Authentication(_)` | `10005` | token 缺失或无效 |

### 下层调用链

```
update_label handler (已实现)
  ↓ LabelDataBase::get_label(label_id) + update_label(&LabelUpdate)
CommonLabelDataBase
  ↓ 委托
SeaOrmLabelRepo::update_label()
```

---

## DELETE `/api/v1/labels/remove` — 解除关联

### 当前状态

✅ 已实现。

### 设计意图

用户移除自己画像上的某个标签（仅解除关联，不删除公有标签本体）。

```
HTTP DELETE /api/v1/labels/remove  (Authorization: Bearer <token>; Json<RemoveLabelRequest>)
  → auth_middleware (JWT 验证)
  → remove_label handler
    → 校验请求体 user_id 为当前登录用户
    → AppState.label_database.remove_label(user_id, label_id)   [LabelDataBase]
      → SeaOrmLabelRepo.delete_user_label_by_ids(user_id, label_id)
    → 用 ok() 包装为 ApiResponse<()> 返回
```

### 请求类型 — `Json<RemoveLabelRequest>`

```rust
#[derive(Debug, Deserialize)]
pub struct RemoveLabelRequest {
    pub user_id: i64,       // 归属用户(需与登录用户一致)
    pub label_id: i64,      // 要解除的标签本体 ID
}
```

**JSON 示例：**

```json
{
    "user_id": 1,
    "label_id": 1
}
```

### 响应

```json
{
    "code": 0,
    "message": "ok",
    "data": null
}
```

### 下层调用链

```
remove_label handler (已实现)
  ↓ LabelDataBase::remove_label(user_id, label_id)
CommonLabelDataBase
  ↓ 委托
SeaOrmLabelRepo::delete_user_label_by_ids(user_id, label_id)
```

---

## Handler 注册方式

```rust
// cmd/feel_api/src/label/mod.rs
pub fn router() -> Route {
    Route::new()
        .at("/list", get(list_user_labels))
        .at("/users", get(list_label_users))
        .at("/add", post(add_label).around(auth_middleware))
        .at("/update", put(update_label).around(auth_middleware))
        .at("/remove", delete(remove_label).around(auth_middleware))
}
```

## 路由挂载与 AppState

```rust
// cmd/feel_api/src/lib.rs
pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
    pub label_database: Arc<dyn LabelDataBase>,   // 新增
}

pub fn app_route() -> Route {
    Route::new()
        .nest("/users", user::router())
        .nest("/labels", label::router())
}
```

最终完整路径为 `/api/v1/...`（由 `main.rs` 的 `.nest("/api/v1", app_route())` 提供）。

### 认证路由说明

- 读取接口（查看用户标签、查看标签下的用户）公开，无需认证
- 写操作（创建标签本体、添加标签、修改备注、解除关联）使用 `.around(auth_middleware)` 保护
- 按业务规则"用户只能管理自己的关联"，handler 需校验请求体 `user_id` 与 `AuthUser` 对应（或具备标签维护权限）

---

## 涉及的 crate 依赖

| crate / 模块 | 作用 |
| --- | --- |
| `feel_api::model` | API DTO（`AddLabelRequest`、`LabelResponse`、`UserLabelItem` 等，规划于 `model/label.rs`） |
| `feel_api::auth` | 认证中间件（`AuthUser`、`auth_middleware`） |
| `feel_entity` | 领域模型（`LabelBase`、`UserLabel`、`LabelCreate`、`LabelUpdate`、`UserLabelCreate`） |
| `feel_storage` | 提供 `LabelDataBase` trait 及其实现（`CommonLabelDataBase`） |
| `feel_sea_orm` | Sea-ORM 实体（`label`、`user_label` 表） |

---

## 与特性设计的对应

| `docs/features/label.md` 核心交互 | 本接口 |
| --- | --- |
| 创建标签本体（添加标签中的"按名称创建本体"部分） | `POST /api/v1/labels/create` |
| 查看用户标签 | `GET /api/v1/labels/list`（body: `user_id`） |
| 查看标签下的用户 | `GET /api/v1/labels/users`（body: `label_id`） |
| 添加标签 | `POST /api/v1/labels/add` |
| 修改标签备注 | `PUT /api/v1/labels/update` |
| 解除关联 | `DELETE /api/v1/labels/remove` |
