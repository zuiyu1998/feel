pub mod user;

use feel_storage::database::{CommonUserDataBase, UserDataBase};
use poem::Route;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
}

pub fn init_app_state() -> AppState {
    let user_database = Arc::new(CommonUserDataBase::new());

    AppState { user_database }
}

pub fn app_route() -> Route {
    Route::new().nest("/user", user::router())
}
