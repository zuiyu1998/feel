//! 内容 API 的请求/响应 DTO。

use crate::model::label::UserBrief;
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

/// 全站内容流（feed）请求体（GET /content/feed）
#[derive(Debug, Deserialize)]
pub struct FeedContentRequest {
    /// 可选：按内容类型过滤（"article" | "video"）
    pub content_type: Option<String>,
    /// 可选：排序方式（"latest" 最新，默认；"popular" 热门）
    pub sort: Option<String>,
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

/// 内容列表项（基表字段 + 类型载荷 + 作者摘要）。
///
/// 作为 Feed 流卡片使用：`author` 内嵌作者信息，客户端无需二次查询；
/// `payload` 对象随 `content_type` 变化（article → `{ "body": "..." }`；
/// video → `{ "video_url": ..., "cover_url": ..., "duration_secs": ... }`）。
#[derive(Debug, Serialize)]
pub struct ContentItem {
    pub id: i64,
    pub user_id: i64,
    pub content_type: String,
    pub title: String,
    pub enabled: bool,
    /// 浏览量（feed 热度排序依据）
    pub view_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub payload: Value,
    /// 作者摘要（user_id / name / avatar）
    pub author: UserBrief,
}

/// 将领域内容与作者摘要组装为 API 列表项。
pub fn content_item(content: Content, author: UserBrief) -> ContentItem {
    ContentItem {
        id: content.base.id,
        user_id: content.base.user_id,
        content_type: content.base.content_type,
        title: content.base.title,
        enabled: content.base.enabled,
        view_count: content.base.view_count,
        created_at: content.base.created_at.to_rfc3339(),
        updated_at: content.base.updated_at.to_rfc3339(),
        payload: content.kind.to_json(),
        author,
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

impl ContentPageResponse {
    /// 由领域分页结果（分页元信息）与已组装的列表项构造分页响应。
    pub fn with_items(items: Vec<ContentItem>, page: &ContentPage) -> Self {
        ContentPageResponse {
            items,
            total: page.total,
            page: page.page,
            page_size: page.page_size,
        }
    }
}
