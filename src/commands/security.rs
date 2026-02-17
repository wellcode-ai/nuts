use crate::config::Config;
use crate::output::{colors, renderer};
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use reqwest::header;
use reqwest::Client;

pub struct SecurityCommand {
    #[allow(dead_code)]
    config: Config,
    deep_scan: bool,
    auth_token: Option<String>,
    save_file: Option<String>,
    http_client: Client,
    ai_client: AnthropicClient,
}

impl SecurityCommand {
    pub fn new(config: Config) -> Self {
        let api_key = config.anthropic_api_key.clone().unwrap_or_default();

        Self {
            config,
            deep_scan: false,
            auth_token: None,
            save_file: None,
            http_client: Client::new(),
            ai_client: ClientBuilder::default().api_key(api_key).build().unwrap(),
        }
    }

    pub fn with_deep_scan(mut self, deep_scan: bool) -> Self {
        self.deep_scan = deep_scan;
        self
    }

    pub fn with_auth(mut self, auth_token: Option<String>) -> Self {
        self.auth_token = auth_token;
        self
    }

    pub fn with_save_file(mut self, save_file: Option<String>) -> Self {
        self.save_file = save_file;
        self
    }

    async fn display_security_analysis(&self, analysis: &str) {
        renderer::render_ai_insight("Security Analysis", analysis);

        println!(
            "\n  {}",
            colors::muted().apply_to(
                "This analysis is based on the API response only. A comprehensive security audit would require additional context."
            )
        );
        println!();
    }

    pub async fn execute(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 2 {
            renderer::render_error(
                "Missing URL",
                "security <url>",
                "security https://api.example.com",
            );
            return Ok(());
        }

        let url = if args[1].starts_with("http") {
            args[1].to_string()
        } else {
            format!("http://{}", args[1])
        };

        println!(
            "\n  {} {}",
            colors::accent().apply_to("Scanning:"),
            colors::muted().apply_to(&url),
        );
        if self.deep_scan {
            println!(
                "  {}",
                colors::muted().apply_to("Deep scan enabled -- this may take a few minutes")
            );
        }

        let mut analysis_data = Vec::new();

        // Basic scan - check main endpoint
        let response = self.http_client.get(&url).send().await?;
        analysis_data.push(self.analyze_response(response).await?);

        // Deep scan - additional checks
        if self.deep_scan {
            // Check common security endpoints
            for endpoint in [
                "/security.txt",
                "/.well-known/security.txt",
                "/robots.txt",
                "/.env",
                "/wp-admin",
                "/api/v1",
                "/graphql",
                "/swagger.json",
                "/openapi.json",
                "/.git/config",
                "/server-status",
            ] {
                let sec_url = format!("{}{}", url, endpoint);
                if let Ok(resp) = self.http_client.get(&sec_url).send().await {
                    analysis_data.push(self.analyze_response(resp).await?);
                }
            }

            // Check HTTP methods
            for method in ["HEAD", "OPTIONS", "TRACE", "PUT", "DELETE", "PATCH"] {
                if let Ok(resp) = self
                    .http_client
                    .request(
                        reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                        &url,
                    )
                    .send()
                    .await
                {
                    analysis_data.push(self.analyze_response(resp).await?);
                }
            }
        }

        // Combine all analyses for AI processing
        let analysis_prompt = if self.deep_scan {
            format!(
                r#"You are an elite application security architect and certified penetration tester (OSCP, CISSP, CEH). Perform an exhaustive security assessment of these API responses.

MAIN ENDPOINT RESPONSE:
{}

ADDITIONAL ENDPOINTS AND METHODS TESTED:
{}

Provide a structured, professional security report with the following sections. Use severity badges [CRITICAL], [HIGH], [MEDIUM], [LOW], [INFO] for each finding.

## 1. EXECUTIVE SUMMARY
- Overall risk rating (CRITICAL / HIGH / MEDIUM / LOW)
- Total findings count by severity
- Top 3 most urgent issues

## 2. COMPLIANCE & CERTIFICATION ASSESSMENT
Evaluate against these frameworks:
- **OWASP Top 10 (2021)**: Map each finding to the relevant OWASP category (A01-A10)
- **SOC 2 Type II**: Trust Service Criteria (Security, Availability, Confidentiality, Processing Integrity, Privacy)
- **PCI DSS v4.0**: Requirements 1-12 as applicable (especially Req 6: Secure Systems, Req 7: Access Control)
- **ISO 27001**: Relevant controls from Annex A
- **NIST Cybersecurity Framework**: Identify, Protect, Detect, Respond, Recover
- **GDPR/CCPA**: Data protection and privacy implications

For each framework, state: PASS / PARTIAL / FAIL with specific control references.

## 3. TRANSPORT LAYER SECURITY
- TLS version and cipher suite analysis
- Certificate validation
- HSTS configuration and preload status
- Certificate transparency

## 4. HTTP SECURITY HEADERS AUDIT
For EACH of these headers, report present/missing/misconfigured with the recommended value:
- Strict-Transport-Security (HSTS)
- Content-Security-Policy (CSP)
- X-Content-Type-Options
- X-Frame-Options
- X-XSS-Protection
- Referrer-Policy
- Permissions-Policy
- Cross-Origin-Opener-Policy (COOP)
- Cross-Origin-Embedder-Policy (COEP)
- Cross-Origin-Resource-Policy (CORP)
- Cache-Control (for sensitive data)

## 5. AUTHENTICATION & AUTHORIZATION
- Authentication mechanism analysis
- Session management assessment
- Token security (JWT analysis if applicable)
- CORS policy evaluation
- Rate limiting presence
- Brute force protection

## 6. DATA EXPOSURE & INFORMATION LEAKAGE
- Server version disclosure
- Technology stack fingerprinting
- Error message verbosity
- Sensitive data in responses
- API endpoint enumeration risks
- Debug/development endpoints accessible

## 7. INJECTION & INPUT VALIDATION RISKS
- SQL injection vectors
- XSS vectors
- Command injection potential
- SSRF risks
- Path traversal risks

## 8. BUSINESS LOGIC & API SECURITY
- IDOR (Insecure Direct Object Reference) risks
- Mass assignment vulnerabilities
- Rate limiting and throttling
- API versioning security
- Error handling consistency

## 9. RISK MATRIX
Create a risk assessment table:
| Finding | Severity | Likelihood | Impact | OWASP Category | Remediation Priority |

## 10. REMEDIATION ROADMAP
- **Immediate (0-24h)**: Critical fixes
- **Short-term (1-2 weeks)**: High-priority items
- **Medium-term (1-3 months)**: Medium findings
- **Ongoing**: Best practices and monitoring

## 11. CERTIFICATION READINESS SCORE
Rate readiness (0-100%) for each:
- SOC 2 Type II: X%
- PCI DSS: X%
- ISO 27001: X%
- OWASP Compliance: X%

Be specific and actionable. Every finding must include the exact header, value, or configuration to fix."#,
                analysis_data[0],
                analysis_data[1..].join("\n---\n")
            )
        } else {
            format!(
                r#"You are an elite application security architect and certified penetration tester (OSCP, CISSP, CEH). Analyze this API response for security vulnerabilities, compliance gaps, and risk exposure.

API RESPONSE:
{}

Provide a professional security assessment with these sections. Use severity badges [CRITICAL], [HIGH], [MEDIUM], [LOW], [INFO] for each finding.

## 1. EXECUTIVE SUMMARY
- Overall risk rating with justification
- Key findings count by severity

## 2. SECURITY HEADERS AUDIT
For each standard security header, report: Present/Missing/Misconfigured with the recommended value:
- Strict-Transport-Security, Content-Security-Policy, X-Content-Type-Options
- X-Frame-Options, Referrer-Policy, Permissions-Policy
- CORS headers, Cache-Control

## 3. COMPLIANCE SNAPSHOT
Quick assessment against:
- **OWASP Top 10**: Which categories are violated (A01-A10)?
- **SOC 2**: Key trust service criteria gaps
- **PCI DSS**: Critical requirement gaps
- **GDPR/CCPA**: Data protection concerns

## 4. INFORMATION DISCLOSURE
- Server/technology fingerprinting
- Sensitive data exposure
- Error message analysis
- Version disclosure

## 5. AUTHENTICATION & ACCESS CONTROL
- Auth mechanism assessment
- Session/token security
- Rate limiting presence

## 6. RISK MATRIX
| Finding | Severity | OWASP Category | Fix Priority |

## 7. REMEDIATION PLAN
- **Immediate**: Critical and high items with exact fixes
- **Short-term**: Medium items
- **Ongoing**: Best practices

Be specific. Include exact header values and configuration changes needed."#,
                analysis_data[0]
            )
        };

        println!("  {}", colors::muted().apply_to("Analyzing with AI..."));

        // Get AI analysis
        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: analysis_prompt,
            }],
        }];

        let messages_request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-sonnet-4-5-20250929".to_string())
            .max_tokens(8192_usize)
            .build()?;

        let messages_response = self.ai_client.messages(messages_request).await?;

        // Print the analysis
        if let Some(ContentBlock::Text { text }) = messages_response.content.first() {
            self.display_security_analysis(text).await;
        } else {
            renderer::render_error("Could not parse AI response", "", "");
        }

        Ok(())
    }

    async fn analyze_response(
        &self,
        response: reqwest::Response,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = response.url().to_string();
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        Ok(format!(
            "URL: {}\nStatus: {}\nHeaders:\n{}\nBody:\n{}\n",
            url,
            status,
            self.format_headers(&headers),
            body
        ))
    }

    fn format_headers(&self, headers: &header::HeaderMap) -> String {
        headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
            .collect::<Vec<String>>()
            .join("\n")
    }
}
