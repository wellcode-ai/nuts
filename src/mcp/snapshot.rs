use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::NutsError;
use crate::mcp::client::McpClient;
use crate::mcp::types::ToolResult;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A captured snapshot of all tool outputs from an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub server_name: String,
    pub server_version: String,
    pub captured_at: String,
    pub tool_results: Vec<ToolSnapshot>,
}

/// A single tool's captured output within a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSnapshot {
    pub tool_name: String,
    pub input: serde_json::Value,
    pub output: ToolResult,
    pub duration_ms: u64,
}

/// A single difference found during comparison.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub tool_name: String,
    pub field: String,
    pub expected: String,
    pub actual: String,
}

/// Summary of comparing two snapshots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    pub matched: usize,
    pub changed: usize,
    pub added: usize,
    pub removed: usize,
    pub diffs: Vec<SnapshotDiff>,
}

// ---------------------------------------------------------------------------
// Capture
// ---------------------------------------------------------------------------

/// Connect to an MCP server, discover tools, call each with empty args,
/// and record the outputs into a Snapshot.
pub async fn capture_snapshot(client: &McpClient) -> Result<Snapshot, NutsError> {
    let caps = client.discover().await?;
    let captured_at = chrono::Utc::now().to_rfc3339();

    let mut tool_results = Vec::new();

    for tool in &caps.tools {
        let input = serde_json::json!({});
        let start = Instant::now();
        let output = client.call_tool(&tool.name, input.clone()).await;
        let duration_ms = start.elapsed().as_millis() as u64;

        let result = match output {
            Ok(r) => r,
            Err(e) => ToolResult {
                is_error: true,
                content: vec![crate::mcp::types::ContentItem::Text {
                    text: format!("Error: {e}"),
                }],
            },
        };

        tool_results.push(ToolSnapshot {
            tool_name: tool.name.clone(),
            input,
            output: result,
            duration_ms,
        });
    }

    Ok(Snapshot {
        server_name: caps.server_name,
        server_version: caps.server_version,
        captured_at,
        tool_results,
    })
}

// ---------------------------------------------------------------------------
// Save / Load
// ---------------------------------------------------------------------------

/// Serialize a snapshot to a JSON file.
pub fn save_snapshot(snapshot: &Snapshot, path: &str) -> Result<(), NutsError> {
    let json = serde_json::to_string_pretty(snapshot)?;
    std::fs::write(path, json).map_err(|e| NutsError::Mcp {
        message: format!("failed to write snapshot to '{path}': {e}"),
    })
}

/// Deserialize a snapshot from a JSON file.
pub fn load_snapshot(path: &str) -> Result<Snapshot, NutsError> {
    let data = std::fs::read_to_string(path).map_err(|e| NutsError::Mcp {
        message: format!("failed to read snapshot from '{path}': {e}"),
    })?;
    let snapshot: Snapshot = serde_json::from_str(&data)?;
    Ok(snapshot)
}

// ---------------------------------------------------------------------------
// Compare
// ---------------------------------------------------------------------------

/// Compare a baseline snapshot against a current snapshot.
///
/// Matches tools by name, reports additions, removals, and content changes.
pub fn compare_snapshots(baseline: &Snapshot, current: &Snapshot) -> CompareResult {
    let mut matched = 0usize;
    let mut changed = 0usize;
    let mut diffs = Vec::new();

    // Index current tools by name for lookup
    let current_map: std::collections::HashMap<&str, &ToolSnapshot> = current
        .tool_results
        .iter()
        .map(|t| (t.tool_name.as_str(), t))
        .collect();

    let baseline_names: std::collections::HashSet<&str> = baseline
        .tool_results
        .iter()
        .map(|t| t.tool_name.as_str())
        .collect();

    let current_names: std::collections::HashSet<&str> = current
        .tool_results
        .iter()
        .map(|t| t.tool_name.as_str())
        .collect();

    // Tools removed (in baseline, not in current)
    let removed_names: Vec<&&str> = baseline_names.difference(&current_names).collect();
    let removed = removed_names.len();
    for name in &removed_names {
        diffs.push(SnapshotDiff {
            tool_name: name.to_string(),
            field: "tool".into(),
            expected: "present".into(),
            actual: "removed".into(),
        });
    }

    // Tools added (in current, not in baseline)
    let added_names: Vec<&&str> = current_names.difference(&baseline_names).collect();
    let added = added_names.len();
    for name in &added_names {
        diffs.push(SnapshotDiff {
            tool_name: name.to_string(),
            field: "tool".into(),
            expected: "absent".into(),
            actual: "added".into(),
        });
    }

    // Compare tools present in both
    for baseline_tool in &baseline.tool_results {
        if let Some(current_tool) = current_map.get(baseline_tool.tool_name.as_str()) {
            let tool_diffs = diff_tool_outputs(baseline_tool, current_tool);
            if tool_diffs.is_empty() {
                matched += 1;
            } else {
                changed += 1;
                diffs.extend(tool_diffs);
            }
        }
    }

    CompareResult {
        matched,
        changed,
        added,
        removed,
        diffs,
    }
}

/// Compare two tool snapshots and return a list of differences.
fn diff_tool_outputs(baseline: &ToolSnapshot, current: &ToolSnapshot) -> Vec<SnapshotDiff> {
    let mut diffs = Vec::new();
    let name = &baseline.tool_name;

    // Compare is_error
    if baseline.output.is_error != current.output.is_error {
        diffs.push(SnapshotDiff {
            tool_name: name.clone(),
            field: "is_error".into(),
            expected: baseline.output.is_error.to_string(),
            actual: current.output.is_error.to_string(),
        });
    }

    // Compare content length
    if baseline.output.content.len() != current.output.content.len() {
        diffs.push(SnapshotDiff {
            tool_name: name.clone(),
            field: "content.length".into(),
            expected: baseline.output.content.len().to_string(),
            actual: current.output.content.len().to_string(),
        });
        return diffs;
    }

    // Compare each content item
    for (i, (b, c)) in baseline
        .output
        .content
        .iter()
        .zip(current.output.content.iter())
        .enumerate()
    {
        let b_json = serde_json::to_string(b).unwrap_or_default();
        let c_json = serde_json::to_string(c).unwrap_or_default();

        if b_json != c_json {
            diffs.push(SnapshotDiff {
                tool_name: name.clone(),
                field: format!("content[{}]", i),
                expected: truncate(&b_json, 200),
                actual: truncate(&c_json, 200),
            });
        }
    }

    diffs
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Format a CompareResult as human-readable text.
pub fn format_compare_human(result: &CompareResult) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "Snapshot Comparison: {} matched, {} changed, {} added, {} removed\n",
        result.matched, result.changed, result.added, result.removed
    ));

    if result.diffs.is_empty() {
        out.push_str("\nNo differences found.\n");
    } else {
        out.push('\n');
        for diff in &result.diffs {
            out.push_str(&format!("  {} [{}]\n", diff.tool_name, diff.field));
            out.push_str(&format!("    expected: {}\n", diff.expected));
            out.push_str(&format!("    actual:   {}\n", diff.actual));
        }
    }

    out
}

/// Format a CompareResult as JSON.
pub fn format_compare_json(result: &CompareResult) -> serde_json::Value {
    serde_json::to_value(result).unwrap_or_else(|_| serde_json::json!({}))
}

/// Format a captured Snapshot summary for display.
pub fn format_capture_human(snapshot: &Snapshot) -> String {
    let mut out = String::new();

    out.push_str(&format!("Server: {} v{}\n", snapshot.server_name, snapshot.server_version));
    out.push_str(&format!("Captured: {}\n", snapshot.captured_at));
    out.push_str(&format!("Tools captured: {}\n\n", snapshot.tool_results.len()));

    for ts in &snapshot.tool_results {
        let status = if ts.output.is_error { "ERROR" } else { "OK" };
        out.push_str(&format!(
            "  {:<30} {}  ({}ms)\n",
            ts.tool_name, status, ts.duration_ms
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::{ContentItem, ToolResult};

    fn make_tool_snapshot(name: &str, text: &str, is_error: bool) -> ToolSnapshot {
        ToolSnapshot {
            tool_name: name.into(),
            input: serde_json::json!({}),
            output: ToolResult {
                is_error,
                content: vec![ContentItem::Text { text: text.into() }],
            },
            duration_ms: 42,
        }
    }

    fn make_snapshot(tools: Vec<ToolSnapshot>) -> Snapshot {
        Snapshot {
            server_name: "test-server".into(),
            server_version: "1.0.0".into(),
            captured_at: "2026-01-01T00:00:00Z".into(),
            tool_results: tools,
        }
    }

    #[test]
    fn compare_identical_snapshots() {
        let baseline = make_snapshot(vec![
            make_tool_snapshot("echo", "hello", false),
            make_tool_snapshot("add", "3", false),
        ]);
        let current = baseline.clone();
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.matched, 2);
        assert_eq!(result.changed, 0);
        assert_eq!(result.added, 0);
        assert_eq!(result.removed, 0);
        assert!(result.diffs.is_empty());
    }

    #[test]
    fn compare_detects_changed_content() {
        let baseline = make_snapshot(vec![make_tool_snapshot("echo", "hello", false)]);
        let current = make_snapshot(vec![make_tool_snapshot("echo", "goodbye", false)]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.matched, 0);
        assert_eq!(result.changed, 1);
        assert_eq!(result.diffs.len(), 1);
        assert_eq!(result.diffs[0].tool_name, "echo");
        assert_eq!(result.diffs[0].field, "content[0]");
    }

    #[test]
    fn compare_detects_error_status_change() {
        let baseline = make_snapshot(vec![make_tool_snapshot("echo", "hello", false)]);
        let current = make_snapshot(vec![make_tool_snapshot("echo", "hello", true)]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.changed, 1);
        let error_diff = result
            .diffs
            .iter()
            .find(|d| d.field == "is_error")
            .unwrap();
        assert_eq!(error_diff.expected, "false");
        assert_eq!(error_diff.actual, "true");
    }

    #[test]
    fn compare_detects_added_tools() {
        let baseline = make_snapshot(vec![make_tool_snapshot("echo", "hi", false)]);
        let current = make_snapshot(vec![
            make_tool_snapshot("echo", "hi", false),
            make_tool_snapshot("new_tool", "data", false),
        ]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.matched, 1);
        assert_eq!(result.added, 1);
        let added_diff = result
            .diffs
            .iter()
            .find(|d| d.tool_name == "new_tool")
            .unwrap();
        assert_eq!(added_diff.actual, "added");
    }

    #[test]
    fn compare_detects_removed_tools() {
        let baseline = make_snapshot(vec![
            make_tool_snapshot("echo", "hi", false),
            make_tool_snapshot("old_tool", "data", false),
        ]);
        let current = make_snapshot(vec![make_tool_snapshot("echo", "hi", false)]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.matched, 1);
        assert_eq!(result.removed, 1);
        let removed_diff = result
            .diffs
            .iter()
            .find(|d| d.tool_name == "old_tool")
            .unwrap();
        assert_eq!(removed_diff.actual, "removed");
    }

    #[test]
    fn compare_detects_content_length_change() {
        let baseline = make_snapshot(vec![ToolSnapshot {
            tool_name: "multi".into(),
            input: serde_json::json!({}),
            output: ToolResult {
                is_error: false,
                content: vec![
                    ContentItem::Text {
                        text: "one".into(),
                    },
                    ContentItem::Text {
                        text: "two".into(),
                    },
                ],
            },
            duration_ms: 10,
        }]);
        let current = make_snapshot(vec![make_tool_snapshot("multi", "one", false)]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.changed, 1);
        let len_diff = result
            .diffs
            .iter()
            .find(|d| d.field == "content.length")
            .unwrap();
        assert_eq!(len_diff.expected, "2");
        assert_eq!(len_diff.actual, "1");
    }

    #[test]
    fn compare_empty_snapshots() {
        let baseline = make_snapshot(vec![]);
        let current = make_snapshot(vec![]);
        let result = compare_snapshots(&baseline, &current);

        assert_eq!(result.matched, 0);
        assert_eq!(result.changed, 0);
        assert_eq!(result.added, 0);
        assert_eq!(result.removed, 0);
        assert!(result.diffs.is_empty());
    }

    #[test]
    fn format_compare_human_no_diffs() {
        let result = CompareResult {
            matched: 3,
            changed: 0,
            added: 0,
            removed: 0,
            diffs: vec![],
        };
        let output = format_compare_human(&result);
        assert!(output.contains("3 matched"));
        assert!(output.contains("No differences found"));
    }

    #[test]
    fn format_compare_human_with_diffs() {
        let result = CompareResult {
            matched: 1,
            changed: 1,
            added: 0,
            removed: 0,
            diffs: vec![SnapshotDiff {
                tool_name: "echo".into(),
                field: "content[0]".into(),
                expected: "hello".into(),
                actual: "goodbye".into(),
            }],
        };
        let output = format_compare_human(&result);
        assert!(output.contains("1 changed"));
        assert!(output.contains("echo"));
        assert!(output.contains("expected: hello"));
        assert!(output.contains("actual:   goodbye"));
    }

    #[test]
    fn format_compare_json_structure() {
        let result = CompareResult {
            matched: 2,
            changed: 1,
            added: 0,
            removed: 0,
            diffs: vec![SnapshotDiff {
                tool_name: "test".into(),
                field: "is_error".into(),
                expected: "false".into(),
                actual: "true".into(),
            }],
        };
        let json = format_compare_json(&result);
        assert_eq!(json["matched"], 2);
        assert_eq!(json["changed"], 1);
        assert_eq!(json["diffs"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn format_capture_human_lists_tools() {
        let snapshot = make_snapshot(vec![
            make_tool_snapshot("echo", "hi", false),
            make_tool_snapshot("broken", "err", true),
        ]);
        let output = format_capture_human(&snapshot);
        assert!(output.contains("test-server"));
        assert!(output.contains("Tools captured: 2"));
        assert!(output.contains("echo"));
        assert!(output.contains("OK"));
        assert!(output.contains("broken"));
        assert!(output.contains("ERROR"));
    }

    #[test]
    fn snapshot_roundtrip_json() {
        let snapshot = make_snapshot(vec![make_tool_snapshot("echo", "hello", false)]);
        let json = serde_json::to_string(&snapshot).unwrap();
        let parsed: Snapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.server_name, "test-server");
        assert_eq!(parsed.tool_results.len(), 1);
        assert_eq!(parsed.tool_results[0].tool_name, "echo");
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(300);
        let result = truncate(&long, 200);
        assert_eq!(result.len(), 203); // 200 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn save_and_load_snapshot() {
        let snapshot = make_snapshot(vec![make_tool_snapshot("echo", "hello", false)]);
        let path = std::env::temp_dir().join("nuts_snapshot_test.json");
        let path_str = path.to_str().unwrap();

        save_snapshot(&snapshot, path_str).unwrap();
        let loaded = load_snapshot(path_str).unwrap();

        assert_eq!(loaded.server_name, "test-server");
        assert_eq!(loaded.tool_results.len(), 1);
        assert_eq!(loaded.tool_results[0].tool_name, "echo");

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_snapshot_missing_file() {
        let result = load_snapshot("/tmp/nonexistent_snapshot_12345.json");
        assert!(result.is_err());
    }
}
