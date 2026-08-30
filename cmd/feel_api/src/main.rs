use feel_api::{ApiConfig, app_route, init_app_state};
use feel_core::tokio;
use poem::{EndpointExt, Route, Server, listener::TcpListener};

/// HTTP 服务监听地址。
const LISTEN_ADDR: &str = "0.0.0.0:3000";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = ApiConfig::default();

    tracing::info!("database: {}", config.database_url);
    tracing::info!("redis: {}", config.redis_url);

    // 初始化应用状态:建立数据库连接、执行迁移、组装各数据访问层
    tracing::info!("initializing app state...");
    let app_state = init_app_state(&config).await;

    let app = Route::new().nest("/api/v1", app_route()).data(app_state);

    // 启动 HTTP 服务
    tracing::info!("HTTP server listening on http://{LISTEN_ADDR} (API prefix: /api/v1)");
    Server::new(TcpListener::bind(LISTEN_ADDR)).run(app).await
}
