use feel_entity::content::ArticlePayload;
use sea_orm::entity::prelude::*;

/// ORM entity for the `content_article` table (article details, 1:1 with content).
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "content_article")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub content_id: i64,
    /// Markdown body
    pub body: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for ArticlePayload {
    fn from(value: Model) -> Self {
        ArticlePayload { body: value.body }
    }
}
