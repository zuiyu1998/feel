pub mod auth;
pub mod content;
pub mod feed;
pub mod follow;
pub mod label;
pub mod model;
pub mod user;

use feel_storage::{
    cache::{content::CommonContentCache, feed::CommonUserFeedCache, user::CommonUserCache},
    database::{
        CommonContentDataBase, CommonLabelDataBase, CommonUserDataBase, CommonUserFeedDataBase,
        CommonUserFollowDataBase, ContentDataBase, LabelDataBase, UserDataBase, UserFeedDataBase,
        UserFollowDataBase,
    },
    repo::{SeaOrmContentRepo, SeaOrmLabelRepo, SeaOrmUserFollowRepo, SeaOrmUserRepo},
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
    pub label_database: Arc<dyn LabelDataBase>,
    pub content_database: Arc<dyn ContentDataBase>,
    pub user_follow_database: Arc<dyn UserFollowDataBase>,
    pub feed_database: Arc<dyn UserFeedDataBase>,
}

pub async fn init_app_state(config: &ApiConfig) -> AppState {
    let conn = Database::connect(&config.database_url)
        .await
        .expect("Database connect failed.");

    // 执行数据库迁移
    migration::Migrator::up(&conn, None)
        .await
        .expect("Database migration failed.");

    // 存储层:user / label / content / follow 共享同一数据库连接
    let user_repo = SeaOrmUserRepo::new(conn.clone());
    let content_repo = SeaOrmContentRepo::new(conn.clone());
    let follow_repo = SeaOrmUserFollowRepo::new(conn.clone());
    let label_repo = SeaOrmLabelRepo::new(conn.clone());
    // 关注流使用独立 repo 实例（共享同一连接池）
    let feed_content_repo = SeaOrmContentRepo::new(conn.clone());
    let feed_follow_repo = SeaOrmUserFollowRepo::new(conn.clone());

    let redis_client =
        redis::Client::open(&*config.redis_url).expect("Redis client create failed.");
    let user_cache = Box::new(CommonUserCache::new(redis_client.clone()));
    let content_cache = CommonContentCache::new(redis_client.clone());
    let feed_cache = CommonUserFeedCache::new(redis_client);

    // 数据访问层
    let user_database = Arc::new(CommonUserDataBase::new(
        user_repo,
        user_cache,
        &config.jwt_secret,
    ));
    let label_database = Arc::new(CommonLabelDataBase::new(label_repo));
    let content_database = Arc::new(CommonContentDataBase::new(content_repo, content_cache));
    let user_follow_database = Arc::new(CommonUserFollowDataBase::new(follow_repo));
    let feed_database = Arc::new(CommonUserFeedDataBase::new(
        feed_cache,
        feed_follow_repo,
        feed_content_repo,
    ));

    AppState {
        user_database,
        label_database,
        content_database,
        user_follow_database,
        feed_database,
    }
}

pub fn app_route() -> Route {
    Route::new()
        .nest("/users", user::router())
        .nest("/labels", label::router())
        .nest("/content", content::router())
        .nest("/follow", follow::router())
        .nest("/feed", feed::router())
}
