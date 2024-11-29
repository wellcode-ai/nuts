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
use crate::collections::manager::CollectionManager;

pub struct NutsShell {
    editor: Editor<NutsCompleter, DefaultHistory>,
    history: Vec<String>,
    suggestions: Vec<String>,
    last_request: Option<(String, String, Option<String>)>,
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
            "anthropic_api_key": api_key
        });
        fs::write(Self::get_config_path(), serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    pub fn load_api_key() -> Option<String> {
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
            collection_manager: CollectionManager::new(),
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
        println!("\n{}", style("🥜 NUTS - Network Universal Testing Suite").cyan().bold());
        println!("{}", style("Available Commands:").yellow());
        
        // API Calls
        println!("\n{}", style("API Testing:").magenta());
        println!("  {} - Make API calls", style("call [METHOD] URL [BODY]").green());
        println!("    Example: call POST https://api.example.com/users '{{\"name\": \"John\"}}'");
        println!("    Example: call GET https://api.example.com/users");

        // Collections
        println!("\n{}", style("Collections:").magenta());
        println!("  {} - Create new collection", style("collection new <name>").green());
        println!("  {} - Run collection", style("collection run <name>").green());
        println!("  {} - Generate documentation site", style("collection docs <name>").green());
        println!("  {} - Save last API call to a collection", style("save <collection> <endpoint>").green());
        println!("    Example: collection new my-api");
        println!("    Example: save my-api get-users");

        // Performance Testing
        println!("\n{}", style("Performance Testing:").magenta());
        println!("  {} - Run performance test on URL", style("perf [METHOD] URL [--users N] [--duration Ns] [BODY]").green());
        println!("  {} - Run performance test on collection", style("collection perf <name> [--users N] [--duration Ns]").green());
        println!("    Example: perf GET https://api.example.com/users --users 100 --duration 30s");
        println!("    Example: collection perf my-api --users 50 --duration 60s");

        // Mocking
        println!("\n{}", style("Mock Server:").magenta());
        println!("  {} - Start mock server for collection", style("collection mock <name>").green());
        println!("  {} - Configure mock data generation", style("collection configure_mock_data <collection> <endpoint>").green());
        println!("    Example: collection mock my-api");
        println!("    Example: collection configure_mock_data my-api get-users");

        // Security
        println!("\n{}", style("Security:").magenta());
        println!("  {} - Run security analysis", style("security [URL]").green());
        println!("    Example: security https://api.example.com/users");

        // Configuration
        println!("\n{}", style("Configuration:").magenta());
        println!("  {} - Configure API keys", style("configure").green());
        println!("  {} - Show this help", style("help").green());
        println!("  {} - Exit the shell", style("exit").green());

        println!("\n{}", style("Tips:").blue());
        println!("• Use TAB for command completion");
        println!("• Commands are case-insensitive");
        println!("• Save API calls to collections for reuse");
        println!("• Configure mock data for automated testing");
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
                                self.collection_manager.create_collection(name)?;
                            } else {
                                println!("Usage: collection new <name>");
                            }
                        }
                        Some("run") => {
                            if let Some(name) = parts.get(2) {
                                self.collection_manager.run_collection(name).await?;
                            } else {
                                println!("Usage: collection run <name>");
                            }
                        }
                        Some("mock") => {
                            if let Some(name) = parts.get(2) {
                                self.collection_manager.start_mock_server(name).await?;
                            } else {
                                println!("Usage: collection mock <name>");
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
                        },
                        Some("perf") => {
                            if let Some(name) = parts.get(2) {
                                self.collection_manager.run_collection_perf(name, &parts[2..]).await?;
                            } else {
                                println!("Usage: collection perf <name> [--users N] [--duration Ns]");
                            }
                        },
                        Some("docs") => {
                            if let Some(name) = parts.get(2) {
                                self.collection_manager.generate_docs(name).await?;
                            } else {
                                println!("Usage: collection docs <name>");
                            }
                        },
                        _ => println!("Available collection commands: new, run, mock, perf, configure_mock_data, docs"),
                    }
                }
                "save" => {
                    if parts.len() >= 3 {
                        let collection_name = &parts[1];
                        let endpoint_name = &parts[2];
                        if let Some(last_request) = &self.last_request {
                            self.collection_manager.save_request_to_collection(
                                collection_name,
                                endpoint_name,
                                last_request
                            )?;
                        } else {
                            println!("❌ No request to save. Make a call first!");
                        }
                    } else {
                        println!("❌ Usage: save <collection_name> <endpoint_name>");
                    }
                }
                "configure_mock_data" => {
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

    fn store_last_request(&mut self, method: String, url: String, body: Option<String>) {
        self.last_request = Some((method, url, body));
    }
}
