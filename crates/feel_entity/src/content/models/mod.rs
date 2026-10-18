use feel_core::chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 内容类型：文章。
pub const CONTENT_TYPE_ARTICLE: &str = "article";
/// 内容类型：视频。
pub const CONTENT_TYPE_VIDEO: &str = "video";

/// 内容公共部分（对应 `content` 基表）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentBase {
    /// Database primary key
    pub id: i64,
    /// 作者（引用 UserBase.id）
    pub user_id: i64,
    /// 内容类型（"article" | "video" | 未来扩展）
    pub content_type: String,
    /// 标题
    pub title: String,
    /// 是否可见/启用
    pub enabled: bool,
    /// 软删除标记
    pub is_delete: bool,
    /// 浏览量（feed 热度排序依据，访问详情时递增）
    pub view_count: i64,
    /// Record creation time
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
}

/// 文章明细载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArticlePayload {
    /// Markdown 正文
    pub body: String,
}

/// 视频明细载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VideoPayload {
    /// 视频地址（对象存储/CDN）
    pub video_url: String,
    /// 封面图地址（可为空串）
    pub cover_url: String,
    /// 时长（秒），未知时为 0
    pub duration_secs: i64,
}

/// 内容类型化载荷。
///
/// 这是**新增内容类型的中心分派点**：加入新类型 = 增加一个变体 +
/// 在 `parse` / `content_type` / `to_json` 的 match 中补一个分支
/// （match 必须穷尽，漏改会编译失败）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentKind {
    Article(ArticlePayload),
    Video(VideoPayload),
}

impl ContentKind {
    /// 返回该载荷对应的内容类型标识。
    pub fn content_type(&self) -> &'static str {
        match self {
            ContentKind::Article(_) => CONTENT_TYPE_ARTICLE,
            ContentKind::Video(_) => CONTENT_TYPE_VIDEO,
        }
    }

    /// 按 `content_type` 将 JSON payload 解析为类型化载荷。
    ///
    /// 解析失败（类型不支持 / 字段缺失或类型不符）时返回可读的错误消息。
    pub fn parse(content_type: &str, payload: &Value) -> Result<ContentKind, String> {
        match content_type {
            CONTENT_TYPE_ARTICLE => serde_json::from_value::<ArticlePayload>(payload.clone())
                .map(ContentKind::Article)
                .map_err(|e| format!("Invalid payload for content_type '{content_type}': {e}")),
            CONTENT_TYPE_VIDEO => serde_json::from_value::<VideoPayload>(payload.clone())
                .map(ContentKind::Video)
                .map_err(|e| format!("Invalid payload for content_type '{content_type}': {e}")),
            other => Err(format!("Unsupported content_type '{other}'")),
        }
    }

    /// 将类型化载荷序列化为 JSON（用于响应 DTO 的 `payload` 字段）。
    pub fn to_json(&self) -> Value {
        match self {
            ContentKind::Article(p) => {
                serde_json::to_value(p).expect("ArticlePayload is always serializable")
            }
            ContentKind::Video(p) => {
                serde_json::to_value(p).expect("VideoPayload is always serializable")
            }
        }
    }
}

/// 组合视图：基表 + 类型载荷。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub base: ContentBase,
    pub kind: ContentKind,
}

/// 创建内容请求（`payload` 由 HTTP 边界携带，落库前校验）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentCreate {
    /// 作者主键（由 handler 从认证用户解析）
    pub user_id: i64,
    /// 内容类型（"article" | "video" | 未来扩展）
    pub content_type: String,
    /// 标题
    pub title: String,
    /// 类型专属字段（JSON，按 content_type 解析校验）
    pub payload: Value,
}

/// 更新内容请求（仅作者可操作；`payload` 需与既有 `content_type` 匹配）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentUpdate {
    /// 内容主键
    pub id: i64,
    /// 操作者主键（用于归属校验）
    pub user_id: i64,
    /// 新标题（None 表示不修改）
    pub title: Option<String>,
    /// 新载荷（None 表示不修改）
    pub payload: Option<Value>,
}

/// 内容分页结果。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentPage {
    pub items: Vec<Content>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

/// Feed 流排序方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedSort {
    /// 最新（按创建时间倒序，默认）
    Latest,
    /// 热门（按浏览量倒序，同浏览量按创建时间倒序）
    Popular,
}

impl FeedSort {
    /// 解析 API 传入的排序标识（"latest" | "popular"）。
    pub fn parse(sort: &str) -> Result<FeedSort, String> {
        match sort {
            "latest" => Ok(FeedSort::Latest),
            "popular" => Ok(FeedSort::Popular),
            other => Err(format!(
                "Unsupported feed sort '{other}', expected 'latest' or 'popular'"
            )),
        }
    }
}
