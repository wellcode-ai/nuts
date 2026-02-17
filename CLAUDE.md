# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NUTS (Network Universal Testing Suite) is a Rust CLI tool for MCP server testing and API testing. It operates in two modes:

1. **Non-interactive CLI** -- `nuts call`, `nuts mcp discover`, `nuts mcp test`, etc. Designed for scripts and CI.
2. **Interactive REPL** -- `nuts shell` enters the original interactive shell with tab completion.

AI features (security scanning, test generation) use Anthropic's Claude API via a centralized `AiService`.

## Development Commands

```bash
cargo build                    # Build the project
cargo run                      # Run the CLI (shows help, not REPL)
cargo run -- shell             # Enter the interactive REPL
cargo install --path .         # Install locally as `nuts` binary

cargo test                     # Run all tests
cargo test --lib               # Library tests only
cargo test --bin nuts          # Binary tests only
cargo test <test_name>         # Run a single test by name
cargo test mcp                 # Run MCP-related tests

cargo fmt                      # Format code (CI enforces --check)
cargo clippy --all-targets --all-features -- -D warnings  # Lint (CI treats warnings as errors)
cargo check                    # Quick compile check
```

## CI Requirements

The GitHub Actions CI pipeline (`.github/workflows/ci.yml`) runs on PRs to `main` and `develop`:
- `cargo fmt -- --check` -- formatting must pass
- `cargo clippy --all-targets --all-features -- -D warnings` -- no clippy warnings allowed
- `cargo test --verbose`
- `cargo doc --no-deps --document-private-items`
- Cross-platform builds (Linux, Windows, macOS)
- `cargo audit` for security vulnerabilities

## Architecture

### Execution Flow (main.rs)

`main.rs` uses `clap` derive macros for CLI parsing. The `Cli` struct has a `Commands` enum with subcommands: `Call`, `Perf`, `Security`, `Ask`, `Mcp`, `Config`, `Shell`. No subcommand prints brief help.

Each `run_*` function creates a `tokio::runtime::Runtime` and calls `block_on` for async work. The `shell` subcommand delegates to `NutsShell::run()` (the original REPL).

Global flags: `--json`, `--quiet`, `--no-color`, `--verbose`, `--env`. TTY detection sets `NO_COLOR` when piped.

### MCP Commands (`nuts mcp <subcommand>`)

MCP subcommands use `McpTransportArgs` for transport selection (`--stdio`, `--sse`, `--http`). The `resolve_transport()` function converts these to a `TransportConfig` enum.

Working subcommands:
- `nuts mcp connect --stdio "cmd"` -- connect, print server info, disconnect
- `nuts mcp discover --stdio "cmd" [--json]` -- full capability listing
- `nuts mcp generate --stdio "cmd" [--json]` -- AI-generate test YAML from discovered schemas

Placeholder subcommands (not yet wired): `test`, `perf`, `security`, `snapshot`.

### Error Handling (`src/error.rs`)

`NutsError` enum with `thiserror` + `miette`:
- Variants: `Http`, `Ai`, `Config`, `Mcp`, `Protocol`, `Flow`, `AuthRequired`, `Io`, `InvalidInput`
- `pub type Result<T> = std::result::Result<T, NutsError>;`
- Auto-conversions from `reqwest::Error`, `serde_json::Error`, `serde_yaml::Error`, `std::io::Error`

### MCP Module (`src/mcp/`)

- **`client.rs`** -- `McpClient` wraps the `rmcp` SDK. Methods: `connect_stdio`, `connect_sse`, `connect_http`, `connect` (from `TransportConfig`), `discover`, `list_tools`, `call_tool`, `list_resources`, `read_resource`, `list_prompts`, `get_prompt`, `disconnect`.
- **`types.rs`** -- Data types: `ServerCapabilities`, `Tool`, `Resource`, `ResourceTemplate`, `Prompt`, `PromptArgument`, `ToolResult`, `ContentItem` (Text/Image/Audio/Resource), `ResourceContent`, `PromptResult`, `PromptMessage`, `TransportConfig` (Stdio/Sse/Http).
- **`discovery.rs`** -- `discover()` convenience function, `format_discovery_human()`, `format_discovery_json()`.
- **`generate.rs`** -- `generate_tests(client, ai)` discovers tools and AI-generates YAML test cases. Handles markdown fence stripping.
- **`test_runner.rs`** -- YAML test file parser and assertion engine. `TestFile`/`ServerConfig`/`TestCase`/`TestStep` structs. `run_tests(path)` connects to server, executes tests, returns `TestSummary`. Supports captures (`$.field` JSONPath) and variable references (`${var}`). Human and JSON summary formatters.

### AI Module (`src/ai/`)

- **`service.rs`** -- `AiService` holds a `Box<dyn AiProvider>`, tracks token usage, maintains conversation buffer. Methods: `complete()`, `complete_with_system()`, `chat()`, `converse()`, `generate_test_cases()`, `security_scan()`, `explain()`, `validate_output()`, `suggest_command()`. Default model: `claude-sonnet-4-5-20250929`.
- **`provider.rs`** -- `AiProvider` trait + `AnthropicProvider`. `MockProvider` for tests.
- **`prompts.rs`** -- All prompt templates as functions. MCP-specific: `mcp_test_generation()`, `mcp_security_scan()`, `mcp_output_validation()`. API: `api_security_analysis()`, `command_suggestion()`, `explain_response()`, `natural_language_command()`, etc.

### Output Module (`src/output/`)

- **`renderer.rs`** -- `render_status_line()`, `render_json_body()` (syntax highlighted), `render_headers()`, `render_table()` (comfy-table), `render_error()` (what/why/fix), `render_ai_insight()`, `render_test_result()`, `render_section()`, `spinner_style()`. `OutputMode` enum: Human/Json/Junit/Quiet.
- **`colors.rs`** -- Semantic color system using `console` crate. `init_colors()`, `colors_enabled()`. Styles: success, warning, error, info, muted, accent (+ bold). JSON: json_key (cyan), json_string (green), json_number (yellow), json_bool (magenta), json_null (dim red). Respects `NO_COLOR`.
- **`welcome.rs`** -- `welcome_message()` (3 lines), `first_run_message()`, `help_text()` (grouped by task: MCP TESTING, MAKING REQUESTS, etc.), `command_help(cmd)` per-command help.

### Legacy Modules (unchanged)

- **`src/shell.rs`** -- Interactive REPL with rustyline, tab completion, command dispatch via `process_command()` match block
- **`src/commands/`** -- Original command implementations (call, perf, security, ask, monitor, generate, etc.)
- **`src/flows/`** -- OpenAPI flow management and collection system
- **`src/story/`** -- AI-guided workflow exploration
- **`src/models/`** -- `ApiAnalysis`, `Metrics`, `MetricsSummary`
- **`src/config.rs`** -- Config stored at `~/.nuts/config.json`
- **`src/completer.rs`** -- Tab completion for shell mode

## Key Patterns

- **Error handling**: New modules use `NutsError` via `thiserror`/`miette`. Legacy code still uses `Box<dyn Error>`. `main.rs` run functions return `Box<dyn Error>` to bridge both.
- **AI integration**: New code uses `AiService` (one instance, shared). Legacy commands still create per-call `anthropic::client::Client`.
- **Async**: Each `run_*` function in main.rs creates its own `tokio::runtime::Runtime`. The shell creates one in `NutsShell::run()`.
- **MCP transport**: All MCP commands resolve `--stdio`/`--sse`/`--http` flags to `TransportConfig`, then call `McpClient::connect()`.
- **User data**: All persisted data lives under `~/.nuts/` (config, flows). No database.

## MCP Test YAML Format

Test files (`.test.yaml`) have `server:` (transport config) and `tests:` (array of test cases). See `docs/mcp-test-format.md` for the full specification. Key structures:

```yaml
server:
  command: "node server.js"   # or sse: / http:
  timeout: 30

tests:
  - name: "test name"
    tool: "tool_name"         # or resource: / prompt:
    input: { key: value }
    assert:
      status: success
      result.type: object
      result.has_field: [id, name]
      duration_ms: { max: 5000 }
    capture:
      var_name: "$.field.path"

  - name: "multi-step"
    steps:
      - tool: "create"
        input: { title: "test" }
        capture: { id: "$.id" }
      - tool: "get"
        input: { id: "${id}" }
```

## Dependencies

Key crates:
- **CLI**: `clap` (derive macros)
- **MCP**: `rmcp` (client, transport-child-process, transport-streamable-http, transport-sse)
- **AI**: `anthropic` client crate
- **Error**: `thiserror`, `miette` (fancy diagnostics)
- **HTTP**: `reqwest`, `axum`, `hyper`, `tower`
- **Output**: `comfy-table`, `console`, `indicatif`, `crossterm`
- **Serialization**: `serde`, `serde_json`, `serde_yaml`
- **Async**: `tokio`, `async-trait`
- **Shell**: `rustyline`
