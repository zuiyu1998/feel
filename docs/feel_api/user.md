# User API — 用户接口

## 概述

用户相关 HTTP 接口定义在 `cmd/feel_api/src/user/mod.rs` 中，通过 Poem 框架提供 RESTful 服务。

所有接口均对外暴露为 `POST` 方法，挂载在 `/api/v1/user/` 路径下。

---

## POST `/api/v1/user/register` — 注册用户

### 当前状态

**Handler 为占位实现**（空函数体），尚未接入 `AppState` 和下层存储逻辑。

```rust
#[handler]
async fn register() {}
```

### 设计意图

完整的 register handler 应实现以下流程：

```
HTTP POST /api/v1/user/register  (Json<RegisterRequest>)
  → register handler
    → 校验请求参数
    → 将 RegisterRequest 转换为 UserRegister（领域模型）
    → AppState.user_database.register(&user_register)    [UserDataBase trait]
      → CommonUserDataBase.register()                     [透传代理]
        → SeaOrmUserRepo.register()                       [UserRepo trait — 真正实现]
          ├─ 1. 开启数据库事务
          ├─ 2. 插入 users 表（自动生成 uid）
          ├─ 3. Argon2 哈希密码
          ├─ 4. 插入 user_credentials 表
          ├─ 5. 提交事务
          └─ 6. 返回 UserBase
    → 将 UserBase 转换为 RegisterResponse（API DTO）
    → 用 ok() 包装为 ApiResponse<RegisterResponse> 返回
```

### 请求类型 — `RegisterRequest`

定义在 `cmd/feel_api/src/model/user.rs`。

```rust
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub avatar: String,
    pub credential_type: String,
    pub credential_name: String,
    pub data: String,            // 明文密码
}
```

| 字段              | 类型     | 说明                          |
|-------------------|----------|-------------------------------|
| `name`            | `String` | 用户昵称                      |
| `avatar`          | `String` | 头像 URL                      |
| `credential_type` | `String` | 凭证类型（如 `"password"`）  |
| `credential_name` | `String` | 凭证名称（如 `"default"`）   |
| `data`            | `String` | 明文密码                      |

> **与领域模型的关系：** `RegisterRequest` 与 `feel_entity::user::UserRegister` 字段一致，
> 是 API 层专用的 DTO。后续可在 DTO 上添加校验注解而不影响领域层。
> 参见 [model/user.md](model/user.md)。

### 响应类型 — `ApiResponse<RegisterResponse>`

所有 API 接口通过统一的 `ApiResponse<T>` 泛型包装返回。

```rust
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}
```

其中 data 字段使用 `RegisterResponse`：

```rust
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: i64,
    pub uid: String,
    pub name: String,
    pub avatar: String,
    pub slogan: String,
    pub enabled: bool,
    pub created_at: String,   // ISO 8601 字符串
    pub updated_at: String,   // ISO 8601 字符串
}
```

| 字段         | 类型              | 说明                              |
|--------------|-------------------|-----------------------------------|
| `code`       | `i32`             | 业务状态码（0 表示成功）          |
| `message`    | `String`          | 提示消息                          |
| `data.id`    | `i64`             | 自增主键                          |
| `data.uid`   | `String`          | 全局唯一标识符（业务键）           |
| `data.name`  | `String`          | 用户昵称                          |
| `data.avatar`| `String`          | 头像 URL                          |
| `data.slogan`| `String`          | 用户签名                          |
| `data.enabled`| `bool`           | 是否启用                          |
| `data.created_at`| `String`     | 创建时间（ISO 8601）              |
| `data.updated_at`| `String`     | 更新时间（ISO 8601）              |

**JSON 示例：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 1,
        "uid": "uid_abc123",
        "name": "张三",
        "avatar": "https://example.com/avatar.png",
        "slogan": "",
        "enabled": true,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T03:00:00Z"
    }
}
```

> **与领域模型的差异：** `RegisterResponse` 相比 `feel_entity::user::UserBase`，
> 时间字段使用 `String` 而非 `DateTime<Utc>`，将时区格式化交给序列化层。
> 参见 [model/user.md](model/user.md) 和 [model/response.md](model/response.md)。

### 实现细节

#### 下层调用链

```
register handler (占位)
  ↓ UserDataBase::register(&UserRegister)
CommonUserDataBase
  ↓ 直接委托
SeaOrmUserRepo::register()
```

**SeaOrmUserRepo::register()** 位于 `crates/feel_storage/src/repo/user.rs:26-53`，具体步骤：

##### 步骤 1：开启事务

```rust
let txn = self.conn.begin().await.map_err(Error::from)?;
```

##### 步骤 2：插入 users 表

使用 `UserActiveModel` 构建插入数据，`uid` 由 `UserId::generate()` 生成：

```rust
use feel_entity::user::UserId;

let uid = UserId::generate();
```

| 模型字段        | 值来源                         |
|----------------|--------------------------------|
| `name`         | `register.name`                |
| `avatar`       | `register.avatar`              |
| `uid`          | `UserId::generate()`           |
| `created_at`   | `Utc::now()`                   |
| `updated_at`   | `Utc::now()`                   |

插入后返回 `UserModel`。

##### 步骤 3：Argon2 密码哈希

位于 `crates/feel_storage/src/utils.rs:6-16`：

```rust
pub fn get_passwod_hash(password: &str) -> (String, String) {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .expect("Hash password failed.")
        .to_string();
    (salt.as_str().to_string(), password_hash)
}
```

- 算法：**Argon2**（通过 `argon2` crate）
- 返回：`(salt, password_hash)`
- 输入：`register.data`（明文密码）

##### 步骤 4：插入 user_credentials 表

使用 `UserCredentialsActiveModel`：

| 模型字段           | 值来源                              |
|-------------------|------------------------------------|
| `user_uid`        | 步骤 2 生成的 `uid`                 |
| `credential_type` | `register.credential_type`         |
| `credential_name` | `register.credential_name`         |
| `encrypted_data`  | `password_hash`（Argon2 输出）       |
| `encryption_key`  | `salt`（Argon2 的盐值）              |
| `enabled`         | `true`                             |

##### 步骤 5：提交事务

```rust
txn.commit().await.map_err(Error::from)?;
```

##### 步骤 6：类型转换

`UserModel` 通过 `From<Model> for UserBase` 转换为 `UserBase` 返回（定义在 `crates/feel_sea_orm/src/user/entities/user.rs:34-47`）：

```rust
impl From<Model> for UserBase {
    fn from(model: Model) -> Self {
        Self {
            id: model.id,
            uid: model.uid,
            name: model.name,
            avatar: model.avatar,
            slogan: model.slogan,
            enabled: model.is_enable,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
```

### 预期的 Handler 实现

```rust
use feel_api::model::{
    response::ok,
    user::{RegisterRequest, RegisterResponse},
};
use feel_entity::user::UserRegister;
use poem::{Data, handler, post, web::Json};

#[handler]
async fn register(
    state: Data<&AppState>,
    body: Json<RegisterRequest>,
) -> Json<ApiResponse<RegisterResponse>> {
    // 1. DTO → 领域模型转换
    let user_register = UserRegister {
        name: body.0.name,
        avatar: body.0.avatar,
        credential_type: body.0.credential_type,
        credential_name: body.0.credential_name,
        data: body.0.data,
    };

    // 2. 调用领域层
    let user_base = match state.user_database.register(&user_register).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 3. 领域模型 → DTO 转换 + 统一响应包装
    let response = RegisterResponse {
        id: user_base.id,
        uid: user_base.uid,
        name: user_base.name,
        avatar: user_base.avatar,
        slogan: user_base.slogan,
        enabled: user_base.enabled,
        created_at: user_base.created_at.to_rfc3339(),
        updated_at: user_base.updated_at.to_rfc3339(),
    };

    ok(response)
}
```

> 三层数据转换：`Json<RegisterRequest>`（HTTP DTO）→ `UserRegister`（领域模型）→ `UserBase`（领域结果）→ `RegisterResponse`（API DTO）→ `Json<ApiResponse<RegisterResponse>>`（HTTP 响应）。

### 涉及的 crate 依赖

| crate / 模块                | 作用                                      |
|-----------------------------|-------------------------------------------|
| `feel_api::model`           | API DTO（`RegisterRequest`、`RegisterResponse`、`ApiResponse`）|
| `feel_entity`               | 领域模型（`UserRegister`、`UserBase`）    |
| `feel_storage`              | 提供 `UserDataBase` trait 及其实现        |
| `feel_sea_orm`              | Sea-ORM 实体（`users`、`user_credentials` 表） |
| `argon2`                    | 密码哈希（通过 `feel_storage` 依赖）      |

---

## POST `/api/v1/user/unregister` — 注销用户

### 当前状态

**Handler 为占位实现**（空函数体），尚未接入 `AppState` 和下层存储逻辑。

```rust
#[handler]
async fn unregister() {}
```

### 设计意图

完整的 unregister handler 应实现以下流程：

```
HTTP POST /api/v1/user/unregister/:user_id
  → unregister handler
    → 从路径提取 user_id
    → AppState.user_database.unregister(user_id)    [UserDataBase trait]
      → CommonUserDataBase.unregister()              [透传代理]
        → SeaOrmUserRepo.unregister()                [UserRepo trait — 真正实现]
          ├─ 1. 按主键查找 users 表
          ├─ 2. 若不存在返回 RecordNotFound 错误
          ├─ 3. 将 is_delete 置为 true（软删除）
          ├─ 4. 更新 updated_at
          └─ 5. 返回 UserBase
    → 将 UserBase 转换为 RegisterResponse（API DTO）
    → 用 ok() 包装为 ApiResponse<RegisterResponse> 返回
```

### 请求参数

当前路由 `POST /api/v1/user/unregister` 使用 POST 方法，参数传递方式待定。

**方案一：路径参数（推荐）**

路由改为 `/api/v1/user/unregister/:user_id`，在 Poem 中通过 `Path<i64>` 提取：

```rust
.at("/unregister/:user_id", post(unregister))
```

```rust
use poem::web::Path;

#[handler]
async fn unregister(
    state: Data<&AppState>,
    Path(user_id): Path<i64>,
) -> Json<ApiResponse<RegisterResponse>> {
    // ...
}
```

**方案二：请求体 DTO**

在 `cmd/feel_api/src/model/user.rs` 中定义 `UnregisterRequest`：

```rust
#[derive(Debug, Deserialize)]
pub struct UnregisterRequest {
    pub user_id: i64,
}
```

### 响应类型 — `ApiResponse<RegisterResponse>`

注销成功后返回被注销的用户信息，响应格式与注册一致：

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 1,
        "uid": "uid_abc123",
        "name": "张三",
        "avatar": "https://example.com/avatar.png",
        "slogan": "",
        "enabled": false,
        "created_at": "2026-07-11T03:00:00Z",
        "updated_at": "2026-07-11T04:00:00Z"
    }
}
```

> 注销后 `data.enabled` 字段仍为注销前的状态（由 `UserBase.enabled` 决定），软删除标记 `is_delete` 对 API 层透明。

### 下层调用链

```
unregister handler (占位)
  ↓ UserDataBase::unregister(user_id)
CommonUserDataBase
  ↓ 直接委托
SeaOrmUserRepo::unregister()
```

**SeaOrmUserRepo::unregister()** 位于 `crates/feel_storage/src/repo/user.rs:55-70`，具体步骤：

#### 步骤 1：查找用户

```rust
let user = UserEntity::find_by_id(user_id)
    .one(&self.conn)
    .await?
    .ok_or_else(|| sea_orm::DbErr::RecordNotFound(
        format!("User {} not found", user_id)
    ))?;
```

- 使用 Sea-ORM 的 `find_by_id` 按主键查找
- 若用户不存在返回 `RecordNotFound` 错误（数据库层）

#### 步骤 2：软删除

```rust
let mut user_active = user.into_active_model();
user_active.is_delete = Set(true);
user_active.updated_at = Set(now.to_utc());
```

| 模型字段        | 值                  | 说明                           |
|----------------|---------------------|--------------------------------|
| `is_delete`    | `true`              | 软删除标记（数据保留在库中）    |
| `updated_at`   | `Utc::now()`        | 更新时间戳                     |

#### 步骤 3：执行更新

```rust
let updated_user: UserModel = user_active.update(&self.conn).await?;
Ok(updated_user.into())
```

- 执行 `UPDATE` 语句更新 `is_delete` 和 `updated_at`
- 通过 `From<Model> for UserBase` 将 ORM 模型转换为领域模型返回

### 预期的 Handler 实现（路径参数方案）

```rust
use feel_api::model::response::{from_storage_error, ok};
use feel_api::model::user::RegisterResponse;
use feel_entity::user::UserBase;
use poem::web::{Data, Json, Path};

#[handler]
async fn unregister(
    state: Data<&AppState>,
    Path(user_id): Path<i64>,
) -> Json<ApiResponse<RegisterResponse>> {
    // 1. 调用领域层
    let user_base = match state.user_database.unregister(user_id).await {
        Ok(user) => user,
        Err(e) => return from_storage_error(e),
    };

    // 2. 领域模型 → DTO 转换 + 统一响应包装
    let response = RegisterResponse {
        id: user_base.id,
        uid: user_base.uid,
        name: user_base.name,
        avatar: user_base.avatar,
        slogan: user_base.slogan,
        enabled: user_base.enabled,
        created_at: user_base.created_at.to_rfc3339(),
        updated_at: user_base.updated_at.to_rfc3339(),
    };

    ok(response)
}
```

### 涉及的 crate 依赖

| crate / 模块      | 作用                               |
|-------------------|------------------------------------|
| `feel_api::model` | API DTO（`RegisterResponse`、`ApiResponse`）|
| `feel_entity`     | 领域模型（`UserBase`）             |
| `feel_storage`    | 提供 `UserDataBase` trait 及其实现 |
| `feel_sea_orm`    | Sea-ORM 实体（`users` 表）         |

---

## POST `/api/v1/user/login` — 用户登录

### 当前状态

占位（Handler 为空）。

```rust
#[handler]
async fn login() {}
```

### 下层 trait 签名

```rust
fn login(&self, login: &UserLogin) -> Result<String>;
```

- `UserLogin` 定义位于 `crates/feel_entity/src/user/models/mod.rs`，当前为**空结构体**，尚未填充字段。
- 返回 `String`，预期为登录凭证（如 token）。

---

## POST `/api/v1/user/logout` — 用户登出

### 当前状态

Handler 为空桩，尚未实现具体逻辑。

```rust
#[handler]
async fn logout() {}
```

### 下层 trait 签名

```rust
fn logout(&self, user_id: u32) -> Result<()>;
```

---

## Handler 注册方式

```rust
// cmd/feel_api/src/user/mod.rs
pub fn router() -> Route {
    Route::new()
        .at("/register", post(register))
        .at("/unregister", post(unregister))
        .at("/login", post(login))
        .at("/logout", post(logout))
}
```

## 路由挂载

```rust
// cmd/feel_api/src/lib.rs
pub fn app_route() -> Route {
    Route::new().nest("/user", user::router())
}
```

最终完整路径为 `/api/v1/user/{action}`。
