use crate::collections::OpenAPISpec;
use std::error::Error;
use std::net::SocketAddr;
use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use tokio::net::TcpListener;
use tracing::{info, warn, error};
use url;
use tokio::signal;
use tower_http::trace::TraceLayer;
use std::time::Duration;
use axum::response::Response;
use axum::http::Request;
use tracing_subscriber::{self, fmt::format::FmtSpan};
use std::collections::HashMap;
use std::sync::Arc;
use rand::Rng;
use axum::extract::Path;
use axum_server::Server;

pub struct MockServer {
    spec: OpenAPISpec,
    port: u16,
}

impl MockServer {
    pub fn new(spec: OpenAPISpec, port: u16) -> Self {
        Self { spec, port }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut router = Router::new();

        // Add routes for each path in the spec
        for (path, item) in &self.spec.paths {
            let clean_path = path.replace("{id}", ":id");
            println!("Adding mock endpoint: {}", clean_path);

            let mock_examples = if let Some(op) = &item.get {
                op.mock_data.as_ref()
                    .and_then(|m| m.examples.as_ref())
                    .cloned()
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let examples = Arc::new(mock_examples);
            
            router = router.route(&clean_path, get(move |_params: Path<HashMap<String, String>>| {
                let examples = examples.clone();
                async move {
                    if examples.is_empty() {
                        (StatusCode::NOT_IMPLEMENTED, String::from("No mock examples found"))
                    } else {
                        let idx = rand::random::<usize>() % examples.len();
                        (StatusCode::OK, examples[idx].clone())
                    }
                }
            }));
        }

        println!("🎭 Starting mock server on http://127.0.0.1:{}", self.port);
        println!("📚 Loaded {} endpoints from OpenAPI spec", self.spec.paths.len());

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        Server::bind(addr)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }
}
