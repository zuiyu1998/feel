use feel_sea_orm::user::entities::user_credentials::{
    Column as UserCredentialsColumn, Entity as UserCredentials,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let table = Table::create()
            .table(UserCredentials)
            .if_not_exists()
            .col(
                ColumnDef::new(UserCredentialsColumn::Id)
                    .big_integer()
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::UserUid)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::CredentialType)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::CredentialName)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::EncryptedData)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::EncryptionKey)
                    .string()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::IsEnabled)
                    .boolean()
                    .not_null()
                    .default(true),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(
                ColumnDef::new(UserCredentialsColumn::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .index(
                &mut Index::create()
                    .name("idx_user_credentials_user_uid_credential_name")
                    .col(UserCredentialsColumn::UserUid)
                    .col(UserCredentialsColumn::CredentialName)
                    .unique()
                    .to_owned(),
            )
            .to_owned();

        manager.create_table(table).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserCredentials).to_owned())
            .await
    }
}
