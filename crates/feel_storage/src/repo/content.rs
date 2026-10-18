use crate::{Error, Result, repo::ContentRepo};
use async_trait::async_trait;
use chrono::Local;
use feel_entity::content::{
    ArticlePayload, CONTENT_TYPE_ARTICLE, CONTENT_TYPE_VIDEO, Content, ContentBase, ContentCreate,
    ContentKind, ContentUpdate, FeedSort, VideoPayload,
};
use feel_sea_orm::content::entities::prelude::*;
use sea_orm::{
    ActiveModelTrait,
    ActiveValue::{NotSet, Set},
    ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, TransactionTrait,
    sea_query::Expr,
};
use std::collections::HashMap;

pub struct SeaOrmContentRepo {
    /// Sea-orm database connection
    pub conn: DatabaseConnection,
}

impl SeaOrmContentRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        SeaOrmContentRepo { conn }
    }
}

impl SeaOrmContentRepo {
    /// 解析 payload 为类型化载荷（按 content_type 分派）。
    fn parse_payload(content_type: &str, payload: &serde_json::Value) -> Result<ContentKind> {
        ContentKind::parse(content_type, payload).map_err(Error::Business)
    }

    /// 加载单条内容明细（1:1），明细缺失时返回 None。
    async fn load_detail(&self, base: &ContentBase) -> Result<Option<ContentKind>> {
        match base.content_type.as_str() {
            CONTENT_TYPE_ARTICLE => Ok(ContentArticleEntity::find_by_id(base.id)
                .one(&self.conn)
                .await?
                .map(|model| ContentKind::Article(model.into()))),
            CONTENT_TYPE_VIDEO => Ok(ContentVideoEntity::find_by_id(base.id)
                .one(&self.conn)
                .await?
                .map(|model| ContentKind::Video(model.into()))),
            other => Err(Error::Business(format!(
                "Unsupported content_type '{other}'"
            ))),
        }
    }

    /// 批量加载明细并组装 `Content` 列表（每类明细一次 IN 查询，避免 N+1）。
    async fn load_contents(&self, bases: Vec<ContentBase>) -> Result<Vec<Content>> {
        let mut article_ids = Vec::new();
        let mut video_ids = Vec::new();
        for base in &bases {
            match base.content_type.as_str() {
                CONTENT_TYPE_ARTICLE => article_ids.push(base.id),
                CONTENT_TYPE_VIDEO => video_ids.push(base.id),
                _ => {}
            }
        }

        let mut article_map: HashMap<i64, ArticlePayload> = HashMap::new();
        if !article_ids.is_empty() {
            let rows = ContentArticleEntity::find()
                .filter(ContentArticleColumn::ContentId.is_in(article_ids))
                .all(&self.conn)
                .await?;
            article_map.extend(rows.into_iter().map(|m| (m.content_id, m.into())));
        }

        let mut video_map: HashMap<i64, VideoPayload> = HashMap::new();
        if !video_ids.is_empty() {
            let rows = ContentVideoEntity::find()
                .filter(ContentVideoColumn::ContentId.is_in(video_ids))
                .all(&self.conn)
                .await?;
            video_map.extend(rows.into_iter().map(|m| (m.content_id, m.into())));
        }

        let mut out = Vec::with_capacity(bases.len());
        for base in bases {
            let kind = match base.content_type.as_str() {
                CONTENT_TYPE_ARTICLE => article_map.remove(&base.id).map(ContentKind::Article),
                CONTENT_TYPE_VIDEO => video_map.remove(&base.id).map(ContentKind::Video),
                // 未知类型 / 明细行缺失：跳过脏数据，不中断列表
                _ => None,
            };
            if let Some(kind) = kind {
                out.push(Content { base, kind });
            }
        }
        Ok(out)
    }
}

#[async_trait]
impl ContentRepo for SeaOrmContentRepo {
    async fn create(&self, create: &ContentCreate) -> Result<Content> {
        let kind = Self::parse_payload(&create.content_type, &create.payload)?;
        let now = Local::now();

        let txn = self.conn.begin().await?;

        let active = ContentActiveModel {
            id: NotSet,
            user_id: Set(create.user_id),
            content_type: Set(create.content_type.clone()),
            title: Set(create.title.clone()),
            enabled: Set(true),
            is_delete: Set(false),
            view_count: Set(0),
            created_at: Set(now.to_utc()),
            updated_at: Set(now.to_utc()),
        };
        let model: ContentModel = active.insert(&txn).await?;
        let content_id = model.id;

        match &kind {
            ContentKind::Article(payload) => {
                let active = ContentArticleActiveModel {
                    content_id: Set(content_id),
                    body: Set(payload.body.clone()),
                };
                active.insert(&txn).await?;
            }
            ContentKind::Video(payload) => {
                let active = ContentVideoActiveModel {
                    content_id: Set(content_id),
                    video_url: Set(payload.video_url.clone()),
                    cover_url: Set(payload.cover_url.clone()),
                    duration_secs: Set(payload.duration_secs as i32),
                };
                active.insert(&txn).await?;
            }
        }

        txn.commit().await?;

        let base: ContentBase = model.into();
        Ok(Content { base, kind })
    }

    async fn find_by_id(&self, content_id: i64) -> Result<Option<Content>> {
        let Some(model) = ContentEntity::find_by_id(content_id)
            .one(&self.conn)
            .await?
        else {
            return Ok(None);
        };

        let base: ContentBase = model.into();
        match self.load_detail(&base).await? {
            Some(kind) => Ok(Some(Content { base, kind })),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: i64, offset: u64, limit: u64) -> Result<Vec<Content>> {
        let bases = ContentEntity::find()
            .filter(ContentColumn::UserId.eq(user_id))
            .filter(ContentColumn::Enabled.eq(true))
            .filter(ContentColumn::IsDelete.eq(false))
            .order_by_desc(ContentColumn::CreatedAt)
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await?
            .into_iter()
            .map(ContentBase::from)
            .collect();

        self.load_contents(bases).await
    }

    async fn count_by_user(&self, user_id: i64) -> Result<u64> {
        let count = ContentEntity::find()
            .filter(ContentColumn::UserId.eq(user_id))
            .filter(ContentColumn::Enabled.eq(true))
            .filter(ContentColumn::IsDelete.eq(false))
            .count(&self.conn)
            .await?;
        Ok(count)
    }

    async fn find_feed(
        &self,
        content_type: Option<&str>,
        sort: FeedSort,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<Content>> {
        let mut query = ContentEntity::find()
            .filter(ContentColumn::Enabled.eq(true))
            .filter(ContentColumn::IsDelete.eq(false));
        if let Some(content_type) = content_type {
            query = query.filter(ContentColumn::ContentType.eq(content_type));
        }

        query = match sort {
            FeedSort::Latest => query.order_by_desc(ContentColumn::CreatedAt),
            FeedSort::Popular => query
                .order_by_desc(ContentColumn::ViewCount)
                .order_by_desc(ContentColumn::CreatedAt),
        };

        let bases = query
            .offset(offset)
            .limit(limit)
            .all(&self.conn)
            .await?
            .into_iter()
            .map(ContentBase::from)
            .collect();

        self.load_contents(bases).await
    }

    async fn count_feed(&self, content_type: Option<&str>) -> Result<u64> {
        let mut query = ContentEntity::find()
            .filter(ContentColumn::Enabled.eq(true))
            .filter(ContentColumn::IsDelete.eq(false));
        if let Some(content_type) = content_type {
            query = query.filter(ContentColumn::ContentType.eq(content_type));
        }
        let count = query.count(&self.conn).await?;
        Ok(count)
    }

    async fn update(&self, update: &ContentUpdate) -> Result<Content> {
        let existing = ContentEntity::find_by_id(update.id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| Error::Business(format!("Content {} not found", update.id)))?;
        let content_type = existing.content_type.clone();

        let new_kind = match &update.payload {
            Some(payload) => Some(Self::parse_payload(&content_type, payload)?),
            None => None,
        };

        let txn = self.conn.begin().await?;

        let mut active = existing.into_active_model();
        if let Some(title) = &update.title {
            active.title = Set(title.clone());
        }
        active.updated_at = Set(Local::now().to_utc());
        let model: ContentModel = active.update(&txn).await?;

        if let Some(kind) = &new_kind {
            match kind {
                ContentKind::Article(payload) => {
                    let row = ContentArticleEntity::find_by_id(model.id).one(&txn).await?;
                    match row {
                        Some(row) => {
                            let mut active = row.into_active_model();
                            active.body = Set(payload.body.clone());
                            active.update(&txn).await?;
                        }
                        None => {
                            ContentArticleActiveModel {
                                content_id: Set(model.id),
                                body: Set(payload.body.clone()),
                            }
                            .insert(&txn)
                            .await?;
                        }
                    }
                }
                ContentKind::Video(payload) => {
                    let row = ContentVideoEntity::find_by_id(model.id).one(&txn).await?;
                    match row {
                        Some(row) => {
                            let mut active = row.into_active_model();
                            active.video_url = Set(payload.video_url.clone());
                            active.cover_url = Set(payload.cover_url.clone());
                            active.duration_secs = Set(payload.duration_secs as i32);
                            active.update(&txn).await?;
                        }
                        None => {
                            ContentVideoActiveModel {
                                content_id: Set(model.id),
                                video_url: Set(payload.video_url.clone()),
                                cover_url: Set(payload.cover_url.clone()),
                                duration_secs: Set(payload.duration_secs as i32),
                            }
                            .insert(&txn)
                            .await?;
                        }
                    }
                }
            }
        }

        txn.commit().await?;

        self.find_by_id(update.id)
            .await?
            .ok_or_else(|| Error::Business(format!("Content {} not found after update", update.id)))
    }

    async fn soft_delete(&self, content_id: i64, user_id: i64) -> Result<()> {
        let existing = ContentEntity::find_by_id(content_id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| Error::Business(format!("Content {content_id} not found")))?;

        if existing.user_id != user_id {
            return Err(Error::Business(format!(
                "User {user_id} cannot manage content of user {}",
                existing.user_id
            )));
        }

        let mut active = existing.into_active_model();
        active.is_delete = Set(true);
        active.updated_at = Set(Local::now().to_utc());
        active.update(&self.conn).await?;
        Ok(())
    }

    async fn increment_view_count(&self, content_id: i64) -> Result<u64> {
        let result = ContentEntity::update_many()
            .col_expr(
                ContentColumn::ViewCount,
                Expr::col(ContentColumn::ViewCount).add(1),
            )
            .filter(ContentColumn::Id.eq(content_id))
            .filter(ContentColumn::IsDelete.eq(false))
            .exec(&self.conn)
            .await?;
        Ok(result.rows_affected)
    }
}
