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
- **`src/commands/`** - Command implementations (call, perf, security, config)
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

### Configuration

- Configuration stored in `~/.nuts_config.json`
- Flow collections stored in `~/.nuts/flows/`
- API key required for AI features (security scanning, story mode)

### Dependencies

- **UI/UX**: `ratatui`, `crossterm`, `console`, `inquire`, `dialoguer`
- **HTTP**: `reqwest`, `axum`, `hyper`, `tower`
- **AI**: `anthropic` client
- **CLI**: `clap` for argument parsing, `rustyline` for shell
- **Serialization**: `serde`, `serde_json`, `serde_yaml`
- **Async**: `tokio` runtime

## Command Structure

All commands follow the pattern: `command [subcommand] [options]`

### Main Commands
- `call` - API testing
- `perf` - Performance testing  
- `security` - Security scanning
- `flow` - OpenAPI flow management
- `config` - Configuration management

### Flow Subcommands
- `flow new <name>` - Create new flow
- `flow add <name> <method> <path>` - Add endpoint
- `flow run <name> <endpoint>` - Execute endpoint
- `flow mock <name>` - Start mock server
- `flow story <name>` - Start AI-guided workflow

## Development Notes

- Uses async/await throughout with tokio runtime
- Error handling with custom `ShellError` type
- Progress indicators with `indicatif` crate
- All user data stored in home directory under `.nuts/`
- AI features require Anthropic API key configuration