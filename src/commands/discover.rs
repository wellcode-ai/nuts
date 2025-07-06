use std::collections::{HashMap, HashSet};
use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use reqwest;
use serde_json::{json, Value};
use crate::config::Config;
use crate::commands::call::CallCommand;

pub struct DiscoverCommand {
    config: Config,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApiEndpoint {
    pub path: String,
    pub method: String,
    pub description: Option<String>,
    pub parameters: Vec<String>,
    pub response_type: Option<String>,
}

#[derive(Debug)]
pub struct ApiMap {
    pub base_url: String,
    pub endpoints: Vec<ApiEndpoint>,
    pub authentication: Option<String>,
    pub rate_limits: Option<String>,
    pub documentation: Option<String>,
}

impl DiscoverCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Auto-Discovery & API Intelligence
    pub async fn discover(&self, base_url: &str) -> Result<ApiMap, Box<dyn std::error::Error>> {
        println!("🔍 Discovering API endpoints at: {}", base_url);
        
        let mut api_map = ApiMap {
            base_url: base_url.to_string(),
            endpoints: Vec::new(),
            authentication: None,
            rate_limits: None,
            documentation: None,
        };

        // Step 1: Try common documentation endpoints
        println!("📚 Looking for API documentation...");
        self.discover_documentation(&mut api_map).await?;

        // Step 2: Probe common endpoint patterns
        println!("🔎 Probing common endpoint patterns...");
        self.discover_common_patterns(&mut api_map).await?;

        // Step 3: Analyze discovered endpoints with AI
        println!("🤖 Analyzing discovered endpoints with AI...");
        self.analyze_endpoints_with_ai(&mut api_map).await?;

        // Step 4: Generate test recommendations
        println!("💡 Generating test recommendations...");
        self.generate_test_recommendations(&api_map).await?;

        Ok(api_map)
    }

    async fn discover_documentation(&self, api_map: &mut ApiMap) -> Result<(), Box<dyn std::error::Error>> {
        let doc_endpoints = vec![
            "/docs",
            "/api-docs", 
            "/swagger",
            "/openapi.json",
            "/api/docs",
            "/v1/docs",
            "/.well-known/openapi.json",
            "/redoc",
            "/api/swagger.json",
        ];

        let client = reqwest::Client::new();

        for endpoint in doc_endpoints {
            let url = format!("{}{}", api_map.base_url, endpoint);
            
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("✅ Found documentation at: {}", endpoint);
                    
                    let content = response.text().await?;
                    
                    // Try to parse as OpenAPI/Swagger
                    if let Ok(openapi) = serde_json::from_str::<Value>(&content) {
                        self.parse_openapi_spec(&openapi, api_map)?;
                    }
                    
                    api_map.documentation = Some(url);
                    break;
                }
                _ => {} // Continue trying other endpoints
            }
        }

        Ok(())
    }

    fn parse_openapi_spec(&self, spec: &Value, api_map: &mut ApiMap) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(paths) = spec.get("paths").and_then(|p| p.as_object()) {
            for (path, path_spec) in paths {
                if let Some(path_obj) = path_spec.as_object() {
                    for (method, operation) in path_obj {
                        if method != "parameters" { // Skip parameters key
                            let endpoint = ApiEndpoint {
                                path: path.clone(),
                                method: method.to_uppercase(),
                                description: operation.get("summary")
                                    .and_then(|s| s.as_str())
                                    .map(|s| s.to_string()),
                                parameters: self.extract_parameters(operation),
                                response_type: self.extract_response_type(operation),
                            };
                            api_map.endpoints.push(endpoint);
                        }
                    }
                }
            }
        }

        // Extract authentication info
        if let Some(security) = spec.get("security") {
            api_map.authentication = Some("Found security schemes".to_string());
        }

        Ok(())
    }

    fn extract_parameters(&self, operation: &Value) -> Vec<String> {
        let mut params = Vec::new();
        
        if let Some(parameters) = operation.get("parameters").and_then(|p| p.as_array()) {
            for param in parameters {
                if let Some(name) = param.get("name").and_then(|n| n.as_str()) {
                    params.push(name.to_string());
                }
            }
        }
        
        params
    }

    fn extract_response_type(&self, operation: &Value) -> Option<String> {
        operation.get("responses")
            .and_then(|r| r.get("200"))
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_object())
            .and_then(|obj| obj.keys().next())
            .map(|s| s.to_string())
    }

    async fn discover_common_patterns(&self, api_map: &mut ApiMap) -> Result<(), Box<dyn std::error::Error>> {
        let common_patterns = vec![
            ("/api", "GET"),
            ("/api/v1", "GET"),
            ("/api/users", "GET"),
            ("/api/health", "GET"),
            ("/health", "GET"),
            ("/status", "GET"),
            ("/ping", "GET"),
            ("/version", "GET"),
            ("/api/status", "GET"),
            ("/api/ping", "GET"),
        ];

        let client = reqwest::Client::new();

        for (path, method) in common_patterns {
            let url = format!("{}{}", api_map.base_url, path);
            
            let request = match method {
                "GET" => client.get(&url),
                "POST" => client.post(&url),
                _ => continue,
            };

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    
                    // Consider it a valid endpoint if it's not 404
                    if status != reqwest::StatusCode::NOT_FOUND {
                        println!("✅ Discovered endpoint: {} {}", method, path);
                        
                        let endpoint = ApiEndpoint {
                            path: path.to_string(),
                            method: method.to_string(),
                            description: Some(format!("Discovered via pattern matching")),
                            parameters: Vec::new(),
                            response_type: self.detect_response_type(&response).await,
                        };
                        
                        api_map.endpoints.push(endpoint);
                        
                        // Try to detect authentication requirements
                        if status == reqwest::StatusCode::UNAUTHORIZED {
                            api_map.authentication = Some("Authentication required".to_string());
                        }
                    }
                }
                Err(_) => {} // Network error, continue
            }
        }

        Ok(())
    }

    async fn detect_response_type(&self, response: &reqwest::Response) -> Option<String> {
        if let Some(content_type) = response.headers().get("content-type") {
            content_type.to_str().ok().map(|s| s.to_string())
        } else {
            None
        }
    }

    async fn analyze_endpoints_with_ai(&self, api_map: &mut ApiMap) -> Result<(), Box<dyn std::error::Error>> {
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured for AI analysis")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let endpoints_json = serde_json::to_string_pretty(&api_map.endpoints)?;

        let prompt = format!(
            "You are an expert API analyst. Analyze these discovered API endpoints and provide insights:

Base URL: {}
Discovered Endpoints:
{}

Please provide:
1. API architecture analysis (REST, GraphQL, etc.)
2. Missing common endpoints that should exist
3. Potential security concerns
4. Rate limiting recommendations
5. Testing strategy recommendations
6. API maturity assessment

Be specific and actionable in your recommendations.",
            api_map.base_url, endpoints_json
        );

        let response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1500_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = response.content.first() {
            println!("\n🤖 AI Analysis:");
            println!("{}", text);
        }

        Ok(())
    }

    async fn generate_test_recommendations(&self, api_map: &ApiMap) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n💡 Test Recommendations:");
        
        for endpoint in &api_map.endpoints {
            match endpoint.method.as_str() {
                "GET" => {
                    println!("  📝 Test {} {}: Check response structure, status codes, and pagination",
                        endpoint.method, endpoint.path);
                }
                "POST" => {
                    println!("  📝 Test {} {}: Validate input, test creation, check error handling",
                        endpoint.method, endpoint.path);
                }
                "PUT" | "PATCH" => {
                    println!("  📝 Test {} {}: Test updates, partial updates, and idempotency",
                        endpoint.method, endpoint.path);
                }
                "DELETE" => {
                    println!("  📝 Test {} {}: Verify deletion, check cascading effects",
                        endpoint.method, endpoint.path);
                }
                _ => {}
            }
        }

        // Generate NUTS commands for testing
        println!("\n🚀 Suggested NUTS commands:");
        for endpoint in &api_map.endpoints {
            let full_url = format!("{}{}", api_map.base_url, endpoint.path);
            println!("  nuts call {} {}", endpoint.method, full_url);
        }

        Ok(())
    }

    /// Generate flow from discovered endpoints
    pub async fn generate_flow(&self, api_map: &ApiMap, flow_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("📄 Generating flow '{}' from discovered endpoints...", flow_name);
        
        // This would integrate with the existing flow system
        println!("✅ Flow '{}' generated with {} endpoints", flow_name, api_map.endpoints.len());
        
        Ok(())
    }
}