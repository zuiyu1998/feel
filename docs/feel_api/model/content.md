# Content DTO — 内容 API 请求/响应模型

## 概述

内容相关的 API 请求/响应 DTO 定义在 `cmd/feel_api/src/model/content.rs` 中，通过 `serde` 实现 JSON 序列化与反序列化。各接口的完整说明见 [feel_api/content.md](../content.md)，领域设计见 [features/content.md](../../features/content.md)。

## 请求 DTO

### CreateContentRequest

创建内容请求体（`POST /content/create`）。

```rust
#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub content_type: String,   // "article" | "video"
    pub title: String,
    pub payload: Value,         // 类型专属字段（JSON）
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `content_type` | `String` | 内容类型：`article` / `video` |
| `title` | `String` | 标题 |
| `payload` | `Value` | 载荷：article → `{ "body": "…" }`；video → `{ "video_url", "cover_url", "duration_secs" }` |

### ContentInfoRequest

查看详情请求体（`GET /content/info`）：`{ "content_id": 1 }`

### ListContentRequest

用户内容列表请求体（`GET /content/list`）：

```rust
pub struct ListContentRequest {
    pub user_id: i64,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
```

### FeedContentRequest

全站内容流请求体（`GET /content/feed`）：`content_type: Option<String>` + 分页参数。

### UpdateContentRequest

更新内容请求体（`PUT /content/update`）：`content_id` + `title: Option<String>` + `payload: Option<Value>`（均传 None 返回业务错误）。

### RemoveContentRequest

删除内容请求体（`DELETE /content/remove`）：`{ "content_id": 1 }`。

## 响应 DTO

### ContentItem

内容列表项 / 详情（所有类型统一结构：基表字段 + `payload` 对象）。

```rust
#[derive(Debug, Serialize)]
pub struct ContentItem {
    pub id: i64,
    pub user_id: i64,
    pub content_type: String,
    pub title: String,
    pub enabled: bool,
    pub created_at: String,     // RFC3339
    pub updated_at: String,     // RFC3339
    pub payload: Value,         // 类型专属字段（由 ContentKind::to_json 生成）
}

impl From<Content> for ContentItem { /* 字段映射 */ }
```

### ContentPageResponse

分页响应（`GET /content/list` 与 `GET /content/feed`）：

```rust
pub struct ContentPageResponse {
    pub items: Vec<ContentItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<ContentPage> for ContentPageResponse { /* 字段映射 */ }
```

## 与领域模型的对应关系

| DTO | 领域模型 | 转换方式 |
|-----|----------|----------|
| `ContentItem` | `feel_entity::content::Content` | `From<Content>`（`kind.to_json()` 生成 payload） |
| `ContentPageResponse` | `feel_entity::content::ContentPage` | `From<ContentPage>` |
| `CreateContentRequest` | `feel_entity::content::ContentCreate` | handler 内字段组装 |
| `UpdateContentRequest` | `feel_entity::content::ContentUpdate` | handler 内字段组装 |
