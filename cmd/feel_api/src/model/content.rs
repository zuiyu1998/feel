//! 内容 API 的请求/响应 DTO。

use feel_entity::content::{Content, ContentPage};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// -- 请求 --

/// 创建内容请求体（POST /content/create）
#[derive(Debug, Deserialize)]
pub struct CreateContentRequest {
    pub content_type: String,
    pub title: String,
    /// 类型专属字段（按 content_type 校验：article → { body }；video → { video_url, cover_url, duration_secs }）
    pub payload: Value,
}

/// 查看内容详情请求体（GET /content/info）
#[derive(Debug, Deserialize)]
pub struct ContentInfoRequest {
    pub content_id: i64,
}

/// 查看某用户内容列表请求体（GET /content/list）
#[derive(Debug, Deserialize)]
pub struct ListContentRequest {
    pub user_id: i64,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 全站内容流请求体（GET /content/feed）
#[derive(Debug, Deserialize)]
pub struct FeedContentRequest {
    /// 可选：按内容类型过滤（"article" | "video"）
    pub content_type: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 更新内容请求体（PUT /content/update）
#[derive(Debug, Deserialize)]
pub struct UpdateContentRequest {
    pub content_id: i64,
    pub title: Option<String>,
    pub payload: Option<Value>,
}

/// 删除内容请求体（DELETE /content/remove）
#[derive(Debug, Deserialize)]
pub struct RemoveContentRequest {
    pub content_id: i64,
}

// -- 响应 --

/// 内容列表项（基表字段 + 类型载荷）。
///
/// 所有内容类型统一用同一 DTO 输出：`payload` 对象随 `content_type` 变化
/// （article → `{ "body": "..." }`；video → `{ "video_url": ..., "cover_url": ..., "duration_secs": ... }`）。
#[derive(Debug, Serialize)]
pub struct ContentItem {
    pub id: i64,
    pub user_id: i64,
    pub content_type: String,
    pub title: String,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub payload: Value,
}

impl From<Content> for ContentItem {
    fn from(value: Content) -> Self {
        ContentItem {
            id: value.base.id,
            user_id: value.base.user_id,
            content_type: value.base.content_type.clone(),
            title: value.base.title,
            enabled: value.base.enabled,
            created_at: value.base.created_at.to_rfc3339(),
            updated_at: value.base.updated_at.to_rfc3339(),
            payload: value.kind.to_json(),
        }
    }
}

/// 内容分页响应。
#[derive(Debug, Serialize)]
pub struct ContentPageResponse {
    pub items: Vec<ContentItem>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

impl From<ContentPage> for ContentPageResponse {
    fn from(value: ContentPage) -> Self {
        ContentPageResponse {
            items: value.items.into_iter().map(Into::into).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}
