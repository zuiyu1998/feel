# migration — 数据库迁移

## 概述

`migration` crate 基于 [sea-orm-migration](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/) 定义数据库结构的版本演进，位于 `crates/migration/`。

- 由 `feel_api` 启动时通过 `Migrator::up` **自动执行**（见 `cmd/feel_api/src/lib.rs` 的 `init_app_state`），无需手工迁移
- 也可独立运行 CLI（用法见仓库内 `crates/migration/README.md`）

## 模块位置

```
crates/migration/src/
├── lib.rs                                  # Migrator 定义与迁移注册
├── main.rs                                 # CLI 入口（cargo run -p migration）
├── m20260207_083750_add_users.rs           # users 表
├── m20260208_000000_add_user_credentials.rs# user_credentials 表
├── m20260209_000000_add_labels.rs          # label 表
├── m20260209_000001_add_user_labels.rs     # user_label 表
└── m20260210_000000_add_content.rs         # content / content_article / content_video 表
```

## Migrator 定义

```rust
pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260207_083750_add_users::Migration),
            Box::new(m20260208_000000_add_user_credentials::Migration),
            Box::new(m20260209_000000_add_labels::Migration),
            Box::new(m20260209_000001_add_user_labels::Migration),
            Box::new(m20260210_000000_add_content::Migration),
        ]
    }
}
```

迁移按**声明顺序**执行（`up`），回滚（`down`）按逆序执行。每个迁移实现 `up()` 建表、`down()` 删表。

## 迁移清单

| 迁移文件 | 创建的表 | 关键约束与说明 |
|----------|----------|----------------|
| `m20260207_083750_add_users` | `users` | `uid` UNIQUE；软删除标记 `is_delete`；`id` BIGSERIAL 主键 |
| `m20260208_000000_add_user_credentials` | `user_credentials` | 唯一索引 `idx_user_credentials_user_uid_credential_name (user_uid, credential_name)` |
| `m20260209_000000_add_labels` | `label` | `name` UNIQUE（标签名全局唯一） |
| `m20260209_000001_add_user_labels` | `user_label` | 唯一索引 `idx_user_label_user_id_label_id (user_id, label_id)`；普通索引 `idx_user_label_label_id` |
| `m20260210_000000_add_content` | `content` / `content_article` / `content_video` | 索引 `idx_content_user_created (user_id, created_at)`、`idx_content_type_created (content_type, created_at)`；明细表 1:1 关联 `content.id` |

> **表名注意**：`label`、`user_label` 为**单数**表名，与 `feel_sea_orm` 实体的 `#[sea_orm(table_name = ...)]` 保持一致；`users`、`user_credentials` 为复数/下划线组合。以实体 `table_name` 为准。

### 表结构概要

| 表 | 列 | 说明 |
|----|----|------|
| `users` | id / uid / name / avatar / slogan / is_enable / is_delete / created_at / updated_at | 用户主体（软删除） |
| `user_credentials` | id / user_uid / credential_type / credential_name / encrypted_data / encryption_key / is_enabled / created_at / updated_at | 登录凭据（加密存储，`user_uid` 逻辑关联 `users.uid`） |
| `label` | id / name / description / remark / influence / enabled / created_at / updated_at | 标签本体（公有资源，字段语义见 [features/label.md](../features/label.md)「数据模型」） |
| `user_label` | id / user_id / label_id / enabled / created_at / updated_at | 用户×标签关联（`user_id` 关联 `users.id`，`label_id` 关联 `label.id`） |
| `content` | id / user_id / content_type / title / enabled / is_delete / created_at / updated_at | 内容基表（公共字段），字段语义见 [features/content.md](../features/content.md) |
| `content_article` | content_id / body | 文章明细（1:1 关联 `content.id`） |
| `content_video` | content_id / video_url / cover_url / duration_secs | 视频明细（1:1 关联 `content.id`） |

所有时间列均为 `TIMESTAMPTZ`（`timestamp_with_time_zone`）。

## 主键与自增约定

- 迁移中主键统一为 `BIGSERIAL`（`big_integer().auto_increment()`）
- 实体层声明 `auto_increment = false`（SeaORM 层不自动回填主键），写入时以 `id: NotSet` 交由数据库序列生成，插入后从返回行取回自增 `id`
- 例外：`users.uid` 由 `typedflake::id!(UserId)` 生成的全局唯一串，并非自增主键

## CLI 用法

见仓库内 `crates/migration/README.md`，常用命令：

```bash
cargo run -p migration            # 应用所有未执行迁移（等价 up）
cargo run -p migration -- down    # 回滚最后一次迁移
cargo run -p migration -- status  # 查看迁移状态
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 迁移文件 | ✅ 5 个 | users / user_credentials / label / user_label / content + content_article + content_video |
| 自动执行 | ✅ | feel_api 启动时 `Migrator::up` |
| 回滚 | ✅ | 每个迁移实现 `down()` |
