use crate::completer::NutsCompleter;
use console::style;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use crate::commands::call::CallCommand;
use crate::commands::security::SecurityCommand;
use crate::commands::perf::PerfCommand;
use crate::flows::manager::CollectionManager;
use crate::config::Config;
use std::path::PathBuf;
use std::fs;
use crate::commands::config::ConfigCommand;
use anthropic::client::ClientBuilder;
use anthropic::types::Message;
use anthropic::types::ContentBlock;
use anthropic::types::MessagesRequestBuilder;
use anthropic::types::Role;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::story::StoryMode;

#[derive(Debug)]
pub enum ShellError {
    ApiError(String),
    ConfigError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellError::ApiError(msg) => write!(f, "API Error: {}", msg),
            ShellError::ConfigError(msg) => write!(f, "Config Error: {}", msg),
            ShellError::IoError(err) => write!(f, "IO Error: {}", err),
        }
    }
}

impl std::error::Error for ShellError {}

pub struct NutsShell {
    editor: Editor<NutsCompleter, DefaultHistory>,
    config: Config,
    history: Vec<String>,
    suggestions: Vec<String>,
    last_request: Option<(String, String, Option<String>)>,
    last_response: Option<String>,
    collection_manager: CollectionManager,
}

impl NutsShell {
    fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".nuts_config.json");
        path
    }

    fn save_api_key(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = serde_json::json!({
            "anthropic_api_key": api_key.to_string()
        });
        fs::write(Self::get_config_path(), serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    pub fn new() -> Self {
        let collections_dir = dirs::home_dir()
            .map(|h| h.join(".nuts").join("flows"))
            .expect("Could not determine home directory");
            
        std::fs::create_dir_all(&collections_dir)
            .expect("Failed to create flows directory");

        // Load config first
        let config = Config::load().unwrap_or_default();

        // Initialize editor with completer
        let mut editor = Editor::new().unwrap();
        editor.set_helper(Some(NutsCompleter::new()));
        editor.bind_sequence(rustyline::KeyEvent::from('\t'), rustyline::Cmd::Complete);

        Self {
            editor,
            config: config.clone(),
            history: Vec::new(),
            suggestions: Vec::new(),
            last_request: None,
            last_response: None,
            collection_manager: CollectionManager::new(
                collections_dir,
                config
            ),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", self.get_welcome_message());
        
        // Create a single runtime for the entire application
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            loop {
                let readline = self.editor.readline("🥜 nuts> ");
                match readline {
                    Ok(line) => {
                        let _ = self.editor.add_history_entry(line.as_str());
                        if let Err(e) = self.process_command(&line).await {
                            println!("❌ Error: {}", e);
                        }
                    }
                    Err(_) => break,
                }
            }
            Ok(())
        })
    }

    fn get_welcome_message(&self) -> String {
        let ascii_art = r#"
    ███╗   ██╗██╗   ██╗██████████████╗
    ████  ██║█║   ██║╚══██╔══╝██╔════╝
    ██╔██╗ ██║██║   ██║   █║   ╚════██║
    ██║╚██╗██║██║   ██║   ██║   ╚═════██║
    ██║ ╚████║╚█████╔╝   ██║   ███████║
    ╚═╝  ╚═══╝ ╚═════╝    ╚═╝   ╚══════╝
    "#;

        format!(
            "{}\n{}\n{}\n",
            style(ascii_art).cyan(),
            style(" Network Universal Testing Suite v0.1.0").magenta(),
            style("Type 'help' to see available commands").green()
        )
    }

    fn show_help(&self) {
        println!("\n{}", style("🥜 NUTS - Network Universal Testing Suite").cyan().bold());
        println!("{}\n", style("Version 0.1.0").dim());

        // Core Commands
        println!("{}", style("🌐 API Testing").yellow());
        println!("  {} - Test an API endpoint", style("call <METHOD> <URL> [BODY]").green());
        println!("  {} - Run performance tests", style("perf <METHOD> <URL> [OPTIONS]").green());
        println!("  {} - Scan for security issues", style("security <URL> [OPTIONS]").green());

        // Flow Management
        println!("\n{}", style("📚 sFlow").yellow());
        println!("  {} - Create new flow", style("flow new <name>").green());
        println!("  {} - Add endpoint to flow", style("flow add <name> <METHOD> <path>").green());
        println!("  {} - Run flow endpoint", style("flow run <name> <endpoint>").green());
        println!("  {} - Generate OpenAPI docs", style("flow docs <name> [format]").green());
        println!("  {} - Save last request", style("save <flow> <name>").green());
        println!("  {} - List flows", style("flow list").green());

        // Mock Server
        println!("\n{}", style("🎭 Mock Server").yellow());
        println!("  {} - Start mock server", style("flow mock <name> [port]").green());
        println!("  {} - Configure mock data", style("flow configure_mock_data <name> <endpoint>").green());

        // Add Story Mode section after Mock Server
        println!("\n{}", style("🎬 Story Mode").yellow());
        println!("  {} - Start AI-guided API workflow", style("flow story <name>").green());
        println!("  {} - Quick story mode alias", style("flow s <name>").green());

        // Configuration
        println!("\n{}", style("⚙️  Configuration").yellow());
        println!("  {} - Configure API key", style("config api-key").green());
        println!("  {} - Show current config", style("config show").green());

        // Pro Tips
        println!("\n{}", style("💡 Tips").blue());
        println!("• Use TAB for command completion");
        println!("• Commands are case-insensitive");
        println!("• Save frequently used calls to flows");
        println!("• Press Ctrl+C to cancel any operation");
    }

    pub async fn process_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<String> = cmd.trim()
            .split_whitespace()
            .map(String::from)
            .collect();

        match parts.first().map(|s| s.as_str()) {
            Some("config") => {
                ConfigCommand::new(self.config.clone())
                    .execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    .await?;
                
                // Reload config and update manager
                self.config = Config::load()?;
                self.collection_manager = CollectionManager::new(
                    self.collection_manager.get_collections_dir(),
                    self.config.clone()
                );
            }
            Some("configure") => {
                match parts.get(1).map(String::as_str) {
                    Some("api-key") => {
                        if let Ok(key) = self.editor.readline_with_initial(
                            "Enter Anthropic API Key: ",
                            ("", "")
                        ) {
                            self.config.anthropic_api_key = Some(key.trim().to_string());
                            self.config.save()?;
                            println!("✅ API key configured successfully");
                        }
                    }
                    Some("show") => {
                        println!("Current Configuration:");
                        println!("  API Key: {}", self.config.anthropic_api_key
                            .as_ref()
                            .map(|k| "********".to_string())
                            .unwrap_or_else(|| "Not set".to_string()));
                    }
                    _ => {
                        println!("Available configure commands:");
                        println!("  {} - Set Anthropic API key", style("api-key").green());
                        println!("  {} - Show current config", style("show").green());
                    }
                }
            }
            Some("call") => {
                if parts.len() > 1 {
                    let (method, url, body) = match parts[1].to_uppercase().as_str() {
                        "POST" | "PUT" | "PATCH" => {
                            if parts.len() < 4 {
                                println!("❌ Usage: call {} URL JSON_BODY", parts[1].to_uppercase());
                                return Ok(());
                            }
                            (parts[1].to_uppercase(), parts[2].clone(), parts.get(3).cloned())
                        },
                        "DELETE" => {
                            if parts.len() < 3 {
                                println!("❌ Usage: call DELETE URL");
                                return Ok(());
                            }
                            ("DELETE".to_string(), parts[2].clone(), None)
                        },
                        "GET" | "HEAD" | "OPTIONS" => {
                            if parts.len() < 3 {
                                ("GET".to_string(), parts[1].clone(), None)
                            } else {
                                (parts[1].to_uppercase(), parts[2].clone(), None)
                            }
                        },
                        _ => {
                            // If no method specified, assume GET
                            ("GET".to_string(), parts[1].clone(), None)
                        }
                    };
                    
                    // Store the request before executing
                    self.store_last_request(method.clone(), url.clone(), body.clone());
                    
                    // Execute call and store response
                    let response = CallCommand::new().execute_with_response(&parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()).await?;
                    self.last_response = Some(response);
                    
                } else {
                    println!("❌ Usage: call [METHOD] URL [JSON_BODY]");
                    println!("Supported methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS");
                    println!("Examples:");
                    println!("  call GET https://api.example.com/users");
                    println!("  call POST https://api.example.com/users {{'name': 'John'}}");
                }
            }
            Some("help") => self.show_help(),
            Some("exit") | Some("quit") => std::process::exit(0),
            Some("perf") => {
                if parts.len() < 2 {
                    println!("❌ Usage: perf [METHOD] URL [--users N] [--duration Ns] [BODY]");
                    println!("Supported methods: GET, POST, PUT, PATCH, DELETE");
                    println!("Example: perf GET https://api.example.com --users 100 --duration 30s");
                    return Ok(());
                }
                
                let (method, url) = match parts[1].to_uppercase().as_str() {
                    "POST" | "PUT" | "PATCH" => {
                        if parts.len() < 3 {
                            println!("❌ Usage: perf {} URL [OPTIONS] JSON_BODY", parts[1].to_uppercase());
                            return Ok(());
                        }
                        (parts[1].to_uppercase(), &parts[2])
                    },
                    "DELETE" => {
                        if parts.len() < 3 {
                            println!("❌ Usage: perf DELETE URL [OPTIONS]");
                            return Ok(());
                        }
                        ("DELETE".to_string(), &parts[2])
                    },
                    "GET" | "HEAD" | "OPTIONS" => {
                        if parts.len() < 3 {
                            ("GET".to_string(), &parts[1])
                        } else {
                            (parts[1].to_uppercase(), &parts[2])
                        }
                    },
                    _ => {
                        // If no method specified, assume GET
                        ("GET".to_string(), &parts[1])
                    }
                };
                
                // Validate URL format
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    println!("⚠️  Warning: URL should start with http:// or https://");
                }
                
                let users = parts.iter()
                    .position(|x| x == "--users")
                    .and_then(|i| parts.get(i + 1))
                    .and_then(|u| u.parse().ok())
                    .unwrap_or(10);
                        
                let duration = parts.iter()
                    .position(|x| x == "--duration")
                    .and_then(|i| parts.get(i + 1))
                    .and_then(|d| d.trim_end_matches('s').parse().ok())
                    .map(|secs| std::time::Duration::from_secs(secs))
                    .unwrap_or(std::time::Duration::from_secs(30));

                // Find body if present (after all flags)
                let body = match method.as_str() {
                    "POST" | "PUT" | "PATCH" => {
                        parts.iter()
                            .skip_while(|&p| {
                                p == "--users" || p == "--duration" || 
                                p.ends_with('s') || p.parse::<u32>().is_ok() ||
                                p == &method || p == url
                            })
                            .last()
                            .map(String::as_str)
                    },
                    _ => None
                };

                PerfCommand::new(&self.config).run(url, users, duration, &method, body).await?;
            }
            Some("security") => {
                if parts.len() < 2 {
                    println!("❌ Usage: security URL [OPTIONS]");
                    println!("Options:");
                    println!("  --deep        Perform deep scan (more thorough but slower)");
                    println!("  --auth TOKEN  Include authorization header for authenticated endpoints");
                    println!("  --save FILE   Save report to specified file");
                    println!("Examples:");
                    println!("  security https://api.example.com");
                    println!("  security https://api.example.com --deep --auth Bearer_token");
                    return Ok(());
                }

                let url = &parts[1];
                
                // Validate URL format
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    println!("⚠️  Warning: URL should start with http:// or https://");
                }

                // Check for API key
                let api_key = self.config.anthropic_api_key.clone()
                    .ok_or("API key not configured. Use 'config api-key' to set it")?;

                // Parse options
                let deep_scan = parts.contains(&"--deep".to_string());
                let auth_token = parts.iter()
                    .position(|x| x == "--auth")
                    .and_then(|i| parts.get(i + 1))
                    .map(|s| s.to_string());
                let save_file = parts.iter()
                    .position(|x| x == "--save")
                    .and_then(|i| parts.get(i + 1))
                    .map(|s| s.to_string());

                println!("🔒 Starting security scan...");
                if deep_scan {
                    println!("📋 Deep scan enabled - this may take a few minutes");
                }

                SecurityCommand::new(self.config.clone())
                    .with_deep_scan(deep_scan)
                    .with_auth(auth_token)
                    .with_save_file(save_file)
                    .execute(&parts.iter().map(|s| s.to_string()).collect::<Vec<String>>())
                    .await?;
            }
            Some("flow") => {
                match parts.get(1).map(String::as_str) {
                    Some("new") => {
                        if let Some(name) = parts.get(2) {
                            println!("🔨 Creating new OpenAPI flow: {}", style(name).cyan());
                            self.collection_manager.create_collection(name)?;
                            println!("✅ Flow created. Use 'flow add {}' to add endpoints", name);
                        } else {
                            println!("❌ Usage: flow new <name>");
                            println!("Creates a new OpenAPI specification flow");
                        }
                    }
                    Some("add") => {
                        if parts.len() >= 4 {
                            let flow = &parts[2];
                            let method = parts[3].to_uppercase();
                            let path = parts.get(4).map(|s| s.to_string());
                            
                            match (method.as_str(), path) {
                                (m, Some(p)) if ["GET", "POST", "PUT", "DELETE", "PATCH"].contains(&m) => {
                                    println!("📝 Adding {} endpoint {} to flow {}", 
                                        style(m).cyan(),
                                        style(&p).green(),
                                        style(flow).yellow()
                                    );
                                    self.collection_manager.add_endpoint(flow, m, &p).await?;
                                },
                                _ => {
                                    println!("❌ Usage: flow add <name> <METHOD> <path>");
                                    println!("Example: flow add my-api GET /users");
                                    println!("Supported methods: GET, POST, PUT, DELETE, PATCH");
                                }
                            }
                        } else {
                            println!("❌ Usage: flow add <name> <METHOD> <path>");
                        }
                    }
                    Some("run") => {
                        if parts.len() >= 4 {
                            let flow = &parts[2];
                            let endpoint = &parts[3];
                            let args = &parts[4..];
                            println!("🚀 Running endpoint {} from flow {}", 
                                style(endpoint).green(),
                                style(flow).yellow()
                            );
                            self.collection_manager.run_endpoint(flow, endpoint, args).await?;
                        } else {
                            println!("❌ Usage: flow run <name> <endpoint> [args...]");
                            println!("Example: flow run my-api /users --data '{{\"name\": \"test\"}}'");
                        }
                    }
                    Some("mock") => {
                        if let Some(name) = parts.get(2) {
                            let port = parts.get(3)
                                .and_then(|p| p.parse().ok())
                                .unwrap_or(3000);
                            println!("🎭 Starting mock server for flow {} on port {}", 
                                style(name).yellow(),
                                style(port).cyan()
                            );
                            self.collection_manager.start_mock_server(name, port).await?;
                        } else {
                            println!("❌ Usage: flow mock <name> [port]");
                            println!("Starts a mock server based on OpenAPI specification");
                        }
                    }
                    Some("configure_mock_data") => {
                        if parts.len() >= 4 {
                            let flow = &parts[2];
                            let endpoint = &parts[3];
                            println!("⚙️  Configuring mock data for endpoint {} in flow {}", 
                                style(endpoint).green(),
                                style(flow).yellow()
                            );
                            self.collection_manager.configure_mock_data(
                                flow, 
                                endpoint,
                                &mut self.editor
                            ).await?;
                        } else {
                            println!("❌ Usage: flow configure_mock_data <name> <endpoint>");
                            println!("Example: flow configure_mock_data my-api /users");
                        }
                    }
                    Some("perf") => {
                        let flow = parts.get(2)
                            .ok_or("Usage: flow perf <name> [endpoint] [--users N] [--duration Ns]")?;
                        let endpoint = parts.get(3);
                        let options = &parts[if endpoint.is_some() { 4 } else { 3 }..];
                        
                        if endpoint.is_some() {
                            println!("🚄 Running performance test for endpoint {} in flow {}", 
                                style(endpoint.unwrap()).green(),
                                style(flow).yellow()
                            );
                        } else {
                            println!("🚄 Running performance tests for flow {}", 
                                style(flow).yellow()
                            );
                        }
                        
                        self.collection_manager.run_endpoint_perf(
                            flow,
                            endpoint.map(String::as_str),
                            options
                        ).await?;
                    }
                    Some("docs") => {
                        if let Some(name) = parts.get(2) {
                            let format = parts.get(3).map(String::as_str).unwrap_or("yaml");
                            match format {
                                "yaml" | "json" => {
                                    println!("📚 Generating OpenAPI documentation for flow {}", 
                                        style(name).yellow()
                                    );
                                    self.collection_manager.generate_openapi(name, format).await?;
                                },
                                _ => println!("❌ Supported formats: yaml, json")
                            }
                        } else {
                            println!("❌ Usage: flow docs <name> [format]");
                            println!("Generates OpenAPI documentation (yaml or json)");
                        }
                    }
                    Some("list") => {
                        println!("📋 Available flows:");
                        self.collection_manager.list_collections().await?;
                    }
                    Some("story") | Some("s") => {
                        if let Some(flow) = parts.get(2) {
                            let api_key = self.config.anthropic_api_key.clone()
                                .ok_or("API key not configured. Use 'config api-key' to set it")?;
                            
                            StoryMode::new(flow.to_string(), api_key)
                                .start(&mut self.editor)
                                .await?;
                        } else {
                            println!("❌ Usage: story <flow>");
                            println!("Start an AI-guided API story session");
                        }
                    }
                    _ => {
                        println!("Available flow commands:");
                        println!("  {} - Create new flow", style("new <name>").green());
                        println!("  {} - Add endpoint to flow", style("add <name> <METHOD> <path>").green());
                        println!("  {} - Run specific endpoint", style("run <name> <endpoint> [args...]").green());
                        println!("  {} - Start mock server", style("mock <name> [port]").green());
                        println!("  {} - Configure mock responses", style("configure_mock_data <name> <endpoint>").green());
                        println!("  {} - Run performance tests", style("perf <name> <endpoint> [options]").green());
                        println!("  {} - Generate OpenAPI docs", style("docs <name> [format]").green());
                        println!("  {} - List all flows", style("list").green());
                    }
                }
            }
            Some("save") => {
                if parts.len() >= 3 {
                    let collection_name = &parts[1];
                    let endpoint_name = &parts[2];
                    if let Some(last_request) = &self.last_request {
                        if let Some(last_response) = &self.last_response {
                            self.collection_manager.save_request_to_collection(
                                collection_name,
                                endpoint_name,
                                last_request,
                                Some(last_response.clone()),
                            ).await?;
                        } else {
                            println!("❌ No response to save. Make a call first!");
                        }
                    } else {
                        println!("❌ No request to save. Make a call first!");
                    }
                } else {
                    println!("❌ Usage: save <collection_name> <endpoint_name>");
                }
            }
            Some("configure_mock_data") => {
                if parts.len() >= 3 {
                    let flow = &parts[1];
                    let endpoint = &parts[2];
                    self.collection_manager.configure_mock_data(
                        flow, 
                        endpoint,
                        &mut self.editor
                    ).await?;
                } else {
                    println!("❌ Usage: configure_mock_data <collection_name> <endpoint_name>");
                }
            }
            Some("daemon") => {
                match parts.get(1).map(String::as_str) {
                    Some("start") => println!("Starting NUTS daemon..."),
                    Some("stop") => println!("Stopping NUTS daemon..."),
                    Some("status") => println!("NUTS daemon status: Not running"),
                    _ => println!("Usage: daemon [start|stop|status]"),
                }
            },
            _ => {
                if let Some(suggestion) = self.ai_suggest_command(cmd).await {
                    println!("🤖 AI Suggests: {}", style(suggestion).blue());
                }
            }
        }
     
        Ok(())
    }

    async fn ai_suggest_command(&self, input: &str) -> Option<String> {
        // Skip if no API key configured
        let api_key = self.config.anthropic_api_key.as_ref()?;
        
        let prompt = format!(
            "You are a CLI assistant for NUTS (Network Universal Testing Suite). \
            The user entered an invalid command: '{}'\n\n\
            Available commands are:\n\
            - call [METHOD] URL [BODY] - Test an API endpoint\n\
            - perf [METHOD] URL [OPTIONS] - Run performance tests\n\
            - flow [new|add|run|mock] - Manage API flows\n\
            - security URL [OPTIONS] - Scan for security issues\n\
            - config [api-key|show] - Configure settings\n\
            - help - Show help\n\n\
            Suggest the most likely command they meant to use. \
            Respond with ONLY the suggested command, no explanation.",
            input
        );

        // Create AI client
        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()
            .ok()?;

        // Get AI response directly - no need for block_on
        match ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(100_usize)
            .build()
            .ok()?
        ).await {
            Ok(response) => {
                if let Some(ContentBlock::Text { text }) = response.content.first() {
                    Some(text.trim().to_string())
                } else {
                    None
                }
            }
            Err(_) => None
        }
    }

    fn store_last_request(&mut self, method: String, url: String, body: Option<String>) {
        self.last_request = Some((method, url, body));
    }

    fn handle_error(&self, error: Box<dyn std::error::Error>) {
        match error.downcast_ref::<ShellError>() {
            Some(ShellError::ApiError(msg)) => {
                println!("❌ API Error: {}", style(msg).red());
                println!("💡 Tip: Check the URL and try again");
            },
            Some(ShellError::ConfigError(msg)) => {
                println!("⚠️  Configuration Error: {}", style(msg).yellow());
                println!("💡 Run 'configure' to set up your environment");
            },
            _ => println!("❌ Error: {}", style(error).red()),
        }
    }

    fn print_info(&self, msg: &str) {
        println!("ℹ️  {}", style(msg).blue());
    }

    fn print_success(&self, msg: &str) {
        println!("✅ {}", style(msg).green());
    }

    fn print_warning(&self, msg: &str) {
        println!("⚠️  {}", style(msg).yellow());
    }

    fn print_error(&self, msg: &str) {
        println!("❌ {}", style(msg).red());
    }

    fn show_command_help(&self, command: &str) {
        match command {
            "call" => {
                println!("{}", style("USAGE:").bold());
                println!("  call [METHOD] URL [BODY]");
                println!("\n{}", style("DESCRIPTION:").bold());
                println!("  Make HTTP requests to test API endpoints");
                println!("\n{}", style("OPTIONS:").bold());
                println!("  METHOD     HTTP method (GET, POST, PUT, DELETE, PATCH)");
                println!("  URL        Target URL");
                println!("  BODY       JSON request body (for POST/PUT/PATCH)");
                println!("\n{}", style("EXAMPLES:").bold());
                println!("  call GET https://api.example.com/users");
                println!("  call POST https://api.example.com/users '{{\"name\":\"test\"}}'");
            },
            "perf" => {
                println!("{}", style("USAGE:").bold());
                println!("  perf [METHOD] URL [OPTIONS]");
                println!("\n{}", style("DESCRIPTION:").bold());
                println!("  Run performance tests against API endpoints");
                println!("\n{}", style("OPTIONS:").bold());
                println!("  --users N        Number of concurrent users");
                println!("  --duration Ns    Test duration in seconds");
                println!("\n{}", style("EXAMPLES:").bold());
                println!("  perf GET https://api.example.com/users --users 100 --duration 30s");
            },
            _ => println!("No detailed help available for '{}'. Use 'help' to see all commands.", command),
        }
    }

    fn with_progress<F, T>(&self, msg: &str, f: F) -> T 
    where 
        F: FnOnce(&ProgressBar) -> T 
    {
        let spinner = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_spinner()
                .template("{spinner} {msg}")
                .unwrap());
        spinner.set_message(msg.to_string());
        
        let result = f(&spinner);
        spinner.finish_with_message("Done!");
        result
    }
}
