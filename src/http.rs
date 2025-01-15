mod handler;

use crate::http::handler::{create_author, find_all_authors, find_author};
use crate::store::AuthorRepository;
use anyhow::Context;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone)]
pub struct AppState<AR: AuthorRepository> {
    author_repo: Arc<AR>,
}

impl<AR: AuthorRepository> AppState<AR> {
    pub fn new(author_repo: AR) -> Self {
        Self {
            author_repo: Arc::new(author_repo),
        }
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

fn api_routes<AR: AuthorRepository>() -> Router<AppState<AR>> {
    Router::new()
        .route("/authors", get(find_all_authors).post(create_author))
        .route("/authors/{id}", get(find_author))
}
