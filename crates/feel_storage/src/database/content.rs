use async_trait::async_trait;

use crate::{Error, Result, repo::ContentRepo};
use feel_entity::prelude::*;

/// 列表分页默认页大小。
pub const DEFAULT_PAGE_SIZE: u64 = 20;
/// 列表分页页大小上限。
pub const MAX_PAGE_SIZE: u64 = 50;
/// 标题最大长度（字符）。
const MAX_TITLE_CHARS: usize = 100;
/// 文章正文最大长度（字符）。
const MAX_ARTICLE_BODY_CHARS: usize = 100_000;

pub struct CommonContentDataBase {
    content_repo: Box<dyn ContentRepo>,
}

impl CommonContentDataBase {
    pub fn new<T: ContentRepo>(content_repo: T) -> Self {
        CommonContentDataBase {
            content_repo: Box::new(content_repo),
        }
    }

    /// 将可选分页参数归一化为 (offset, page, page_size)。
    fn paginate(page: Option<u64>, page_size: Option<u64>) -> (u64, u64, u64) {
        let page = page.unwrap_or(1).max(1);
        let size = page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAX_PAGE_SIZE);
        ((page - 1) * size, page, size)
    }

    /// 校验并归一化标题（去首尾空白，长度 1~100 字符）。
    fn validate_title(title: &str) -> Result<String> {
        let title = title.trim();
        if title.is_empty() {
            return Err(Error::Business("Content title must not be empty".into()));
        }
        if title.chars().count() > MAX_TITLE_CHARS {
            return Err(Error::Business(format!(
                "Content title must be at most {MAX_TITLE_CHARS} characters"
            )));
        }
        Ok(title.to_string())
    }

    /// 校验并解析类型载荷。
    fn validate_payload(content_type: &str, payload: &serde_json::Value) -> Result<ContentKind> {
        let kind = ContentKind::parse(content_type, payload).map_err(Error::Business)?;
        match &kind {
            ContentKind::Article(payload) => {
                if payload.body.trim().is_empty() {
                    return Err(Error::Business("Article body must not be empty".into()));
                }
                if payload.body.chars().count() > MAX_ARTICLE_BODY_CHARS {
                    return Err(Error::Business(format!(
                        "Article body must be at most {MAX_ARTICLE_BODY_CHARS} characters"
                    )));
                }
            }
            ContentKind::Video(payload) => {
                if !payload.video_url.starts_with("http://")
                    && !payload.video_url.starts_with("https://")
                {
                    return Err(Error::Business(
                        "Video video_url must start with http:// or https://".into(),
                    ));
                }
                if !payload.cover_url.is_empty()
                    && !payload.cover_url.starts_with("http://")
                    && !payload.cover_url.starts_with("https://")
                {
                    return Err(Error::Business(
                        "Video cover_url must start with http:// or https://".into(),
                    ));
                }
                if payload.duration_secs < 0 {
                    return Err(Error::Business(
                        "Video duration_secs must not be negative".into(),
                    ));
                }
            }
        }
        Ok(kind)
    }
}

#[async_trait]
pub trait ContentDataBase: 'static + Send + Sync {
    /// 创建内容（标题/载荷校验后落库）
    async fn create(&self, create: &ContentCreate) -> Result<Content>;

    /// 按 ID 获取内容（已软删除的视为不存在）
    async fn get(&self, content_id: i64) -> Result<Option<Content>>;

    /// 分页查询某用户的内容列表（enabled 且未删除，按创建时间倒序）
    async fn list_by_user(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<ContentPage>;

    /// 分页查询全站内容流（enabled 且未删除，可按 content_type 过滤）
    async fn list_feed(
        &self,
        content_type: Option<&str>,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<ContentPage>;

    /// 更新内容标题/载荷（仅作者，归属校验由本层完成）
    async fn update(&self, update: &ContentUpdate) -> Result<Content>;

    /// 软删除内容（仅作者）
    async fn remove(&self, content_id: i64, user_id: i64) -> Result<()>;
}

#[async_trait]
impl ContentDataBase for CommonContentDataBase {
    async fn create(&self, create: &ContentCreate) -> Result<Content> {
        let title = Self::validate_title(&create.title)?;
        Self::validate_payload(&create.content_type, &create.payload)?;

        let create = ContentCreate {
            user_id: create.user_id,
            content_type: create.content_type.clone(),
            title,
            payload: create.payload.clone(),
        };
        self.content_repo.create(&create).await
    }

    async fn get(&self, content_id: i64) -> Result<Option<Content>> {
        Ok(self
            .content_repo
            .find_by_id(content_id)
            .await?
            .filter(|content| !content.base.is_delete))
    }

    async fn list_by_user(
        &self,
        user_id: i64,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<ContentPage> {
        let (offset, page, page_size) = Self::paginate(page, page_size);
        let items = self
            .content_repo
            .find_by_user(user_id, offset, page_size)
            .await?;
        let total = self.content_repo.count_by_user(user_id).await?;
        Ok(ContentPage {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn list_feed(
        &self,
        content_type: Option<&str>,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<ContentPage> {
        let (offset, page, page_size) = Self::paginate(page, page_size);
        let items = self
            .content_repo
            .find_feed(content_type, offset, page_size)
            .await?;
        let total = self.content_repo.count_feed(content_type).await?;
        Ok(ContentPage {
            items,
            total,
            page,
            page_size,
        })
    }

    async fn update(&self, update: &ContentUpdate) -> Result<Content> {
        let existing = self
            .content_repo
            .find_by_id(update.id)
            .await?
            .filter(|content| !content.base.is_delete)
            .ok_or_else(|| Error::Business(format!("Content {} not found", update.id)))?;

        if existing.base.user_id != update.user_id {
            return Err(Error::Business(format!(
                "User {} cannot manage content of user {}",
                update.user_id, existing.base.user_id
            )));
        }

        let mut normalized = update.clone();
        if let Some(title) = &update.title {
            normalized.title = Some(Self::validate_title(title)?);
        }
        if let Some(payload) = &update.payload {
            Self::validate_payload(&existing.base.content_type, payload)?;
        }
        if normalized.title.is_none() && normalized.payload.is_none() {
            return Err(Error::Business("Nothing to update".into()));
        }

        self.content_repo.update(&normalized).await
    }

    async fn remove(&self, content_id: i64, user_id: i64) -> Result<()> {
        let existing = self
            .content_repo
            .find_by_id(content_id)
            .await?
            .filter(|content| !content.base.is_delete)
            .ok_or_else(|| Error::Business(format!("Content {content_id} not found")))?;

        if existing.base.user_id != user_id {
            return Err(Error::Business(format!(
                "User {user_id} cannot manage content of user {}",
                existing.base.user_id
            )));
        }

        self.content_repo.soft_delete(content_id, user_id).await
    }
}
