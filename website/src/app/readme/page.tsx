import Link from 'next/link';

export default function ReadmePage() {
  return (
    <div className="flash-bg min-h-screen px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="text-center mb-12">
          <h1 className="nuts-logo mb-4">NUTS</h1>
          <p className="subtitle mb-8">AI-Powered API & MCP Server Testing Suite</p>
          <div className="flex justify-center gap-4 mb-8">
            <Link href="/" className="cyberpunk-button">← Home</Link>
            <a href="https://github.com/wellcode-ai/nuts" className="cyberpunk-button">GitHub</a>
          </div>
        </div>

        <div className="readme-content">
          
          <section className="mb-12">
            <h2 className="section-title">⚡ 30-Second Start (Try It Now!)</h2>
            <div className="code-block">
              <div className="code-header">Copy & Paste to Get Started</div>
              <pre className="code-content">
{`# Install (choose one)
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
> monitor https://httpbin.org/get --smart`}
              </pre>
            </div>
            <div className="text-center mt-6">
              <p className="text-flash-orange text-xl font-bold">
                That&apos;s it! You&apos;re now using the most powerful API testing tool ever built. 🚀
              </p>
            </div>
          </section>

          <section className="mb-12">
            <h2 className="section-title">🤔 What Makes NUTS Different?</h2>
            
            <div className="grid md:grid-cols-2 gap-8 mb-8">
              <div className="cyberpunk-border p-6">
                <h3 className="text-red-400 text-lg font-bold mb-4">❌ Traditional Tools</h3>
                <div className="code-block">
                  <pre className="code-content text-sm">
{`# Complex curl commands
curl -X POST https://api.example.com/users \\
  -H "Content-Type: application/json" \\
  -H "Authorization: Bearer token123" \\
  -d '{"name":"John","email":"john@example.com"}'

# Multiple tools needed
curl + jq + ab + custom scripts...`}
                  </pre>
                </div>
              </div>
              
              <div className="cyberpunk-border p-6">
                <h3 className="text-flash-orange text-lg font-bold mb-4">✅ NUTS (AI-Powered)</h3>
                <div className="code-block">
                  <pre className="code-content text-sm">
{`# Simple, powerful commands
> call POST https://api.example.com/users '{"name":"John"}'
> ask "Create a realistic user for testing"
> perf GET https://api.example.com/users --users 100
> monitor https://api.example.com --smart`}
                  </pre>
                </div>
              </div>
            </div>

            <div className="cyberpunk-border p-6">
              <h3 className="text-flash-orange text-lg font-bold mb-4">🎯 Why Developers Love NUTS</h3>
              <div className="grid md:grid-cols-2 gap-6">
                <div className="flex items-start gap-3">
                  <span className="text-flash-orange text-xl">🚀</span>
                  <div>
                    <h4 className="text-white font-bold">Zero Learning Curve</h4>
                    <p className="text-gray-300">If you know curl, you know NUTS</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <span className="text-flash-orange text-xl">🤖</span>
                  <div>
                    <h4 className="text-white font-bold">AI-Powered</h4>
                    <p className="text-gray-300">Natural language commands, smart monitoring, auto-fix</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <span className="text-flash-orange text-xl">⚡</span>
                  <div>
                    <h4 className="text-white font-bold">All-in-One</h4>
                    <p className="text-gray-300">Testing, monitoring, security, performance in one tool</p>
                  </div>
                </div>
                <div className="flex items-start gap-3">
                  <span className="text-flash-orange text-xl">🎯</span>
                  <div>
                    <h4 className="text-white font-bold">Production-Ready</h4>
                    <p className="text-gray-300">Built with Rust for reliability and speed</p>
                  </div>
                </div>
              </div>
            </div>
          </section>

          <section className="mb-12">
            <h2 className="section-title">🤖 AI Superpowers (CURL Killer!)</h2>
            
            <div className="feature-grid">
              <div className="feature-card">
                <h3 className="feature-title">💬 Natural Language API Calls</h3>
                <p className="feature-description">
                  Stop memorizing curl syntax! Just tell NUTS what you want in plain English.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# Instead of complex curl commands:
nuts ask "Create a user with email john@example.com"
nuts ask "Get all products from the API"
nuts ask "Test if the login endpoint works"`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">🎲 AI Test Data Generation</h3>
                <p className="feature-description">
                  Generate unlimited realistic test data with AI. No more manual JSON crafting!
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# Generate realistic test data
nuts generate users 50
nuts generate products 25  
nuts generate orders 10

# Automatically creates diverse, realistic data`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">📊 Smart Monitoring</h3>
                <p className="feature-description">
                  AI monitors your APIs and predicts issues before they happen. Health checks every 30 seconds, AI analysis every 3rd check.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# Basic monitoring (30-second intervals)
nuts monitor https://api.myapp.com

# Smart monitoring with AI insights (every 90 seconds)
nuts monitor https://api.myapp.com --smart

# AI analyzes patterns, trends, and predictions`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">🧠 AI Response Explanation</h3>
                <p className="feature-description">
                  Confused by API responses? Let AI explain them in human terms.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# Make any API call
nuts call GET https://api.example.com/users

# Then get AI explanation
nuts explain`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">🔧 Auto-Fix Broken APIs</h3>
                <p className="feature-description">
                  AI diagnoses API problems and suggests specific fixes automatically.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# AI diagnoses and fixes APIs
nuts fix https://api.broken.com

# Get specific actionable recommendations`}
                  </pre>
                </div>
              </div>
            </div>
          </section>

          <section className="mb-12">
            <h2 className="section-title">MCP Server Testing</h2>

            <div className="feature-grid">
              <div className="feature-card">
                <h3 className="feature-title">Discover & Connect</h3>
                <p className="feature-description">
                  Automatically discover MCP server capabilities, tools, resources, and prompts across all transport types.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`nuts mcp discover --http https://api.example.com/mcp
nuts mcp discover --stdio "node server.js"
nuts mcp connect --http https://api.example.com/mcp --bearer TOKEN`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">YAML Test Suites</h3>
                <p className="feature-description">
                  Write declarative test suites in YAML and run them with a single command for repeatable MCP server validation.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# Write declarative tests in YAML
# Run with a single command
nuts mcp test mcp-tests.yaml
nuts mcp test --file tests/integration.yaml`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">Security Scanner</h3>
                <p className="feature-description">
                  AI-powered security analysis that checks for injection vulnerabilities, auth bypass, and schema validation issues.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`# AI-powered security analysis
nuts mcp security --http https://api.example.com/mcp
# Checks: injection, auth bypass, schema validation`}
                  </pre>
                </div>
              </div>

              <div className="feature-card">
                <h3 className="feature-title">Performance & Snapshots</h3>
                <p className="feature-description">
                  Benchmark MCP tool performance and capture snapshots for regression testing across server versions.
                </p>
                <div className="code-block">
                  <pre className="code-content">
{`nuts mcp perf --http https://example.com/mcp --tool search --iterations 100
nuts mcp snapshot capture --http https://example.com/mcp -o baseline.json
nuts mcp snapshot compare --compare baseline.json --http https://example.com/mcp`}
                  </pre>
                </div>
              </div>
            </div>
          </section>

          <section className="mb-12">
            <h2 className="section-title">📚 Complete Command Reference</h2>
            
            <div className="mb-8">
              <h3 className="feature-title">🔧 Core Commands</h3>
              <div className="code-block">
                <div className="code-header">Essential API Testing Commands</div>
                <pre className="code-content">
{`# Advanced HTTP Client (CURL killer)
nuts call GET https://api.example.com/users
nuts call POST https://api.example.com/users '{"name": "John"}'
nuts call -H "Authorization: Bearer token" GET https://api.example.com/users

# Natural Language Interface
nuts ask "Create a user with realistic data"
nuts ask "Get all products from the API"
nuts ask "Delete user with ID 123"

# Performance Testing
nuts perf GET https://api.example.com/users --users 100 --duration 30s
nuts perf POST https://api.example.com/users --users 50 '{"name": "Test"}'

# Security Scanning
nuts security https://api.example.com --deep
nuts security https://api.example.com --auth "Bearer token" --save report.json

# Health Monitoring
nuts monitor https://api.example.com              # Basic (30s intervals)
nuts monitor https://api.example.com --smart      # AI analysis (every 3rd check)

# API Discovery & Testing
nuts discover https://api.example.com             # Auto-discover endpoints
nuts test "Check if user registration works"      # AI test generation
nuts generate users 10                            # Generate test data
nuts predict https://api.example.com              # AI health prediction
nuts explain                                      # Explain last response
nuts fix https://api.example.com/broken           # Auto-fix issues`}
                </pre>
              </div>
            </div>

            <div className="mb-8">
              <h3 className="feature-title">🔌 MCP Server Testing Commands</h3>
              <div className="code-block">
                <div className="code-header">MCP Server Testing Commands</div>
                <pre className="code-content">
{`# Discover server capabilities
nuts mcp discover --http https://example.com/mcp
nuts mcp discover --stdio "node server.js"
nuts mcp discover --http https://example.com/mcp --bearer TOKEN

# Run test suites
nuts mcp test mcp-tests.yaml
nuts mcp test --file tests/suite.yaml --json

# Security scanning (requires API key)
nuts mcp security --http https://example.com/mcp

# Performance benchmarking
nuts mcp perf --http https://example.com/mcp --tool echo --iterations 100

# Snapshot regression testing
nuts mcp snapshot capture --http https://example.com/mcp -o baseline.json
nuts mcp snapshot compare --compare baseline.json --http https://example.com/mcp

# AI test generation (requires API key)
nuts mcp generate --http https://example.com/mcp

# Transport options (available on all MCP commands)
--http URL          # Streamable HTTP (recommended)
--sse URL           # Legacy Server-Sent Events
--stdio "cmd args"  # Spawn child process
--bearer TOKEN      # Bearer auth for HTTP
--set-env KEY=VALUE # Env vars for stdio`}
                </pre>
              </div>
            </div>

            <div className="mb-8">
              <h3 className="feature-title">⚙️ Configuration & Shortcuts</h3>
              <div className="code-block">
                <div className="code-header">Setup & Aliases</div>
                <pre className="code-content">
{`# Configuration
nuts config api-key                               # Set Anthropic API key
nuts config show                                  # Show current config

# Command Aliases (shortcuts)
nuts c GET https://api.example.com                # 'c' = call
nuts p GET https://api.example.com --users 10     # 'p' = perf
nuts s myapi                                      # 's' = flow story
nuts h                                            # 'h' = help
nuts q                                            # 'q' = quit`}
                </pre>
              </div>
            </div>
          </section>

          <section className="mb-12">
            <h2 className="section-title">💖 Why Developers Love NUTS</h2>
            
            <div className="love-grid">
              <div className="love-item">
                <span className="love-emoji">🚀</span>
                <h3>Zero Learning Curve</h3>
                <p>Just talk to NUTS like a human. No complex syntax to memorize.</p>
              </div>
              
              <div className="love-item">
                <span className="love-emoji">🤖</span>
                <h3>AI-Powered Everything</h3>
                <p>Every command is enhanced with AI to make you more productive.</p>
              </div>
              
              <div className="love-item">
                <span className="love-emoji">⚡</span>
                <h3>Instant Productivity</h3>
                <p>Stop writing boilerplate. Focus on testing, not tool configuration.</p>
              </div>
              
              <div className="love-item">
                <span className="love-emoji">🔮</span>
                <h3>Predictive Intelligence</h3>
                <p>Catch issues before they become problems. Monitor smartly.</p>
              </div>
              
              <div className="love-item">
                <span className="love-emoji">🎯</span>
                <h3>Perfect Simplicity</h3>
                <p>Clean, focused interface. No overwhelming features or complexity.</p>
              </div>
              
              <div className="love-item">
                <span className="love-emoji">🌟</span>
                <h3>Future-Proof</h3>
                <p>NUTS learns and improves. The more you use it, the smarter it gets.</p>
              </div>
            </div>
          </section>

          <section className="text-center">
            <div className="cyberpunk-border p-8">
              <h2 className="section-title">Ready to Revolutionize API Testing?</h2>
              <p className="text-white text-lg mb-6">
                Join the AI revolution. Make API testing effortless.
              </p>
              <div className="flex justify-center gap-4">
                <a href="https://github.com/wellcode-ai/nuts" className="cyberpunk-button">
                  🚀 Get Started on GitHub
                </a>
                <Link href="/" className="cyberpunk-button">
                  ← Back to Home
                </Link>
              </div>
            </div>
          </section>

        </div>
      </div>
    </div>
  );
}