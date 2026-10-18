use feel_entity::content::VideoPayload;
use sea_orm::entity::prelude::*;

/// ORM entity for the `content_video` table (video details, 1:1 with content).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content_video")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub content_id: i64,
    /// Video URL (object storage / CDN)
    pub video_url: String,
    /// Cover image URL (may be empty)
    pub cover_url: String,
    /// Duration in seconds (0 when unknown)
    pub duration_secs: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for VideoPayload {
    fn from(value: Model) -> Self {
        VideoPayload {
            video_url: value.video_url,
            cover_url: value.cover_url,
            duration_secs: value.duration_secs as i64,
        }
    }
}
