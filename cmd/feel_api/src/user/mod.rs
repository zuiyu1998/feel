use poem::{Route, handler, post};

pub fn router() -> Route {
    Route::new()
        .at("/register", post(register))
        .at("/unregister", post(unregister))
        .at("/login", post(login))
        .at("/logout", post(login))
}

#[handler]
async fn register() {}

#[handler]
async fn unregister() {}

#[handler]
async fn login() {}

#[handler]
async fn logout() {}
