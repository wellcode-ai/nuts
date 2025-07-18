use crate::completer::NutsCompleter;
use console::style;
use rustyline::Editor;
use rustyline::history::DefaultHistory;
use crate::commands::call::CallCommand;
use crate::commands::security::SecurityCommand;
use crate::commands::perf::PerfCommand;
use crate::commands::test::TestCommand;
use crate::commands::discover::DiscoverCommand;
use crate::commands::predict::PredictCommand;
use crate::commands::ask::AskCommand;
use crate::commands::generate::GenerateCommand;
use crate::commands::monitor::MonitorCommand;
use crate::commands::explain::ExplainCommand;
use crate::commands::fix::FixCommand;
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

#[derive(Debug)]
#[allow(dead_code)]
pub enum ShellError {
    #[allow(dead_code)]
    ApiError(String),
    #[allow(dead_code)]
    ConfigError(String),
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    history: Vec<String>,
    #[allow(dead_code)]
    suggestions: Vec<String>,
    #[allow(dead_code)]
    last_request: Option<(String, String, Option<String>)>,
    last_response: Option<String>,
}

impl NutsShell {
    #[allow(dead_code)]
    fn get_config_path() -> PathBuf {
        let mut path = dirs::home_dir().expect("Could not find home directory");
        path.push(".nuts_config.json");
        path
    }

    #[allow(dead_code)]
    fn save_api_key(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = serde_json::json!({
            "anthropic_api_key": api_key.to_string()
        });
        fs::write(Self::get_config_path(), serde_json::to_string_pretty(&config)?)?;
        Ok(())
    }

    pub fn new() -> Self {
        // Load config first
        let config = Config::load().unwrap_or_default();

        // Initialize editor with completer
        let mut editor = Editor::new().unwrap();
        editor.set_helper(Some(NutsCompleter::new()));
        editor.bind_sequence(rustyline::KeyEvent::from('\t'), rustyline::Cmd::Complete);

        Self {
            editor,
            config,
            history: Vec::new(),
            suggestions: Vec::new(),
            last_request: None,
            last_response: None,
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
    ███╗   ██╗ ██╗   ██╗ ████████╗ ███████╗
    ████╗  ██║ ██║   ██║ ╚══██╔══╝ ██╔════╝ 
    ██╔██╗ ██║ ██║   ██║    ██║    ███████╗
    ██║╚██╗██║ ██║   ██║    ██║    ╚════██║
    ██║ ╚████║ ╚██████╔╝    ██║    ███████║
    ╚═╝  ╚═══╝  ╚═════╝     ╚═╝    ╚══════╝
                                           
    ╔═══════════════════════════════════════════════════════╗
    ║            🤖 AI-POWERED CURL KILLER 🚀              ║
    ║        The Revolutionary API Testing Revolution       ║
    ╚═══════════════════════════════════════════════════════╝
    "#;

        format!(
            "{}\n{}\n{}\n{}\n",
            style(ascii_art).cyan().bold(),
            style("🥜 NUTS v0.1.0 - Talk to APIs Like a Human!").magenta().bold(),
            style("💡 Just say: nuts ask \"Create 5 test users\" and watch the magic!").yellow(),
            style("🎯 Type 'help' to see all AI superpowers").green()
        )
    }

    fn show_help(&self) {
        println!("\n{}", style("🥜 NUTS - API Testing, Performance & Security CLI Tool").cyan().bold());
        println!("{}\n", style("Version 0.1.0 - The Future of API Testing").dim());

        // Revolutionary AI Features
        println!("{}", style("🚀 AI SUPERPOWERS (CURL Killer!)").magenta().bold());
        println!("  {} - AI-powered CURL alternative", style("ask \"Create 5 test users with realistic data\"").green());
        println!("  {} - Generate realistic test data", style("generate users 10").green());
        println!("  {} - Smart API monitoring", style("monitor <URL> --smart").green());
        println!("  {} - AI explains API responses", style("explain").green());
        println!("  {} - Auto-diagnose and fix APIs", style("fix <URL>").green());

        // Smart API Testing
        println!("\n{}", style("⚡ Smart API Testing").yellow());
        println!("  {} - Test with natural language", style("test \"Check if user registration works\"").green());
        println!("  {} - Smart endpoint testing", style("call <METHOD> <URL> [BODY]").green());
        println!("  {} - Auto-discover API endpoints", style("discover <BASE_URL>").green());
        println!("  {} - Predict API health issues", style("predict <BASE_URL>").green());
        println!("  {} - AI-enhanced performance tests", style("perf <METHOD> <URL> [OPTIONS]").green());
        println!("  {} - AI-powered security scanning", style("security <URL> [OPTIONS]").green());

        // Advanced Call Options (CURL-like)
        println!("\n{}", style("🔧 Advanced Call Options (CURL Killer!)").blue());
        println!("  {} - Add custom headers", style("-H \"Content-Type: application/json\"").green());
        println!("  {} - Basic authentication", style("-u username:password").green());
        println!("  {} - Bearer token auth", style("--bearer <token>").green());
        println!("  {} - Send data/body", style("-d '{\"name\": \"test\"}'").green());
        println!("  {} - Form data upload", style("-F \"file=@data.txt\"").green());
        println!("  {} - Verbose debug output", style("-v").green());
        println!("  {} - Include response headers", style("-i").green());
        println!("  {} - Save to file", style("-o response.json").green());
        println!("  {} - Follow redirects", style("-L").green());
        println!("  {} - Set timeout", style("--timeout 30").green());
        println!("  {} - Auto retry requests", style("--retry 3").green());
        println!("  {} - Skip SSL verification", style("-k").green());


        // Configuration
        println!("\n{}", style("⚙️  Configuration").yellow());
        println!("  {} - Configure API key", style("config api-key").green());
        println!("  {} - Show current config", style("config show").green());

        // Revolutionary Examples  
        println!("\n{}", style("🚀 Revolutionary Examples").blue().bold());
        println!("• {}", style("ask \"Create a POST request with user data\"").cyan());
        println!("• {}", style("call -X POST -H \"Content-Type: application/json\" -d '{\"name\":\"test\"}' https://api.example.com/users").cyan());
        println!("• {}", style("call -v -L --bearer abc123 GET https://api.secure.com/data").cyan());
        println!("• {}", style("generate users 50").cyan());
        println!("• {}", style("monitor https://api.myapp.com --smart").cyan());
        println!("• {}", style("test \"Verify pagination works correctly\"").cyan());
        println!("• {}", style("call --retry 3 --timeout 10 POST https://api.unreliable.com").cyan());
        println!("• {}", style("discover https://api.github.com").cyan());
        println!("• {}", style("fix https://api.broken.com").cyan());

        // Pro Tips
        println!("\n{}", style("💡 Pro Tips").blue());
        println!("• Talk to NUTS like a human - it understands natural language!");
        println!("• Use 'ask' instead of memorizing curl commands");
        println!("• Generate unlimited realistic test data with AI");
        println!("• Let AI explain confusing API responses");
        println!("• Monitor APIs smartly to prevent issues");
        println!("• NUTS gets smarter the more you use it!");
    }

    pub async fn process_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<String> = cmd.trim()
            .split_whitespace()
            .map(String::from)
            .collect();

        match parts.first().map(|s| s.as_str()) {
            Some("test") => {
                if parts.len() < 2 {
                    println!("❌ Usage: test \"natural language description\" [base_url]");
                    println!("Examples:");
                    println!("  test \"Check if user registration works with valid email\"");
                    println!("  test \"Verify pagination works correctly\" https://api.example.com");
                    println!("  test \"Ensure rate limiting kicks in after 100 requests\"");
                    return Ok(());
                }

                // Extract the test description (remove quotes if present)
                let description = parts[1..].join(" ").trim_matches('"').to_string();
                
                // Check if last argument looks like a URL
                let base_url = if parts.len() > 2 {
                    let last_part = parts.last().unwrap();
                    if last_part.starts_with("http") {
                        Some(last_part.as_str())
                    } else {
                        None
                    }
                } else {
                    None
                };

                let test_command = TestCommand::new(self.config.clone());
                test_command.execute_natural_language(&description, base_url).await?;
            }
            Some("discover") => {
                if parts.len() < 2 {
                    println!("❌ Usage: discover <BASE_URL>");
                    println!("Examples:");
                    println!("  discover https://api.github.com");
                    println!("  discover https://jsonplaceholder.typicode.com");
                    println!("  discover https://api.myapp.com");
                    return Ok(());
                }

                let base_url = &parts[1];
                let discover_command = DiscoverCommand::new(self.config.clone());
                
                match discover_command.discover(base_url).await {
                    Ok(api_map) => {
                        println!("\n✅ Discovery complete! Found {} endpoints", api_map.endpoints.len());
                        
                        // Ask if user wants to generate a flow
                        if !api_map.endpoints.is_empty() {
                            println!("\n💡 Generate a flow from discovered endpoints? (y/n)");
                            if let Ok(response) = self.editor.readline("🚀 ") {
                                if response.trim().eq_ignore_ascii_case("y") {
                                    let flow_name = format!("discovered-{}", 
                                        base_url.replace("https://", "").replace("http://", "").replace("/", "-"));
                                    discover_command.generate_flow(&api_map, &flow_name).await?;
                                }
                            }
                        }
                    }
                    Err(e) => println!("❌ Discovery failed: {}", e),
                }
            }
            Some("predict") => {
                if parts.len() < 2 {
                    println!("❌ Usage: predict <BASE_URL>");
                    println!("Examples:");
                    println!("  predict https://api.myapp.com");
                    println!("  predict https://api.github.com");
                    println!("  predict https://jsonplaceholder.typicode.com");
                    return Ok(());
                }

                let base_url = &parts[1];
                let predict_command = PredictCommand::new(self.config.clone());
                
                match predict_command.predict_health(base_url).await {
                    Ok(_prediction) => {
                        // Results are already displayed in the predict_health method
                        println!("\n🎯 Prediction complete! Use these insights to prevent issues.");
                    }
                    Err(e) => println!("❌ Prediction failed: {}", e),
                }
            }
            Some("ask") => {
                if parts.len() < 2 {
                    println!("❌ Usage: ask \"natural language request\"");
                    println!("Examples:");
                    println!("  ask \"Create a POST request to add a new user\"");
                    println!("  ask \"Generate 10 test users with realistic data\"");
                    println!("  ask \"Check if the API is working properly\"");
                    println!("  ask \"Make a request to get all products\"");
                    return Ok(());
                }

                let request = parts[1..].join(" ").trim_matches('"').to_string();
                let ask_command = AskCommand::new(self.config.clone());
                
                match ask_command.execute(&request).await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Ask failed: {}", e),
                }
            }
            Some("generate") => {
                if parts.len() < 2 {
                    println!("❌ Usage: generate <data_type> [count]");
                    println!("Examples:");
                    println!("  generate users 10");
                    println!("  generate products 25");
                    println!("  generate orders 5");
                    return Ok(());
                }

                let data_type = &parts[1];
                let count = parts.get(2)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(5);
                
                let generate_command = GenerateCommand::new(self.config.clone());
                
                match generate_command.generate(data_type, count).await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Generate failed: {}", e),
                }
            }
            Some("monitor") => {
                if parts.len() < 2 {
                    println!("❌ Usage: monitor <URL> [--smart]");
                    println!("Examples:");
                    println!("  monitor https://api.example.com");
                    println!("  monitor https://api.example.com --smart");
                    return Ok(());
                }

                let url = &parts[1];
                let smart = parts.contains(&"--smart".to_string());
                
                let monitor_command = MonitorCommand::new(self.config.clone());
                
                match monitor_command.monitor(url, smart).await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Monitor failed: {}", e),
                }
            }
            Some("explain") => {
                if let Some(last_response) = &self.last_response {
                    let explain_command = ExplainCommand::new(self.config.clone());
                    
                    match explain_command.explain_response(last_response, None).await {
                        Ok(_) => {},
                        Err(e) => println!("❌ Explain failed: {}", e),
                    }
                } else {
                    println!("❌ No previous response to explain. Make an API call first!");
                    println!("Usage: call GET https://api.example.com/users, then use 'explain'");
                }
            }
            Some("fix") => {
                if parts.len() < 2 {
                    println!("❌ Usage: fix <URL>");
                    println!("Examples:");
                    println!("  fix https://api.broken.com");
                    println!("  fix https://api.example.com/slow-endpoint");
                    return Ok(());
                }

                let url = &parts[1];
                let fix_command = FixCommand::new(self.config.clone());
                
                match fix_command.auto_fix(url).await {
                    Ok(_) => {},
                    Err(e) => println!("❌ Fix failed: {}", e),
                }
            }
            Some("config") => {
                ConfigCommand::new(self.config.clone())
                    .execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    .await?;
                
                // Reload config
                self.config = Config::load()?;
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
                            .map(|_k| "********".to_string())
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
                    // Use the new enhanced call command
                    let call_command = CallCommand::new();
                    let args: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
                    
                    match call_command.execute(&args).await {
                        Ok(_) => {
                            // For now, we don't store response for advanced calls
                            // TODO: Enhance this to work with the new CallOptions system
                        }
                        Err(e) => println!("❌ Call failed: {}", e),
                    }
                } else {
                    println!("❌ Usage: call [OPTIONS] [METHOD] URL [BODY]");
                    println!("🔧 Advanced Options:");
                    println!("  -H \"Header: Value\"    Add custom headers");
                    println!("  -u username:password  Basic authentication");
                    println!("  --bearer <token>      Bearer token auth");
                    println!("  -d 'data'             Send data/body");
                    println!("  -v                    Verbose output");
                    println!("  -i                    Include headers");
                    println!("  -L                    Follow redirects");
                    println!("  --timeout <sec>       Request timeout");
                    println!("  --retry <num>         Retry failed requests");
                    println!("Examples:");
                    println!("  call GET https://api.example.com/users");
                    println!("  call -v -H \"Authorization: Bearer token\" POST https://api.example.com/users");
                    println!("  call -d '{{\"name\": \"John\"}}' https://api.example.com/users");
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
                let _api_key = self.config.anthropic_api_key.clone()
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

    #[allow(dead_code)]
    fn store_last_request(&mut self, method: String, url: String, body: Option<String>) {
        self.last_request = Some((method, url, body));
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    fn print_info(&self, msg: &str) {
        println!("ℹ️  {}", style(msg).blue());
    }

    #[allow(dead_code)]
    fn print_success(&self, msg: &str) {
        println!("✅ {}", style(msg).green());
    }

    #[allow(dead_code)]
    fn print_warning(&self, msg: &str) {
        println!("⚠️  {}", style(msg).yellow());
    }

    #[allow(dead_code)]
    fn print_error(&self, msg: &str) {
        println!("❌ {}", style(msg).red());
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
