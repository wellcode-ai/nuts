# NUTS Website

This is the official documentation website for NUTS (Network Universal Testing Suite) - The AI-Powered CURL Killer.

## NUTS - Network Universal Testing Suite

**The AI-Powered CURL Killer** 🚀

NUTS is a revolutionary CLI tool that combines the power of traditional API testing with cutting-edge AI capabilities. It's like having a smart assistant that understands your testing needs and helps you work faster and more efficiently.

### 🌟 Key Features

- **Natural Language Interface** - Just describe what you want: `ask "create a user with realistic data"`
- **Smart API Testing** - Advanced HTTP client with CURL-like features
- **AI-Powered Security Scanning** - Intelligent vulnerability detection
- **Real-time Health Monitoring** - Smart monitoring with predictive insights
- **Performance Testing** - Concurrent load testing with AI analysis
- **OpenAPI Flow Management** - Create, manage, and execute API collections
- **Mock Server Generation** - Generate mock servers from OpenAPI specs
- **Interactive Shell** - Tab completion and command history

### 🚀 Quick Start

#### 1. Installation
```bash
# From releases
curl -L https://github.com/wellcode-ai/nuts/releases/latest/download/nuts-linux-amd64 -o nuts
chmod +x nuts
sudo mv nuts /usr/local/bin/

# From source
cargo install --git https://github.com/wellcode-ai/nuts
```

#### 2. Basic Usage
```bash
# Start the interactive shell
nuts

# Basic API testing
> call GET https://jsonplaceholder.typicode.com/users

# Natural language commands (AI required)
> ask "Create a POST request to register a new user with realistic data"

# Performance testing
> perf GET https://api.example.com/users --users 100 --duration 30s

# Security scanning
> security https://api.example.com --deep

# Health monitoring
> monitor https://api.example.com --smart
```

### 📚 Complete Command Reference

#### Core Commands

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

#### Flow Management

| Command | Description | Example |
|---------|-------------|---------|
| `flow new NAME` | Create new flow | `flow new myapi` |
| `flow add NAME METHOD PATH` | Add endpoint | `flow add myapi GET /users` |
| `flow run NAME ENDPOINT` | Execute endpoint | `flow run myapi /users` |
| `flow list` | List flows | `flow list` |
| `flow docs NAME` | Generate docs | `flow docs myapi` |
| `flow mock NAME [PORT]` | Start mock server | `flow mock myapi 8080` |
| `flow story NAME` | AI-guided workflow | `flow story myapi` |

#### Monitor Command Details

The `monitor` command provides real-time API health monitoring:

- **Basic monitoring**: `monitor https://api.example.com`
  - Performs health checks every 30 seconds
  - Monitors response times and status codes
  - Detects issues (slow responses, errors, empty responses)

- **Smart monitoring**: `monitor https://api.example.com --smart`
  - All basic monitoring features
  - AI analysis every 3rd check (every 90 seconds)
  - Provides trend analysis, predictions, and recommendations
  - Requires Anthropic API key configuration

### 🤖 AI Features

NUTS leverages Anthropic's Claude AI for intelligent automation:

- **Natural Language Processing** - Convert plain English to API calls
- **Security Analysis** - Intelligent vulnerability detection
- **Performance Insights** - AI-powered performance analysis
- **Predictive Monitoring** - Forecast potential issues
- **Smart Test Generation** - Create comprehensive test suites
- **Automatic Fixing** - AI-suggested solutions for API issues

### 🔗 Links

- [GitHub Repository](https://github.com/wellcode-ai/nuts)
- [Issue Tracker](https://github.com/wellcode-ai/nuts/issues)
- [Releases](https://github.com/wellcode-ai/nuts/releases)

---

## Website Development

This website is built with [Next.js](https://nextjs.org) and bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

### Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

### Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

### Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.
