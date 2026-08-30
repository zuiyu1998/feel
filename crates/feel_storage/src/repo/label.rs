use crate::{Result, repo::LabelRepo};
use async_trait::async_trait;
use chrono::Local;
use feel_entity::label::{LabelBase, LabelCreate, LabelUpdate, UserLabel, UserLabelCreate};
use feel_sea_orm::label::entities::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ActiveValue::Set, ColumnTrait, DatabaseConnection,
    EntityTrait, IntoActiveModel, PaginatorTrait, QueryFilter, QueryOrder,
};

pub struct SeaOrmLabelRepo {
    /// Sea-orm database connection
    pub conn: DatabaseConnection,
}

impl SeaOrmLabelRepo {
    pub fn new(conn: DatabaseConnection) -> Self {
        SeaOrmLabelRepo { conn }
    }
}

#[async_trait]
impl LabelRepo for SeaOrmLabelRepo {
    async fn find_label_by_id(&self, label_id: i64) -> Result<Option<LabelBase>> {
        let label = LabelEntity::find_by_id(label_id).one(&self.conn).await?;
        Ok(label.map(Into::into))
    }

    async fn find_label_by_name(&self, name: &str) -> Result<Option<LabelBase>> {
        let label = LabelEntity::find()
            .filter(LabelColumn::Name.eq(name))
            .one(&self.conn)
            .await?;
        Ok(label.map(Into::into))
    }

    async fn find_all_labels(&self) -> Result<Vec<LabelBase>> {
        let list = LabelEntity::find()
            .order_by_asc(LabelColumn::CreatedAt)
            .all(&self.conn)
            .await?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn create_label(&self, create: &LabelCreate) -> Result<LabelBase> {
        let now = Local::now();

        let active = LabelActiveModel {
            id: NotSet, // database auto-increment
            name: Set(create.name.clone()),
            description: Set(create.description.clone()),
            remark: Set(create.remark.clone()),
            influence: Set(create.influence),
            enabled: Set(true),
            created_at: Set(now.to_utc()),
            updated_at: Set(now.to_utc()),
        };

        let model: LabelModel = active.insert(&self.conn).await?;
        Ok(model.into())
    }

    async fn update_label(&self, update: &LabelUpdate) -> Result<LabelBase> {
        let now = Local::now();

        let mut active = LabelEntity::find_by_id(update.id)
            .one(&self.conn)
            .await?
            .ok_or_else(|| {
                sea_orm::DbErr::RecordNotFound(format!("Label {} not found", update.id))
            })?
            .into_active_model();

        active.name = Set(update.name.clone());
        active.description = Set(update.description.clone());
        active.remark = Set(update.remark.clone());
        active.influence = Set(update.influence);
        active.enabled = Set(update.enabled);
        active.updated_at = Set(now.to_utc());

        let model: LabelModel = active.update(&self.conn).await?;
        Ok(model.into())
    }

    async fn find_user_labels(&self, user_id: i64) -> Result<Vec<UserLabel>> {
        let list = UserLabelEntity::find()
            .filter(UserLabelColumn::UserId.eq(user_id))
            .order_by_asc(UserLabelColumn::CreatedAt)
            .all(&self.conn)
            .await?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn find_users_by_label(&self, label_id: i64) -> Result<Vec<UserLabel>> {
        let list = UserLabelEntity::find()
            .filter(UserLabelColumn::LabelId.eq(label_id))
            .all(&self.conn)
            .await?;
        Ok(list.into_iter().map(Into::into).collect())
    }

    async fn count_user_labels(&self, user_id: i64) -> Result<u64> {
        let count = UserLabelEntity::find()
            .filter(UserLabelColumn::UserId.eq(user_id))
            .count(&self.conn)
            .await?;
        Ok(count)
    }

    async fn create_user_label(&self, create: &UserLabelCreate) -> Result<UserLabel> {
        let now = Local::now();

        let active = UserLabelActiveModel {
            id: NotSet, // database auto-increment
            user_id: Set(create.user_id),
            label_id: Set(create.label_id),
            enabled: Set(true),
            created_at: Set(now.to_utc()),
            updated_at: Set(now.to_utc()),
        };

        let model: UserLabelModel = active.insert(&self.conn).await?;
        Ok(model.into())
    }

    async fn delete_user_label(&self, user_label_id: i64) -> Result<()> {
        UserLabelEntity::delete_by_id(user_label_id)
            .exec(&self.conn)
            .await?;
        Ok(())
    }

    async fn delete_user_label_by_ids(&self, user_id: i64, label_id: i64) -> Result<()> {
        UserLabelEntity::delete_many()
            .filter(UserLabelColumn::UserId.eq(user_id))
            .filter(UserLabelColumn::LabelId.eq(label_id))
            .exec(&self.conn)
            .await?;
        Ok(())
    }
}
