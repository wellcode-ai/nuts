use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::error::NutsError;
use crate::mcp::client::McpClient;
use crate::mcp::types::{ContentItem, TransportConfig};

// ---------------------------------------------------------------------------
// YAML test file structures
// ---------------------------------------------------------------------------

/// Top-level structure of a `.test.yaml` file.
#[derive(Debug, Clone, Deserialize)]
pub struct TestFile {
    pub server: ServerConfig,
    pub tests: Vec<TestCase>,
}

/// Server connection configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Display name (optional, used in reports).
    pub name: Option<String>,
    /// stdio transport: the command to spawn.
    pub command: Option<String>,
    /// SSE transport URL.
    pub sse: Option<String>,
    /// HTTP (Streamable HTTP) transport URL.
    pub http: Option<String>,
    /// Connection timeout in seconds (default: 30).
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Environment variables for stdio transport.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Working directory for stdio transport.
    pub cwd: Option<String>,
}

fn default_timeout() -> u64 {
    30
}

impl ServerConfig {
    /// Convert to a `TransportConfig` for connecting via `McpClient`.
    pub fn to_transport_config(&self) -> Result<TransportConfig, NutsError> {
        match (&self.command, &self.sse, &self.http) {
            (Some(command), None, None) => {
                let parts: Vec<&str> = command.split_whitespace().collect();
                let (cmd, args) = parts.split_first().ok_or_else(|| NutsError::InvalidInput {
                    message: "server.command is empty".into(),
                })?;
                Ok(TransportConfig::Stdio {
                    command: cmd.to_string(),
                    args: args.iter().map(|s| s.to_string()).collect(),
                    env: self
                        .env
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                })
            }
            (None, Some(url), None) => Ok(TransportConfig::Sse {
                url: url.clone(),
                bearer: None,
            }),
            (None, None, Some(url)) => Ok(TransportConfig::Http {
                url: url.clone(),
                bearer: None,
            }),
            _ => Err(NutsError::InvalidInput {
                message: "server config must have exactly one of: command, sse, http".into(),
            }),
        }
    }

    /// Human-readable transport description for reports.
    pub fn transport_description(&self) -> String {
        if let Some(cmd) = &self.command {
            format!("{cmd} (stdio)")
        } else if let Some(url) = &self.sse {
            format!("{url} (sse)")
        } else if let Some(url) = &self.http {
            format!("{url} (http)")
        } else {
            "(unknown)".to_string()
        }
    }
}

/// A single test case, either single-step or multi-step.
#[derive(Debug, Clone, Deserialize)]
pub struct TestCase {
    /// Human-readable test name.
    pub name: String,
    /// Optional longer description.
    pub description: Option<String>,
    /// Skip this test.
    #[serde(default)]
    pub skip: bool,
    /// Tags for filtering.
    #[serde(default)]
    pub tags: Vec<String>,

    // Single-step fields (ignored if `steps` is present)
    /// Tool to call.
    pub tool: Option<String>,
    /// Resource to read.
    pub resource: Option<String>,
    /// Prompt to get.
    pub prompt: Option<String>,
    /// Input arguments.
    pub input: Option<serde_json::Value>,
    /// Assertions.
    #[serde(rename = "assert")]
    pub assertions: Option<TestAssertions>,
    /// Capture values from the result.
    pub capture: Option<HashMap<String, String>>,

    // Multi-step
    /// Steps for a multi-step test.
    pub steps: Option<Vec<TestStep>>,
}

/// A single step in a multi-step test.
#[derive(Debug, Clone, Deserialize)]
pub struct TestStep {
    pub tool: Option<String>,
    pub resource: Option<String>,
    pub prompt: Option<String>,
    pub input: Option<serde_json::Value>,
    #[serde(rename = "assert")]
    pub assertions: Option<TestAssertions>,
    pub capture: Option<HashMap<String, String>>,
}

/// Assertion block within a test or step.
#[derive(Debug, Clone, Deserialize)]
pub struct TestAssertions {
    /// Expected status: "success", "error", or a list of acceptable values.
    pub status: Option<StatusAssertion>,

    /// Assert the JSON type of the result.
    #[serde(rename = "result.type")]
    pub result_type: Option<String>,

    /// Assert that the result has specific field(s).
    #[serde(rename = "result.has_field")]
    pub result_has_field: Option<FieldAssertion>,

    /// Assert that the result contains a value.
    #[serde(rename = "result.contains")]
    pub result_contains: Option<ContainsAssertion>,

    /// Assert the result equals a value or field equals a value.
    #[serde(rename = "result.equals")]
    pub result_equals: Option<serde_json::Value>,

    /// Assert the result length.
    #[serde(rename = "result.length")]
    pub result_length: Option<LengthAssertion>,

    /// Assert max/min duration in milliseconds.
    pub duration_ms: Option<DurationAssertion>,

    /// Assert specific JSON-RPC error code.
    #[serde(rename = "error.code")]
    pub error_code: Option<i64>,

    /// Assert error code is one of a list.
    #[serde(rename = "error.code_in")]
    pub error_code_in: Option<Vec<i64>>,

    /// Assert error message (exact match).
    #[serde(rename = "error.message")]
    pub error_message: Option<String>,

    /// Assert error message contains substring.
    #[serde(rename = "error.message_contains")]
    pub error_message_contains: Option<String>,

    /// Assert the result text matches a regex pattern.
    #[serde(rename = "result.matches")]
    pub result_matches: Option<String>,
}

/// Status can be a single string or a list of acceptable values.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum StatusAssertion {
    Single(String),
    Multiple(Vec<String>),
}

/// Field assertion: a single field name or a list of field names.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum FieldAssertion {
    Single(String),
    Multiple(Vec<String>),
}

/// Contains assertion with field and value.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ContainsAssertion {
    /// Simple string contains check.
    Simple(String),
    /// Field/value check for objects/arrays.
    FieldValue {
        field: String,
        value: serde_json::Value,
    },
}

/// Length assertion: exact number or min/max.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum LengthAssertion {
    Exact(usize),
    Range {
        min: Option<usize>,
        max: Option<usize>,
    },
}

/// Duration assertion: min/max in milliseconds.
#[derive(Debug, Clone, Deserialize)]
pub struct DurationAssertion {
    pub max: Option<u64>,
    pub min: Option<u64>,
}

// ---------------------------------------------------------------------------
// Test results
// ---------------------------------------------------------------------------

/// The result of running a single test (or step).
#[derive(Debug, Clone, Serialize)]
pub struct TestResult {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub operation: String,
    /// If failed, what went wrong.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<AssertionFailure>,
}

/// A single assertion failure.
#[derive(Debug, Clone, Serialize)]
pub struct AssertionFailure {
    pub assertion: String,
    pub expected: String,
    pub actual: String,
}

/// Status of a test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

/// Summary of an entire test suite run.
#[derive(Debug, Clone, Serialize)]
pub struct TestSummary {
    pub suite: String,
    pub server: String,
    pub transport: String,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub tests: Vec<TestResult>,
}

// ---------------------------------------------------------------------------
// Test execution
// ---------------------------------------------------------------------------

/// Parse a YAML test file from disk.
pub fn parse_test_file(path: &str) -> Result<TestFile, NutsError> {
    let content = std::fs::read_to_string(path).map_err(|e| NutsError::Mcp {
        message: format!("failed to read test file '{}': {}", path, e),
    })?;
    let test_file: TestFile = serde_yaml::from_str(&content)?;

    // Validate: must have at least one test
    if test_file.tests.is_empty() {
        return Err(NutsError::InvalidInput {
            message: format!("test file '{}' has no tests", path),
        });
    }

    Ok(test_file)
}

/// Run all tests in a parsed test file.
///
/// Connects to the MCP server once, runs all tests, then disconnects.
pub async fn run_tests(test_file_path: &str) -> Result<TestSummary, NutsError> {
    let test_file = parse_test_file(test_file_path)?;
    let suite_name = test_file.server.name.clone().unwrap_or_else(|| {
        Path::new(test_file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string()
    });
    let transport_desc = test_file.server.transport_description();
    let transport_config = test_file.server.to_transport_config()?;

    // Connect to the MCP server
    let client = McpClient::connect(&transport_config).await?;

    let suite_start = Instant::now();
    let mut results = Vec::new();

    for test_case in &test_file.tests {
        if test_case.skip {
            results.push(TestResult {
                name: test_case.name.clone(),
                status: TestStatus::Skipped,
                duration_ms: 0,
                operation: describe_operation(test_case),
                failures: vec![],
            });
            continue;
        }

        if let Some(steps) = &test_case.steps {
            // Multi-step test
            let step_results = run_multi_step_test(&client, test_case, steps).await;
            results.extend(step_results);
        } else {
            // Single-step test
            let result = run_single_test(&client, test_case).await;
            results.push(result);
        }
    }

    // Disconnect
    let _ = client.disconnect().await;

    let suite_duration = suite_start.elapsed();
    let passed = results
        .iter()
        .filter(|r| r.status == TestStatus::Passed)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == TestStatus::Skipped)
        .count();

    Ok(TestSummary {
        suite: suite_name,
        server: transport_desc,
        transport: transport_type_name(&transport_config),
        passed,
        failed,
        skipped,
        duration_ms: suite_duration.as_millis() as u64,
        tests: results,
    })
}

/// Run a single-step test case.
async fn run_single_test(client: &McpClient, test: &TestCase) -> TestResult {
    let operation = describe_operation(test);
    let start = Instant::now();

    let exec_result = execute_operation(
        client,
        test.tool.as_deref(),
        test.resource.as_deref(),
        test.prompt.as_deref(),
        test.input.as_ref(),
        &HashMap::new(),
    )
    .await;

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;

    match exec_result {
        Ok(output) => {
            let failures = if let Some(assertions) = &test.assertions {
                check_assertions(assertions, &output, duration_ms)
            } else {
                vec![]
            };
            let status = if failures.is_empty() {
                TestStatus::Passed
            } else {
                TestStatus::Failed
            };
            TestResult {
                name: test.name.clone(),
                status,
                duration_ms,
                operation,
                failures,
            }
        }
        Err(e) => TestResult {
            name: test.name.clone(),
            status: TestStatus::Failed,
            duration_ms,
            operation,
            failures: vec![AssertionFailure {
                assertion: "execution".into(),
                expected: "successful MCP call".into(),
                actual: e.to_string(),
            }],
        },
    }
}

/// Run a multi-step test case.
async fn run_multi_step_test(
    client: &McpClient,
    test: &TestCase,
    steps: &[TestStep],
) -> Vec<TestResult> {
    let mut results = Vec::new();
    let mut captures: HashMap<String, serde_json::Value> = HashMap::new();
    let mut step_failed = false;

    for (i, step) in steps.iter().enumerate() {
        let step_name = format!(
            "{} / Step {}: {}",
            test.name,
            i + 1,
            step_operation_name(step)
        );

        if step_failed {
            results.push(TestResult {
                name: step_name,
                status: TestStatus::Skipped,
                duration_ms: 0,
                operation: step_describe_operation(step),
                failures: vec![],
            });
            continue;
        }

        // Resolve variable references in input
        let resolved_input = step
            .input
            .as_ref()
            .map(|input| resolve_variables(input, &captures));

        let start = Instant::now();
        let exec_result = execute_operation(
            client,
            step.tool.as_deref(),
            step.resource.as_deref(),
            step.prompt.as_deref(),
            resolved_input.as_ref(),
            &captures,
        )
        .await;
        let duration = start.elapsed();
        let duration_ms = duration.as_millis() as u64;

        match exec_result {
            Ok(output) => {
                // Process captures
                if let Some(capture_defs) = &step.capture {
                    for (var_name, json_path) in capture_defs {
                        if let Some(value) = extract_json_path(&output.content_json, json_path) {
                            captures.insert(var_name.clone(), value);
                        }
                    }
                }

                // Check assertions
                let failures = if let Some(assertions) = &step.assertions {
                    check_assertions(assertions, &output, duration_ms)
                } else {
                    vec![]
                };
                let status = if failures.is_empty() {
                    TestStatus::Passed
                } else {
                    step_failed = true;
                    TestStatus::Failed
                };
                results.push(TestResult {
                    name: step_name,
                    status,
                    duration_ms,
                    operation: step_describe_operation(step),
                    failures,
                });
            }
            Err(e) => {
                step_failed = true;
                results.push(TestResult {
                    name: step_name,
                    status: TestStatus::Failed,
                    duration_ms,
                    operation: step_describe_operation(step),
                    failures: vec![AssertionFailure {
                        assertion: "execution".into(),
                        expected: "successful MCP call".into(),
                        actual: e.to_string(),
                    }],
                });
            }
        }
    }

    results
}

// ---------------------------------------------------------------------------
// Operation execution
// ---------------------------------------------------------------------------

/// The output of executing an MCP operation, normalized for assertion checking.
struct OperationOutput {
    is_error: bool,
    /// The text content concatenated from all content items.
    text_content: String,
    /// The result parsed as JSON (or Value::Null if not parseable).
    content_json: serde_json::Value,
    /// Raw content items.
    #[allow(dead_code)]
    content_items: Vec<ContentItem>,
}

/// Execute an MCP operation (tool call, resource read, or prompt get).
async fn execute_operation(
    client: &McpClient,
    tool: Option<&str>,
    resource: Option<&str>,
    prompt: Option<&str>,
    input: Option<&serde_json::Value>,
    _captures: &HashMap<String, serde_json::Value>,
) -> Result<OperationOutput, NutsError> {
    match (tool, resource, prompt) {
        (Some(tool_name), None, None) => {
            let args = input.cloned().unwrap_or(serde_json::json!({}));
            let result = client.call_tool(tool_name, args).await?;
            let text = result
                .content
                .iter()
                .filter_map(|c| c.as_text())
                .collect::<Vec<_>>()
                .join("");
            let json =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text.clone()));
            Ok(OperationOutput {
                is_error: result.is_error,
                text_content: text,
                content_json: json,
                content_items: result.content,
            })
        }
        (None, Some(uri), None) => {
            let result = client.read_resource(uri).await?;
            let text = result
                .contents
                .iter()
                .filter_map(|c| c.as_text())
                .collect::<Vec<_>>()
                .join("");
            let json =
                serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text.clone()));
            Ok(OperationOutput {
                is_error: false,
                text_content: text,
                content_json: json,
                content_items: result.contents,
            })
        }
        (None, None, Some(prompt_name)) => {
            let args = input.cloned();
            let result = client.get_prompt(prompt_name, args).await?;
            let json = serde_json::to_value(&result).unwrap_or_default();
            Ok(OperationOutput {
                is_error: false,
                text_content: serde_json::to_string(&result).unwrap_or_default(),
                content_json: json,
                content_items: vec![],
            })
        }
        _ => Err(NutsError::InvalidInput {
            message: "test must specify exactly one of: tool, resource, prompt".into(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Assertion checking
// ---------------------------------------------------------------------------

/// Structured error information extracted from error content.
struct ErrorInfo {
    /// The error code, if found in structured JSON.
    code: Option<i64>,
    /// The error message — extracted from JSON "message" field, or the raw text.
    message: String,
}

/// Extract structured error info from the operation output.
///
/// When a tool returns `is_error=true`, the text content may be a JSON object
/// with "code" and "message" fields (JSON-RPC style). If it is not valid JSON
/// or doesn't contain those fields, we fall back to using the raw text.
fn extract_error_info(output: &OperationOutput) -> ErrorInfo {
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(&output.text_content) {
        let code = obj.get("code").and_then(|c| c.as_i64());
        let message = obj
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or(&output.text_content)
            .to_string();
        ErrorInfo { code, message }
    } else {
        ErrorInfo {
            code: None,
            message: output.text_content.clone(),
        }
    }
}

/// Check all assertions against the operation output. Returns a list of failures.
fn check_assertions(
    assertions: &TestAssertions,
    output: &OperationOutput,
    duration_ms: u64,
) -> Vec<AssertionFailure> {
    let mut failures = Vec::new();

    // status assertion
    if let Some(status) = &assertions.status {
        let actual_status = if output.is_error { "error" } else { "success" };
        let expected_statuses = match status {
            StatusAssertion::Single(s) => vec![s.as_str()],
            StatusAssertion::Multiple(list) => list.iter().map(|s| s.as_str()).collect(),
        };
        if !expected_statuses.contains(&actual_status) {
            failures.push(AssertionFailure {
                assertion: "status".into(),
                expected: format!("{}", expected_statuses.join(" or ")),
                actual: actual_status.to_string(),
            });
        }
    }

    // result.type assertion
    if let Some(expected_type) = &assertions.result_type {
        let actual_type = json_type_name(&output.content_json);
        if actual_type != expected_type.as_str() {
            failures.push(AssertionFailure {
                assertion: "result.type".into(),
                expected: expected_type.clone(),
                actual: actual_type.to_string(),
            });
        }
    }

    // result.has_field assertion
    if let Some(field_assert) = &assertions.result_has_field {
        let fields = match field_assert {
            FieldAssertion::Single(f) => vec![f.as_str()],
            FieldAssertion::Multiple(list) => list.iter().map(|s| s.as_str()).collect(),
        };
        for field in fields {
            if resolve_dot_path(&output.content_json, field).is_none() {
                failures.push(AssertionFailure {
                    assertion: "result.has_field".into(),
                    expected: format!("field '{}' exists", field),
                    actual: "field not found".into(),
                });
            }
        }
    }

    // result.contains assertion
    if let Some(contains) = &assertions.result_contains {
        match contains {
            ContainsAssertion::Simple(text) => {
                if !output.text_content.contains(text.as_str()) {
                    failures.push(AssertionFailure {
                        assertion: "result.contains".into(),
                        expected: format!("contains \"{}\"", text),
                        actual: truncate_string(&output.text_content, 200),
                    });
                }
            }
            ContainsAssertion::FieldValue { field, value } => {
                let found = match &output.content_json {
                    serde_json::Value::Array(arr) => arr
                        .iter()
                        .any(|item| item.get(field).map(|v| v == value).unwrap_or(false)),
                    obj @ serde_json::Value::Object(_) => {
                        obj.get(field).map(|v| v == value).unwrap_or(false)
                    }
                    _ => false,
                };
                if !found {
                    failures.push(AssertionFailure {
                        assertion: "result.contains".into(),
                        expected: format!("field '{}' == {:?}", field, value),
                        actual: truncate_string(
                            &serde_json::to_string(&output.content_json).unwrap_or_default(),
                            200,
                        ),
                    });
                }
            }
        }
    }

    // result.equals assertion
    if let Some(equals) = &assertions.result_equals {
        // If it has "field" and "value" keys, it's a field-specific comparison
        if let (Some(field), Some(value)) = (
            equals.get("field").and_then(|f| f.as_str()),
            equals.get("value"),
        ) {
            let actual = resolve_dot_path(&output.content_json, field);
            if actual.as_ref() != Some(value) {
                failures.push(AssertionFailure {
                    assertion: "result.equals".into(),
                    expected: format!("{}.{} == {:?}", "result", field, value),
                    actual: format!("{:?}", actual),
                });
            }
        } else {
            // Compare entire result
            if &output.content_json != equals {
                failures.push(AssertionFailure {
                    assertion: "result.equals".into(),
                    expected: serde_json::to_string(equals).unwrap_or_default(),
                    actual: truncate_string(
                        &serde_json::to_string(&output.content_json).unwrap_or_default(),
                        200,
                    ),
                });
            }
        }
    }

    // result.length assertion
    if let Some(length) = &assertions.result_length {
        let actual_len = match &output.content_json {
            serde_json::Value::Array(arr) => Some(arr.len()),
            serde_json::Value::String(s) => Some(s.len()),
            _ => None,
        };
        match (length, actual_len) {
            (LengthAssertion::Exact(expected), Some(actual)) => {
                if actual != *expected {
                    failures.push(AssertionFailure {
                        assertion: "result.length".into(),
                        expected: format!("{}", expected),
                        actual: format!("{}", actual),
                    });
                }
            }
            (LengthAssertion::Range { min, max }, Some(actual)) => {
                if let Some(min_val) = min {
                    if actual < *min_val {
                        failures.push(AssertionFailure {
                            assertion: "result.length".into(),
                            expected: format!("min {}", min_val),
                            actual: format!("{}", actual),
                        });
                    }
                }
                if let Some(max_val) = max {
                    if actual > *max_val {
                        failures.push(AssertionFailure {
                            assertion: "result.length".into(),
                            expected: format!("max {}", max_val),
                            actual: format!("{}", actual),
                        });
                    }
                }
            }
            (_, None) => {
                failures.push(AssertionFailure {
                    assertion: "result.length".into(),
                    expected: "array or string".into(),
                    actual: json_type_name(&output.content_json).to_string(),
                });
            }
        }
    }

    // duration_ms assertion
    if let Some(dur) = &assertions.duration_ms {
        if let Some(max) = dur.max {
            if duration_ms > max {
                failures.push(AssertionFailure {
                    assertion: "duration_ms".into(),
                    expected: format!("max {}ms", max),
                    actual: format!("{}ms", duration_ms),
                });
            }
        }
        if let Some(min) = dur.min {
            if duration_ms < min {
                failures.push(AssertionFailure {
                    assertion: "duration_ms".into(),
                    expected: format!("min {}ms", min),
                    actual: format!("{}ms", duration_ms),
                });
            }
        }
    }

    // --- Error assertions: extract structured info once ---
    let needs_error_info = assertions.error_code.is_some()
        || assertions.error_code_in.is_some()
        || assertions.error_message.is_some()
        || assertions.error_message_contains.is_some();

    if needs_error_info {
        let err_info = extract_error_info(output);

        // error.code assertion
        if let Some(expected_code) = assertions.error_code {
            match err_info.code {
                Some(actual_code) if actual_code == expected_code => {}
                Some(actual_code) => {
                    failures.push(AssertionFailure {
                        assertion: "error.code".into(),
                        expected: format!("{}", expected_code),
                        actual: format!("{}", actual_code),
                    });
                }
                None => {
                    failures.push(AssertionFailure {
                        assertion: "error.code".into(),
                        expected: format!("{}", expected_code),
                        actual: "no error code found in response".into(),
                    });
                }
            }
        }

        // error.code_in assertion
        if let Some(expected_codes) = &assertions.error_code_in {
            match err_info.code {
                Some(actual_code) if expected_codes.contains(&actual_code) => {}
                Some(actual_code) => {
                    failures.push(AssertionFailure {
                        assertion: "error.code_in".into(),
                        expected: format!("one of {:?}", expected_codes),
                        actual: format!("{}", actual_code),
                    });
                }
                None => {
                    failures.push(AssertionFailure {
                        assertion: "error.code_in".into(),
                        expected: format!("one of {:?}", expected_codes),
                        actual: "no error code found in response".into(),
                    });
                }
            }
        }

        // error.message assertion (uses extracted message, not raw text)
        if let Some(expected_msg) = &assertions.error_message {
            if err_info.message != *expected_msg {
                failures.push(AssertionFailure {
                    assertion: "error.message".into(),
                    expected: expected_msg.clone(),
                    actual: truncate_string(&err_info.message, 200),
                });
            }
        }

        // error.message_contains assertion (uses extracted message)
        if let Some(substring) = &assertions.error_message_contains {
            if !err_info.message.contains(substring.as_str()) {
                failures.push(AssertionFailure {
                    assertion: "error.message_contains".into(),
                    expected: format!("contains \"{}\"", substring),
                    actual: truncate_string(&err_info.message, 200),
                });
            }
        }
    }

    // result.matches regex assertion
    if let Some(pattern) = &assertions.result_matches {
        match regex::Regex::new(pattern) {
            Ok(re) => {
                if !re.is_match(&output.text_content) {
                    failures.push(AssertionFailure {
                        assertion: "result.matches".into(),
                        expected: format!("matches /{}/", pattern),
                        actual: truncate_string(&output.text_content, 200),
                    });
                }
            }
            Err(e) => {
                failures.push(AssertionFailure {
                    assertion: "result.matches".into(),
                    expected: format!("valid regex /{}/", pattern),
                    actual: format!("invalid regex: {}", e),
                });
            }
        }
    }

    failures
}

// ---------------------------------------------------------------------------
// Formatting helpers for test output
// ---------------------------------------------------------------------------

/// Format a `TestSummary` as human-readable terminal output.
pub fn format_summary_human(summary: &TestSummary) -> String {
    let mut out = String::new();
    out.push_str(&format!("MCP Test Suite: {}\n", summary.suite));
    out.push_str(&format!("Server: {}\n\n", summary.server));

    for result in &summary.tests {
        let badge = match result.status {
            TestStatus::Passed => "[PASS]",
            TestStatus::Failed => "[FAIL]",
            TestStatus::Skipped => "[SKIP]",
        };
        out.push_str(&format!(
            "  {} {:<50} {}ms\n",
            badge, result.name, result.duration_ms
        ));
        for failure in &result.failures {
            out.push_str(&format!(
                "         Expected: {} {}\n         Got: {}\n",
                failure.assertion, failure.expected, failure.actual
            ));
        }
    }

    out.push_str(&format!(
        "\nResults: {} passed, {} failed, {} skipped\n",
        summary.passed, summary.failed, summary.skipped
    ));
    out.push_str(&format!("Duration: {}ms\n", summary.duration_ms));
    out
}

/// Format a `TestSummary` as a JSON value for machine-readable output.
pub fn format_summary_json(summary: &TestSummary) -> serde_json::Value {
    serde_json::to_value(summary).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

/// Describe the operation for a test case.
fn describe_operation(test: &TestCase) -> String {
    if let Some(tool) = &test.tool {
        format!("tool:{}", tool)
    } else if let Some(resource) = &test.resource {
        format!("resource:{}", resource)
    } else if let Some(prompt) = &test.prompt {
        format!("prompt:{}", prompt)
    } else if test.steps.is_some() {
        "multi-step".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Describe the operation for a test step.
fn step_describe_operation(step: &TestStep) -> String {
    if let Some(tool) = &step.tool {
        format!("tool:{}", tool)
    } else if let Some(resource) = &step.resource {
        format!("resource:{}", resource)
    } else if let Some(prompt) = &step.prompt {
        format!("prompt:{}", prompt)
    } else {
        "unknown".to_string()
    }
}

/// Get the operation name (for step labels).
fn step_operation_name(step: &TestStep) -> String {
    step.tool
        .as_deref()
        .or(step.resource.as_deref())
        .or(step.prompt.as_deref())
        .unwrap_or("unknown")
        .to_string()
}

/// Get the transport type name.
fn transport_type_name(config: &TransportConfig) -> String {
    match config {
        TransportConfig::Stdio { .. } => "stdio".to_string(),
        TransportConfig::Sse { .. } => "sse".to_string(),
        TransportConfig::Http { .. } => "http".to_string(),
    }
}

/// Return the JSON type name for a value.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

/// Resolve a dot-separated path like "user.address.city" in a JSON value.
fn resolve_dot_path(value: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let mut current = value.clone();
    for segment in path.split('.') {
        // Handle array index: segment like "items[0]"
        if let Some(bracket_pos) = segment.find('[') {
            let key = &segment[..bracket_pos];
            let idx_str = &segment[bracket_pos + 1..segment.len() - 1];
            if !key.is_empty() {
                current = current.get(key)?.clone();
            }
            let idx: usize = idx_str.parse().ok()?;
            current = current.get(idx)?.clone();
        } else {
            current = current.get(segment)?.clone();
        }
    }
    Some(current)
}

/// Extract a value from JSON using a simplified JSONPath expression.
/// Supports: `$`, `$.field`, `$.field.nested`, `$.array[0]`, `$.array[0].field`
fn extract_json_path(value: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let path = path.trim();
    if path == "$" {
        return Some(value.clone());
    }
    let path = path.strip_prefix("$.")?;
    resolve_dot_path(value, path)
}

/// Resolve `${var}` references in a JSON value using captured variables.
fn resolve_variables(
    input: &serde_json::Value,
    captures: &HashMap<String, serde_json::Value>,
) -> serde_json::Value {
    match input {
        serde_json::Value::String(s) => {
            // Check if the entire string is a single variable reference
            if s.starts_with("${") && s.ends_with('}') && s.matches("${").count() == 1 {
                let var_name = &s[2..s.len() - 1];
                if let Some(value) = captures.get(var_name) {
                    return value.clone();
                }
            }
            // Replace inline references
            let mut result = s.clone();
            for (name, value) in captures {
                let placeholder = format!("${{{}}}", name);
                if result.contains(&placeholder) {
                    let replacement = match value {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(&placeholder, &replacement);
                }
            }
            serde_json::Value::String(result)
        }
        serde_json::Value::Object(map) => {
            let resolved: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), resolve_variables(v, captures)))
                .collect();
            serde_json::Value::Object(resolved)
        }
        serde_json::Value::Array(arr) => {
            let resolved: Vec<serde_json::Value> =
                arr.iter().map(|v| resolve_variables(v, captures)).collect();
            serde_json::Value::Array(resolved)
        }
        other => other.clone(),
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated.
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_server_config_stdio() {
        let yaml = r#"
server:
  name: "test-server"
  command: "node server.js"
  timeout: 10
tests:
  - name: "hello"
    tool: "echo"
    input:
      message: "hi"
    assert:
      status: success
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(tf.server.name, Some("test-server".into()));
        assert_eq!(tf.server.command, Some("node server.js".into()));
        assert_eq!(tf.server.timeout, 10);
        assert_eq!(tf.tests.len(), 1);
        assert_eq!(tf.tests[0].name, "hello");
        assert_eq!(tf.tests[0].tool, Some("echo".into()));
    }

    #[test]
    fn parse_server_config_sse() {
        let yaml = r#"
server:
  sse: "http://localhost:3001/sse"
tests:
  - name: "basic"
    tool: "ping"
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(tf.server.sse, Some("http://localhost:3001/sse".into()));
        let config = tf.server.to_transport_config().unwrap();
        match config {
            TransportConfig::Sse { url, .. } => assert_eq!(url, "http://localhost:3001/sse"),
            _ => panic!("expected SSE transport"),
        }
    }

    #[test]
    fn parse_server_config_http() {
        let yaml = r#"
server:
  http: "http://localhost:8080/mcp"
tests:
  - name: "basic"
    tool: "stats"
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let config = tf.server.to_transport_config().unwrap();
        match config {
            TransportConfig::Http { url, .. } => assert_eq!(url, "http://localhost:8080/mcp"),
            _ => panic!("expected HTTP transport"),
        }
    }

    #[test]
    fn to_transport_config_stdio_splits_args() {
        let config = ServerConfig {
            name: None,
            command: Some("npx -y @mcp/server".into()),
            sse: None,
            http: None,
            timeout: 30,
            env: HashMap::new(),
            cwd: None,
        };
        let tc = config.to_transport_config().unwrap();
        match tc {
            TransportConfig::Stdio { command, args, .. } => {
                assert_eq!(command, "npx");
                assert_eq!(args, vec!["-y", "@mcp/server"]);
            }
            _ => panic!("expected Stdio"),
        }
    }

    #[test]
    fn to_transport_config_rejects_multiple() {
        let config = ServerConfig {
            name: None,
            command: Some("node server.js".into()),
            sse: Some("http://localhost:3001/sse".into()),
            http: None,
            timeout: 30,
            env: HashMap::new(),
            cwd: None,
        };
        assert!(config.to_transport_config().is_err());
    }

    #[test]
    fn to_transport_config_rejects_none() {
        let config = ServerConfig {
            name: None,
            command: None,
            sse: None,
            http: None,
            timeout: 30,
            env: HashMap::new(),
            cwd: None,
        };
        assert!(config.to_transport_config().is_err());
    }

    #[test]
    fn parse_assertions_status_single() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "echo"
    assert:
      status: success
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.status {
            Some(StatusAssertion::Single(s)) => assert_eq!(s, "success"),
            _ => panic!("expected single status"),
        }
    }

    #[test]
    fn parse_assertions_status_multiple() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "echo"
    assert:
      status: [success, error]
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.status {
            Some(StatusAssertion::Multiple(list)) => {
                assert_eq!(list, &["success", "error"]);
            }
            _ => panic!("expected multiple status"),
        }
    }

    #[test]
    fn parse_duration_assertion() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "echo"
    assert:
      duration_ms:
        max: 5000
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        assert_eq!(a.duration_ms.as_ref().unwrap().max, Some(5000));
    }

    #[test]
    fn parse_multi_step_test() {
        let yaml = r#"
server:
  command: "node server.js"
tests:
  - name: "workflow"
    steps:
      - tool: "create"
        input:
          title: "test"
        capture:
          doc_id: "$.id"
        assert:
          status: success
      - tool: "get"
        input:
          id: "${doc_id}"
        assert:
          status: success
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let steps = tf.tests[0].steps.as_ref().unwrap();
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].tool, Some("create".into()));
        assert_eq!(steps[1].tool, Some("get".into()));
        let captures = steps[0].capture.as_ref().unwrap();
        assert_eq!(captures.get("doc_id"), Some(&"$.id".to_string()));
    }

    #[test]
    fn parse_result_type_assertion() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "search"
    assert:
      result.type: array
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        assert_eq!(a.result_type.as_deref(), Some("array"));
    }

    #[test]
    fn parse_has_field_single() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "get"
    assert:
      result.has_field: id
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.result_has_field {
            Some(FieldAssertion::Single(f)) => assert_eq!(f, "id"),
            _ => panic!("expected single field"),
        }
    }

    #[test]
    fn parse_has_field_multiple() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "get"
    assert:
      result.has_field: [id, name, email]
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.result_has_field {
            Some(FieldAssertion::Multiple(list)) => {
                assert_eq!(list, &["id", "name", "email"]);
            }
            _ => panic!("expected multiple fields"),
        }
    }

    #[test]
    fn parse_contains_field_value() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "search"
    assert:
      result.contains:
        field: "title"
        value: "Test Document"
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.result_contains {
            Some(ContainsAssertion::FieldValue { field, value }) => {
                assert_eq!(field, "title");
                assert_eq!(value, &serde_json::json!("Test Document"));
            }
            _ => panic!("expected field/value contains"),
        }
    }

    #[test]
    fn parse_length_exact() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "list"
    assert:
      result.length: 5
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.result_length {
            Some(LengthAssertion::Exact(n)) => assert_eq!(*n, 5),
            _ => panic!("expected exact length"),
        }
    }

    #[test]
    fn parse_length_range() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "list"
    assert:
      result.length:
        min: 1
        max: 100
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        match &a.result_length {
            Some(LengthAssertion::Range { min, max }) => {
                assert_eq!(*min, Some(1));
                assert_eq!(*max, Some(100));
            }
            _ => panic!("expected length range"),
        }
    }

    #[test]
    fn parse_error_code_in() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "bad"
    assert:
      status: error
      error.code_in: [-32602, -32603]
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        assert_eq!(a.error_code_in, Some(vec![-32602, -32603]));
    }

    #[test]
    fn parse_skip_and_tags() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    skip: true
    tags: [smoke, api]
    tool: "echo"
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        assert!(tf.tests[0].skip);
        assert_eq!(tf.tests[0].tags, vec!["smoke", "api"]);
    }

    // --- Assertion evaluation tests ---

    fn make_output(is_error: bool, text: &str) -> OperationOutput {
        let json =
            serde_json::from_str(text).unwrap_or(serde_json::Value::String(text.to_string()));
        OperationOutput {
            is_error,
            text_content: text.to_string(),
            content_json: json,
            content_items: vec![],
        }
    }

    #[test]
    fn assert_status_success_passes() {
        let assertions = TestAssertions {
            status: Some(StatusAssertion::Single("success".into())),
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, "ok");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_status_success_fails_on_error() {
        let assertions = TestAssertions {
            status: Some(StatusAssertion::Single("success".into())),
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, "error occurred");
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "status");
    }

    #[test]
    fn assert_status_either_passes() {
        let assertions = TestAssertions {
            status: Some(StatusAssertion::Multiple(vec![
                "success".into(),
                "error".into(),
            ])),
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, "error");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_result_type_object() {
        let assertions = TestAssertions {
            status: None,
            result_type: Some("object".into()),
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"{"name":"alice"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_result_type_array_fails_on_object() {
        let assertions = TestAssertions {
            status: None,
            result_type: Some("array".into()),
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"{"name":"alice"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "result.type");
    }

    #[test]
    fn assert_has_field_passes() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: Some(FieldAssertion::Multiple(vec!["id".into(), "name".into()])),
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"{"id":1,"name":"alice","email":"a@b.com"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_has_field_fails_missing() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: Some(FieldAssertion::Single("missing_field".into())),
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"{"id":1}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn assert_contains_simple_text() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: Some(ContainsAssertion::Simple("hello".into())),
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, "hello world");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_contains_field_value_in_array() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: Some(ContainsAssertion::FieldValue {
                field: "name".into(),
                value: serde_json::json!("alice"),
            }),
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"[{"name":"bob"},{"name":"alice"}]"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_equals_field_value() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: Some(serde_json::json!({"field": "name", "value": "alice"})),
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"{"name":"alice","id":1}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_length_exact() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: Some(LengthAssertion::Exact(3)),
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"[1,2,3]"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_length_range_fails() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: Some(LengthAssertion::Range {
                min: None,
                max: Some(2),
            }),
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, r#"[1,2,3]"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "result.length");
    }

    #[test]
    fn assert_duration_max_fails() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: Some(DurationAssertion {
                max: Some(100),
                min: None,
            }),
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, "ok");
        let failures = check_assertions(&assertions, &output, 200);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "duration_ms");
    }

    #[test]
    fn assert_duration_max_passes() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: Some(DurationAssertion {
                max: Some(500),
                min: None,
            }),
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(false, "ok");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    // --- Error code / message assertion tests ---

    #[test]
    fn assert_error_code_passes() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: Some(-32602),
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32602,"message":"Invalid params"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_error_code_fails_wrong_code() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: Some(-32602),
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32603,"message":"Internal error"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "error.code");
        assert!(failures[0].actual.contains("-32603"));
    }

    #[test]
    fn assert_error_code_fails_no_code_in_text() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: Some(-32602),
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, "plain text error");
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "error.code");
        assert!(failures[0].actual.contains("no error code"));
    }

    #[test]
    fn assert_error_code_in_passes() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: Some(vec![-32602, -32603]),
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32603,"message":"Internal error"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_error_code_in_fails() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: Some(vec![-32602, -32603]),
            error_message: None,
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32600,"message":"Invalid Request"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "error.code_in");
    }

    #[test]
    fn assert_error_message_extracts_from_json() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: Some("Invalid params".into()),
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32602,"message":"Invalid params"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_error_message_falls_back_to_raw_text() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: Some("something went wrong".into()),
            error_message_contains: None,
            result_matches: None,
        };
        let output = make_output(true, "something went wrong");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_error_message_contains_extracts_from_json() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: Some("Invalid".into()),
            result_matches: None,
        };
        let output = make_output(true, r#"{"code":-32602,"message":"Invalid params"}"#);
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    // --- result.matches regex assertion tests ---

    #[test]
    fn assert_result_matches_passes() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: Some(r#"^\d{3}-\d{4}$"#.into()),
        };
        let output = make_output(false, "123-4567");
        let failures = check_assertions(&assertions, &output, 100);
        assert!(failures.is_empty());
    }

    #[test]
    fn assert_result_matches_fails() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: Some(r#"^\d{3}-\d{4}$"#.into()),
        };
        let output = make_output(false, "not-a-number");
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].assertion, "result.matches");
    }

    #[test]
    fn assert_result_matches_invalid_regex() {
        let assertions = TestAssertions {
            status: None,
            result_type: None,
            result_has_field: None,
            result_contains: None,
            result_equals: None,
            result_length: None,
            duration_ms: None,
            error_code: None,
            error_code_in: None,
            error_message: None,
            error_message_contains: None,
            result_matches: Some("[invalid".into()),
        };
        let output = make_output(false, "anything");
        let failures = check_assertions(&assertions, &output, 100);
        assert_eq!(failures.len(), 1);
        assert!(failures[0].actual.contains("invalid regex"));
    }

    #[test]
    fn parse_result_matches_from_yaml() {
        let yaml = r#"
server:
  command: "test"
tests:
  - name: "t1"
    tool: "echo"
    assert:
      result.matches: "^hello\\s+world$"
"#;
        let tf: TestFile = serde_yaml::from_str(yaml).unwrap();
        let a = tf.tests[0].assertions.as_ref().unwrap();
        assert_eq!(a.result_matches.as_deref(), Some("^hello\\s+world$"));
    }

    // --- Utility function tests ---

    #[test]
    fn resolve_dot_path_simple() {
        let v = serde_json::json!({"name": "alice", "age": 30});
        assert_eq!(
            resolve_dot_path(&v, "name"),
            Some(serde_json::json!("alice"))
        );
        assert_eq!(resolve_dot_path(&v, "age"), Some(serde_json::json!(30)));
        assert_eq!(resolve_dot_path(&v, "missing"), None);
    }

    #[test]
    fn resolve_dot_path_nested() {
        let v = serde_json::json!({"user": {"address": {"city": "NYC"}}});
        assert_eq!(
            resolve_dot_path(&v, "user.address.city"),
            Some(serde_json::json!("NYC"))
        );
    }

    #[test]
    fn resolve_dot_path_array_index() {
        let v = serde_json::json!({"items": [{"id": 1}, {"id": 2}]});
        assert_eq!(
            resolve_dot_path(&v, "items[0].id"),
            Some(serde_json::json!(1))
        );
        assert_eq!(
            resolve_dot_path(&v, "items[1].id"),
            Some(serde_json::json!(2))
        );
    }

    #[test]
    fn extract_json_path_root() {
        let v = serde_json::json!({"id": 42});
        assert_eq!(extract_json_path(&v, "$"), Some(v.clone()));
    }

    #[test]
    fn extract_json_path_field() {
        let v = serde_json::json!({"id": 42});
        assert_eq!(extract_json_path(&v, "$.id"), Some(serde_json::json!(42)));
    }

    #[test]
    fn resolve_variables_replaces_full() {
        let mut captures = HashMap::new();
        captures.insert("user_id".to_string(), serde_json::json!(42));
        let input = serde_json::json!({"id": "${user_id}"});
        let resolved = resolve_variables(&input, &captures);
        assert_eq!(resolved, serde_json::json!({"id": 42}));
    }

    #[test]
    fn resolve_variables_replaces_inline() {
        let mut captures = HashMap::new();
        captures.insert("name".to_string(), serde_json::json!("alice"));
        let input = serde_json::json!({"greeting": "hello ${name}!"});
        let resolved = resolve_variables(&input, &captures);
        assert_eq!(resolved, serde_json::json!({"greeting": "hello alice!"}));
    }

    #[test]
    fn resolve_variables_preserves_non_string() {
        let captures = HashMap::new();
        let input = serde_json::json!({"count": 5, "active": true});
        let resolved = resolve_variables(&input, &captures);
        assert_eq!(resolved, input);
    }

    #[test]
    fn json_type_name_all_types() {
        assert_eq!(json_type_name(&serde_json::json!(null)), "null");
        assert_eq!(json_type_name(&serde_json::json!(true)), "boolean");
        assert_eq!(json_type_name(&serde_json::json!(42)), "number");
        assert_eq!(json_type_name(&serde_json::json!("hello")), "string");
        assert_eq!(json_type_name(&serde_json::json!([1, 2])), "array");
        assert_eq!(json_type_name(&serde_json::json!({"a": 1})), "object");
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        assert_eq!(truncate_string("hello world", 5), "hello...");
    }

    #[test]
    fn format_summary_human_basic() {
        let summary = TestSummary {
            suite: "test-server".into(),
            server: "node server.js (stdio)".into(),
            transport: "stdio".into(),
            passed: 2,
            failed: 1,
            skipped: 0,
            duration_ms: 500,
            tests: vec![
                TestResult {
                    name: "test one".into(),
                    status: TestStatus::Passed,
                    duration_ms: 100,
                    operation: "tool:echo".into(),
                    failures: vec![],
                },
                TestResult {
                    name: "test two".into(),
                    status: TestStatus::Failed,
                    duration_ms: 200,
                    operation: "tool:search".into(),
                    failures: vec![AssertionFailure {
                        assertion: "status".into(),
                        expected: "success".into(),
                        actual: "error".into(),
                    }],
                },
            ],
        };
        let out = format_summary_human(&summary);
        assert!(out.contains("MCP Test Suite: test-server"));
        assert!(out.contains("[PASS]"));
        assert!(out.contains("[FAIL]"));
        assert!(out.contains("2 passed, 1 failed, 0 skipped"));
    }

    #[test]
    fn format_summary_json_roundtrips() {
        let summary = TestSummary {
            suite: "s".into(),
            server: "cmd".into(),
            transport: "stdio".into(),
            passed: 1,
            failed: 0,
            skipped: 0,
            duration_ms: 100,
            tests: vec![],
        };
        let json = format_summary_json(&summary);
        assert_eq!(json["suite"], "s");
        assert_eq!(json["passed"], 1);
    }
}
