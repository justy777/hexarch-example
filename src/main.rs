use hexarch_example::config::Config;
use hexarch_example::http::{AppState, HttpServer, HttpServerConfig};
use hexarch_example::sqlite::{DefaultAuthorRepository, create_pool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::from_env()?;

    tracing_subscriber::fmt::init();

    let pool = create_pool(config.database_url()).await?;
    let repo = DefaultAuthorRepository::new(pool);
    let state = AppState::new(repo);

    let server_config = HttpServerConfig::new(config.server_port());
    let http_server = HttpServer::new(state, server_config).await?;
    http_server.run().await
}
