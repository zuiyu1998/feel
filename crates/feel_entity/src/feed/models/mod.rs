use serde::{Deserialize, Serialize};

use crate::content::Content;

/// 关注流分页结果（推拉结合后的合并视图）。
///
/// `has_more` 表示**合并窗口**内是否还有更多内容：
/// 关注流只保证最近窗口的完整性（推模式收件箱窗口 + 拉模式大 V 窗口），
/// 窗口耗尽后 `has_more = false`，客户端可停止翻页。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowingFeedPage {
    pub items: Vec<Content>,
    pub page: u64,
    pub page_size: u64,
    pub has_more: bool,
}
