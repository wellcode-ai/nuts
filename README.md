# NUTS - Network Universal Testing Suite

**The AI-Powered CURL** 🚀

NUTS is a revolutionary CLI tool that combines the power of traditional API testing with cutting-edge AI capabilities. It's like having a smart assistant that understands your testing needs and helps you work faster and more efficiently.

## 🌟 Key Features

- **Natural Language Interface** - Just describe what you want: `ask "create a user with realistic data"`
- **Smart API Testing** - Advanced HTTP client with CURL-like features
- **AI-Powered Security Scanning** - Intelligent vulnerability detection
- **Real-time Health Monitoring** - Smart monitoring with predictive insights
- **Performance Testing** - Concurrent load testing with AI analysis
- **OpenAPI Flow Management** - Create, manage, and execute API collections
- **Mock Server Generation** - Generate mock servers from OpenAPI specs
- **Interactive Shell** - Tab completion and command history

## 🚀 Installation

### From Releases

1. Download the appropriate binary for your system from the [releases page](https://github.com/wellcode-ai/nuts/releases)
2. Make it executable (Linux/macOS):
   ```bash
   chmod +x nuts
   ```
3. Move it to your PATH:
   ```bash
   # Linux/macOS
   sudo mv nuts /usr/local/bin/
   
   # Windows
   # Move nuts.exe to a location in your PATH
   ```

### From Source

```bash
# Clone and install
git clone https://github.com/wellcode-ai/nuts
cd nuts
cargo install --path .

# Or install directly from git
cargo install --git https://github.com/wellcode-ai/nuts
```

## ⚡ 30-Second Start (Try It Now!)

```bash
# Install (choose one)
cargo install --git https://github.com/wellcode-ai/nuts
# OR download from releases: https://github.com/wellcode-ai/nuts/releases

# Start the interactive shell
nuts

# Try these commands immediately (no setup needed!)
> call GET https://jsonplaceholder.typicode.com/users
> call POST https://httpbin.org/post '{"name": "Test"}'
> perf GET https://httpbin.org/get --users 10

# AI features (optional - requires API key)
> config api-key  # Enter your Anthropic API key
> ask "Create a user with realistic data"
> monitor https://httpbin.org/get --smart
```

**That's it!** You're now using the most powerful API testing tool ever built. 🚀

## 🤔 What Makes NUTS Different?

**Instead of this (traditional tools):**
```bash
# Complex curl commands
curl -X POST https://api.example.com/users \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer token123" \
  -d '{"name":"John","email":"john@example.com","role":"admin"}'

# Multiple tools needed
curl + jq + ab + custom scripts...
```

**You get this (NUTS):**
```bash
# Simple, powerful commands
> call POST https://api.example.com/users '{"name":"John"}'
> ask "Create a realistic user for testing"
> perf GET https://api.example.com/users --users 100
> monitor https://api.example.com --smart
```

**Why developers love NUTS:**
- 🚀 **Zero learning curve** - If you know curl, you know NUTS
- 🤖 **AI-powered** - Natural language commands, smart monitoring, auto-fix
- ⚡ **All-in-one** - Testing, monitoring, security, performance in one tool
- 🎯 **Production-ready** - Built with Rust for reliability and speed

## 🎯 Detailed Quick Start

### 1. Basic API Testing (Works Immediately)
```bash
# Simple GET request
> call GET https://jsonplaceholder.typicode.com/users

# POST with data
> call POST https://httpbin.org/post '{"name": "John", "email": "john@example.com"}'

# With headers and auth
> call -H "Content-Type: application/json" --bearer "token123" GET https://api.example.com/users
```

### 2. Performance Testing (No Setup Required)
```bash
# Basic load test
> perf GET https://httpbin.org/get

# Advanced load test
> perf GET https://api.example.com/users --users 100 --duration 30s
```

### 3. Configure AI Features (Optional but Recommended)
```bash
> config api-key
# Enter your Anthropic API key for AI features
```

### 4. Natural Language Commands (AI Required)
```bash
# Let AI create the perfect request for you
> ask "Create a POST request to register a new user with realistic data"
> ask "Get all products from an e-commerce API"
> ask "Delete user with ID 123"
```

### 5. Security Scanning (AI Required)
```bash
# Basic security scan
> security https://api.example.com

# Deep security analysis
> security https://api.example.com --deep --save security_report.json
```

### 6. Health Monitoring
```bash
# Basic monitoring
> monitor https://api.example.com

# Smart monitoring with AI insights
> monitor https://api.example.com --smart
```

## 📚 Complete Command Reference

### Core Commands

| Command | Description | Example |
|---------|-------------|---------|
| `call [OPTIONS] [METHOD] URL [BODY]` | Advanced HTTP client | `call GET https://api.example.com/users` |
| `ask "description"` | Natural language to API call | `ask "Create a user with test data"` |
| `perf [METHOD] URL [OPTIONS]` | Performance testing | `perf GET https://api.example.com --users 50` |
| `security URL [OPTIONS]` | AI security scanning | `security https://api.example.com --deep` |
| `monitor URL [--smart]` | Health monitoring | `monitor https://api.example.com --smart` |
| `discover BASE_URL` | Auto-discover endpoints | `discover https://api.example.com` |
| `test "description"` | AI test generation | `test "Check user registration works"` |
| `generate TYPE [count]` | Generate test data | `generate users 10` |
| `predict BASE_URL` | AI health prediction | `predict https://api.example.com` |
| `explain` | Explain last response | `explain` |
| `fix URL` | Auto-fix API issues | `fix https://api.example.com/broken` |
| `config [api-key\|show]` | Configuration | `config api-key` |

### Flow Management

| Command | Description | Example |
|---------|-------------|---------|
| `flow new NAME` | Create new flow | `flow new myapi` |
| `flow add NAME METHOD PATH` | Add endpoint | `flow add myapi GET /users` |
| `flow run NAME ENDPOINT` | Execute endpoint | `flow run myapi /users` |
| `flow list` | List flows | `flow list` |
| `flow docs NAME` | Generate docs | `flow docs myapi` |
| `flow mock NAME [PORT]` | Start mock server | `flow mock myapi 8080` |
| `flow story NAME` | AI-guided workflow | `flow story myapi` |

### Command Aliases
- `c` → `call`
- `p` → `perf`
- `s` → `flow story`
- `h` → `help`
- `q` → `quit`

## 🔧 Call Command Options

The `call` command supports extensive options similar to curl:

```bash
# Headers
call -H "Content-Type: application/json" -H "Authorization: Bearer token" GET https://api.example.com

# Authentication
call -u username:password GET https://api.example.com
call --bearer "token123" GET https://api.example.com

# Verbose output
call -v GET https://api.example.com

# Follow redirects
call -L GET https://api.example.com

# Timeout and retries
call --timeout 30 --retry 3 GET https://api.example.com
```

## 🤖 AI Features

NUTS leverages Anthropic's Claude AI for intelligent automation:

- **Natural Language Processing** - Convert plain English to API calls
- **Security Analysis** - Intelligent vulnerability detection
- **Performance Insights** - AI-powered performance analysis
- **Predictive Monitoring** - Forecast potential issues
- **Smart Test Generation** - Create comprehensive test suites
- **Automatic Fixing** - AI-suggested solutions for API issues

## 🏗️ Architecture

- **Interactive Shell** - Built with `rustyline` for excellent UX
- **Async Runtime** - Powered by `tokio` for high performance
- **HTTP Client** - Uses `reqwest` for reliable HTTP communication
- **AI Integration** - Direct integration with Anthropic's API
- **Configuration** - Stored in `~/.nuts_config.json`
- **Flow Storage** - Collections stored in `~/.nuts/flows/`

## 🤝 Contributing

We welcome contributions! Please check out our [contributing guidelines](CONTRIBUTING.md) for details.

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 🔗 Links

- [Documentation](https://nuts-cli.dev)
- [GitHub Repository](https://github.com/wellcode-ai/nuts)
- [Issue Tracker](https://github.com/wellcode-ai/nuts/issues)
- [Releases](https://github.com/wellcode-ai/nuts/releases)
