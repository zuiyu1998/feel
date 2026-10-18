# feel — 印象

印象（feel）是减少人与人沟通与协作成本的平台：通过对用户进行客观评价来精准描述其自我画像。用户通过表达描绘自我画像——理智的表达（文章）不加限制，情绪化的表达（评论）需要限制。

平台愿景与功能规划见 [docs/主要体验.md](docs/主要体验.md)。

## 架构

Rust workspace，分层架构：

```
cmd/feel_api        HTTP API 服务（Poem；路由 / Handler / DTO / JWT 认证）
        │
        ▼
crates/feel_storage 数据访问层：Repo 持久化接口 + DataBase（组合 Repo + 缓存）
crates/feel_entity  领域模型（UserBase / LabelBase / UserLabel …）
crates/feel_core    核心公共依赖（chrono / tokio 统一导出）
        │
        ▼
crates/feel_sea_orm Sea-ORM 实体与 Repo 实现
crates/migration    数据库迁移（启动时自动执行）
        │
        ▼
    PostgreSQL（业务数据） + Redis（缓存）
```

| Crate | 职责 | 文档 |
|-------|------|------|
| [`feel_api`](cmd/feel_api/) | HTTP API 服务（唯一二进制入口） | [docs/feel_api/](docs/feel_api/README.md) |
| [`feel_storage`](crates/feel_storage/) | 持久化接口 + 数据访问网关 + 缓存 | [docs/feel_storage/](docs/feel_storage/UserDataBase.md) |
| [`feel_entity`](crates/feel_entity/) | 领域模型定义 | [docs/feel_entity/](docs/feel_entity/README.md) |
| [`feel_sea_orm`](crates/feel_sea_orm/) | Sea-ORM 实体与 Repo 实现 | [docs/feel_sea_orm/](docs/feel_sea_orm/UserEntity.md) |
| [`migration`](crates/migration/) | 数据库迁移 | [docs/migration/](docs/migration/README.md) |
| [`feel_core`](crates/feel_core/) | 核心公共依赖 | [docs/feel_core/](docs/feel_core/README.md) |

> 根包 `src/lib.rs` 目前为 `add` 占位脚手架，不承载业务代码。

## 快速开始

```bash
# 启动 API 服务（默认连接本地 PostgreSQL / Redis，见 docs/feel_api/deploy.md）
cargo run -p feel_api

# 自定义数据库
DATABASE_URL=postgresql://user:password@host:5432/feel cargo run -p feel_api
```

- 服务默认监听 `0.0.0.0:3000`，接口前缀 `/api/v1`
- 启动时自动执行数据库迁移（`migration::Migrator::up`）
- 配置项（数据库 / Redis / JWT 密钥）见 [docs/feel_api/README.md](docs/feel_api/README.md#配置apiconfig)

## 文档

完整的文档目录（按项目架构组织）见 [**docs/README.md**](docs/README.md)。

## 开发

```bash
cargo check --workspace        # 类型检查
cargo test --workspace         # 运行测试
```

代码风格与提交规范见 `.pre-commit-config.yaml` / `reasonix.toml`。
