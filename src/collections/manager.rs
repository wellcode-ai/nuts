use crate::collections::*;
use crate::commands::perf::PerfCommand;
use rustyline::Editor;
use std::path::PathBuf;
use std::fs;
use std::time::Duration;
use std::collections::HashMap;
use crate::commands::call::CallCommand;
use crate::commands::mock::MockServer;
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use console::style;
use crate::config::Config;
use url;

pub struct CollectionManager {
    collections_dir: PathBuf,
    config: Config,
    ai_client: AnthropicClient,
}

impl CollectionManager {
    pub fn new(collections_dir: PathBuf, config: Config) -> Self {
        let api_key = config.anthropic_api_key.clone()
            .unwrap_or_default();

        Self {
            collections_dir,
            config,
            ai_client: ClientBuilder::default()
                .api_key(api_key)
                .build()
                .unwrap(),
        }
    }

    fn get_collection_path(&self, name: &str) -> PathBuf {
        self.collections_dir.join(format!("{}.yaml", name))
    }

    pub fn create_collection(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.get_collection_path(name);
        
        let template = OpenAPISpec::new(name);
        template.save(&path)?;
        
        println!("✅ Created OpenAPI collection at: {}", path.display());
        Ok(())
    }

    pub async fn add_endpoint(
        &self,
        collection: &str,
        method: &str,
        path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.get_collection_path(collection);
        let mut spec = OpenAPISpec::load(&spec_path)?;

        // Parse and clean the URL/path
        let (server_url, clean_path) = if path.starts_with("http") {
            let url = url::Url::parse(path)?;
            let base = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));
            (base, url.path().to_string())
        } else {
            ("http://localhost:3000".to_string(), 
             if path.starts_with('/') { path.to_string() } else { format!("/{}", path) })
        };

        // Update servers
        if !spec.servers.iter().any(|s| s.url == server_url) {
            spec.servers.push(Server {
                url: server_url,
                description: Some("API Server".to_string()),
            });
        }

        // Create path item
        let path_item = spec.paths.entry(clean_path.clone()).or_insert(PathItem::new());

        // Create operation with better defaults
        let operation = Operation {
            summary: Some(format!("{} {}", method, &clean_path)),
            description: Some("API endpoint".to_string()),
            parameters: None,
            request_body: if ["POST", "PUT", "PATCH"].contains(&method) {
                Some(RequestBody {
                    description: Some("Request payload".to_string()),
                    required: Some(true),
                    content: {
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: "object".to_string(),
                                format: None,
                                properties: None,
                                items: None,
                            },
                            example: Some(serde_json::json!({})),
                        });
                        content
                    },
                })
            } else {
                None
            },
            responses: {
                let mut responses = HashMap::new();
                responses.insert("200".to_string(), Response {
                    description: "Successful response".to_string(),
                    content: Some({
                        let mut content = HashMap::new();
                        content.insert("application/json".to_string(), MediaType {
                            schema: Schema {
                                schema_type: "object".to_string(),
                                format: None,
                                properties: None,
                                items: None,
                            },
                            example: None,
                        });
                        content
                    }),
                });
                responses
            },
            security: None,
            tags: Some(vec![clean_path.split('/').nth(1).unwrap_or("default").to_string()]),
            mock_data: None,
        };

        // Add operation to path item
        match method {
            "GET" => path_item.get = Some(operation),
            "POST" => path_item.post = Some(operation),
            "PUT" => path_item.put = Some(operation),
            "DELETE" => path_item.delete = Some(operation),
            "PATCH" => path_item.patch = Some(operation),
            _ => return Err("Unsupported HTTP method".into()),
        }

        spec.save(&spec_path)?;
        println!("✅ Added {} endpoint {} to collection", method, clean_path);
        Ok(())
    }

    pub async fn run_endpoint(
        &self,
        collection: &str,
        endpoint: &str,
        _args: &[String]
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.get_collection_path(collection);
        let spec = OpenAPISpec::load(&spec_path)?;

        // Find the endpoint in the spec
        let (path, item) = spec.paths.iter()
            .find(|(p, _)| p.contains(endpoint))
            .ok_or("Endpoint not found in collection")?;

        // Determine method and operation
        let (method, _operation) = item.get_operation()
            .ok_or("No operation found for endpoint")?;

        // Build the full URL
        let base_url = spec.servers.first()
            .map(|s| s.url.as_str())
            .unwrap_or("http://localhost:3000");
        let full_url = format!("{}{}", base_url, path);

        // Execute the request
        println!(" Executing {} {}", method, full_url);
        CallCommand::new().execute(&[method, &full_url]).await?;
        Ok(())
    }

    pub async fn start_mock_server(
        &self,
        name: &str,
        port: u16
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.get_collection_path(name);
        let spec = OpenAPISpec::load(&spec_path)?;
        
        println!("Starting mock server for {} on port {}", name, port);
        MockServer::new(spec, port).start().await?;
        Ok(())
    }

    pub async fn configure_mock_data(
        &self,
        collection: &str,
        endpoint: &str,
        editor: &mut Editor<impl rustyline::Helper, impl rustyline::history::History>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check for API key
        let api_key = self.config.anthropic_api_key.clone()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        // Verify API key is not empty
        if api_key.trim().is_empty() {
            return Err("API key is empty. Use 'config api-key' to set it".into());
        }

        let spec_path = self.get_collection_path(collection);
        let mut spec = OpenAPISpec::load(&spec_path)?;

        println!("Available endpoints:");
        for path in spec.paths.keys() {
            println!("  • {}", path);
        }

        // Find the endpoint by exact path match
        if let Some((_path, item)) = spec.paths.iter_mut().find(|(path, _)| path == &endpoint) {
            println!("⚙️  Analyzing endpoint and generating mock data...");

            // Improve the prompt with actual schema information
            let prompt = format!(
                "You are a mock data generator for API testing. Generate diverse test data examples for this endpoint.\n\
                URL: {}\n\
                Response Schema: {}\n\n\
                Generate 10 different examples in this format:\n\
                Description: <what this example tests>\n\
                {{\n  // JSON response example\n}}\n\n\
                Include examples for:\n\
                1. Happy path with typical data\n\
                2. Edge cases (empty values, very long values)\n\
                3. Special characters and Unicode\n\
                4. Error responses (404, 500)\n\
                5. Boundary testing\n\
                Make each example valid JSON.",
                endpoint,
                serde_json::to_string_pretty(&item.get.as_ref()
                    .and_then(|op| op.responses.get("200"))
                    .and_then(|resp| resp.content.as_ref())
                    .unwrap_or(&HashMap::new()))?
            );

            // Get AI response
            let messages = vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt.into() }]
            }];

            let messages_request = MessagesRequestBuilder::default()
                .messages(messages)
                .model("claude-3-sonnet-20240229".to_string())
                .max_tokens(2000_usize)
                .build()?;

            let response = self.ai_client.messages(messages_request).await?;
            
            // Debug the AI response
            if let Some(ContentBlock::Text { text }) = response.content.first() {
                println!("AI Response:\n{}", text);  // Debug print
                let examples = Self::parse_mock_examples(&text)?;
                if examples.is_empty() {
                    println!("⚠️  No valid examples could be parsed from AI response");
                } else {
                    // Save examples to the OpenAPI spec
                    let examples_clone = examples.clone();
                    item.mock_data = Some(MockDataConfig {
                        description: "AI-generated mock responses".to_string(),
                        schema: None,
                        examples: Some(examples),
                    });

                    spec.save(&spec_path)?;
                    println!("✅ Generated and saved {} mock examples", examples_clone.len());
                    
                    // Print example summaries
                    println!("\n📋 Generated mock examples:");
                    for (i, example) in examples_clone.iter().enumerate() {
                        println!("  {}. {}", i + 1, style(example).cyan());
                    }
                }
            }
        } else {
            println!("❌ Endpoint not found in collection: {}", endpoint);
        }

        Ok(())
    }

    fn parse_mock_examples(response: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut examples = Vec::new();
        let mut current_json = String::new();
        let mut in_json = false;

        for line in response.lines() {
            let line = line.trim();
            
            if line.contains("{") {
                in_json = true;
                current_json = line.to_string();
            } else if in_json {
                current_json.push_str("\n");
                current_json.push_str(line);
                
                if line.contains("}") {
                    in_json = false;
                    // Try to parse and validate JSON
                    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&current_json) {
                        examples.push(serde_json::to_string_pretty(&parsed)?);
                    }
                    current_json.clear();
                }
            }
        }

        if examples.is_empty() {
            // Fallback: create a default example
            examples.push(serde_json::to_string_pretty(&serde_json::json!({
                "id": 1,
                "title": "Example item",
                "completed": false
            }))?);
        }

        Ok(examples)
    }

    async fn generate_user_flow(&self, spec: &OpenAPISpec) -> Result<Vec<(String, String, Option<String>)>, Box<dyn std::error::Error>> {
        let mut endpoints = Vec::new();
        for (path, item) in &spec.paths {
            if let Some(op) = &item.get {
                endpoints.push(format!("GET {}\nDescription: {}\n", path, op.summary.as_deref().unwrap_or("")));
            }
            if let Some(op) = &item.post {
                endpoints.push(format!("POST {}\nDescription: {}\n", path, op.summary.as_deref().unwrap_or("")));
            }
            // Add other methods as needed
        }

        let prompt = format!(
            "You are an API testing expert. Analyze these endpoints and create a realistic test flow:\n\
            \n\
            Available Endpoints:\n{}\n\
            Create a sequence of 3-5 API calls that simulates a realistic user journey.\n\
            Focus on testing core functionality and common user paths.\n\
            Format each line as: METHOD /path [JSON body] | Brief explanation\n\
            Example: GET /users | List all users\n\
            Keep it focused and realistic.",
            endpoints.join("\n")
        );

        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt }],
        }];

        let message_request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-haiku-20240307".to_string())
            .max_tokens(800_usize)
            .build()?;

        let response = self.ai_client.messages(message_request).await?;
        
        if let Some(ContentBlock::Text { text }) = response.content.first() {
            let mut flow = Vec::new();
            for line in text.lines() {
                if let Some((call, explanation)) = line.split_once('|') {
                    let parts: Vec<&str> = call.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        let method = parts[0].to_string();
                        let path = parts[1].to_string();
                        let body = if parts.len() > 2 {
                            Some(parts[2..].join(" "))
                        } else {
                            None
                        };
                        println!("   • {} {} | {}", 
                            style(&method).cyan().to_string(),
                            style(&path).green().to_string(),
                            style(explanation.trim()).dim().to_string()
                        );
                        flow.push((method, path, body));
                    }
                }
            }
            Ok(flow)
        } else {
            Ok(Vec::new())
        }
    }

    async fn parse_options(options: &[String]) -> Result<(u32, Duration), Box<dyn std::error::Error>> {
        let users = options.iter()
            .position(|x| x == "--users")
            .and_then(|i| options.get(i + 1))
            .and_then(|u| u.parse().ok())
            .unwrap_or(10);

        let duration = options.iter()
            .position(|x| x == "--duration")
            .and_then(|i| options.get(i + 1))
            .and_then(|d| d.trim_end_matches('s').parse().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(30));

        Ok((users, duration))
    }

    pub async fn run_endpoint_perf(
        &self,
        collection: &str,
        endpoint: Option<&str>,
        options: &[String]
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.get_collection_path(collection);
        let spec = OpenAPISpec::load(&spec_path)?;
        let (users, duration) = Self::parse_options(options).await?;
        let base_url = spec.servers.first()
            .map(|s| s.url.as_str())
            .unwrap_or("http://localhost:8000");

        // If no specific endpoint is provided, analyze all endpoints
        if endpoint.is_none() {
            println!("🔍 Analyzing collection endpoints...");
            
            // Try AI flow generation if API key is available
            if self.config.api_key.is_some() {
                println!("🤖 Generating realistic test scenarios...\n");
                if let Ok(flow) = self.generate_user_flow(&spec).await {
                    if !flow.is_empty() {
                        let perf = PerfCommand::new(&self.config);
                        for (method, path, body) in flow {
                            println!("\n🚀 Testing {} {}", style(&method).cyan(), style(&path).green());
                            let url = if path.starts_with("http://") || path.starts_with("https://") {
                                path.to_string()
                            } else {
                                format!("{}{}", &base_url, &path)
                            };
                            perf.run(
                                &url,
                                users,
                                duration,
                                &method,
                                body.as_deref()
                            ).await?;
                        }
                        return Ok(());
                    }
                }
            }

            // Fallback to testing all GET endpoints
            println!("ℹ️  Testing all GET endpoints...");
            let perf = PerfCommand::new(&self.config);
            for (path, item) in &spec.paths {
                if let Some(op) = &item.get {
                    println!("\n🚀 Testing GET {}", style(path).green());
                    self.run_single_endpoint_test(path, "GET", users, duration, base_url).await?;
                }
                if let Some(op) = &item.post {
                    println!("\n🚀 Testing POST {}", style(path).green());
                    self.run_single_endpoint_test(path, "POST", users, duration, base_url).await?;
                }
            }
            return Ok(());
        }

        // Single endpoint test
        let endpoint = endpoint.unwrap();
        let item = spec.paths.iter()
            .find(|(p, _)| p.contains(endpoint))
            .ok_or("Endpoint not found in collection")?
            .1;
        
        let (method, _operation) = item.get_operation()
            .ok_or("No operation found for endpoint")?;

        let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", base_url, endpoint)
        };
        self.run_single_endpoint_test(&url, method, users, duration, base_url).await
    }

    pub async fn generate_openapi(
        &self,
        name: &str,
        format: &str
    ) -> Result<(), Box<dyn std::error::Error>> {
        let spec_path = self.get_collection_path(name);
        let mut spec = OpenAPISpec::load(&spec_path)?;

        println!("🤖 Analyzing API endpoints and generating documentation...");

        // Get API key from config
        let api_key = self.config.anthropic_api_key.clone()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        // Verify API key is not empty
        if api_key.trim().is_empty() {
            return Err("API key is empty. Use 'config api-key' to set it".into());
        }

        // Generate documentation for each endpoint
        for (path, item) in spec.paths.iter_mut() {
            if let Some(operation) = &mut item.get {
                let prompt = format!(
                    "You are a technical writer creating OpenAPI documentation. \
                    Analyze this API endpoint and generate a clear, detailed description:\n\
                    Path: {}\n\
                    Method: GET\n\
                    Response Example: {:?}\n\n\
                    Please provide:\n\
                    1. A concise summary (one line)\n\
                    2. A detailed description including:\n\
                       - What the endpoint does\n\
                       - Common use cases\n\
                       - Response structure explanation\n\
                       - Any important notes or considerations",
                    path,
                    operation.responses.get("200").and_then(|r| r.content.as_ref())
                );

                let messages = vec![Message {
                    role: Role::User,
                    content: vec![ContentBlock::Text { text: prompt.into() }]
                }];

                let messages_request = MessagesRequestBuilder::default()
                    .messages(messages)
                    .model("claude-3-sonnet-20240229".to_string())
                    .max_tokens(1000_usize)
                    .build()?;

                let response = self.ai_client.messages(messages_request).await?;
                
                if let Some(ContentBlock::Text { text }) = response.content.first() {
                    // Parse AI response into summary and description
                    let lines: Vec<&str> = text.lines().collect();
                    if let Some((summary, description)) = lines.split_first() {
                        operation.summary = Some(summary.trim().to_string());
                        operation.description = Some(description.join("\n").trim().to_string());
                    }
                }
            }
        }

        // Save the updated spec
        let output_path = match format {
            "json" => self.collections_dir.join(format!("{}.json", name)),
            _ => spec_path.clone(),
        };

        match format {
            "json" => {
                let json = serde_json::to_string_pretty(&spec)?;
                fs::write(&output_path, json)?;
            },
            "yaml" => {
                let yaml = serde_yaml::to_string(&spec)?;
                fs::write(&output_path, yaml)?;
            },
            _ => return Err("Unsupported format".into()),
        }

        println!("✅ Generated enhanced OpenAPI documentation: {}", output_path.display());
        Ok(())
    }

    pub async fn list_collections(&self) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(&self.collections_dir)? {
            let entry = entry?;
            if let Some(name) = entry.path().file_stem() {
                if let Some(name_str) = name.to_str() {
                    println!("  • {}", name_str);
                }
            }
        }
        Ok(())
    }

    pub async fn save_request_to_collection(
        &self,
        collection: &str,
        endpoint_name: &str,
        request: &(String, String, Option<String>),
        response: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (method, url, _body) = request;
        let spec_path = self.get_collection_path(collection);
        let mut spec = OpenAPISpec::load(&spec_path)?;

        // Parse URL and setup servers
        let url = url::Url::parse(&url)?;
        let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));
        
        // Extract path parameters and clean path
        let path_segments: Vec<&str> = url.path().split('/').collect();
        let mut path_params = Vec::new();
        let clean_path = path_segments.iter().enumerate()
            .map(|(i, segment)| {
                if segment.parse::<i64>().is_ok() {
                    let param_name = if i == path_segments.len() - 1 { "id" } else { &format!("id_{}", i) };
                    path_params.push(param_name.to_string());
                    format!("{{{}}}", param_name)
                } else {
                    segment.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join("/");

        // Update servers
        if !spec.servers.iter().any(|s| s.url == base_url) {
            spec.servers.clear();
            spec.servers.push(Server {
                url: base_url,
                description: Some("API Server".to_string()),
            });
        }

        // Generate AI documentation
        let doc_prompt = format!(
            "You are a technical writer creating OpenAPI documentation. \
            Analyze this API endpoint and generate a clear, detailed description:\n\
            Path: {}\n\
            Method: {}\n\
            Response Example: {}\n\n\
            Please provide:\n\
            1. A concise summary (one line)\n\
            2. A detailed description including:\n\
               - What the endpoint does\n\
               - Common use cases\n\
               - Response structure explanation\n\
               - Any important notes or considerations",
            clean_path, method,
            response.as_deref().unwrap_or("{}")
        );

        let doc_response = self.get_ai_response(&doc_prompt).await?;
        let (summary, description) = Self::parse_ai_doc_response(&doc_response)?;

        // Generate mock data
        let mock_prompt = format!(
            "Generate diverse test data examples for this endpoint.\n\
            URL: {}\n\
            Method: {}\n\
            Example Response: {}\n\n\
            Generate 5 different examples covering:\n\
            1. Happy path\n\
            2. Edge cases\n\
            3. Error scenarios\n\
            4. Special characters\n\
            5. Boundary values\n\
            Make each example valid JSON.",
            clean_path, method,
            response.as_deref().unwrap_or("{}")
        );

        let mock_response = self.get_ai_response(&mock_prompt).await?;
        let mock_examples = Self::parse_mock_examples(&mock_response)?;

        // Create operation with all the generated content
        let operation = Operation {
            summary: Some(summary),
            description: Some(description),
            parameters: if !path_params.is_empty() {
                Some(path_params.iter().map(|param| Parameter {
                    name: param.to_string(),
                    in_: "path".to_string(),
                    description: Some(format!("Path parameter {}", param)),
                    required: true,
                    schema: Schema {
                        schema_type: "integer".to_string(),
                        format: Some("int64".to_string()),
                        properties: None,
                        items: None,
                    },
                }).collect())
            } else {
                None
            },
            responses: {
                let mut responses = HashMap::new();
                if let Some(resp) = response {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&resp) {
                        responses.insert("200".to_string(), Response {
                            description: "Successful response".to_string(),
                            content: Some({
                                let mut content = HashMap::new();
                                content.insert("application/json".to_string(), MediaType {
                                    schema: Schema {
                                        schema_type: "object".to_string(),
                                        properties: None,
                                        items: None,
                                        format: None,
                                    },
                                    example: Some(json),
                                });
                                content
                            }),
                        });
                    }
                }
                responses
            },
            mock_data: Some(MockDataConfig {
                description: "AI-generated mock responses".to_string(),
                schema: None,
                examples: Some(mock_examples),
            }),
            ..Default::default()
        };

        // Add operation to path item
        let path_item = spec.paths.entry(clean_path.clone()).or_insert(PathItem::new());
        match method.to_uppercase().as_str() {
            "GET" => path_item.get = Some(operation),
            "POST" => path_item.post = Some(operation),
            "PUT" => path_item.put = Some(operation),
            "DELETE" => path_item.delete = Some(operation),
            "PATCH" => path_item.patch = Some(operation),
            _ => return Err("Unsupported HTTP method".into()),
        }

        spec.save(&spec_path)?;
        println!("✅ Saved {} {} to collection {} with documentation and mock data", method, url, collection);
        Ok(())
    }
    async fn get_ai_response(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt.into() }]
        }];

        let request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(2000_usize)
            .build()?;

        let response = self.ai_client.messages(request).await?;
        
        if let Some(ContentBlock::Text { text }) = response.content.first() {
            Ok(text.clone())
        } else {
            Err("No response from AI".into())
        }
    }

    fn parse_ai_doc_response(response: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
        let lines: Vec<&str> = response.lines().collect();
        if let Some((summary, rest)) = lines.split_first() {
            Ok((
                summary.trim().to_string(),
                rest.join("\n").trim().to_string()
            ))
        } else {
            Err("Could not parse AI documentation response".into())
        }
    }




// Add a fallback for when AI is not available
async fn run_single_endpoint_test(
    &self,
    endpoint: &str,
    method: &str,
    users: u32,
    duration: Duration,
    base_url: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running single endpoint test...");
    let perf = PerfCommand::new(&self.config);
    let url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        format!("{}{}", base_url, endpoint)
    };
    perf.run(
        &url,
        users,
        duration,
        method,
        None
    ).await
}

    pub fn get_collections_dir(&self) -> PathBuf {
        self.collections_dir.clone()
    }
}