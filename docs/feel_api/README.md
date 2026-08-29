# feel_api — HTTP API 服务

## 概述

`feel_api` 是 feel 平台的 **HTTP API 服务**，基于 [Poem](https://github.com/poem-web/poem) 框架构建，位于 `cmd/feel_api/`。

负责提供用户注册、登录、注销等 RESTful 接口，通过 `feel_storage` 和 `feel_core` 与数据库交互。

## 模块位置

```
cmd/feel_api/src/
├── main.rs      # 入口：启动 HTTP 服务器
├── lib.rs       # 路由、AppState、ApiConfig
├── auth.rs      # JWT 认证中间件
├── model/       # API 专用 DTO（请求/响应数据模型）
│   ├── mod.rs
│   ├── user.rs      # 用户接口 DTO
│   └── response.rs  # 通用响应封装 ApiResponse<T>
└── user/
    └── mod.rs   # 用户相关接口
    （label/ 与 model/label.rs 为规划中,见 label.md）
```

## 运行方式

```bash
# 开发模式（默认连接 postgresql://postgres:bj123456@192.168.0.107:5432/feel）
cargo run -p feel_api

# 指定数据库连接
DATABASE_URL=postgresql://user:password@host:5432/feel cargo run -p feel_api
```

服务默认监听 `0.0.0.0:3000`。

## 配置（ApiConfig）

```rust
pub struct ApiConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
}
```

通过环境变量覆盖默认值：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `database_url` | `postgresql://postgres:bj123456@192.168.0.107:5432/feel` | PostgreSQL 连接 |
| `redis_url` | `redis://192.168.0.107:6379` | Redis 连接 |
| `jwt_secret` | `feel-jwt-secret` | JWT 签名密钥 |

## 应用状态（AppState）

```rust
pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
}
```

启动时初始化数据库连接池和 Redis 连接，创建 `CommonUserDataBase` 实例（含 `UserRepo` + `UserCache` + `jwt_secret`）注入到 Poem 的共享状态中，供各 handler 使用。

## 认证中间件（AuthMiddleware）

`auth.rs` 提供 JWT Bearer token 认证中间件 `auth_middleware`，配合 Poem 的 `EndpointExt::around` 使用：

```rust
.at("/info", get(info).around(auth_middleware))
```

中间件流程：
1. 从 `Authorization: Bearer <token>` 请求头提取 token
2. 调用 `UserDataBase::parse_token` 验证签名和过期
3. 将 `AuthUser { uid }` 注入请求扩展
4. 调用内部 handler

认证失败时返回 HTTP 401 + 统一 JSON 错误体：

```json
{
    "code": 10005,
    "message": "Missing or invalid Authorization header",
    "data": null
}
```

## API 路由

所有接口挂载在 `/api/v1` 前缀下：

| 方法 | 路径                                | Handler       | 描述             | 认证 | 状态 |
|------|-------------------------------------|---------------|------------------|------|------|
| POST | `/api/v1/users/register`             | `register`    | 注册用户         | ❌    | ✅ 已实现 |
| POST | `/api/v1/users/unregister/:user_id`  | `unregister`  | 注销用户         | ❌    | ✅ 已实现 |
| POST | `/api/v1/users/login`                | `login`       | 用户登录         | ❌    | ✅ 已实现 |
| POST | `/api/v1/users/logout`               | `logout`      | 用户登出         | ❌    | 🚧 占位 |
| GET  | `/api/v1/users/info`                 | `info`        | 获取当前用户信息 | ✅ Bearer | ✅ 已实现 |
| GET  | `/api/v1/labels/list`               | `list_user_labels` | 查看用户标签 | ❌ | ✅ 已实现 |
| GET  | `/api/v1/labels/users`               | `list_label_users` | 查看标签下的用户 | ❌ | ✅ 已实现 |
| POST | `/api/v1/labels/add`                 | `add_label`   | 为用户添加标签   | ✅ Bearer | ✅ 已实现 |
| PUT  | `/api/v1/labels/update`              | `update_label`| 修改标签备注     | ✅ Bearer | ✅ 已实现 |
| DELETE | `/api/v1/labels/remove`            | `remove_label` | 解除关联 | ✅ Bearer | ✅ 已实现 |

> `register`、`unregister`、`login` 均已接入 `AppState` 和 `feel_storage`，包含错误处理。`login` 自动签发 JWT token。`info` 需要通过 `Authorization: Bearer <token>` 认证。标签相关接口已实现，见 [label.md](label.md)。

## 架构说明

```
feel_api (HTTP 路由 & Handler)
    │
    ├── migration (自动执行数据库迁移)
    │
    ▼
feel_storage (数据库操作接口 + 缓存)
    │
    ▼
feel_sea_orm (Sea-ORM 实体 / 查询)
    │
    ▼
  PostgreSQL
```

- `feel_api` 依赖 `feel_storage`、`feel_core` 和 `migration`
- `feel_storage` 中的 `UserRepo` trait 提供了数据访问抽象
- `UserDataBase` 封装了 `UserRepo` 并包含缓存逻辑
- 启动时 `init_app_state()` 自动执行数据库迁移（`migration::Migrator::up`）
