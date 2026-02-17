use crate::output::colors;

/// Clean welcome message. 3 lines max. No ASCII art. No marketing.
pub fn welcome_message() -> String {
    let version = env!("CARGO_PKG_VERSION");
    format!(
        "\n  {}  {}\n  {}\n",
        colors::accent_bold().apply_to("NUTS"),
        colors::muted().apply_to(format!("v{} -- MCP & API Testing", version)),
        colors::muted().apply_to("Type 'help' for commands, 'mcp connect' to test MCP servers."),
    )
}

/// First-run message shown when no config exists.
/// Guides the user through initial setup.
pub fn first_run_message() -> String {
    let version = env!("CARGO_PKG_VERSION");
    let mut out = String::new();

    out.push_str(&format!(
        "\n  {} {}\n\n",
        colors::accent_bold().apply_to("NUTS"),
        colors::muted().apply_to(format!("v{}", version)),
    ));

    out.push_str(&format!("  {}\n\n", "Welcome. Let's get you set up.",));

    out.push_str(&format!(
        "  {}\n",
        colors::muted().apply_to("NUTS uses AI for security scanning, test generation, and more."),
    ));
    out.push_str(&format!(
        "  {}\n\n",
        colors::muted().apply_to("To enable AI features, configure your Anthropic API key:"),
    ));

    out.push_str(&format!(
        "    {}\n\n",
        colors::accent().apply_to("config api-key"),
    ));

    out.push_str(&format!("  {}\n\n", "Try these to start:",));

    out.push_str(&format!(
        "    {:<44} {}\n",
        colors::accent().apply_to("call GET https://httpbin.org/get"),
        colors::muted().apply_to("Make your first request"),
    ));
    out.push_str(&format!(
        "    {:<44} {}\n",
        colors::accent().apply_to("ask \"list users from jsonplaceholder\""),
        colors::muted().apply_to("Let AI build the request"),
    ));
    out.push_str(&format!(
        "    {:<44} {}\n",
        colors::accent().apply_to("help"),
        colors::muted().apply_to("See all commands"),
    ));

    out
}

/// Organized help text. Grouped by task, not marketing category.
/// No emoji. No branding fluff. Just the commands.
pub fn help_text() -> String {
    let mut out = String::new();

    let version = env!("CARGO_PKG_VERSION");
    out.push_str(&format!(
        "\n  {}\n",
        colors::accent_bold().apply_to(format!("NUTS v{} -- MCP & API Testing Suite", version)),
    ));

    // MCP Testing
    out.push_str(&format!(
        "\n  {}\n",
        colors::info_bold().apply_to("MCP TESTING"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("mcp connect <transport> <target>"),
        colors::muted().apply_to("Connect to an MCP server"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("mcp discover <transport> <target>"),
        colors::muted().apply_to("Discover tools, resources, prompts"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("mcp test <file>"),
        colors::muted().apply_to("Run MCP test suite"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("mcp security <transport> <target>"),
        colors::muted().apply_to("Security scan an MCP server"),
    ));

    // Making Requests
    out.push_str(&format!(
        "\n  {}\n",
        colors::info_bold().apply_to("MAKING REQUESTS"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("call [METHOD] <url> [body]"),
        colors::muted().apply_to("HTTP request (alias: c)"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("ask \"description\""),
        colors::muted().apply_to("Natural language request"),
    ));

    // Testing & Analysis
    out.push_str(&format!(
        "\n  {}\n",
        colors::info_bold().apply_to("TESTING & ANALYSIS"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("perf [METHOD] <url> [options]"),
        colors::muted().apply_to("Load testing (alias: p)"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("security <url> [--deep]"),
        colors::muted().apply_to("Security scan"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("test \"description\" [url]"),
        colors::muted().apply_to("AI test generation"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("explain"),
        colors::muted().apply_to("Explain last response"),
    ));

    // API Management
    out.push_str(&format!(
        "\n  {}\n",
        colors::info_bold().apply_to("API MANAGEMENT"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("flow new|add|run|list|mock"),
        colors::muted().apply_to("Manage API flows"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("discover <url>"),
        colors::muted().apply_to("Auto-discover endpoints"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("monitor <url> [--smart]"),
        colors::muted().apply_to("Health monitoring"),
    ));

    // Data & Utilities
    out.push_str(&format!(
        "\n  {}\n",
        colors::info_bold().apply_to("DATA & UTILITIES"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("generate <type> [count]"),
        colors::muted().apply_to("Generate test data"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("predict <url>"),
        colors::muted().apply_to("Health prediction"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("fix <url>"),
        colors::muted().apply_to("Auto-diagnose issues"),
    ));

    // Config
    out.push_str(&format!("\n  {}\n", colors::info_bold().apply_to("CONFIG"),));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("config api-key"),
        colors::muted().apply_to("Set API key"),
    ));
    out.push_str(&format!(
        "    {:<40} {}\n",
        colors::accent().apply_to("config show"),
        colors::muted().apply_to("Show configuration"),
    ));

    out.push_str(&format!(
        "\n  {}\n",
        colors::muted().apply_to("Type '<command> --help' for detailed usage."),
    ));

    out
}

/// Per-command help. Returns a focused help block for a single command.
pub fn command_help(cmd: &str) -> String {
    match cmd {
        "call" | "c" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    call [options] [METHOD] <url> [body]\n\
\n\
  {options_label}\n\
    -H \"Key: Value\"    Add request header\n\
    -d 'data'          Send request body (implies POST)\n\
    -v                 Show request/response headers\n\
    -L                 Follow redirects\n\
    -i                 Include response headers\n\
    -o <file>          Save response to file\n\
    -k                 Skip SSL verification\n\
    --bearer <token>   Bearer authentication\n\
    -u user:pass       Basic authentication\n\
    --timeout <secs>   Request timeout (default: 30)\n\
    --retry <n>        Retry on failure\n\
\n\
  {examples_label}\n\
    call https://api.example.com/users\n\
    call POST https://api.example.com/users '{{\"name\":\"test\"}}'\n\
    call -v -H \"Authorization: Bearer tok\" GET https://api.example.com\n",
            title = colors::accent_bold().apply_to("call -- Make HTTP requests"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            options_label = colors::info_bold().apply_to("OPTIONS"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "perf" | "p" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    perf [METHOD] <url> [--users N] [--duration Ns] [body]\n\
\n\
  {options_label}\n\
    --users <n>        Concurrent users (default: 10)\n\
    --duration <Ns>    Test duration (default: 10s)\n\
\n\
  {examples_label}\n\
    perf GET https://api.example.com/users\n\
    perf GET https://api.example.com/users --users 100 --duration 30s\n\
    perf POST https://api.example.com/users --users 50 '{{\"name\": \"Test\"}}'\n",
            title = colors::accent_bold().apply_to("perf -- Performance / load testing"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            options_label = colors::info_bold().apply_to("OPTIONS"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "security" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    security <url> [--deep] [--auth TOKEN] [--save FILE]\n\
\n\
  {options_label}\n\
    --deep             Thorough analysis\n\
    --auth <token>     Authentication token\n\
    --save <file>      Save results to file\n\
\n\
  {examples_label}\n\
    security https://api.example.com\n\
    security https://api.example.com --deep --auth \"Bearer tok\"\n",
            title = colors::accent_bold().apply_to("security -- AI-powered security scanning"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            options_label = colors::info_bold().apply_to("OPTIONS"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "ask" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    ask \"natural language description\"\n\
\n\
  {examples_label}\n\
    ask \"Create a POST request with user data\"\n\
    ask \"Get all products from the API\"\n\
    ask \"Delete user with ID 123\"\n",
            title = colors::accent_bold().apply_to("ask -- Natural language to API call"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "monitor" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    monitor <url> [--smart]\n\
\n\
  {options_label}\n\
    --smart            Enable AI analysis every 3rd check\n\
\n\
  {examples_label}\n\
    monitor https://api.example.com\n\
    monitor https://api.example.com --smart\n",
            title = colors::accent_bold().apply_to("monitor -- Real-time API health monitoring"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            options_label = colors::info_bold().apply_to("OPTIONS"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "test" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    test \"description\" [base_url]\n\
\n\
  {examples_label}\n\
    test \"Check if user registration works\"\n\
    test \"Verify pagination works correctly\" https://api.example.com\n",
            title = colors::accent_bold().apply_to("test -- AI-driven test case generation"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "generate" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    generate <data_type> [count]\n\
\n\
  {examples_label}\n\
    generate users 10\n\
    generate products 5\n\
    generate orders 20\n",
            title = colors::accent_bold().apply_to("generate -- AI-powered test data generation"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "discover" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    discover <base_url>\n\
\n\
  {examples_label}\n\
    discover https://api.example.com\n",
            title = colors::accent_bold().apply_to("discover -- Auto-discover API endpoints"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "predict" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    predict <base_url>\n\
\n\
  {examples_label}\n\
    predict https://api.example.com\n",
            title = colors::accent_bold().apply_to("predict -- AI-powered health prediction"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "explain" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    explain\n\
\n\
  Explains the last API response in human-friendly terms using AI.\n",
            title = colors::accent_bold().apply_to("explain -- AI explains last response"),
            usage_label = colors::info_bold().apply_to("USAGE"),
        ),

        "fix" => format!(
            "\n\
  {title}\n\
\n\
  {usage_label}\n\
    fix <url>\n\
\n\
  {examples_label}\n\
    fix https://api.example.com/broken-endpoint\n",
            title = colors::accent_bold().apply_to("fix -- Auto-diagnose and fix API issues"),
            usage_label = colors::info_bold().apply_to("USAGE"),
            examples_label = colors::info_bold().apply_to("EXAMPLES"),
        ),

        "flow" => format!(
            "\n\
  {title}\n\
\n\
  {subcommands_label}\n\
    flow new <name>                        Create a new flow\n\
    flow add <name> <METHOD> <path>        Add endpoint to flow\n\
    flow run <name> <endpoint>             Execute an endpoint\n\
    flow list                              List all flows\n\
    flow docs <name>                       Generate documentation\n\
    flow mock <name> [port]                Start mock server\n\
    flow story <name>                      AI-guided workflow\n",
            title = colors::accent_bold().apply_to("flow -- Manage API flow collections"),
            subcommands_label = colors::info_bold().apply_to("SUBCOMMANDS"),
        ),

        "config" => format!(
            "\n\
  {title}\n\
\n\
  {subcommands_label}\n\
    config api-key     Set Anthropic API key\n\
    config show        Show current configuration\n",
            title = colors::accent_bold().apply_to("config -- Configuration management"),
            subcommands_label = colors::info_bold().apply_to("SUBCOMMANDS"),
        ),

        "mcp" => format!(
            "\n\
  {title}\n\
\n\
  {subcommands_label}\n\
    mcp connect <transport> <target>       Connect to MCP server\n\
    mcp discover <transport> <target>      Discover capabilities\n\
    mcp test <file>                        Run test suite\n\
    mcp perf <transport> <target>          Performance testing\n\
    mcp security <transport> <target>      Security scanning\n\
    mcp snapshot <transport> <target>      Snapshot testing\n\
    mcp generate <transport> <target>      Generate test suite\n\
\n\
  {transports_label}\n\
    --stdio <cmd>      Spawn process, communicate via stdin/stdout\n\
    --sse <url>        Connect via Server-Sent Events\n\
    --http <url>       Connect via HTTP (Streamable HTTP)\n",
            title = colors::accent_bold().apply_to("mcp -- MCP server testing"),
            subcommands_label = colors::info_bold().apply_to("SUBCOMMANDS"),
            transports_label = colors::info_bold().apply_to("TRANSPORTS"),
        ),

        _ => format!(
            "\n  {} '{}'\n  {}\n",
            colors::warning().apply_to("No help available for"),
            cmd,
            colors::muted().apply_to("Type 'help' to see all commands."),
        ),
    }
}
