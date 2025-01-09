use anyhow::Context;
use axum::Router;
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct HttpServerConfig {
    port: String,
}

impl HttpServerConfig {
    pub fn new(port: &str) -> Self {
        Self { port: port.into() }
    }
}

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    pub async fn new(state: AppState, config: HttpServerConfig) -> anyhow::Result<Self> {
        let router = Router::new()
            .nest("/api/v1", api_routes())
            .with_state(state);

        let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("Failed to bind to port {}", config.port))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        axum::serve(self.listener, self.router)
            .await
            .context("Received error from running server")?;
        Ok(())
    }
}

fn api_routes() -> Router<AppState> {
    Router::new()
}

