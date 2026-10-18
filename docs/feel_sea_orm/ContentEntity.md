# ContentEntity — content 表 ORM 实体

## 概述

`ContentEntity` 是 `feel_sea_orm` crate 中定义的 **Sea-ORM 实体**，对应数据库中的 `content` 表（内容公共字段，基表），位于 `crates/feel_sea_orm/src/content/entities/content.rs`。提供 `ContentModel → ContentBase` 的类型转换，供 `SeaOrmContentRepo` 使用。

## 模块位置

```
crates/feel_sea_orm/src/content/entities/
├── mod.rs        # 模块声明
├── prelude.rs    # 重新导出（Content* / ContentArticle* / ContentVideo*）
├── content.rs    # content 表实体 ←
├── content_article.rs # 文章明细表
└── content_video.rs   # 视频明细表
```

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(index)]
    pub user_id: i64,
    #[sea_orm(index)]
    pub content_type: String,
    pub title: String,
    pub enabled: bool,
    pub is_delete: bool,
    #[sea_orm(index)]
    pub view_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## DDL 对应（PostgreSQL，由 migration 创建）

```sql
CREATE TABLE content (
    id           BIGSERIAL    PRIMARY KEY,
    user_id      BIGINT       NOT NULL,
    content_type VARCHAR      NOT NULL,
    title        VARCHAR      NOT NULL,
    enabled      BOOLEAN      NOT NULL DEFAULT true,
    is_delete    BOOLEAN      NOT NULL DEFAULT false,
    view_count   BIGINT       NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ  NOT NULL,
    updated_at   TIMESTAMPTZ  NOT NULL
);

CREATE INDEX idx_content_user_created ON content (user_id, created_at);
CREATE INDEX idx_content_type_created ON content (content_type, created_at);
-- 浏览量（feed 热度），由 m20260211_000000_add_content_view_count 追加
CREATE INDEX idx_content_view_created ON content (view_count, created_at);
```

## 关系定义（Relation）

```rust
pub enum Relation {}
```

**未定义 ORM 关系**：与明细表（`content_article` / `content_video`）的关联通过 `content.id = 明细表.content_id` 逻辑承载，由 `SeaOrmContentRepo` 手动组装。

## 类型转换：ContentModel → ContentBase

```rust
impl From<Model> for ContentBase {
    // 字段一一映射
}
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | 9 个字段（含 view_count） |
| 索引 | ✅ | user_id + created_at、content_type + created_at、view_count + created_at 复合索引 |
| Relation | 🚧 空 | 未定义 ORM 关系 |
| From 转换 | ✅ | ContentModel → ContentBase |
| 迁移建表 | ✅ | `m20260210_000000_add_content` |
