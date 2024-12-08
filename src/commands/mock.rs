use crate::collections::{OpenAPISpec, Operation};
use std::error::Error;
use std::net::SocketAddr;
use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::{Value, json};
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
use std::future::Future;

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

            // Handle each HTTP method
            if let Some(op) = &item.get {
                let examples = Arc::new(Self::get_mock_examples(op));
                router = router.route(&clean_path, get(move |params| Self::handle_request(examples.clone(), params)));
            }
            if let Some(op) = &item.post {
                let examples = Arc::new(Self::get_mock_examples(op));
                router = router.route(&clean_path, post(move |params| Self::handle_request(examples.clone(), params)));
            }
            // Add other methods similarly
        }

        println!("🎭 Starting mock server on http://127.0.0.1:{}", self.port);
        println!("📚 Loaded {} endpoints from OpenAPI spec", self.spec.paths.len());

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        Server::bind(addr)
            .serve(router.into_make_service())
            .await?;

        Ok(())
    }

    fn get_mock_examples(op: &Operation) -> Vec<String> {
        op.mock_data.as_ref()
            .and_then(|m| m.examples.as_ref())
            .cloned()
            .unwrap_or_default()
    }

    async fn handle_request(examples: Arc<Vec<String>>, _params: Path<HashMap<String, String>>) -> (StatusCode, Json<Value>) {
        if examples.is_empty() {
            (StatusCode::NOT_IMPLEMENTED, Json(json!({
                "error": "No mock examples found"
            })))
        } else {
            let idx = rand::random::<usize>() % examples.len();
            let example = &examples[idx];
            match serde_json::from_str(example) {
                Ok(json) => (StatusCode::OK, Json(json)),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "Invalid JSON in mock data"
                })))
            }
        }
    }
}
