use crate::store::AuthorRepository;
use anyhow::Context;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, Clone)]
pub struct AppState<AR: AuthorRepository> {
    author_repo: Arc<AR>,
}

impl<AR: AuthorRepository> AppState<AR> {
    pub fn new(author_repo: AR) -> Self {
        Self { author_repo: Arc::new(author_repo) }
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
    pub async fn new<AR: AuthorRepository>(
        state: AppState<AR>,
        config: HttpServerConfig,
    ) -> anyhow::Result<Self> {
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

fn api_routes<AR: AuthorRepository>() -> Router<AppState<AR>> {
    Router::new()
}
