# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NUTS (Network Universal Testing Suite) is a Rust CLI tool for API testing, performance testing, and security scanning. It features an interactive shell with tab completion, AI-powered command suggestions, and OpenAPI flow management.

## Development Commands

### Build and Run
```bash
cargo build                    # Build the project
cargo run                      # Run the CLI tool
cargo install --path .         # Install locally
```

### Testing
```bash
cargo test                     # Run all tests
cargo test --lib               # Run library tests only
cargo test --bin nuts          # Run binary tests only
```

### Code Quality
```bash
cargo fmt                      # Format code
cargo clippy                   # Run linter
cargo check                    # Check for compile errors
```

## Architecture

### Core Components

- **`src/main.rs`** - Entry point that initializes the shell
- **`src/shell.rs`** - Main shell implementation with command processing
- **`src/commands/`** - Command implementations (call, perf, security, config, monitor, etc.)
- **`src/flows/`** - OpenAPI flow management and collection system
- **`src/models/`** - Data structures for analysis and metrics
- **`src/config.rs`** - Configuration management with API key storage
- **`src/completer.rs`** - Tab completion for shell commands
- **`src/story/`** - AI-guided workflow system

### Key Features

1. **Interactive Shell** - Uses `rustyline` for command line editing with tab completion
2. **API Testing** - HTTP client with support for all common methods
3. **Performance Testing** - Concurrent load testing with configurable parameters
4. **Security Scanning** - AI-powered security analysis using Anthropic's Claude
5. **OpenAPI Flows** - Create, manage, and execute API collections
6. **Mock Server** - Generate mock servers from OpenAPI specifications
7. **Story Mode** - AI-guided API workflow exploration
8. **Health Monitoring** - Real-time API health monitoring with AI insights
9. **Natural Language Interface** - AI-powered command generation from natural language

### Configuration

- Configuration stored in `~/.nuts_config.json`
- Flow collections stored in `~/.nuts/flows/`
- API key required for AI features (security scanning, story mode, monitoring, natural language)

### Dependencies

- **UI/UX**: `ratatui`, `crossterm`, `console`, `inquire`, `dialoguer`
- **HTTP**: `reqwest`, `axum`, `hyper`, `tower`
- **AI**: `anthropic` client
- **CLI**: `clap` for argument parsing, `rustyline` for shell
- **Serialization**: `serde`, `serde_json`, `serde_yaml`
- **Async**: `tokio` runtime

## Complete Command Reference

### Core Commands

#### `call [OPTIONS] [METHOD] URL [BODY]`
Advanced HTTP client with CURL-like features
- **Options**: `-H` (headers), `-u` (basic auth), `--bearer` (token), `-v` (verbose), `-L` (follow redirects)
- **Examples**: 
  ```bash
  call GET https://api.example.com/users
  call POST https://api.example.com/users '{"name": "John"}'
  call -H "Content-Type: application/json" -v POST https://api.example.com/users '{"name": "John"}'
  ```

#### `ask "natural language request"`
AI-powered natural language to API call conversion
- **Examples**: 
  ```bash
  ask "Create a POST request with user data"
  ask "Get all products from the API"
  ask "Delete user with ID 123"
  ```

#### `perf [METHOD] URL [--users N] [--duration Ns] [BODY]`
Performance testing with concurrent load testing
- **Options**: `--users` (concurrent users), `--duration` (test duration)
- **Examples**: 
  ```bash
  perf GET https://api.example.com/users
  perf GET https://api.example.com/users --users 100 --duration 30s
  perf POST https://api.example.com/users --users 50 '{"name": "Test"}'
  ```

#### `security URL [--deep] [--auth TOKEN] [--save FILE]`
AI-powered security vulnerability scanning
- **Options**: `--deep` (thorough analysis), `--auth` (authentication token), `--save` (save results)
- **Examples**: 
  ```bash
  security https://api.example.com
  security https://api.example.com --deep --auth "Bearer token123"
  security https://api.example.com --save security_report.json
  ```

#### `monitor <URL> [--smart]`
Real-time API health monitoring with AI insights
- **Functionality**: 
  - Performs health checks every 30 seconds
  - Monitors response times and status codes
  - Detects issues (slow responses, errors, empty responses)
  - With `--smart` flag: AI analysis every 3rd check providing trend analysis, predictions, and recommendations
- **Examples**: 
  ```bash
  monitor https://api.example.com
  monitor https://api.example.com --smart
  ```

#### `discover <BASE_URL>`
Auto-discover API endpoints and generate OpenAPI specifications
- **Examples**: 
  ```bash
  discover https://api.example.com
  ```

#### `test "description" [base_url]`
AI-driven test case generation from natural language
- **Examples**: 
  ```bash
  test "Check if user registration works"
  test "Verify pagination works correctly" https://api.example.com
  ```

#### `generate <data_type> [count]`
AI-powered realistic test data generation
- **Examples**: 
  ```bash
  generate users 10
  generate products 5
  generate orders 20
  ```

#### `predict <BASE_URL>`
AI-powered API health prediction and forecasting
- **Examples**: 
  ```bash
  predict https://api.example.com
  ```

#### `explain`
AI explains the last API response in human-friendly terms
- **Examples**: 
  ```bash
  explain
  ```

#### `fix <URL>`
AI-powered automatic API issue detection and fixing
- **Examples**: 
  ```bash
  fix https://api.example.com/broken-endpoint
  ```

#### `config [api-key|show]`
Configuration management
- **Examples**: 
  ```bash
  config api-key
  config show
  ```

### Flow Management Commands

#### `flow new <name>`
Create a new OpenAPI flow collection
- **Examples**: 
  ```bash
  flow new myapi
  flow new user-management
  ```

#### `flow add <name> <METHOD> <path>`
Add an endpoint to an existing flow
- **Examples**: 
  ```bash
  flow add myapi GET /users
  flow add myapi POST /users
  ```

#### `flow run <name> <endpoint>`
Execute a specific endpoint from a flow
- **Examples**: 
  ```bash
  flow run myapi /users
  flow run myapi /users/123
  ```

#### `flow list`
List all available flows
- **Examples**: 
  ```bash
  flow list
  ```

#### `flow docs <name>`
Generate documentation for a flow
- **Examples**: 
  ```bash
  flow docs myapi
  ```

#### `flow mock <name> [port]`
Start a mock server from OpenAPI specification
- **Examples**: 
  ```bash
  flow mock myapi
  flow mock myapi 8080
  ```

#### `flow story <name>`
Start AI-guided interactive workflow exploration
- **Examples**: 
  ```bash
  flow story myapi
  flow s myapi  # shorthand
  ```

#### `flow configure_mock_data <name> <endpoint>`
Configure mock data for specific endpoints
- **Examples**: 
  ```bash
  flow configure_mock_data myapi /users
  ```

### Command Aliases
- `c` → `call`
- `p` → `perf`
- `s` → `flow story`
- `h` → `help`
- `q` → `quit`

## Development Notes

- Uses async/await throughout with tokio runtime
- Error handling with custom `ShellError` type
- Progress indicators with `indicatif` crate
- All user data stored in home directory under `.nuts/`
- AI features require Anthropic API key configuration
- Monitor command performs health checks every 30 seconds with optional AI analysis
- Natural language commands leverage Claude AI for intelligent command generation