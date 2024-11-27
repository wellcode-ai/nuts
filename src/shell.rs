use crate::completer::NutsCompleter;
use console::style;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use crate::commands::call::CallCommand;
use crate::commands::security::SecurityCommand;
use crate::commands::perf::PerfCommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::collections::{Collection, Endpoint};

pub struct NutsShell {
    editor: Editor<NutsCompleter, DefaultHistory>,
    history: Vec<String>,
    suggestions: Vec<String>,
    last_request: Option<(String, String, Option<String>)>,
}

impl NutsShell {
    fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".nuts_config.json");
        path
    }

    fn save_api_key(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = serde_json::json!({
            "anthropic_api_key": api_key
        });
        fs::write(Self::get_config_path(), serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    fn load_api_key() -> Option<String> {
        fs::read_to_string(Self::get_config_path())
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|json| json["anthropic_api_key"].as_str().map(String::from))
    }

    pub fn new() -> Self {
        let mut editor = Editor::<NutsCompleter, DefaultHistory>::new().unwrap();
        let completer = NutsCompleter::new();
        editor.set_helper(Some(completer));
        
        let mut shell = Self {
            editor,
            history: Vec::new(),
            suggestions: vec![
                "call".to_string(),
                "test".to_string(),
                "perf".to_string(),
                "mock".to_string(),
                "security".to_string(),
                "configure".to_string(),
            ],
            last_request: None,
        };
        
        // Load API key on startup
        if let Some(key) = Self::load_api_key() {
            std::env::set_var("ANTHROPIC_API_KEY", key);
        }
        
        shell
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
                        self.editor.add_history_entry(line.as_str());
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
    ███╗   ██╗██╗   ██╗████████╗███████╗
    ████╗  ██║██║   ██║╚══██╔══╝██╔════╝
    ██╔██╗ ██║██║   ██║   ██║   ███████╗
    ██║╚██╗██║██║   ██║   ██║   ╚════██║
    ██║ ╚████║╚██████╔╝   ██║   ███████║
    ╚═╝  ╚═══╝ ╚═════╝    ╚═╝   ╚══════╝
    "#;

        format!(
            "{}\n{}\n{}\n",
            style(ascii_art).cyan(),
            style("🌐 Network Universal Testing Suite v0.1.0").magenta(),
            style("Type 'help' to see available commands").green()
        )
    }

    fn show_help(&self) {
        println!("\n{}",
            style("Available Commands:").cyan().bold()
        );
        println!("  {} - Make API calls", style("call").green());
        println!("  {} - Run test collections", style("test").green());
        println!("  {} - Performance testing (e.g., perf URL --users 100 --duration 30s)", 
            style("perf").green());
        println!("  {} - Start mock server", style("mock").green());
        println!("  {} - Security scanning", style("security").green());
        println!("  {} - Configure API keys", style("configure").green());
        println!("  {} - Show this help", style("help").green());
        println!("  {} - Exit the shell", style("exit").green());
    }

    async fn process_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<String> = cmd.trim()
            .split_whitespace()
            .map(String::from)
            .collect();
        
        match parts.first() {
            Some(cmd) => match cmd.as_str() {
                "call" => {
                    if parts.len() > 1 {
                        let (method, url, body) = if parts[1].to_uppercase() == "POST" {
                            ("POST", parts[2].clone(), parts.get(3).cloned())
                        } else {
                            ("GET", parts[1].clone(), None)
                        };
                        
                        // Store the request before executing
                        self.store_last_request(method.to_string(), url.clone(), body.clone());
                        
                        CallCommand::new().execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()).await?;
                    } else {
                        println!("❌ Usage: call [METHOD] URL [JSON_BODY]");
                    }
                }
                "help" => self.show_help(),
                "configure" => {
                    if let Ok(line) = self.editor.readline_with_initial("Enter Anthropic API Key: ", ("", "")) {
                        let key = line.trim();
                        Self::save_api_key(key)?;
                        std::env::set_var("ANTHROPIC_API_KEY", key);
                        println!("✅ {}", style("Anthropic API Key configured successfully").green());
                    }
                },
                "exit" | "quit" => std::process::exit(0),
                "test" | "mock" => {
                    println!("⚠️  {} is comming soon!", style(cmd.trim()).yellow());
                },
                "perf" => {
                    if parts.len() < 2 {
                        println!("❌ Usage: perf [METHOD] URL [--users N] [--duration Ns] [BODY]");
                        return Ok(());
                    }
                    
                    let (method, url) = if parts[1].to_uppercase() == "POST" {
                        ("POST", &parts[2])
                    } else {
                        ("GET", &parts[1])
                    };
                    
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
                    let body = if method == "POST" {
                        parts.iter()
                            .skip_while(|&p| p == "--users" || p == "--duration" || p.ends_with('s') || p.parse::<u32>().is_ok())
                            .last()
                            .map(String::as_str)
                    } else {
                        None
                    };

                    PerfCommand::new().run(url, users, duration, method, body).await?;
                },
                "security" => {
                    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY")
                        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
                    SecurityCommand::new(&anthropic_api_key).execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()).await?;
                }
                "collection" => {
                    match parts.get(1).map(String::as_str) {
                        Some("new") => {
                            if let Some(name) = parts.get(2) {
                                self.create_collection(name)?;
                            } else {
                                println!("Usage: collection new <name>");
                            }
                        }
                        Some("run") => {
                            if let Some(name) = parts.get(2) {
                                self.run_collection(name).await?;
                            } else {
                                println!("Usage: collection run <name>");
                            }
                        }
                        Some("mock") => {
                            if let Some(name) = parts.get(2) {
                                self.start_mock_server(name).await?;
                            } else {
                                println!("Usage: collection mock <name>");
                            }
                        }
                        Some("perf") => {
                            if let Some(name) = parts.get(2) {
                                self.run_collection_perf(name, &parts[2..]).await?;
                            } else {
                                println!("Usage: collection perf <name> [--users N] [--duration Ns]");
                            }
                        },
                        _ => println!("Available collection commands: new, run, mock, perf"),
                    }
                }
                "save" => {
                    if parts.len() >= 3 {
                        let collection_name = &parts[1];
                        let endpoint_name = &parts[2];
                        self.save_last_request_to_collection(collection_name, endpoint_name)?;
                    } else {
                        println!("❌ Usage: save <collection_name> <endpoint_name>");
                    }
                }
                _ => {
                    if let Some(suggestion) = self.ai_suggest_command(cmd) {
                        println!("🤖 AI Suggests: {}", style(suggestion).blue());
                    }
                }
            },
            _ => {}
        }
     
        Ok(())
    }

    fn ai_suggest_command(&self, input: &str) -> Option<String> {
        // This would integrate with Claude AI
        // For now, return a mock suggestion
        Some(format!("Did you mean 'nuts call {}' ?", input))
    }

    fn get_collections_dir() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".nuts");
        path.push("collections");
        std::fs::create_dir_all(&path).expect("Could not create collections directory");
        path
    }

    fn create_collection(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = Self::get_collections_dir();
        path.push(format!("{}.yaml", name));
        
        let template = Collection {
            name: name.to_string(),
            base_url: "https://api.example.com".to_string(),
            endpoints: vec![
                Endpoint {
                    name: "Example Endpoint".to_string(),
                    path: "/example".to_string(),
                    method: "GET".to_string(),
                    headers: None,
                    body: None,
                    tests: None,
                    mock: None,
                    perf: None,
                }
            ],
        };
        
        template.save(path)?;
        println!("✅ Created collection: {}", name);
        Ok(())
    }

    async fn run_collection(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = Self::get_collections_dir();
        path.push(format!("{}.yaml", name));
        
        let collection = Collection::load(path)?;
        println!("Running collection: {}", collection.name);
        
        for endpoint in collection.endpoints {
            println!("Testing endpoint: {}", endpoint.name);
            // Implement your test logic here
        }
        
        Ok(())
    }

    async fn start_mock_server(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = Self::get_collections_dir();
        path.push(format!("{}.yaml", name));
        
        let collection = Collection::load(path)?;
        println!("Starting mock server for: {}", collection.name);
        // Implement mock server logic here
        
        Ok(())
    }

    fn store_last_request(&mut self, method: String, url: String, body: Option<String>) {
        self.last_request = Some((method, url, body));
    }

    fn save_last_request_to_collection(&self, collection_name: &str, endpoint_name: &str) 
        -> Result<(), Box<dyn std::error::Error>> 
    {
        if let Some((method, url, body)) = &self.last_request {
            let mut path = Self::get_collections_dir();
            path.push(format!("{}.yaml", collection_name));
            
            let mut collection = if path.exists() {
                Collection::load(path.clone())?
            } else {
                Collection {
                    name: collection_name.to_string(),
                    base_url: "".to_string(),
                    endpoints: Vec::new(),
                }
            };

            let endpoint = Endpoint {
                name: endpoint_name.to_string(),
                path: url.clone(),
                method: method.clone(),
                headers: None,
                body: body.clone(),
                tests: None,
                mock: None,
                perf: None,
            };

            collection.endpoints.push(endpoint);
            collection.save(path)?;
            println!("✅ Saved endpoint '{}' to collection '{}'", endpoint_name, collection_name);
        } else {
            println!("❌ No request to save. Make a call first!");
        }
        Ok(())
    }

    async fn run_collection_perf(&self, name: &str, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = Self::get_collections_dir();
        path.push(format!("{}.yaml", name));
        
        let collection = Collection::load(path)?;
        println!("🚀 Running performance tests for collection: {}", collection.name);

        // Parse common arguments
        let users = args.iter()
            .position(|x| x == "--users")
            .and_then(|i| args.get(i + 1))
            .and_then(|u| u.parse().ok())
            .unwrap_or(10);
            
        let duration = args.iter()
            .position(|x| x == "--duration")
            .and_then(|i| args.get(i + 1))
            .and_then(|d| d.trim_end_matches('s').parse().ok())
            .map(|secs| std::time::Duration::from_secs(secs))
            .unwrap_or(std::time::Duration::from_secs(30));

        for endpoint in collection.endpoints {
            println!("\n📌 Testing endpoint: {}", endpoint.name);
            
            let full_url = if endpoint.path.starts_with("http") {
                endpoint.path
            } else {
                format!("{}{}", collection.base_url, endpoint.path)
            };

            PerfCommand::new().run(
                &full_url,
                users,
                duration,
                &endpoint.method,
                endpoint.body.as_deref()
            ).await?;
        }
        
        Ok(())
    }
}