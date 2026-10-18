pub use sea_orm_migration::prelude::*;

mod m20260207_083750_add_users;
mod m20260208_000000_add_user_credentials;
mod m20260209_000000_add_labels;
mod m20260209_000001_add_user_labels;
mod m20260210_000000_add_content;
mod m20260211_000000_add_content_view_count;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260207_083750_add_users::Migration),
            Box::new(m20260208_000000_add_user_credentials::Migration),
            Box::new(m20260209_000000_add_labels::Migration),
            Box::new(m20260209_000001_add_user_labels::Migration),
            Box::new(m20260210_000000_add_content::Migration),
            Box::new(m20260211_000000_add_content_view_count::Migration),
        ]
    }
}
