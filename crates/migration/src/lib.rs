pub use sea_orm_migration::prelude::*;

mod m20260207_083750_add_users;
mod m20260208_000000_add_user_credentials;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260207_083750_add_users::Migration),
            Box::new(m20260208_000000_add_user_credentials::Migration),
        ]
    }
}
