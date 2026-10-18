# ContentArticleEntity — content_article 表 ORM 实体

## 概述

`ContentArticleEntity` 是文章明细的 **Sea-ORM 实体**，对应 `content_article` 表（1:1 关联 `content.id`），位于 `crates/feel_sea_orm/src/content/entities/content_article.rs`。提供 `ContentArticleModel → ArticlePayload` 的转换。

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content_article")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub content_id: i64,
    pub body: String,
}
```

## DDL 对应（PostgreSQL）

```sql
CREATE TABLE content_article (
    content_id BIGINT  PRIMARY KEY,
    body       TEXT    NOT NULL
);
```

## 关系定义（Relation）

```rust
pub enum Relation {}
```

无 ORM 关系 / 外键约束；`content_id` 为主键，与 `content.id` 逻辑 1:1 关联。

## 类型转换：ContentArticleModel → ArticlePayload

```rust
impl From<Model> for ArticlePayload {
    fn from(value: Model) -> Self {
        ArticlePayload { body: value.body }
    }
}
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | content_id / body |
| From 转换 | ✅ | ContentArticleModel → ArticlePayload |
| 迁移建表 | ✅ | `m20260210_000000_add_content` |
