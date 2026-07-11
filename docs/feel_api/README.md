# feel_api — HTTP API 服务

## 概述

`feel_api` 是 feel 平台的 **HTTP API 服务**，基于 [Poem](https://github.com/poem-web/poem) 框架构建，位于 `cmd/feel_api/`。

负责提供用户注册、登录、注销等 RESTful 接口，通过 `feel_storage` 和 `feel_core` 与数据库交互。

## 模块位置

```
cmd/feel_api/src/
├── main.rs      # 入口：启动 HTTP 服务器
├── lib.rs       # 路由、AppState、ApiConfig
└── user/
    └── mod.rs   # 用户相关接口
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
}
```

通过环境变量 `DATABASE_URL` 覆盖默认值，默认连接本地 PostgreSQL。

## 应用状态（AppState）

```rust
pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
}
```

启动时初始化数据库连接池，创建 `UserDataBase` 实例注入到 Poem 的共享状态中，供各 handler 使用。

## API 路由

所有接口挂载在 `/api/v1` 前缀下：

| 方法 | 路径                        | Handler     | 描述         |
|------|-----------------------------|-------------|--------------|
| POST | `/api/v1/user/register`     | `register`  | 注册用户     |
| POST | `/api/v1/user/unregister`   | `unregister`| 注销用户     |
| POST | `/api/v1/user/login`        | `login`     | 用户登录     |
| POST | `/api/v1/user/logout`       | `logout`    | 用户登出     |

> 当前 Handler 均为占位实现（空函数），功能待填充。

## 架构说明

```
feel_api (HTTP 路由 & Handler)
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

- `feel_api` 依赖 `feel_storage` 和 `feel_core`
- `feel_storage` 中的 `UserRepo` trait 提供了数据访问抽象
- `UserDataBase` 封装了 `UserRepo` 并包含缓存逻辑
