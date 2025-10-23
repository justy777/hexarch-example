mod handlers;

use crate::http::handlers::{
    create_author, delete_author, find_all_authors, find_author, update_author,
};

use crate::repositories::AuthorRepository;
use anyhow::Context;
use axum::Router;
use axum::routing::get;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    author_repo: Arc<dyn AuthorRepository>,
}

impl AppState {
    pub fn new(author_repo: impl AuthorRepository) -> Self {
        Self {
            author_repo: Arc::new(author_repo),
        }
    }
}

#[derive(Debug)]
pub struct HttpServerConfig {
    port: u16,
}

impl HttpServerConfig {
    #[must_use]
    pub const fn new(port: u16) -> Self {
        Self { port }
    }
}

pub struct HttpServer {
    router: Router,
    listener: TcpListener,
}

impl HttpServer {
    pub async fn new(state: AppState, config: HttpServerConfig) -> anyhow::Result<Self> {
        let trace_layer =
            TraceLayer::new_for_http().make_span_with(|request: &axum::extract::Request<_>| {
                let uri = request.uri().to_string();
                tracing::info_span!("http_request", method = ?request.method(), uri)
            });

        let router = Router::new()
            .nest("/api/v1", api_routes())
            .layer(trace_layer)
            .with_state(state);

        let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
            .await
            .with_context(|| format!("Failed to bind to port {}", config.port))?;

        Ok(Self { router, listener })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("Listening on {}", self.listener.local_addr()?);
        axum::serve(self.listener, self.router)
            .await
            .context("Received error from running server")?;
        Ok(())
    }
}

fn api_routes() -> Router<AppState> {
    let author_routes = Router::new()
        .route("/", get(find_all_authors).post(create_author))
        .route(
            "/{id}",
            get(find_author).patch(update_author).delete(delete_author),
        );
    Router::new().nest("/authors", author_routes)
}
