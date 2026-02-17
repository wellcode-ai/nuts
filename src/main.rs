mod ai;
mod commands;
mod completer;
mod config;
mod error;
mod flows;
mod mcp;
mod models;
mod output;
mod shell;
mod story;

use clap::{Args, Parser, Subcommand};
use std::io::IsTerminal;

use shell::NutsShell;

/// NUTS -- Network Universal Testing Suite
///
/// Test MCP servers and APIs with AI-powered intelligence.
/// Run with no subcommand to see help. Use `nuts shell` for the interactive REPL.
#[derive(Parser)]
#[command(
    name = "nuts",
    version = env!("CARGO_PKG_VERSION"),
    about = "NUTS - Network Universal Testing Suite",
    long_about = "Test MCP servers and APIs with AI-powered intelligence.\nRun `nuts shell` for the interactive REPL."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Output results as JSON (machine-readable)
    #[arg(long, global = true)]
    json: bool,

    /// Suppress non-essential output
    #[arg(long, global = true)]
    quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Enable verbose / debug output
    #[arg(long, short, global = true)]
    verbose: bool,

    /// Environment name to use (e.g. staging, production)
    #[arg(long, global = true)]
    env: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Make an HTTP request
    Call(CallArgs),

    /// Run performance / load tests
    Perf(PerfArgs),

    /// AI-powered security scan
    Security(SecurityArgs),

    /// Natural-language request via AI
    Ask(AskArgs),

    /// MCP server testing (connect, discover, test, security)
    Mcp(McpArgs),

    /// Manage configuration
    Config(ConfigArgs),

    /// Start the interactive REPL shell
    Shell,
}

// ---- Subcommand argument structs ----

#[derive(Args)]
struct CallArgs {
    /// HTTP method (GET, POST, PUT, PATCH, DELETE). Defaults to GET.
    method: Option<String>,

    /// Target URL
    url: Option<String>,

    /// Request body (JSON string)
    body: Option<String>,

    /// Add a header (-H "Key: Value"), can be repeated
    #[arg(short = 'H', num_args = 1)]
    headers: Vec<String>,

    /// Basic auth (-u user:pass)
    #[arg(short = 'u')]
    user: Option<String>,

    /// Bearer token authentication
    #[arg(long)]
    bearer: Option<String>,

    /// Verbose output
    #[arg(short = 'V')]
    call_verbose: bool,

    /// Follow redirects
    #[arg(short = 'L')]
    follow_redirects: bool,
}

#[derive(Args)]
struct PerfArgs {
    /// HTTP method (defaults to GET)
    method: Option<String>,

    /// Target URL
    url: Option<String>,

    /// Number of concurrent users
    #[arg(long, default_value = "10")]
    users: u32,

    /// Test duration (e.g. "30s")
    #[arg(long, default_value = "30s")]
    duration: String,

    /// Request body for POST/PUT/PATCH
    body: Option<String>,
}

#[derive(Args)]
struct SecurityArgs {
    /// Target URL to scan
    url: String,

    /// Enable deep / thorough scan
    #[arg(long)]
    deep: bool,

    /// Auth token to include in requests
    #[arg(long)]
    auth: Option<String>,

    /// Save report to file
    #[arg(long)]
    save: Option<String>,
}

#[derive(Args)]
struct AskArgs {
    /// Natural-language description of what you want
    description: Vec<String>,
}

#[derive(Args)]
struct McpArgs {
    #[command(subcommand)]
    command: Option<McpCommands>,
}

#[derive(Subcommand)]
enum McpCommands {
    /// Connect to an MCP server and print server info
    Connect(McpTransportArgs),
    /// Discover tools, resources, and prompts
    Discover(McpTransportArgs),
    /// Run test suite against an MCP server
    Test(McpTestArgs),
    /// Performance test MCP tool calls
    Perf(McpPerfArgs),
    /// Security scan an MCP server
    Security(McpTransportArgs),
    /// Capture or compare output snapshots
    Snapshot(McpSnapshotArgs),
    /// AI-generate test suite from discovered schema
    Generate(McpTransportArgs),
}

/// Shared transport arguments for MCP subcommands.
#[derive(Args)]
struct McpTransportArgs {
    /// Connect via stdio by spawning a command (e.g. "npx my-server")
    #[arg(long)]
    stdio: Option<String>,

    /// Connect via Server-Sent Events transport
    #[arg(long)]
    sse: Option<String>,

    /// Connect via Streamable HTTP transport
    #[arg(long)]
    http: Option<String>,

    /// Bearer token for SSE/HTTP authentication
    #[arg(long)]
    bearer: Option<String>,

    /// Set environment variable for stdio transport (KEY=VALUE), can be repeated
    #[arg(long = "set-env", value_name = "KEY=VALUE")]
    env_vars: Vec<String>,

    /// Connection / call timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
}

/// Arguments for `nuts mcp test` which adds a test file path.
#[derive(Args)]
struct McpTestArgs {
    /// Path to the YAML test file or directory
    test_path: Option<String>,

    /// Connect via stdio by spawning a command
    #[arg(long)]
    stdio: Option<String>,

    /// Connect via Server-Sent Events transport
    #[arg(long)]
    sse: Option<String>,

    /// Connect via Streamable HTTP transport
    #[arg(long)]
    http: Option<String>,

    /// Bearer token for SSE/HTTP authentication
    #[arg(long)]
    bearer: Option<String>,

    /// Set environment variable for stdio transport (KEY=VALUE), can be repeated
    #[arg(long = "set-env", value_name = "KEY=VALUE")]
    env_vars: Vec<String>,

    /// Connection / call timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,
}

/// Arguments for `nuts mcp perf` which adds tool-specific perf options.
#[derive(Args)]
struct McpPerfArgs {
    /// Connect via stdio by spawning a command (e.g. "npx my-server")
    #[arg(long)]
    stdio: Option<String>,

    /// Connect via Server-Sent Events transport
    #[arg(long)]
    sse: Option<String>,

    /// Connect via Streamable HTTP transport
    #[arg(long)]
    http: Option<String>,

    /// Bearer token for SSE/HTTP authentication
    #[arg(long)]
    bearer: Option<String>,

    /// Set environment variable for stdio transport (KEY=VALUE), can be repeated
    #[arg(long = "set-env", value_name = "KEY=VALUE")]
    env_vars: Vec<String>,

    /// Connection / call timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Tool name to benchmark (required)
    #[arg(long)]
    tool: String,

    /// JSON input for the tool (default: {})
    #[arg(long, default_value = "{}")]
    input: String,

    /// Number of iterations to run (default: 100)
    #[arg(long, default_value = "100")]
    iterations: u32,

    /// Number of concurrent calls (default: 1, >1 is future work)
    #[arg(long, default_value = "1")]
    concurrency: u32,

    /// Number of warmup iterations to discard (default: 5)
    #[arg(long, default_value = "5")]
    warmup: u32,
}

/// Arguments for `nuts mcp snapshot` with capture/compare modes.
#[derive(Args)]
struct McpSnapshotArgs {
    /// Connect via stdio by spawning a command (e.g. "npx my-server")
    #[arg(long)]
    stdio: Option<String>,

    /// Connect via Server-Sent Events transport
    #[arg(long)]
    sse: Option<String>,

    /// Connect via Streamable HTTP transport
    #[arg(long)]
    http: Option<String>,

    /// Bearer token for SSE/HTTP authentication
    #[arg(long)]
    bearer: Option<String>,

    /// Set environment variable for stdio transport (KEY=VALUE), can be repeated
    #[arg(long = "set-env", value_name = "KEY=VALUE")]
    env_vars: Vec<String>,

    /// Connection / call timeout in seconds
    #[arg(long, default_value = "30")]
    timeout: u64,

    /// Capture mode: connect, call all tools, save snapshot
    #[arg(long)]
    capture: bool,

    /// Compare mode: path to baseline snapshot JSON to compare against
    #[arg(long, value_name = "BASELINE")]
    compare: Option<String>,

    /// Output file path for capture mode (default: print to stdout)
    #[arg(short = 'o', long, value_name = "FILE")]
    output: Option<String>,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: Option<ConfigCommands>,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a configuration value (e.g. `config set api-key <KEY>`)
    Set {
        /// Key to set
        key: String,
        /// Value
        value: String,
    },
    /// Show current configuration
    Show,
}

// ---- Execution ----

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Detect whether stdout is a terminal (useful for auto-disabling colors)
    let is_tty = std::io::stdout().is_terminal();

    // Disable colors when piped or explicitly requested
    if cli.no_color || !is_tty {
        // console crate respects NO_COLOR env
        std::env::set_var("NO_COLOR", "1");
    }

    // Initialize the color system for non-shell CLI paths
    output::colors::init_colors(cli.no_color);

    match cli.command {
        // No subcommand -> show help
        None => {
            print_brief_help();
            Ok(())
        }

        Some(Commands::Shell) => {
            let mut shell = NutsShell::new();
            shell.run()
        }

        Some(Commands::Call(ref args)) => run_call(args, &cli),
        Some(Commands::Perf(ref args)) => run_perf(args, &cli),
        Some(Commands::Security(ref args)) => run_security(args, &cli),
        Some(Commands::Ask(ref args)) => run_ask(args),
        Some(Commands::Mcp(ref args)) => run_mcp(args, &cli),
        Some(Commands::Config(ref args)) => run_config(args),
    }
}

// ---- Brief help (shown when invoked with no subcommand) ----

fn print_brief_help() {
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("nuts {version} - Network Universal Testing Suite\n");
    eprintln!("Usage: nuts <COMMAND> [OPTIONS]\n");
    eprintln!("Commands:");
    eprintln!("  call       Make an HTTP request");
    eprintln!("  perf       Run performance / load tests");
    eprintln!("  security   AI-powered security scan");
    eprintln!("  ask        Natural-language request via AI");
    eprintln!("  mcp        MCP server testing");
    eprintln!("  config     Manage configuration");
    eprintln!("  shell      Start the interactive REPL\n");
    eprintln!("Global flags: --json  --quiet  --no-color  --verbose  --env <NAME>\n");
    eprintln!("Run `nuts <COMMAND> --help` for details on a specific command.");
    eprintln!("Run `nuts shell` for the interactive experience.");
}

// ---- Command runners ----

fn run_call(args: &CallArgs, cli: &Cli) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Resolve method and url.  The first positional could be a method or a URL.
    let (method, url, body) = resolve_call_args(&args)?;

    // Build a token list compatible with the existing CallCommand::execute(&[&str]) interface.
    let mut tokens: Vec<String> = vec!["call".into()];

    if args.call_verbose || cli.verbose {
        tokens.push("-v".into());
    }
    if args.follow_redirects {
        tokens.push("-L".into());
    }
    for h in &args.headers {
        tokens.push("-H".into());
        tokens.push(h.clone());
    }
    if let Some(ref u) = args.user {
        tokens.push("-u".into());
        tokens.push(u.clone());
    }
    if let Some(ref b) = args.bearer {
        tokens.push("--bearer".into());
        tokens.push(b.clone());
    }

    tokens.push(method);
    tokens.push(url);
    if let Some(b) = body {
        tokens.push(b);
    }

    let token_refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
    let call_cmd = crate::commands::call::CallCommand::new();

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(call_cmd.execute(&token_refs))?;
    Ok(())
}

fn resolve_call_args(
    args: &CallArgs,
) -> std::result::Result<(String, String, Option<String>), Box<dyn std::error::Error>> {
    let known_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

    match (&args.method, &args.url) {
        (Some(first), Some(second)) => {
            if known_methods.contains(&first.to_uppercase().as_str()) {
                // nuts call GET <url> [body]
                Ok((first.to_uppercase(), second.clone(), args.body.clone()))
            } else {
                // first is actually the url, second is the body
                Ok(("GET".into(), first.clone(), Some(second.clone())))
            }
        }
        (Some(first), None) => {
            // Only one positional -- treat as URL with GET
            Ok(("GET".into(), first.clone(), None))
        }
        _ => Err("URL is required. Usage: nuts call [METHOD] <URL> [BODY]".into()),
    }
}

fn run_perf(args: &PerfArgs, cli: &Cli) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let known_methods = ["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
    let (method, url) = match (&args.method, &args.url) {
        (Some(first), Some(second)) => {
            if known_methods.contains(&first.to_uppercase().as_str()) {
                (first.to_uppercase(), second.clone())
            } else {
                ("GET".into(), first.clone())
            }
        }
        (Some(first), None) => ("GET".into(), first.clone()),
        _ => return Err("URL is required. Usage: nuts perf [METHOD] <URL>".into()),
    };

    let duration_secs: u64 = args.duration.trim_end_matches('s').parse().unwrap_or(30);
    let duration = std::time::Duration::from_secs(duration_secs);

    let cfg = crate::config::Config::load().unwrap_or_default();
    let perf_cmd = crate::commands::perf::PerfCommand::new(&cfg);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(perf_cmd.run(&url, args.users, duration, &method, args.body.as_deref()))?;

    if cli.quiet {
        // In quiet mode the perf command already printed; future work will
        // use an OutputRenderer to respect --quiet and --json.
    }
    Ok(())
}

fn run_security(
    args: &SecurityArgs,
    _cli: &Cli,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let cfg = crate::config::Config::load()?;
    let _api_key = cfg
        .anthropic_api_key
        .as_ref()
        .ok_or("API key not configured. Run: nuts config set api-key <KEY>")?;

    let mut cmd = crate::commands::security::SecurityCommand::new(cfg.clone());
    cmd = cmd.with_deep_scan(args.deep);
    if let Some(ref token) = args.auth {
        cmd = cmd.with_auth(Some(token.clone()));
    }
    if let Some(ref file) = args.save {
        cmd = cmd.with_save_file(Some(file.clone()));
    }

    let tokens: Vec<String> = {
        let mut t = vec!["security".into(), args.url.clone()];
        if args.deep {
            t.push("--deep".into());
        }
        if let Some(ref token) = args.auth {
            t.push("--auth".into());
            t.push(token.clone());
        }
        if let Some(ref file) = args.save {
            t.push("--save".into());
            t.push(file.clone());
        }
        t
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(cmd.execute(&tokens))?;
    Ok(())
}

fn run_ask(args: &AskArgs) -> std::result::Result<(), Box<dyn std::error::Error>> {
    if args.description.is_empty() {
        return Err("Description is required. Usage: nuts ask \"your request here\"".into());
    }

    let description = args.description.join(" ");
    let cfg = crate::config::Config::load()?;
    let ask_cmd = crate::commands::ask::AskCommand::new(cfg);

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(ask_cmd.execute(&description))?;
    Ok(())
}

fn run_mcp(args: &McpArgs, cli: &Cli) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        None => {
            eprintln!("MCP server testing commands:\n");
            eprintln!("  nuts mcp connect  --stdio \"cmd\" | --sse <url> | --http <url>");
            eprintln!("  nuts mcp discover --stdio \"cmd\" | --sse <url> | --http <url>");
            eprintln!("  nuts mcp test     [test_file.yaml]  (server config in YAML)");
            eprintln!("  nuts mcp perf     --stdio \"cmd\" | --sse <url> | --http <url>");
            eprintln!("  nuts mcp security --stdio \"cmd\" | --sse <url> | --http <url>");
            eprintln!("  nuts mcp snapshot --stdio \"cmd\" | --sse <url> | --http <url>");
            eprintln!("  nuts mcp generate --stdio \"cmd\" | --sse <url> | --http <url>\n");
            eprintln!("Run `nuts mcp <COMMAND> --help` for details.");
            Ok(())
        }
        Some(McpCommands::Connect(ref transport)) => mcp_connect(transport),
        Some(McpCommands::Discover(ref transport)) => mcp_discover(transport, cli.json),
        Some(McpCommands::Test(ref test_args)) => mcp_test(test_args, cli.json),
        Some(McpCommands::Perf(ref perf_args)) => mcp_perf(perf_args, cli.json),
        Some(McpCommands::Security(ref transport)) => mcp_security(transport, cli.json),
        Some(McpCommands::Snapshot(ref snap_args)) => mcp_snapshot(snap_args, cli.json),
        Some(McpCommands::Generate(ref transport)) => mcp_generate(transport, cli.json),
    }
}

// ---------------------------------------------------------------------------
// MCP transport resolution
// ---------------------------------------------------------------------------

/// Parse transport args into a `TransportConfig`, returning a helpful error
/// if no transport is specified.
fn resolve_transport(
    stdio: &Option<String>,
    sse: &Option<String>,
    http: &Option<String>,
    bearer: &Option<String>,
    env_vars: &[String],
) -> std::result::Result<crate::mcp::types::TransportConfig, Box<dyn std::error::Error>> {
    use crate::mcp::types::TransportConfig;

    if let Some(ref cmd) = stdio {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return Err("--stdio requires a command string".into());
        }
        let command = parts[0].to_string();
        let args = parts[1..].iter().map(|s| s.to_string()).collect();
        let env = parse_env_vars(env_vars)?;
        return Ok(TransportConfig::Stdio { command, args, env });
    }
    if let Some(ref url) = sse {
        return Ok(TransportConfig::Sse {
            url: url.clone(),
            bearer: bearer.clone(),
        });
    }
    if let Some(ref url) = http {
        return Ok(TransportConfig::Http {
            url: url.clone(),
            bearer: bearer.clone(),
        });
    }

    Err("A transport is required. Use --stdio, --sse, or --http.".into())
}

/// Parse `KEY=VALUE` strings into (key, value) pairs.
fn parse_env_vars(
    vars: &[String],
) -> std::result::Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    vars.iter()
        .map(|s| {
            let (key, value) = s
                .split_once('=')
                .ok_or_else(|| format!("Invalid --env format: '{}'. Expected KEY=VALUE", s))?;
            Ok((key.to_string(), value.to_string()))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// MCP subcommand implementations
// ---------------------------------------------------------------------------

fn mcp_connect(
    transport: &McpTransportArgs,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &transport.stdio,
        &transport.sse,
        &transport.http,
        &transport.bearer,
        &transport.env_vars,
    )?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = crate::mcp::client::McpClient::connect(&config).await?;
        let caps = client.discover().await?;

        crate::output::renderer::render_section(
            "Connected",
            &format!(
                "Server: {} v{}\nProtocol: {}\nTools: {}  Resources: {}  Prompts: {}",
                caps.server_name,
                caps.server_version,
                caps.protocol_version,
                caps.tools.len(),
                caps.resources.len() + caps.resource_templates.len(),
                caps.prompts.len(),
            ),
        );

        client.disconnect().await?;
        Ok(())
    })
}

fn mcp_discover(
    transport: &McpTransportArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &transport.stdio,
        &transport.sse,
        &transport.http,
        &transport.bearer,
        &transport.env_vars,
    )?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = crate::mcp::client::McpClient::connect(&config).await?;
        let caps = crate::mcp::discovery::discover(&client).await?;

        if json_output {
            let json = crate::mcp::discovery::format_discovery_json(&caps);
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            crate::output::renderer::render_discovery(&caps);
        }

        client.disconnect().await?;
        Ok(())
    })
}

fn mcp_test(
    test_args: &McpTestArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let path = resolve_test_path(test_args.test_path.as_deref())?;

    let rt = tokio::runtime::Runtime::new()?;
    let summary = rt.block_on(crate::mcp::test_runner::run_tests(&path))?;

    if json_output {
        let json = crate::mcp::test_runner::format_summary_json(&summary);
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        crate::output::renderer::render_test_summary(&summary);
    }

    if summary.failed > 0 {
        std::process::exit(1);
    }
    Ok(())
}

/// Resolve the test file path, checking defaults if none provided.
fn resolve_test_path(
    provided: Option<&str>,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    if let Some(p) = provided {
        return Ok(p.to_string());
    }

    // Check default locations
    let defaults = [
        "mcp-tests.yaml",
        "mcp-tests.yml",
        ".nuts/mcp/tests/mcp-tests.yaml",
        ".nuts/mcp/tests/mcp-tests.yml",
    ];
    for candidate in &defaults {
        if std::path::Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }

    Err("No test file specified and no default found.\n\
         Usage: nuts mcp test <test_file.yaml>\n\
         Or place a file at mcp-tests.yaml or .nuts/mcp/tests/mcp-tests.yaml"
        .into())
}

fn mcp_perf(
    perf_args: &McpPerfArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &perf_args.stdio,
        &perf_args.sse,
        &perf_args.http,
        &perf_args.bearer,
        &perf_args.env_vars,
    )?;

    let input: serde_json::Value =
        serde_json::from_str(&perf_args.input).map_err(|e| format!("Invalid --input JSON: {e}"))?;

    let perf_config = crate::mcp::perf::PerfConfig {
        iterations: perf_args.iterations,
        concurrency: perf_args.concurrency,
        warmup: perf_args.warmup,
    };

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let spinner =
            indicatif::ProgressBar::new((perf_config.warmup + perf_config.iterations) as u64);
        spinner.set_style(
            indicatif::ProgressStyle::with_template(
                "  {spinner:.cyan} {msg} [{bar:30.cyan/dim}] {pos}/{len}",
            )
            .unwrap_or_else(|_| indicatif::ProgressStyle::default_bar()),
        );
        spinner.set_message(format!("Benchmarking '{}'...", perf_args.tool));

        let report = crate::mcp::perf::run_perf(
            &config,
            &perf_args.tool,
            input,
            &perf_config,
            Some(&|done, _total| {
                spinner.set_position(done as u64);
            }),
        )
        .await?;

        spinner.finish_and_clear();

        if json_output {
            let json = crate::mcp::perf::format_report_json(&report);
            println!("{}", serde_json::to_string_pretty(&json)?);
        } else {
            crate::output::renderer::render_perf_report(&report);
        }

        Ok(())
    })
}

fn mcp_snapshot(
    snap_args: &McpSnapshotArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &snap_args.stdio,
        &snap_args.sse,
        &snap_args.http,
        &snap_args.bearer,
        &snap_args.env_vars,
    )?;

    if !snap_args.capture && snap_args.compare.is_none() {
        return Err(
            "Specify --capture to take a snapshot or --compare <baseline.json> to compare.\n\
             Examples:\n  \
             nuts mcp snapshot --capture --stdio \"my-server\" -o baseline.json\n  \
             nuts mcp snapshot --compare baseline.json --stdio \"my-server\""
                .into(),
        );
    }

    let rt = tokio::runtime::Runtime::new()?;

    if snap_args.capture {
        // Capture mode
        rt.block_on(async {
            let spinner = indicatif::ProgressBar::new_spinner();
            spinner.set_style(crate::output::renderer::spinner_style());
            spinner.set_message("Capturing snapshot...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(120));

            let client = crate::mcp::client::McpClient::connect(&config).await?;
            let snapshot = crate::mcp::snapshot::capture_snapshot(&client).await?;
            client.disconnect().await?;

            spinner.finish_and_clear();

            if let Some(ref path) = snap_args.output {
                crate::mcp::snapshot::save_snapshot(&snapshot, path)?;
                if !json_output {
                    crate::output::renderer::render_snapshot_capture(&snapshot);
                    println!("  Saved to: {}\n", crate::output::colors::accent().apply_to(path));
                }
            } else if json_output {
                let json = serde_json::to_string_pretty(&snapshot)?;
                println!("{}", json);
            } else {
                // Print snapshot JSON to stdout (useful for piping)
                let json = serde_json::to_string_pretty(&snapshot)?;
                println!("{}", json);
            }

            Ok(())
        })
    } else {
        // Compare mode
        let baseline_path = snap_args.compare.as_ref().unwrap();
        let baseline = crate::mcp::snapshot::load_snapshot(baseline_path)?;

        rt.block_on(async {
            let spinner = indicatif::ProgressBar::new_spinner();
            spinner.set_style(crate::output::renderer::spinner_style());
            spinner.set_message("Capturing current snapshot for comparison...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(120));

            let client = crate::mcp::client::McpClient::connect(&config).await?;
            let current = crate::mcp::snapshot::capture_snapshot(&client).await?;
            client.disconnect().await?;

            spinner.finish_and_clear();

            let result = crate::mcp::snapshot::compare_snapshots(&baseline, &current);

            if json_output {
                let json = crate::mcp::snapshot::format_compare_json(&result);
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                crate::output::renderer::render_snapshot_compare(&result);
            }

            if result.changed > 0 || result.added > 0 || result.removed > 0 {
                std::process::exit(1);
            }

            Ok(())
        })
    }
}

fn mcp_security(
    transport: &McpTransportArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &transport.stdio,
        &transport.sse,
        &transport.http,
        &transport.bearer,
        &transport.env_vars,
    )?;

    // Load API key
    let cfg = crate::config::Config::load().unwrap_or_default();
    let api_key = match cfg.anthropic_api_key.as_deref() {
        Some(key) if !key.is_empty() => key.to_string(),
        _ => {
            crate::output::renderer::render_error(
                "API key required for MCP security scanning",
                "The 'mcp security' command uses AI to analyze tool schemas and probe for vulnerabilities.",
                "nuts config set api-key <YOUR_ANTHROPIC_API_KEY>",
            );
            return Ok(());
        }
    };

    let ai = crate::ai::AiService::new(&api_key)
        .map_err(|e| format!("Failed to initialize AI service: {e}"))?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = crate::mcp::client::McpClient::connect(&config).await?;
        let report = crate::mcp::security::security_scan(&client, &ai).await?;

        if json_output {
            let json = crate::mcp::security::format_report_json(&report)?;
            println!("{}", json);
        } else {
            crate::mcp::security::render_report(&report);
        }

        client.disconnect().await?;
        Ok(())
    })
}

fn mcp_generate(
    transport: &McpTransportArgs,
    json_output: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = resolve_transport(
        &transport.stdio,
        &transport.sse,
        &transport.http,
        &transport.bearer,
        &transport.env_vars,
    )?;

    // Load API key
    let cfg = crate::config::Config::load().unwrap_or_default();
    let api_key = match cfg.anthropic_api_key.as_deref() {
        Some(key) if !key.is_empty() => key.to_string(),
        _ => {
            crate::output::renderer::render_error(
                "API key required for AI test generation",
                "The 'mcp generate' command uses AI to create test cases from discovered tool schemas.",
                "nuts config set api-key <YOUR_ANTHROPIC_API_KEY>",
            );
            return Ok(());
        }
    };

    let ai = crate::ai::AiService::new(&api_key)
        .map_err(|e| format!("Failed to initialize AI service: {e}"))?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let client = crate::mcp::client::McpClient::connect(&config).await?;
        let yaml = crate::mcp::generate::generate_tests(&client, &ai).await?;

        if yaml.is_empty() {
            // No tools found, message already printed by generate_tests
        } else if json_output {
            // Parse YAML to JSON for --json output
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(&yaml).unwrap_or(serde_yaml::Value::String(yaml.clone()));
            let json = serde_json::to_string_pretty(&yaml_value)?;
            println!("{}", json);
        } else {
            println!("{}", yaml);
        }

        client.disconnect().await?;
        Ok(())
    })
}

fn run_config(args: &ConfigArgs) -> std::result::Result<(), Box<dyn std::error::Error>> {
    match &args.command {
        None => {
            eprintln!("Usage:");
            eprintln!("  nuts config set <KEY> <VALUE>  Set a config value");
            eprintln!("  nuts config show               Show current config");
            Ok(())
        }
        Some(ConfigCommands::Show) => {
            let cfg = crate::config::Config::load().unwrap_or_default();
            let masked_key = cfg
                .anthropic_api_key
                .as_ref()
                .map(|_| "********".to_string())
                .unwrap_or_else(|| "not set".into());
            println!("Configuration:");
            println!("  anthropic_api_key: {masked_key}");
            Ok(())
        }
        Some(ConfigCommands::Set { key, value }) => {
            let mut cfg = crate::config::Config::load().unwrap_or_default();
            match key.as_str() {
                "api-key" | "anthropic-api-key" | "anthropic_api_key" => {
                    cfg.anthropic_api_key = Some(value.clone());
                    cfg.save()?;
                    println!("API key saved.");
                }
                other => {
                    eprintln!("Unknown config key: {other}");
                    eprintln!("Available keys: api-key");
                }
            }
            Ok(())
        }
    }
}
