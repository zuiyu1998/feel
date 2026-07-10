use feel_api::{ApiConfig, app_route, init_app_state};
use feel_core::tokio;
use poem::{EndpointExt, Route, Server, listener::TcpListener};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = ApiConfig::default();

    tracing::info!("Starting server at {}", config.database_url);

    let app_state = init_app_state(&config).await;

    let app = Route::new().nest("/api/v1", app_route()).data(app_state);
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
}
