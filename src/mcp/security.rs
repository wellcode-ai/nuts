use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

use crate::ai::AiService;
use crate::error::NutsError;
use crate::mcp::client::McpClient;
use crate::mcp::types::{Resource, Tool};
use crate::output::{colors, renderer};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Overall risk level for the scanned server.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Critical => write!(f, "CRITICAL"),
            RiskLevel::High => write!(f, "HIGH"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::Low => write!(f, "LOW"),
        }
    }
}

/// Severity of an individual finding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// A single security finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityFinding {
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    pub recommendation: String,
}

/// Complete security scan report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub findings: Vec<SecurityFinding>,
    pub summary: String,
    pub risk_level: RiskLevel,
}

// ---------------------------------------------------------------------------
// Core scan logic
// ---------------------------------------------------------------------------

/// Run a full security scan against the connected MCP server.
///
/// 1. Discovers tools and resources
/// 2. Performs static schema analysis per tool
/// 3. Probes tools with adversarial inputs
/// 4. Analyzes resources for sensitive exposure
/// 5. Sends findings to AI for deeper analysis and recommendations
pub async fn security_scan(
    client: &McpClient,
    ai: &AiService,
) -> Result<SecurityReport, NutsError> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(renderer::spinner_style());
    spinner.set_message("Discovering server capabilities...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let tools = client.list_tools().await?;
    let resources = client.list_resources().await?;
    spinner.finish_and_clear();

    eprintln!(
        "  Found {} tool(s), {} resource(s). Starting security scan...\n",
        tools.len(),
        resources.len()
    );

    let mut all_findings: Vec<SecurityFinding> = Vec::new();

    // Phase 1: Static schema analysis
    for (i, tool) in tools.iter().enumerate() {
        let label = format!(
            "[{}/{}] Analyzing schema for '{}'...",
            i + 1,
            tools.len(),
            tool.name
        );
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(renderer::spinner_style());
        spinner.set_message(label);
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let mut schema_findings = analyze_schema(tool);
        all_findings.append(&mut schema_findings);
        spinner.finish_and_clear();
    }

    // Phase 2: Adversarial probing
    for (i, tool) in tools.iter().enumerate() {
        let label = format!(
            "[{}/{}] Probing '{}' with adversarial inputs...",
            i + 1,
            tools.len(),
            tool.name
        );
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(renderer::spinner_style());
        spinner.set_message(label);
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let mut probe_findings = probe_tool(client, tool).await;
        all_findings.append(&mut probe_findings);
        spinner.finish_and_clear();
    }

    // Phase 3: Resource analysis
    if !resources.is_empty() {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(renderer::spinner_style());
        spinner.set_message("Analyzing resources for sensitive exposure...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let mut resource_findings = analyze_resources(&resources);
        all_findings.append(&mut resource_findings);
        spinner.finish_and_clear();
    }

    // Phase 4: AI analysis -- send static findings to AI for deeper insight
    if !tools.is_empty() {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(renderer::spinner_style());
        spinner.set_message("AI analyzing findings and generating recommendations...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(80));

        let mut ai_findings = ai_analyze_tools(ai, &tools, &all_findings).await?;
        all_findings.append(&mut ai_findings);
        spinner.finish_and_clear();
    }

    // Build report
    let risk_level = compute_risk_level(&all_findings);
    let summary = build_summary(&all_findings, &risk_level);

    Ok(SecurityReport {
        findings: all_findings,
        summary,
        risk_level,
    })
}

// ---------------------------------------------------------------------------
// Phase 1: Static schema analysis
// ---------------------------------------------------------------------------

/// Analyze a tool's JSON Schema for common weaknesses.
fn analyze_schema(tool: &Tool) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    let tool_name = &tool.name;

    let schema = match &tool.input_schema {
        Some(s) => s,
        None => {
            findings.push(SecurityFinding {
                severity: Severity::Medium,
                category: "schema".into(),
                title: "No input schema defined".into(),
                description: format!(
                    "Tool '{}' has no input schema, meaning any input is accepted without validation.",
                    tool_name
                ),
                tool_name: Some(tool_name.clone()),
                recommendation: "Define a strict JSON Schema with required fields and type constraints.".into(),
            });
            return findings;
        }
    };

    // Check for missing required fields
    if schema.get("required").is_none() {
        if let Some(props) = schema.get("properties") {
            if props.as_object().map_or(false, |p| !p.is_empty()) {
                findings.push(SecurityFinding {
                    severity: Severity::Low,
                    category: "schema".into(),
                    title: "No required fields specified".into(),
                    description: format!(
                        "Tool '{}' has properties but no 'required' array, allowing all fields to be omitted.",
                        tool_name
                    ),
                    tool_name: Some(tool_name.clone()),
                    recommendation: "Add a 'required' array listing mandatory parameters.".into(),
                });
            }
        }
    }

    // Check for additionalProperties: true (or missing, which defaults to true)
    if let Some(additional) = schema.get("additionalProperties") {
        if additional.as_bool() == Some(true) {
            findings.push(SecurityFinding {
                severity: Severity::Medium,
                category: "schema".into(),
                title: "Additional properties allowed".into(),
                description: format!(
                    "Tool '{}' explicitly allows additional properties, meaning arbitrary extra fields can be injected.",
                    tool_name
                ),
                tool_name: Some(tool_name.clone()),
                recommendation: "Set 'additionalProperties: false' to reject unexpected fields.".into(),
            });
        }
    }

    // Check string params for missing validation constraints
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (param_name, param_schema) in props {
            let param_type = param_schema
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("");

            if param_type == "string" {
                let has_max_length = param_schema.get("maxLength").is_some();
                let has_pattern = param_schema.get("pattern").is_some();
                let has_enum = param_schema.get("enum").is_some();

                if !has_max_length && !has_pattern && !has_enum {
                    findings.push(SecurityFinding {
                        severity: Severity::Low,
                        category: "validation".into(),
                        title: format!("Unconstrained string parameter '{}'", param_name),
                        description: format!(
                            "Tool '{}' parameter '{}' is a string with no maxLength, pattern, or enum constraint. This may allow injection attacks or oversized inputs.",
                            tool_name, param_name
                        ),
                        tool_name: Some(tool_name.clone()),
                        recommendation: format!(
                            "Add maxLength, pattern regex, or enum values to constrain '{}'.",
                            param_name
                        ),
                    });
                }
            }
        }
    }

    // Check for sensitive-sounding tool names
    let sensitive_keywords = [
        "exec",
        "execute",
        "run",
        "shell",
        "command",
        "cmd",
        "eval",
        "file",
        "read_file",
        "write_file",
        "delete",
        "remove",
        "admin",
        "password",
        "token",
        "secret",
        "credential",
        "key",
        "sudo",
    ];
    let name_lower = tool_name.to_lowercase();
    for keyword in &sensitive_keywords {
        if name_lower.contains(keyword) {
            findings.push(SecurityFinding {
                severity: Severity::High,
                category: "sensitive_tool".into(),
                title: format!("Sensitive tool name: '{}'", tool_name),
                description: format!(
                    "Tool '{}' name suggests it may perform sensitive operations ({}-related). Ensure proper authorization and input sanitization.",
                    tool_name, keyword
                ),
                tool_name: Some(tool_name.clone()),
                recommendation: "Implement strict input validation, authorization checks, and audit logging for this tool.".into(),
            });
            break; // One finding per tool is enough
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// Phase 2: Adversarial probing
// ---------------------------------------------------------------------------

/// Probe a tool with adversarial inputs and analyze the responses.
async fn probe_tool(client: &McpClient, tool: &Tool) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();
    let tool_name = &tool.name;

    // Build a set of probe payloads based on the tool's schema
    let probes = build_probes(tool);

    for probe in &probes {
        let result = client.call_tool(tool_name, probe.payload.clone()).await;

        match result {
            Ok(tool_result) => {
                // Check if adversarial input was processed without error
                if !tool_result.is_error && probe.should_error {
                    findings.push(SecurityFinding {
                        severity: probe.severity.clone(),
                        category: probe.category.clone(),
                        title: format!("{} - accepted by '{}'", probe.name, tool_name),
                        description: format!(
                            "Tool '{}' accepted adversarial input without error: {}",
                            tool_name, probe.description
                        ),
                        tool_name: Some(tool_name.clone()),
                        recommendation: probe.recommendation.clone(),
                    });
                }

                // Check response content for information leakage
                for content in &tool_result.content {
                    if let crate::mcp::types::ContentItem::Text { text } = content {
                        check_information_leakage(&mut findings, tool_name, &probe.name, text);
                    }
                }
            }
            Err(_) => {
                // Tool rejected the input -- this is generally the expected/safe behavior
            }
        }
    }

    findings
}

/// A single adversarial probe.
struct Probe {
    name: String,
    category: String,
    payload: serde_json::Value,
    should_error: bool,
    severity: Severity,
    description: String,
    recommendation: String,
}

/// Build adversarial probe payloads based on the tool's input schema.
fn build_probes(tool: &Tool) -> Vec<Probe> {
    let mut probes = Vec::new();

    // Find the first string parameter to target
    let string_param = tool
        .input_schema
        .as_ref()
        .and_then(|s| s.get("properties"))
        .and_then(|p| p.as_object())
        .and_then(|props| {
            props
                .iter()
                .find(|(_, v)| v.get("type").and_then(|t| t.as_str()) == Some("string"))
        })
        .map(|(name, _)| name.clone());

    if let Some(ref param) = string_param {
        // Path traversal
        probes.push(Probe {
            name: "Path traversal (unix)".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "../../etc/passwd" }),
            should_error: true,
            severity: Severity::High,
            description: "Unix path traversal attempt was accepted.".into(),
            recommendation:
                "Validate and sanitize file path inputs. Reject paths containing '..' sequences."
                    .into(),
        });
        probes.push(Probe {
            name: "Path traversal (windows)".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "..\\..\\windows\\system32" }),
            should_error: true,
            severity: Severity::High,
            description: "Windows path traversal attempt was accepted.".into(),
            recommendation:
                "Validate and sanitize file path inputs. Reject paths containing '..' sequences."
                    .into(),
        });

        // Command injection
        probes.push(Probe {
            name: "Command injection (semicolon)".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "; ls -la /" }),
            should_error: true,
            severity: Severity::Critical,
            description: "Shell command injection via semicolon was accepted.".into(),
            recommendation:
                "Never pass user input to shell commands. Use parameterized APIs instead.".into(),
        });
        probes.push(Probe {
            name: "Command injection (subshell)".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "$(whoami)" }),
            should_error: true,
            severity: Severity::Critical,
            description: "Shell subshell injection was accepted.".into(),
            recommendation: "Sanitize inputs to prevent shell metacharacter interpretation.".into(),
        });
        probes.push(Probe {
            name: "Command injection (backtick)".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "`id`" }),
            should_error: true,
            severity: Severity::Critical,
            description: "Backtick command injection was accepted.".into(),
            recommendation: "Sanitize inputs to prevent shell metacharacter interpretation.".into(),
        });

        // SQL injection
        probes.push(Probe {
            name: "SQL injection".into(),
            category: "injection".into(),
            payload: serde_json::json!({ param: "'; DROP TABLE users; --" }),
            should_error: true,
            severity: Severity::High,
            description: "SQL injection payload was accepted without error.".into(),
            recommendation:
                "Use parameterized queries. Never concatenate user input into SQL strings.".into(),
        });

        // Oversized input
        probes.push(Probe {
            name: "Oversized input".into(),
            category: "validation".into(),
            payload: serde_json::json!({ param: "A".repeat(100_000) }),
            should_error: true,
            severity: Severity::Medium,
            description: "Extremely large input (100KB) was accepted without rejection.".into(),
            recommendation: "Enforce maxLength constraints on string parameters.".into(),
        });
    }

    // Type confusion: send string where number might be expected
    let number_param = tool
        .input_schema
        .as_ref()
        .and_then(|s| s.get("properties"))
        .and_then(|p| p.as_object())
        .and_then(|props| {
            props.iter().find(|(_, v)| {
                let t = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
                t == "number" || t == "integer"
            })
        })
        .map(|(name, _)| name.clone());

    if let Some(ref param) = number_param {
        probes.push(Probe {
            name: "Type confusion (string for number)".into(),
            category: "validation".into(),
            payload: serde_json::json!({ param: "not_a_number" }),
            should_error: true,
            severity: Severity::Low,
            description: "String value accepted for a numeric parameter.".into(),
            recommendation: "Enforce strict type validation on all parameters.".into(),
        });
    }

    // Empty input probe
    probes.push(Probe {
        name: "Empty input".into(),
        category: "validation".into(),
        payload: serde_json::json!({}),
        should_error: false, // Not necessarily an error -- depends on schema
        severity: Severity::Info,
        description: "Empty input was sent to test default handling.".into(),
        recommendation: "Ensure the tool handles missing parameters gracefully.".into(),
    });

    probes
}

/// Check tool response text for information leakage patterns.
fn check_information_leakage(
    findings: &mut Vec<SecurityFinding>,
    tool_name: &str,
    probe_name: &str,
    response_text: &str,
) {
    let leakage_patterns: &[(&str, &str, Severity)] = &[
        (
            "/etc/passwd",
            "System file content leaked in response",
            Severity::Critical,
        ),
        (
            "root:x:",
            "Unix passwd file content exposed",
            Severity::Critical,
        ),
        (
            "WINDOWS\\system32",
            "Windows system path exposed",
            Severity::Critical,
        ),
        (
            "stack trace",
            "Stack trace leaked in error response",
            Severity::High,
        ),
        ("at line", "Code location leaked in error", Severity::Medium),
        (
            "SQL",
            "SQL-related information in error response",
            Severity::Medium,
        ),
        (
            "connection refused",
            "Internal network topology exposed",
            Severity::Medium,
        ),
        ("ENOENT", "Internal error codes exposed", Severity::Low),
        ("errno", "System error details exposed", Severity::Low),
    ];

    let text_lower = response_text.to_lowercase();
    for (pattern, description, severity) in leakage_patterns {
        if text_lower.contains(&pattern.to_lowercase()) {
            findings.push(SecurityFinding {
                severity: severity.clone(),
                category: "information_leakage".into(),
                title: format!("Information leakage via '{}' probe", probe_name),
                description: format!(
                    "Tool '{}' response to '{}' probe: {}",
                    tool_name, probe_name, description
                ),
                tool_name: Some(tool_name.into()),
                recommendation: "Sanitize error messages. Never expose internal paths, stack traces, or system details to clients.".into(),
            });
            break; // One leakage finding per probe is sufficient
        }
    }
}

// ---------------------------------------------------------------------------
// Phase 3: Resource analysis
// ---------------------------------------------------------------------------

/// Analyze resources for sensitive path patterns and overly broad access.
fn analyze_resources(resources: &[Resource]) -> Vec<SecurityFinding> {
    let mut findings = Vec::new();

    let sensitive_patterns = [
        ("file://", "File system access", Severity::High),
        ("/etc/", "System configuration path", Severity::Critical),
        ("/proc/", "Process information path", Severity::Critical),
        ("env", "Environment variable access", Severity::High),
        ("secret", "Secret/credential access", Severity::High),
        ("password", "Password-related resource", Severity::High),
        ("token", "Token-related resource", Severity::High),
        ("config", "Configuration access", Severity::Medium),
        ("admin", "Administrative resource", Severity::Medium),
        ("log", "Log file access", Severity::Medium),
    ];

    for resource in resources {
        let uri_lower = resource.uri.to_lowercase();
        let name_lower = resource.name.to_lowercase();
        let desc_lower = resource.description.as_deref().unwrap_or("").to_lowercase();

        for (pattern, label, severity) in &sensitive_patterns {
            if uri_lower.contains(pattern)
                || name_lower.contains(pattern)
                || desc_lower.contains(pattern)
            {
                findings.push(SecurityFinding {
                    severity: severity.clone(),
                    category: "resource_exposure".into(),
                    title: format!("Sensitive resource: '{}' ({})", resource.name, label),
                    description: format!(
                        "Resource '{}' (URI: {}) appears to provide {}. Ensure proper access controls.",
                        resource.name, resource.uri, label
                    ),
                    tool_name: None,
                    recommendation: "Restrict access to sensitive resources. Implement authentication and authorization checks.".into(),
                });
                break;
            }
        }
    }

    findings
}

// ---------------------------------------------------------------------------
// Phase 4: AI analysis
// ---------------------------------------------------------------------------

/// Send tool schemas and current findings to AI for deeper analysis.
async fn ai_analyze_tools(
    ai: &AiService,
    tools: &[Tool],
    existing_findings: &[SecurityFinding],
) -> Result<Vec<SecurityFinding>, NutsError> {
    let mut ai_findings = Vec::new();

    // Summarize existing findings for context
    let findings_summary = if existing_findings.is_empty() {
        "No static findings so far.".to_string()
    } else {
        existing_findings
            .iter()
            .map(|f| format!("[{}] {}: {}", f.severity, f.title, f.description))
            .collect::<Vec<_>>()
            .join("\n")
    };

    for tool in tools {
        let description = tool.description.as_deref().unwrap_or("(no description)");
        let schema_str = tool
            .input_schema
            .as_ref()
            .map(|v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string()))
            .unwrap_or_else(|| "{}".to_string());

        let ai_response = ai
            .security_scan(
                &tool.name,
                description,
                &schema_str,
                Some(&findings_summary),
            )
            .await
            .map_err(|e| NutsError::Ai {
                message: format!("AI security analysis for '{}' failed: {}", tool.name, e),
            })?;

        // Parse AI response -- it should be a JSON array of attack objects
        if let Ok(attacks) =
            serde_json::from_str::<Vec<serde_json::Value>>(&strip_json_fences(&ai_response))
        {
            for attack in attacks {
                let severity = match attack
                    .get("severity_if_found")
                    .and_then(|s| s.as_str())
                    .unwrap_or("MEDIUM")
                    .to_uppercase()
                    .as_str()
                {
                    "CRITICAL" => Severity::Critical,
                    "HIGH" => Severity::High,
                    "LOW" => Severity::Low,
                    "INFO" => Severity::Info,
                    _ => Severity::Medium,
                };

                let category = attack
                    .get("category")
                    .and_then(|c| c.as_str())
                    .unwrap_or("ai_analysis")
                    .to_string();

                let name = attack
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("AI-identified concern")
                    .to_string();

                let safe_behavior = attack
                    .get("expected_safe_behavior")
                    .and_then(|b| b.as_str())
                    .unwrap_or("Reject or sanitize the input")
                    .to_string();

                ai_findings.push(SecurityFinding {
                    severity,
                    category,
                    title: format!("[AI] {}", name),
                    description: format!(
                        "AI-identified potential vulnerability in tool '{}'. Expected safe behavior: {}",
                        tool.name, safe_behavior
                    ),
                    tool_name: Some(tool.name.clone()),
                    recommendation: safe_behavior,
                });
            }
        }
        // If parsing fails, the AI response wasn't structured -- skip silently
    }

    Ok(ai_findings)
}

/// Strip markdown JSON fences that the AI might wrap around the response.
fn strip_json_fences(text: &str) -> String {
    let trimmed = text.trim();
    let without_open = if trimmed.starts_with("```json") {
        trimmed
            .strip_prefix("```json")
            .unwrap_or(trimmed)
            .trim_start()
    } else if trimmed.starts_with("```") {
        trimmed.strip_prefix("```").unwrap_or(trimmed).trim_start()
    } else {
        trimmed
    };
    let without_close = if without_open.ends_with("```") {
        without_open
            .strip_suffix("```")
            .unwrap_or(without_open)
            .trim_end()
    } else {
        without_open
    };
    without_close.to_string()
}

// ---------------------------------------------------------------------------
// Report building
// ---------------------------------------------------------------------------

/// Compute overall risk level from findings.
fn compute_risk_level(findings: &[SecurityFinding]) -> RiskLevel {
    let has_critical = findings.iter().any(|f| f.severity == Severity::Critical);
    let has_high = findings.iter().any(|f| f.severity == Severity::High);
    let medium_count = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .count();

    if has_critical {
        RiskLevel::Critical
    } else if has_high {
        RiskLevel::High
    } else if medium_count >= 3 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

/// Build a human-readable summary of the scan.
fn build_summary(findings: &[SecurityFinding], risk_level: &RiskLevel) -> String {
    let critical = findings
        .iter()
        .filter(|f| f.severity == Severity::Critical)
        .count();
    let high = findings
        .iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    let medium = findings
        .iter()
        .filter(|f| f.severity == Severity::Medium)
        .count();
    let low = findings
        .iter()
        .filter(|f| f.severity == Severity::Low)
        .count();
    let info = findings
        .iter()
        .filter(|f| f.severity == Severity::Info)
        .count();

    format!(
        "Risk Level: {}\nFindings: {} total ({} critical, {} high, {} medium, {} low, {} info)",
        risk_level,
        findings.len(),
        critical,
        high,
        medium,
        low,
        info
    )
}

// ---------------------------------------------------------------------------
// Output formatting
// ---------------------------------------------------------------------------

/// Render a security report to the terminal with color-coded severity.
pub fn render_report(report: &SecurityReport) {
    // Summary header
    let risk_style = match report.risk_level {
        RiskLevel::Critical => colors::error_bold(),
        RiskLevel::High => colors::error_bold(),
        RiskLevel::Medium => colors::warning_bold(),
        RiskLevel::Low => colors::success_bold(),
    };

    renderer::render_section(
        "MCP Security Scan",
        &format!(
            "Overall Risk: {}",
            risk_style.apply_to(report.risk_level.to_string())
        ),
    );

    if report.findings.is_empty() {
        eprintln!("\n  No security findings. The server appears well-configured.\n");
        return;
    }

    eprintln!();
    eprintln!("  {}", report.summary);
    eprintln!();

    // Group findings by severity
    let severity_order = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ];

    for severity in &severity_order {
        let group: Vec<_> = report
            .findings
            .iter()
            .filter(|f| &f.severity == severity)
            .collect();

        if group.is_empty() {
            continue;
        }

        let style = match severity {
            Severity::Critical => colors::error_bold(),
            Severity::High => colors::error_bold(),
            Severity::Medium => colors::warning_bold(),
            Severity::Low => colors::muted(),
            Severity::Info => colors::muted(),
        };

        for finding in &group {
            let tool_str = finding
                .tool_name
                .as_deref()
                .map(|t| format!(" ({})", t))
                .unwrap_or_default();

            eprintln!(
                "  {} {}{}",
                style.apply_to(format!("[{}]", finding.severity)),
                finding.title,
                colors::muted().apply_to(&tool_str),
            );
            eprintln!("    {}", colors::muted().apply_to(&finding.description));
            eprintln!(
                "    {}",
                colors::accent().apply_to(format!("Fix: {}", finding.recommendation))
            );
            eprintln!();
        }
    }
}

/// Format the report as JSON.
pub fn format_report_json(report: &SecurityReport) -> Result<String, NutsError> {
    serde_json::to_string_pretty(report).map_err(|e| NutsError::Ai {
        message: format!("Failed to serialize security report: {e}"),
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::Tool;

    fn make_tool(name: &str, desc: &str, schema: Option<serde_json::Value>) -> Tool {
        Tool {
            name: name.into(),
            description: Some(desc.into()),
            input_schema: schema,
        }
    }

    #[test]
    fn analyze_schema_no_schema() {
        let tool = Tool {
            name: "test".into(),
            description: None,
            input_schema: None,
        };
        let findings = analyze_schema(&tool);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].category, "schema");
        assert!(findings[0].title.contains("No input schema"));
    }

    #[test]
    fn analyze_schema_no_required_fields() {
        let tool = make_tool(
            "search",
            "Search",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                }
            })),
        );
        let findings = analyze_schema(&tool);
        assert!(findings
            .iter()
            .any(|f| f.title.contains("No required fields")));
    }

    #[test]
    fn analyze_schema_additional_properties() {
        let tool = make_tool(
            "search",
            "Search",
            Some(serde_json::json!({
                "type": "object",
                "additionalProperties": true,
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            })),
        );
        let findings = analyze_schema(&tool);
        assert!(findings
            .iter()
            .any(|f| f.title.contains("Additional properties")));
    }

    #[test]
    fn analyze_schema_unconstrained_string() {
        let tool = make_tool(
            "search",
            "Search",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            })),
        );
        let findings = analyze_schema(&tool);
        assert!(findings
            .iter()
            .any(|f| f.title.contains("Unconstrained string")));
    }

    #[test]
    fn analyze_schema_constrained_string_no_finding() {
        let tool = make_tool(
            "search",
            "Search",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "maxLength": 100 }
                },
                "required": ["query"]
            })),
        );
        let findings = analyze_schema(&tool);
        assert!(!findings
            .iter()
            .any(|f| f.title.contains("Unconstrained string")));
    }

    #[test]
    fn analyze_schema_sensitive_tool_name() {
        let tool = make_tool(
            "execute_command",
            "Run a shell command",
            Some(serde_json::json!({})),
        );
        let findings = analyze_schema(&tool);
        assert!(findings.iter().any(|f| f.category == "sensitive_tool"));
    }

    #[test]
    fn analyze_schema_safe_tool_name() {
        let tool = make_tool(
            "search_docs",
            "Search documents",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "q": { "type": "string", "maxLength": 200 }
                },
                "required": ["q"]
            })),
        );
        let findings = analyze_schema(&tool);
        assert!(!findings.iter().any(|f| f.category == "sensitive_tool"));
    }

    #[test]
    fn analyze_resources_detects_sensitive() {
        let resources = vec![
            Resource {
                uri: "file:///etc/config".into(),
                name: "config".into(),
                description: Some("Server configuration".into()),
                mime_type: None,
            },
            Resource {
                uri: "data://safe".into(),
                name: "safe_data".into(),
                description: Some("Public data".into()),
                mime_type: None,
            },
        ];
        let findings = analyze_resources(&resources);
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.category == "resource_exposure"));
    }

    #[test]
    fn analyze_resources_safe() {
        let resources = vec![Resource {
            uri: "data://public/items".into(),
            name: "items".into(),
            description: Some("Public item list".into()),
            mime_type: Some("application/json".into()),
        }];
        let findings = analyze_resources(&resources);
        assert!(findings.is_empty());
    }

    #[test]
    fn compute_risk_critical() {
        let findings = vec![SecurityFinding {
            severity: Severity::Critical,
            category: "test".into(),
            title: "test".into(),
            description: "test".into(),
            tool_name: None,
            recommendation: "test".into(),
        }];
        assert_eq!(compute_risk_level(&findings), RiskLevel::Critical);
    }

    #[test]
    fn compute_risk_high() {
        let findings = vec![SecurityFinding {
            severity: Severity::High,
            category: "test".into(),
            title: "test".into(),
            description: "test".into(),
            tool_name: None,
            recommendation: "test".into(),
        }];
        assert_eq!(compute_risk_level(&findings), RiskLevel::High);
    }

    #[test]
    fn compute_risk_medium_threshold() {
        let findings: Vec<SecurityFinding> = (0..3)
            .map(|_| SecurityFinding {
                severity: Severity::Medium,
                category: "test".into(),
                title: "test".into(),
                description: "test".into(),
                tool_name: None,
                recommendation: "test".into(),
            })
            .collect();
        assert_eq!(compute_risk_level(&findings), RiskLevel::Medium);
    }

    #[test]
    fn compute_risk_low() {
        let findings = vec![SecurityFinding {
            severity: Severity::Low,
            category: "test".into(),
            title: "test".into(),
            description: "test".into(),
            tool_name: None,
            recommendation: "test".into(),
        }];
        assert_eq!(compute_risk_level(&findings), RiskLevel::Low);
    }

    #[test]
    fn build_summary_formats_correctly() {
        let findings = vec![
            SecurityFinding {
                severity: Severity::Critical,
                category: "t".into(),
                title: "t".into(),
                description: "t".into(),
                tool_name: None,
                recommendation: "t".into(),
            },
            SecurityFinding {
                severity: Severity::High,
                category: "t".into(),
                title: "t".into(),
                description: "t".into(),
                tool_name: None,
                recommendation: "t".into(),
            },
            SecurityFinding {
                severity: Severity::Low,
                category: "t".into(),
                title: "t".into(),
                description: "t".into(),
                tool_name: None,
                recommendation: "t".into(),
            },
        ];
        let summary = build_summary(&findings, &RiskLevel::Critical);
        assert!(summary.contains("Risk Level: CRITICAL"));
        assert!(summary.contains("3 total"));
        assert!(summary.contains("1 critical"));
        assert!(summary.contains("1 high"));
        assert!(summary.contains("1 low"));
    }

    #[test]
    fn strip_json_fences_works() {
        let input = "```json\n[{\"test\": true}]\n```";
        let result = strip_json_fences(input);
        assert_eq!(result, "[{\"test\": true}]");
    }

    #[test]
    fn strip_json_fences_no_fences() {
        let input = "[{\"test\": true}]";
        let result = strip_json_fences(input);
        assert_eq!(result, "[{\"test\": true}]");
    }

    #[test]
    fn check_leakage_detects_passwd() {
        let mut findings = Vec::new();
        check_information_leakage(
            &mut findings,
            "read_file",
            "path traversal",
            "root:x:0:0:root:/root:/bin/bash",
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn check_leakage_clean_response() {
        let mut findings = Vec::new();
        check_information_leakage(
            &mut findings,
            "search",
            "sql injection",
            "No results found for your query.",
        );
        assert!(findings.is_empty());
    }

    #[test]
    fn build_probes_targets_string_param() {
        let tool = make_tool(
            "search",
            "Search",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                }
            })),
        );
        let probes = build_probes(&tool);
        // Should have path traversal, command injection, sql injection, oversized, empty
        assert!(probes.len() >= 7);
        assert!(probes.iter().any(|p| p.name.contains("Path traversal")));
        assert!(probes.iter().any(|p| p.name.contains("Command injection")));
        assert!(probes.iter().any(|p| p.name.contains("SQL injection")));
        assert!(probes.iter().any(|p| p.name.contains("Oversized")));
    }

    #[test]
    fn build_probes_with_number_param() {
        let tool = make_tool(
            "get_item",
            "Get item by ID",
            Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" }
                }
            })),
        );
        let probes = build_probes(&tool);
        assert!(probes.iter().any(|p| p.name.contains("Type confusion")));
    }

    #[test]
    fn severity_ordering() {
        assert!(Severity::Critical < Severity::High);
        assert!(Severity::High < Severity::Medium);
        assert!(Severity::Medium < Severity::Low);
        assert!(Severity::Low < Severity::Info);
    }

    #[test]
    fn report_serializes_to_json() {
        let report = SecurityReport {
            findings: vec![SecurityFinding {
                severity: Severity::High,
                category: "injection".into(),
                title: "SQL injection".into(),
                description: "Tool accepts SQL payloads".into(),
                tool_name: Some("search".into()),
                recommendation: "Use parameterized queries".into(),
            }],
            summary: "1 finding".into(),
            risk_level: RiskLevel::High,
        };
        let json = format_report_json(&report).unwrap();
        assert!(json.contains("SQL injection"));
        assert!(json.contains("\"risk_level\": \"high\""));
    }
}
