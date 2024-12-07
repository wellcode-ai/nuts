use crate::completer::NutsCompleter;
use console::style;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use crate::commands::call::CallCommand;
use crate::commands::security::SecurityCommand;
use crate::commands::perf::PerfCommand;
use crate::collections::manager::CollectionManager;
use crate::config::Config;
use std::path::PathBuf;
use std::fs;
use crate::commands::config::ConfigCommand;

#[derive(Debug)]
pub enum ShellError {
    ApiError(String),
    ConfigError(String),
    IoError(std::io::Error),
}

impl std::fmt::Display for ShellError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(msg) => write!(f, "API Error: {}", msg),
            Self::ConfigError(msg) => write!(f, "Configuration Error: {}", msg),
            Self::IoError(e) => write!(f, "IO Error: {}", e),
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
            .map(|h| h.join(".nuts").join("collections"))
            .expect("Could not determine home directory");
            
        std::fs::create_dir_all(&collections_dir)
            .expect("Failed to create collections directory");

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
    ██║╚██╗██║██║   ██║   ██║   ╚════██║
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
        // Title and Version
        println!("\n{}", style("🥜 NUTS - Network Universal Testing Suite").cyan().bold());
        println!("{}\n", style("Version 0.1.0").dim());

        // Quick Start
        println!("{}", style("🚀 Quick Start:").yellow());
        println!("  {} - Test an API endpoint", style("call GET https://api.example.com").green());
        println!("  {} - Run performance test", style("perf GET https://api.example.com").green());
        println!("  {} - Scan for security issues", style("security https://api.example.com").green());
        println!("");

        // Core Commands
        println!("{}", style("🛠️  Core Commands:").yellow());
        println!("  {} - Make API calls", style("call [METHOD] URL [BODY]").green());
        println!("  {} - Run performance tests", style("perf [METHOD] URL [OPTIONS]").green());
        println!("  {} - Security analysis", style("security [URL]").green());
        println!("  {} - Start mock server", style("mock [PORT]").green());
        println!("");

        // Collection Management
        println!("{}", style("📚 Collections:").yellow());
        println!("  {} - Create collection", style("collection new <name>").green());
        println!("  {} - Run collection", style("collection run <name>").green());
        println!("  {} - Generate docs", style("collection docs <name>").green());
        println!("  {} - Save last request", style("save <collection> <name>").green());
        println!("");

        // Performance Options
        println!("{}", style("🚄 Performance Options:").yellow());
        println!("  --users N        Number of concurrent users");
        println!("  --duration Ns    Test duration in seconds");
        println!("  Example: {} ", style("perf GET api/users --users 100 --duration 30s").dim());
        println!("");

        // Mock Server
        println!("{}", style("🎭 Mock Server:").yellow());
        println!("  {} - Start mock server", style("collection mock <name>").green());
        println!("  {} - Configure mocks", style("collection configure_mock_data <name> <endpoint>").green());
        println!("");

        // System Commands
        println!("{}", style("⚙️  System:").yellow());
        println!("  {} - Configure API keys", style("configure").green());
        println!("  {} - Manage background service", style("daemon [start|stop|status]").green());
        println!("  {} - Show this help", style("help").green());
        println!("  {} - Exit NUTS", style("exit").green());
        println!("");

        // Configuration
        println!("{}", style("⚙️  Configuration:").yellow());
        println!("  {} - Set API key", style("configure api-key").green());
        println!("  {} - Show current config", style("configure show").green());

        // Pro Tips
        println!("{}", style("💡 Pro Tips:").blue());
        println!("• Use TAB for command completion");
        println!("• Commands are case-insensitive");
        println!("• Save frequently used calls to collections");
        println!("• Use --help with any command for detailed options");
        println!("• Press Ctrl+C to cancel any running operation");
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
            Some("collection") => {
                match parts.get(1).map(String::as_str) {
                    Some("new") => {
                        if let Some(name) = parts.get(2) {
                            println!("🔨 Creating new OpenAPI collection: {}", style(name).cyan());
                            self.collection_manager.create_collection(name)?;
                            println!("✅ Collection created. Use 'collection add {}' to add endpoints", name);
                        } else {
                            println!("❌ Usage: collection new <name>");
                            println!("Creates a new OpenAPI specification collection");
                        }
                    }
                    Some("add") => {
                        if parts.len() >= 4 {
                            let collection = &parts[2];
                            let method = parts[3].to_uppercase();
                            let path = parts.get(4).map(|s| s.to_string());
                            
                            match (method.as_str(), path) {
                                (m, Some(p)) if ["GET", "POST", "PUT", "DELETE", "PATCH"].contains(&m) => {
                                    println!("📝 Adding {} endpoint {} to collection {}", 
                                        style(m).cyan(),
                                        style(&p).green(),
                                        style(collection).yellow()
                                    );
                                    self.collection_manager.add_endpoint(collection, m, &p).await?;
                                },
                                _ => {
                                    println!("❌ Usage: collection add <name> <METHOD> <path>");
                                    println!("Example: collection add my-api GET /users");
                                    println!("Supported methods: GET, POST, PUT, DELETE, PATCH");
                                }
                            }
                        } else {
                            println!("❌ Usage: collection add <name> <METHOD> <path>");
                        }
                    }
                    Some("run") => {
                        if parts.len() >= 4 {
                            let collection = &parts[2];
                            let endpoint = &parts[3];
                            let args = &parts[4..];
                            println!("🚀 Running endpoint {} from collection {}", 
                                style(endpoint).green(),
                                style(collection).yellow()
                            );
                            self.collection_manager.run_endpoint(collection, endpoint, args).await?;
                        } else {
                            println!("❌ Usage: collection run <name> <endpoint> [args...]");
                            println!("Example: collection run my-api /users --data '{{\"name\": \"test\"}}'");
                        }
                    }
                    Some("mock") => {
                        if let Some(name) = parts.get(2) {
                            let port = parts.get(3)
                                .and_then(|p| p.parse().ok())
                                .unwrap_or(3000);
                            println!("🎭 Starting mock server for collection {} on port {}", 
                                style(name).yellow(),
                                style(port).cyan()
                            );
                            self.collection_manager.start_mock_server(name, port).await?;
                        } else {
                            println!("❌ Usage: collection mock <name> [port]");
                            println!("Starts a mock server based on OpenAPI specification");
                        }
                    }
                    Some("configure_mock_data") => {
                        if parts.len() >= 4 {
                            let collection = &parts[2];
                            let endpoint = &parts[3];
                            println!("⚙️  Configuring mock data for endpoint {} in collection {}", 
                                style(endpoint).green(),
                                style(collection).yellow()
                            );
                            self.collection_manager.configure_mock_data(
                                collection, 
                                endpoint,
                                &mut self.editor
                            ).await?;
                        } else {
                            println!("❌ Usage: collection configure_mock_data <name> <endpoint>");
                            println!("Example: collection configure_mock_data my-api /users");
                        }
                    }
                    Some("perf") => {
                        let collection = parts.get(2)
                            .ok_or("Usage: collection perf <name> [endpoint] [--users N] [--duration Ns]")?;
                        let endpoint = parts.get(3);
                        let options = &parts[if endpoint.is_some() { 4 } else { 3 }..];
                        
                        if endpoint.is_some() {
                            println!("🚄 Running performance test for endpoint {} in collection {}", 
                                style(endpoint.unwrap()).green(),
                                style(collection).yellow()
                            );
                        } else {
                            println!("🚄 Running performance tests for collection {}", 
                                style(collection).yellow()
                            );
                        }
                        
                        self.collection_manager.run_endpoint_perf(
                            collection,
                            endpoint.map(String::as_str),
                            options
                        ).await?;
                    }
                    Some("docs") => {
                        if let Some(name) = parts.get(2) {
                            let format = parts.get(3).map(String::as_str).unwrap_or("yaml");
                            match format {
                                "yaml" | "json" => {
                                    println!("📚 Generating OpenAPI documentation for collection {}", 
                                        style(name).yellow()
                                    );
                                    self.collection_manager.generate_openapi(name, format).await?;
                                },
                                _ => println!("❌ Supported formats: yaml, json")
                            }
                        } else {
                            println!("❌ Usage: collection docs <name> [format]");
                            println!("Generates OpenAPI documentation (yaml or json)");
                        }
                    }
                    Some("list") => {
                        println!("📋 Available collections:");
                        self.collection_manager.list_collections().await?;
                    }
                    _ => {
                        println!("Available collection commands:");
                        println!("  {} - Create new collection", style("new <name>").green());
                        println!("  {} - Add endpoint to collection", style("add <name> <METHOD> <path>").green());
                        println!("  {} - Run specific endpoint", style("run <name> <endpoint> [args...]").green());
                        println!("  {} - Start mock server", style("mock <name> [port]").green());
                        println!("  {} - Configure mock responses", style("configure_mock_data <name> <endpoint>").green());
                        println!("  {} - Run performance tests", style("perf <name> <endpoint> [options]").green());
                        println!("  {} - Generate OpenAPI docs", style("docs <name> [format]").green());
                        println!("  {} - List all collections", style("list").green());
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
                            )?;
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
                    let collection = &parts[1];
                    let endpoint = &parts[2];
                    self.collection_manager.configure_mock_data(
                        collection, 
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
                if let Some(suggestion) = self.ai_suggest_command(cmd) {
                    println!("🤖 AI Suggests: {}", style(suggestion).blue());
                }
            }
        }
     
        Ok(())
    }

    fn ai_suggest_command(&self, input: &str) -> Option<String> {
        // This would integrate with Claude AI
        // For now, return a mock suggestion
        Some(format!("Did you mean 'nuts call {}' ?", input))
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
}
