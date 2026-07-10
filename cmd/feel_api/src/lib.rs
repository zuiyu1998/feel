pub mod user;

use feel_storage::{
    database::{CommonUserDataBase, UserDataBase},
    repo::SeaOrmUserRepo,
};
use poem::Route;
use sea_orm::Database;
use std::sync::Arc;

pub struct ApiConfig {
    pub database_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://postgres:bj123456@192.168.0.107:5432/feel".to_string(),
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

    let user_repo = SeaOrmUserRepo::new(conn);

    let user_database = Arc::new(CommonUserDataBase::new(user_repo));

    AppState { user_database }
}

pub fn app_route() -> Route {
    Route::new().nest("/user", user::router())
}
