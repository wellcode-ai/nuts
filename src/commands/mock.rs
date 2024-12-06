use crate::collections::OpenAPISpec;
use std::error::Error;
use std::net::SocketAddr;
use axum::{
    Router,
    routing::{get, post, put, delete, patch},
    Json, extract::Path,
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
use axum::middleware::map_response;
use axum::response::Response;
use axum::http::Request;
use tracing_subscriber::{self, fmt::format::FmtSpan};

pub struct MockServer {
    spec: OpenAPISpec,
    port: u16,
}

impl MockServer {
    pub fn new(spec: OpenAPISpec, port: u16) -> Self {
        Self { spec, port }
    }

    pub async fn start(&self) -> Result<(), Box<dyn Error>> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_span_events(FmtSpan::CLOSE)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_level(true)
            .init();

        let app = self.build_router()
            .layer(TraceLayer::new_for_http()
                .on_request(|req: &Request<_>, _: &tracing::Span| {
                    info!("→ {} {}", req.method(), req.uri());
                })
                .on_response(|res: &Response, latency: Duration, _: &tracing::Span| {
                    let status = res.status();
                    if status.is_success() {
                        info!("← {} ({:?})", status, latency);
                    } else if status.is_client_error() {
                        warn!("← {} ({:?})", status, latency);
                    } else if status.is_server_error() {
                        error!("← {} ({:?})", status, latency);
                    }
                }));

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        
        info!("🎭 Starting mock server on http://{}", addr);
        info!("📚 Loaded {} endpoints from OpenAPI spec", self.spec.paths.len());

        let listener = TcpListener::bind(&addr).await?;
        
        // Create shutdown signal handler
        let shutdown = async {
            signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
            println!("\n🛑 Shutting down mock server...");
        };

        // Run server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await?;

        println!("✅ Server stopped gracefully");
        Ok(())
    }

    fn build_router(&self) -> Router {
        let mut router = Router::new();

        // Add routes for each path in the OpenAPI spec
        for (path, item) in &self.spec.paths {
            // Extract path from full URL and ensure it starts with /
            let path = if let Ok(url) = url::Url::parse(path) {
                url.path().to_string()
            } else {
                // If not a full URL, ensure path starts with /
                if !path.starts_with('/') {
                    format!("/{}", path)
                } else {
                    path.to_string()
                }
            };

            let path = self.convert_path_to_axum(&path);
            info!("Adding mock endpoint: {}", path);

            let examples = item.mock_data.as_ref()
                .and_then(|m| m.examples.as_ref())
                .cloned()
                .unwrap_or_default();

            // Add handlers for each HTTP method
            if item.get.is_some() {
                let examples = examples.clone();
                router = router.route(&path, get(move || handle_mock(examples.clone())));
            }
            if item.post.is_some() {
                let examples = examples.clone();
                router = router.route(&path, post(move || handle_mock(examples.clone())));
            }
            if item.put.is_some() {
                let examples = examples.clone();
                router = router.route(&path, put(move || handle_mock(examples.clone())));
            }
            if item.delete.is_some() {
                let examples = examples.clone();
                router = router.route(&path, delete(move || handle_mock(examples.clone())));
            }
            if item.patch.is_some() {
                let examples = examples.clone();
                router = router.route(&path, patch(move || handle_mock(examples.clone())));
            }
        }

        router
    }

    fn convert_path_to_axum(&self, path: &str) -> String {
        // Convert OpenAPI path params ({param}) to Axum format (:param)
        path.replace('{', ":").replace('}', "")
    }
}

async fn handle_mock(examples: Vec<String>) -> impl IntoResponse {
    if examples.is_empty() {
        warn!("No mock examples found for endpoint");
        return (
            StatusCode::NOT_IMPLEMENTED,
            Json(serde_json::json!({
                "error": "No mock data available",
                "code": 501,
                "message": "This endpoint has no mock examples configured"
            }))
        ).into_response();
    }

    // Randomly select one example
    use rand::seq::SliceRandom;
    match examples.choose(&mut rand::thread_rng())
        .and_then(|e| serde_json::from_str::<Value>(e).ok()) 
    {
        Some(example) => {
            info!("Serving mock response");
            (StatusCode::OK, Json(example)).into_response()
        }
        None => {
            error!("Failed to parse mock example as JSON");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Invalid mock data",
                    "code": 500,
                    "message": "Failed to parse mock example"
                }))
            ).into_response()
        }
    }
}

// Add a catch-all handler for unmatched routes
async fn handle_not_found() -> impl IntoResponse {
    warn!("Endpoint not found");
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Not Found",
            "code": 404,
            "message": "The requested endpoint does not exist"
        }))
    )
} 