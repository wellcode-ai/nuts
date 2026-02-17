use crate::commands::ask::AskCommand;
use crate::commands::call::CallCommand;
use crate::commands::config::ConfigCommand;
use crate::commands::discover::DiscoverCommand;
use crate::commands::explain::ExplainCommand;
use crate::commands::fix::FixCommand;
use crate::commands::generate::GenerateCommand;
use crate::commands::monitor::MonitorCommand;
use crate::commands::perf::PerfCommand;
use crate::commands::predict::PredictCommand;
use crate::commands::security::SecurityCommand;
use crate::commands::test::TestCommand;
use crate::completer::NutsCompleter;
use crate::config::Config;
use crate::output::{colors, renderer, welcome};
use anthropic::client::ClientBuilder;
use anthropic::types::ContentBlock;
use anthropic::types::Message;
use anthropic::types::MessagesRequestBuilder;
use anthropic::types::Role;
use rustyline::history::DefaultHistory;
use rustyline::Editor;

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
        // Initialize the color system
        colors::init_colors(false);

        // Show first-run message if no API key configured, otherwise normal welcome
        if self.config.anthropic_api_key.is_none() {
            println!("{}", welcome::first_run_message());
        } else {
            println!("{}", welcome::welcome_message());
        }

        // Create a single runtime for the entire application
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            loop {
                let readline = self.editor.readline("nuts> ");
                match readline {
                    Ok(line) => {
                        let _ = self.editor.add_history_entry(line.as_str());
                        if let Err(e) = self.process_command(&line).await {
                            renderer::render_error(&e.to_string(), "", "");
                        }
                    }
                    Err(_) => break,
                }
            }
            Ok(())
        })
    }

    fn show_help(&self) {
        print!("{}", welcome::help_text());
    }

    pub async fn process_command(&mut self, cmd: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parts: Vec<String> = cmd.trim().split_whitespace().map(String::from).collect();

        match parts.first().map(|s| s.as_str()) {
            Some("test") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("test"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("test"));
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
                test_command
                    .execute_natural_language(&description, base_url)
                    .await?;
            }
            Some("discover") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("discover"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("discover"));
                    return Ok(());
                }

                let base_url = &parts[1];
                let discover_command = DiscoverCommand::new(self.config.clone());

                match discover_command.discover(base_url).await {
                    Ok(api_map) => {
                        println!(
                            "\n  {} Found {} endpoints",
                            colors::success().apply_to("Discovery complete."),
                            api_map.endpoints.len(),
                        );

                        // Ask if user wants to generate a flow
                        if !api_map.endpoints.is_empty() {
                            println!(
                                "\n  {} Generate a flow from discovered endpoints? (y/n)",
                                colors::muted().apply_to("Hint:"),
                            );
                            if let Ok(response) = self.editor.readline("  > ") {
                                if response.trim().eq_ignore_ascii_case("y") {
                                    let flow_name = format!(
                                        "discovered-{}",
                                        base_url
                                            .replace("https://", "")
                                            .replace("http://", "")
                                            .replace("/", "-")
                                    );
                                    discover_command.generate_flow(&api_map, &flow_name).await?;
                                }
                            }
                        }
                    }
                    Err(e) => renderer::render_error(&format!("Discovery failed: {}", e), "", ""),
                }
            }
            Some("predict") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("predict"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("predict"));
                    return Ok(());
                }

                let base_url = &parts[1];
                let predict_command = PredictCommand::new(self.config.clone());

                match predict_command.predict_health(base_url).await {
                    Ok(_prediction) => {
                        println!("\n  {}", colors::success().apply_to("Prediction complete."));
                    }
                    Err(e) => renderer::render_error(&format!("Prediction failed: {}", e), "", ""),
                }
            }
            Some("ask") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("ask"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("ask"));
                    return Ok(());
                }

                let request = parts[1..].join(" ").trim_matches('"').to_string();
                let ask_command = AskCommand::new(self.config.clone());

                match ask_command.execute(&request).await {
                    Ok(_) => {}
                    Err(e) => renderer::render_error(&format!("Ask failed: {}", e), "", ""),
                }
            }
            Some("generate") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("generate"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("generate"));
                    return Ok(());
                }

                let data_type = &parts[1];
                let count = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);

                let generate_command = GenerateCommand::new(self.config.clone());

                match generate_command.generate(data_type, count).await {
                    Ok(_) => {}
                    Err(e) => renderer::render_error(&format!("Generate failed: {}", e), "", ""),
                }
            }
            Some("monitor") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("monitor"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("monitor"));
                    return Ok(());
                }

                let url = &parts[1];
                let smart = parts.contains(&"--smart".to_string());

                let monitor_command = MonitorCommand::new(self.config.clone());

                match monitor_command.monitor(url, smart).await {
                    Ok(_) => {}
                    Err(e) => renderer::render_error(&format!("Monitor failed: {}", e), "", ""),
                }
            }
            Some("explain") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("explain"));
                    return Ok(());
                }
                if let Some(last_response) = &self.last_response {
                    let explain_command = ExplainCommand::new(self.config.clone());

                    match explain_command.explain_response(last_response, None).await {
                        Ok(_) => {}
                        Err(e) => renderer::render_error(&format!("Explain failed: {}", e), "", ""),
                    }
                } else {
                    renderer::render_error(
                        "No previous response to explain",
                        "Make an API call first, then use 'explain'.",
                        "call GET https://api.example.com/users",
                    );
                }
            }
            Some("fix") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("fix"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("fix"));
                    return Ok(());
                }

                let url = &parts[1];
                let fix_command = FixCommand::new(self.config.clone());

                match fix_command.auto_fix(url).await {
                    Ok(_) => {}
                    Err(e) => renderer::render_error(&format!("Fix failed: {}", e), "", ""),
                }
            }
            Some("config") => {
                ConfigCommand::new(self.config.clone())
                    .execute(&parts.iter().map(|s| s.as_str()).collect::<Vec<_>>())
                    .await?;

                // Reload config
                self.config = Config::load()?;
            }
            Some("configure") => match parts.get(1).map(String::as_str) {
                Some("api-key") => {
                    if let Ok(key) = self
                        .editor
                        .readline_with_initial("Enter Anthropic API Key: ", ("", ""))
                    {
                        self.config.anthropic_api_key = Some(key.trim().to_string());
                        self.config.save()?;
                        println!(
                            "  {}",
                            colors::success().apply_to("API key configured successfully")
                        );
                    }
                }
                Some("show") => {
                    println!("\n  {}", colors::accent().apply_to("Configuration"));
                    println!(
                        "  API Key: {}",
                        self.config
                            .anthropic_api_key
                            .as_ref()
                            .map(|_k| "********".to_string())
                            .unwrap_or_else(|| "Not set".to_string())
                    );
                }
                _ => {
                    print!("{}", welcome::command_help("config"));
                }
            },
            Some("call") | Some("c") => {
                // Handle --help for call command
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("call"));
                    return Ok(());
                }
                if parts.len() > 1 {
                    let call_command = CallCommand::new();
                    let args: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

                    match call_command.execute(&args).await {
                        Ok(_) => {}
                        Err(e) => renderer::render_error(&format!("Call failed: {}", e), "", ""),
                    }
                } else {
                    print!("{}", welcome::command_help("call"));
                }
            }
            Some("help") => {
                // Check if requesting help for a specific command: help <cmd>
                if parts.len() > 1 {
                    print!("{}", welcome::command_help(parts[1].as_str()));
                } else {
                    self.show_help();
                }
            }
            Some("exit") | Some("quit") => std::process::exit(0),
            Some("perf") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("perf"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("perf"));
                    return Ok(());
                }

                let (method, url) = match parts[1].to_uppercase().as_str() {
                    "POST" | "PUT" | "PATCH" => {
                        if parts.len() < 3 {
                            print!("{}", welcome::command_help("perf"));
                            return Ok(());
                        }
                        (parts[1].to_uppercase(), &parts[2])
                    }
                    "DELETE" => {
                        if parts.len() < 3 {
                            print!("{}", welcome::command_help("perf"));
                            return Ok(());
                        }
                        ("DELETE".to_string(), &parts[2])
                    }
                    "GET" | "HEAD" | "OPTIONS" => {
                        if parts.len() < 3 {
                            ("GET".to_string(), &parts[1])
                        } else {
                            (parts[1].to_uppercase(), &parts[2])
                        }
                    }
                    _ => {
                        // If no method specified, assume GET
                        ("GET".to_string(), &parts[1])
                    }
                };

                // Validate URL format
                if !url.starts_with("http://") && !url.starts_with("https://") {
                    println!(
                        "  {}",
                        colors::warning()
                            .apply_to("Warning: URL should start with http:// or https://")
                    );
                }

                let users = parts
                    .iter()
                    .position(|x| x == "--users")
                    .and_then(|i| parts.get(i + 1))
                    .and_then(|u| u.parse().ok())
                    .unwrap_or(10);

                let duration = parts
                    .iter()
                    .position(|x| x == "--duration")
                    .and_then(|i| parts.get(i + 1))
                    .and_then(|d| d.trim_end_matches('s').parse().ok())
                    .map(|secs| std::time::Duration::from_secs(secs))
                    .unwrap_or(std::time::Duration::from_secs(30));

                // Find body if present (after all flags)
                let body = match method.as_str() {
                    "POST" | "PUT" | "PATCH" => parts
                        .iter()
                        .skip_while(|&p| {
                            p == "--users"
                                || p == "--duration"
                                || p.ends_with('s')
                                || p.parse::<u32>().is_ok()
                                || p == &method
                                || p == url
                        })
                        .last()
                        .map(String::as_str),
                    _ => None,
                };

                PerfCommand::new(&self.config)
                    .run(url, users, duration, &method, body)
                    .await?;
            }
            Some("security") => {
                if parts.iter().any(|p| p == "--help" || p == "-h") {
                    print!("{}", welcome::command_help("security"));
                    return Ok(());
                }
                if parts.len() < 2 {
                    print!("{}", welcome::command_help("security"));
                    return Ok(());
                }

                let url = &parts[1];

                if !url.starts_with("http://") && !url.starts_with("https://") {
                    println!(
                        "  {}",
                        colors::warning()
                            .apply_to("Warning: URL should start with http:// or https://")
                    );
                }

                // Check for API key
                if self.config.anthropic_api_key.is_none() {
                    renderer::render_error(
                        "API key not configured",
                        "AI features require an Anthropic API key.",
                        "config api-key",
                    );
                    return Ok(());
                }

                // Parse options
                let deep_scan = parts.contains(&"--deep".to_string());
                let auth_token = parts
                    .iter()
                    .position(|x| x == "--auth")
                    .and_then(|i| parts.get(i + 1))
                    .map(|s| s.to_string());
                let save_file = parts
                    .iter()
                    .position(|x| x == "--save")
                    .and_then(|i| parts.get(i + 1))
                    .map(|s| s.to_string());

                println!(
                    "\n  {} {}",
                    colors::accent().apply_to("Security Scan:"),
                    colors::muted().apply_to(url.as_str()),
                );
                if deep_scan {
                    println!("  {}", colors::muted().apply_to("Mode: deep scan"));
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
                    println!(
                        "  {} {}",
                        colors::muted().apply_to("Did you mean:"),
                        colors::accent().apply_to(&suggestion),
                    );
                } else {
                    renderer::render_error(
                        &format!(
                            "Unknown command: {}",
                            cmd.split_whitespace().next().unwrap_or(cmd)
                        ),
                        "Type 'help' to see available commands.",
                        "",
                    );
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
        match ai_client
            .messages(
                MessagesRequestBuilder::default()
                    .messages(vec![Message {
                        role: Role::User,
                        content: vec![ContentBlock::Text { text: prompt }],
                    }])
                    .model("claude-3-sonnet-20240229".to_string())
                    .max_tokens(100_usize)
                    .build()
                    .ok()?,
            )
            .await
        {
            Ok(response) => {
                if let Some(ContentBlock::Text { text }) = response.content.first() {
                    Some(text.trim().to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}
