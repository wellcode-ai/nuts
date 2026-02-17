use crate::mcp::security::{RiskLevel, SecurityFinding, SecurityReport, Severity};
use crate::mcp::test_runner::{TestResult, TestStatus, TestSummary};
use crate::output::colors;
use comfy_table::{presets, ContentArrangement, Table};
use std::io::IsTerminal;
use std::time::Duration;

/// Compact one-line status: "200 OK  143ms  2.4 KB"
/// Colors by status code: 2xx=green, 3xx=yellow, 4xx=yellow, 5xx=red.
pub fn render_status_line(status: u16, duration: Duration, size: usize) {
    let status_text = format!("{} {}", status, status_reason(status));
    let time_text = format_duration(duration);
    let size_text = format_bytes(size);

    let styled_status = match status {
        200..=299 => colors::success_bold().apply_to(&status_text),
        300..=399 => colors::warning_bold().apply_to(&status_text),
        400..=499 => colors::warning_bold().apply_to(&status_text),
        _ => colors::error_bold().apply_to(&status_text),
    };

    println!(
        "  {}  {}  {}",
        styled_status,
        colors::muted().apply_to(&time_text),
        colors::muted().apply_to(&size_text),
    );
}

/// Syntax-highlighted JSON output.
/// Keys=cyan, strings=green, numbers=yellow, booleans=magenta, null=dim red.
pub fn render_json_body(value: &serde_json::Value) {
    let highlighted = highlight_json(value, 0);
    println!();
    for line in highlighted.lines() {
        println!("  {}", line);
    }
}

/// Render response headers as a clean aligned table.
pub fn render_headers(headers: &[(String, String)]) {
    if headers.is_empty() {
        return;
    }
    println!();
    for (key, val) in headers {
        println!(
            "  {}  {}",
            colors::muted().apply_to(format!("{:<30}", key)),
            colors::accent().apply_to(val),
        );
    }
}

/// Render a data table with headers and rows using comfy-table.
pub fn render_table(headers: &[&str], rows: &[Vec<String>]) {
    let mut table = Table::new();
    table
        .load_preset(presets::UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(headers);
    for row in rows {
        table.add_row(row);
    }

    // Apply header styling
    if colors::colors_enabled() {
        println!("\n{}", table);
    } else {
        println!("\n{}", table);
    }
}

/// Structured error display: what / why / fix.
/// Red header, dim context.
pub fn render_error(what: &str, why: &str, fix: &str) {
    println!();
    println!(
        "  {}",
        colors::error_bold().apply_to(format!("Error: {}", what))
    );
    if !why.is_empty() {
        println!();
        println!("  {}", colors::muted().apply_to(why));
    }
    if !fix.is_empty() {
        println!();
        println!("  {}", colors::muted().apply_to("Try:"));
        println!("    {}", colors::accent().apply_to(fix));
    }
    println!();
}

/// AI insight block with blue accent and clear attribution.
pub fn render_ai_insight(title: &str, content: &str) {
    let header = if title.is_empty() {
        "AI Analysis".to_string()
    } else {
        title.to_string()
    };

    let width = terminal_width().min(76);
    let separator: String = "\u{2500}".repeat(width.saturating_sub(2));

    println!();
    println!("  {}", colors::info_bold().apply_to(&header));
    println!("  {}", colors::info().apply_to(&separator));
    for line in content.lines() {
        println!("  {}", line);
    }
    println!();
}

/// Render a test result line: [PASS] green / [FAIL] red.
pub fn render_test_result(name: &str, passed: bool, duration: Duration) {
    let (badge, style) = if passed {
        ("[PASS]", colors::success_bold())
    } else {
        ("[FAIL]", colors::error_bold())
    };

    let time_text = format_duration(duration);
    println!(
        "  {} {:<50} {}",
        style.apply_to(badge),
        name,
        colors::muted().apply_to(time_text),
    );
}

/// Render a complete test suite summary with colored badges and failure details.
///
/// This is the rich TTY alternative to `test_runner::format_summary_human()`.
pub fn render_test_summary(summary: &TestSummary) {
    let width = terminal_width().min(76);
    let separator: String = "\u{2500}".repeat(width.saturating_sub(2));

    // Header
    println!();
    println!(
        "  {}",
        colors::accent_bold().apply_to(format!("MCP Test Results: {}", summary.suite))
    );
    println!(
        "  {} {}",
        colors::muted().apply_to("Server:"),
        summary.server,
    );
    println!("  {}", colors::muted().apply_to(&separator));

    // Individual test results
    for result in &summary.tests {
        render_test_result_line(result);
    }

    // Footer totals
    println!("  {}", colors::muted().apply_to(&separator));

    let total_time = if summary.duration_ms < 1000 {
        format!("{}ms", summary.duration_ms)
    } else {
        format!("{:.1}s", summary.duration_ms as f64 / 1000.0)
    };

    let mut parts = Vec::new();
    if summary.passed > 0 {
        parts.push(format!(
            "{}",
            colors::success_bold().apply_to(format!("{} passed", summary.passed))
        ));
    }
    if summary.failed > 0 {
        parts.push(format!(
            "{}",
            colors::error_bold().apply_to(format!("{} failed", summary.failed))
        ));
    }
    if summary.skipped > 0 {
        parts.push(format!(
            "{}",
            colors::warning().apply_to(format!("{} skipped", summary.skipped))
        ));
    }

    println!(
        "  {} in {}",
        parts.join(", "),
        colors::muted().apply_to(&total_time),
    );
    println!();
}

/// Render a single test result line with badge, name, duration, and failure details.
fn render_test_result_line(result: &TestResult) {
    let (badge, badge_style) = match result.status {
        TestStatus::Passed => ("PASS", colors::success_bold()),
        TestStatus::Failed => ("FAIL", colors::error_bold()),
        TestStatus::Skipped => ("SKIP", colors::warning()),
    };

    let time_text = if result.duration_ms > 0 {
        if result.duration_ms < 1000 {
            format!("{}ms", result.duration_ms)
        } else {
            format!("{:.1}s", result.duration_ms as f64 / 1000.0)
        }
    } else {
        String::new()
    };

    println!(
        "  {} {:<50} {}",
        badge_style.apply_to(format!("[{}]", badge)),
        result.name,
        colors::muted().apply_to(&time_text),
    );

    // Show failures indented below
    for failure in &result.failures {
        println!(
            "         {} {} {}",
            colors::error().apply_to(&failure.assertion),
            colors::muted().apply_to("expected:"),
            failure.expected,
        );
        println!(
            "         {}      {} {}",
            " ".repeat(failure.assertion.len()),
            colors::muted().apply_to("got:"),
            colors::error().apply_to(&failure.actual),
        );
    }
}

/// Show progress during test execution: "Running [3/15] test_name..."
pub fn render_test_progress(test_name: &str, index: usize, total: usize) {
    println!(
        "  {} {}",
        colors::muted().apply_to(format!("[{}/{}]", index, total)),
        colors::accent().apply_to(test_name),
    );
}

/// Render a full MCP security scan report with color-coded severity badges.
///
/// This is the rich TTY alternative to `mcp::security::render_report()`.
pub fn render_security_report(report: &SecurityReport) {
    let width = terminal_width().min(76);
    let separator: String = "\u{2500}".repeat(width.saturating_sub(2));

    // Header with risk level badge
    let risk_style = match report.risk_level {
        RiskLevel::Critical => colors::error_bold(),
        RiskLevel::High => colors::error_bold(),
        RiskLevel::Medium => colors::warning_bold(),
        RiskLevel::Low => colors::success_bold(),
    };

    println!();
    println!(
        "  {}",
        colors::accent_bold().apply_to("MCP Security Report")
    );
    println!(
        "  {} {}",
        colors::muted().apply_to("Risk Level:"),
        risk_style.apply_to(report.risk_level.to_string()),
    );
    println!("  {}", colors::muted().apply_to(&separator));

    if report.findings.is_empty() {
        println!(
            "\n  {}\n",
            colors::success().apply_to("No security findings. The server appears well-configured.")
        );
        return;
    }

    // Group findings by severity in priority order
    let severity_order = [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
        Severity::Info,
    ];

    for severity in &severity_order {
        let group: Vec<&SecurityFinding> = report
            .findings
            .iter()
            .filter(|f| &f.severity == severity)
            .collect();

        if group.is_empty() {
            continue;
        }

        println!();
        let (badge_style, count_style) = match severity {
            Severity::Critical => (colors::error_bold(), colors::error_bold()),
            Severity::High => (colors::error_bold(), colors::error()),
            Severity::Medium => (colors::warning_bold(), colors::warning()),
            Severity::Low => (colors::muted(), colors::muted()),
            Severity::Info => (colors::muted(), colors::muted()),
        };

        println!(
            "  {} {}",
            badge_style.apply_to(format!("[{}]", severity)),
            count_style.apply_to(format!("{} finding(s)", group.len())),
        );

        for finding in &group {
            let tool_str = finding
                .tool_name
                .as_deref()
                .map(|t| format!(" ({})", t))
                .unwrap_or_default();

            println!(
                "    {} {}{}",
                colors::muted().apply_to(&finding.category),
                finding.title,
                colors::muted().apply_to(&tool_str),
            );
            println!(
                "      {}",
                colors::muted().apply_to(&finding.description),
            );
            println!(
                "      {}",
                colors::accent().apply_to(format!("Fix: {}", finding.recommendation)),
            );
        }
    }

    // Footer totals
    println!();
    println!("  {}", colors::muted().apply_to(&separator));

    let mut counts = Vec::new();
    let count_by = |sev: &Severity| -> usize {
        report.findings.iter().filter(|f| &f.severity == sev).count()
    };

    let critical = count_by(&Severity::Critical);
    let high = count_by(&Severity::High);
    let medium = count_by(&Severity::Medium);
    let low = count_by(&Severity::Low);
    let info = count_by(&Severity::Info);

    if critical > 0 {
        counts.push(format!(
            "{}",
            colors::error_bold().apply_to(format!("{} Critical", critical))
        ));
    }
    if high > 0 {
        counts.push(format!(
            "{}",
            colors::error().apply_to(format!("{} High", high))
        ));
    }
    if medium > 0 {
        counts.push(format!(
            "{}",
            colors::warning().apply_to(format!("{} Medium", medium))
        ));
    }
    if low > 0 {
        counts.push(format!(
            "{}",
            colors::muted().apply_to(format!("{} Low", low))
        ));
    }
    if info > 0 {
        counts.push(format!(
            "{}",
            colors::muted().apply_to(format!("{} Info", info))
        ));
    }

    println!(
        "  {} ({} total)",
        counts.join(", "),
        report.findings.len(),
    );
    println!();
}

/// Section with underline header.
pub fn render_section(title: &str, content: &str) {
    let width = title.len().max(20);
    let separator: String = "\u{2500}".repeat(width);

    println!();
    println!("  {}", colors::accent_bold().apply_to(title));
    println!("  {}", colors::muted().apply_to(&separator));
    for line in content.lines() {
        println!("  {}", line);
    }
}

/// A simple progress spinner message (for use with indicatif).
/// Returns a formatted prefix string, not an actual spinner --
/// the caller should use `indicatif::ProgressBar` with this message.
pub fn spinner_style() -> indicatif::ProgressStyle {
    indicatif::ProgressStyle::with_template("  {spinner:.cyan} {msg}")
        .unwrap_or_else(|_| indicatif::ProgressStyle::default_spinner())
        .tick_strings(&["\u{25cb}", "\u{25d4}", "\u{25d1}", "\u{25d5}", "\u{25cf}"])
}

// ---------------------------------------------------------------------------
// JSON syntax highlighter
// ---------------------------------------------------------------------------

fn highlight_json(value: &serde_json::Value, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let inner_pad = "  ".repeat(indent + 1);

    match value {
        serde_json::Value::Null => {
            format!("{}", colors::json_null().apply_to("null"))
        }
        serde_json::Value::Bool(b) => {
            format!("{}", colors::json_bool().apply_to(b))
        }
        serde_json::Value::Number(n) => {
            format!("{}", colors::json_number().apply_to(n))
        }
        serde_json::Value::String(s) => {
            let escaped = serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s));
            format!("{}", colors::json_string().apply_to(escaped))
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return format!(
                    "{}{}",
                    colors::json_punct().apply_to("["),
                    colors::json_punct().apply_to("]")
                );
            }
            let mut lines = Vec::new();
            lines.push(format!("{}", colors::json_punct().apply_to("[")));
            for (i, item) in arr.iter().enumerate() {
                let comma = if i < arr.len() - 1 {
                    format!("{}", colors::json_punct().apply_to(","))
                } else {
                    String::new()
                };
                lines.push(format!(
                    "{}{}{}",
                    inner_pad,
                    highlight_json(item, indent + 1),
                    comma,
                ));
            }
            lines.push(format!("{}{}", pad, colors::json_punct().apply_to("]")));
            lines.join("\n")
        }
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                return format!(
                    "{}{}",
                    colors::json_punct().apply_to("{"),
                    colors::json_punct().apply_to("}")
                );
            }
            let mut lines = Vec::new();
            lines.push(format!("{}", colors::json_punct().apply_to("{")));
            let entries: Vec<_> = map.iter().collect();
            for (i, (key, val)) in entries.iter().enumerate() {
                let comma = if i < entries.len() - 1 {
                    format!("{}", colors::json_punct().apply_to(","))
                } else {
                    String::new()
                };
                let key_str = format!("\"{}\"", key);
                lines.push(format!(
                    "{}{}{} {}{}",
                    inner_pad,
                    colors::json_key().apply_to(&key_str),
                    colors::json_punct().apply_to(":"),
                    highlight_json(val, indent + 1),
                    comma,
                ));
            }
            lines.push(format!("{}{}", pad, colors::json_punct().apply_to("}")));
            lines.join("\n")
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn status_reason(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "",
    }
}

fn format_duration(d: Duration) -> String {
    let ms = d.as_millis();
    if ms < 1000 {
        format!("{}ms", ms)
    } else {
        format!("{:.1}s", d.as_secs_f64())
    }
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

// ---------------------------------------------------------------------------
// Output mode: controls what gets printed
// ---------------------------------------------------------------------------

/// Output mode determines format and verbosity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    /// Full color, progress indicators, hints (default when TTY).
    Human,
    /// Structured JSON for scripting and CI.
    Json,
    /// JUnit XML for CI dashboards.
    Junit,
    /// No output except final result / exit code.
    Quiet,
}

impl OutputMode {
    /// Detect the right mode from flags and environment.
    pub fn detect(json_flag: bool, junit_flag: bool, quiet_flag: bool) -> Self {
        if json_flag {
            return OutputMode::Json;
        }
        if junit_flag {
            return OutputMode::Junit;
        }
        if quiet_flag {
            return OutputMode::Quiet;
        }
        if !std::io::stdout().is_terminal() {
            // When piped, default to quiet human (no colors already handled by init_colors)
            return OutputMode::Human;
        }
        OutputMode::Human
    }

    pub fn is_human(&self) -> bool {
        *self == OutputMode::Human
    }
}
