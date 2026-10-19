//! 关注流（Feed）API 的请求/响应 DTO。

use crate::model::content::ContentItem;
use serde::{Deserialize, Serialize};

/// 关注流请求体（GET /feed/following）
#[derive(Debug, Deserialize)]
pub struct FollowingFeedRequest {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

/// 关注流响应（推拉合并后的时间线，按内容倒序）。
#[derive(Debug, Serialize)]
pub struct FollowingFeedResponse {
    pub items: Vec<ContentItem>,
    pub page: u64,
    pub page_size: u64,
    /// 合并窗口内是否还有更多（窗口外内容不再返回，客户端可停止翻页）
    pub has_more: bool,
}
