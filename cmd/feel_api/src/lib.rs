pub mod user;

use feel_storage::UserRepo;
use poem::Route;
use std::sync::Arc;

pub struct AppState {
    pub user_database: Arc<dyn UserRepo>,
}

pub fn app_route() -> Route {
    Route::new().nest("/user", user::router())
}
