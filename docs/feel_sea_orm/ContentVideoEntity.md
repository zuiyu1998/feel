# ContentVideoEntity — content_video 表 ORM 实体

## 概述

`ContentVideoEntity` 是视频明细的 **Sea-ORM 实体**，对应 `content_video` 表（1:1 关联 `content.id`），位于 `crates/feel_sea_orm/src/content/entities/content_video.rs`。提供 `ContentVideoModel → VideoPayload` 的转换。

## 数据模型（Model）

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content_video")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub content_id: i64,
    pub video_url: String,
    pub cover_url: String,
    pub duration_secs: i32,
}
```

## DDL 对应（PostgreSQL）

```sql
CREATE TABLE content_video (
    content_id    BIGINT   PRIMARY KEY,
    video_url     VARCHAR  NOT NULL,
    cover_url     VARCHAR  NOT NULL DEFAULT '',
    duration_secs INTEGER  NOT NULL DEFAULT 0
);
```

## 关系定义（Relation）

```rust
pub enum Relation {}
```

无 ORM 关系 / 外键约束；`content_id` 为主键，与 `content.id` 逻辑 1:1 关联。

## 类型转换：ContentVideoModel → VideoPayload

```rust
impl From<Model> for VideoPayload {
    fn from(value: Model) -> Self {
        VideoPayload {
            video_url: value.video_url,
            cover_url: value.cover_url,
            duration_secs: value.duration_secs as i64,
        }
    }
}
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 字段定义 | ✅ | content_id / video_url / cover_url / duration_secs |
| From 转换 | ✅ | ContentVideoModel → VideoPayload |
| 迁移建表 | ✅ | `m20260210_000000_add_content` |
