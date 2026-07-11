# feel_api 启动与部署

## 环境要求

- Rust 2024 edition
- PostgreSQL 数据库

## 环境变量

| 变量名         | 默认值                                                     | 说明                |
|----------------|------------------------------------------------------------|---------------------|
| `DATABASE_URL` | `postgresql://postgres:bj123456@192.168.0.107:5432/feel`   | PostgreSQL 连接地址 |
| `RUST_LOG`     | `info`                                                     | 日志级别            |

## 启动步骤

### 1. 启动 API 服务（自动执行迁移）

```bash
# 默认配置
cargo run -p feel_api

# 自定义数据库
DATABASE_URL=postgresql://user:pass@localhost:5432/feel cargo run -p feel_api

# 启用 debug 日志
RUST_LOG=debug cargo run -p feel_api
```

服务启动时 `init_app_state()` 会自动执行数据库迁移，无需手动运行 `cargo run -p migration`。

### 2. 验证服务

```bash
curl -X POST http://localhost:3000/api/v1/user/register
```

## 代码入口

```rust
// cmd/feel_api/src/main.rs
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = ApiConfig::default();

    // 初始化数据库连接和 AppState
    let app_state = init_app_state(&config).await;

    let app = Route::new()
        .nest("/api/v1", app_route())
        .data(app_state);

    // 监听 0.0.0.0:3000
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
```

## 依赖关系

```mermaid
graph TD
    feel_api --> feel_storage
    feel_api --> feel_core
    feel_api --> migration
    feel_storage --> feel_sea_orm
    feel_sea_orm --> sea-orm
    migration --> sea-orm-migration
    sea-orm-migration --> sea-orm
    sea_orm --> PostgreSQL
```
