pub mod model;
pub mod user;

use feel_storage::{
    cache::user::CommonUserCache,
    database::{CommonUserDataBase, UserDataBase},
    repo::SeaOrmUserRepo,
};
use migration::MigratorTrait;
use poem::Route;
use sea_orm::Database;
use std::sync::Arc;

pub struct ApiConfig {
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:bj123456@192.168.0.107:5432/feel".to_string(),
            redis_url: "redis://192.168.0.107:6379".to_string(),
            jwt_secret: "feel-jwt-secret".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
}

pub async fn init_app_state(config: &ApiConfig) -> AppState {
    let conn = Database::connect(&config.database_url)
        .await
        .expect("Database connect failed.");

    // 执行数据库迁移
    migration::Migrator::up(&conn, None)
        .await
        .expect("Database migration failed.");

    let user_repo = SeaOrmUserRepo::new(conn);

    let redis_client =
        redis::Client::open(&*config.redis_url).expect("Redis client create failed.");
    let user_cache = Box::new(CommonUserCache::new(redis_client));

    let user_database = Arc::new(CommonUserDataBase::new(
        user_repo,
        user_cache,
        &config.jwt_secret,
    ));

    AppState { user_database }
}

pub fn app_route() -> Route {
    Route::new().nest("/user", user::router())
}
