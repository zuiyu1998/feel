use feel_entity::UserDataBase;
use std::sync::Arc;

pub struct AppState {
    pub user_database: Arc<dyn UserDataBase>,
}
