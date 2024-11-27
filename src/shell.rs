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

pub struct NutsShell {
    editor: Editor<NutsCompleter, DefaultHistory>,
    history: Vec<String>,
    suggestions: Vec<String>,
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
                        println!("❌ Usage: perf URL [--users N] [--duration Ns]");
                        return Ok(());
                    }
                    
                    let url = &parts[1];
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

                    PerfCommand::new().run(url, users, duration).await?;
                },
                "security" => {
                    let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY")
                        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
                    SecurityCommand::new(&anthropic_api_key).execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<&str>>()).await?;
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
}