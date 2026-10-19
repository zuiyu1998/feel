# Feed DTO — 关注流 API 请求/响应模型

## 概述

关注流的 API 请求/响应 DTO 定义在 `cmd/feel_api/src/model/feed.rs` 中，复用 `model/content.rs` 的 `ContentItem` 卡片结构。

## 请求 DTO

### FollowingFeedRequest

关注流请求体（`GET /feed/following`）：

```rust
#[derive(Debug, Deserialize)]
pub struct FollowingFeedRequest {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
```

## 响应 DTO

### FollowingFeedResponse

推拉合并后的时间线响应：

```rust
#[derive(Debug, Serialize)]
pub struct FollowingFeedResponse {
    pub items: Vec<ContentItem>,
    pub page: u64,
    pub page_size: u64,
    /// 合并窗口内是否还有更多（false 时客户端停止翻页）
    pub has_more: bool,
}
```

- `items` 元素复用 `ContentItem`（基表字段 + `payload` + `author` 摘要）
- 无 `total`：关注流基于推拉窗口，总量不精确，改用 `has_more` 语义

## 与领域模型的对应关系

| DTO | 领域模型 | 转换方式 |
|-----|----------|----------|
| `FollowingFeedResponse` | `feel_entity::feed::FollowingFeedPage` | handler 内组装（items 经作者摘要补齐） |
| `ContentItem` | `feel_entity::content::Content` | `model::content::content_item(content, author)` |
