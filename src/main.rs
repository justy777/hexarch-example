use hexarch_example::config::Config;
use hexarch_example::http::{AppState, HttpServer, HttpServerConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    let state = AppState::new();
    let server_config = HttpServerConfig::new(config.server_port());
    let http_server = HttpServer::new(state, server_config).await?;
    http_server.run().await
}
