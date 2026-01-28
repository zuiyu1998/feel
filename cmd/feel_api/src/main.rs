use feel_api::{app_route, init_app_state};
use feel_core::tokio;
use poem::{EndpointExt, Route, Server, listener::TcpListener};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app_state = init_app_state();

    let app = Route::new().nest("/api/v1", app_route()).data(app_state);
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
