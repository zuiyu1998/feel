use feel_api::app_route;
use feel_core::tokio;
use poem::{Route, Server, listener::TcpListener};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().nest("/api/v1", app_route());
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
