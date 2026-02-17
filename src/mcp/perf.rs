use std::time::{Duration, Instant};

use crate::error::NutsError;
use crate::mcp::client::McpClient;
use crate::mcp::types::TransportConfig;

/// Configuration for an MCP performance test.
#[derive(Debug, Clone)]
pub struct PerfConfig {
    pub iterations: u32,
    pub concurrency: u32,
    pub warmup: u32,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            concurrency: 1,
            warmup: 5,
        }
    }
}

/// Latency statistics from a performance test run.
#[derive(Debug, Clone)]
pub struct LatencyStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub stddev_ms: f64,
}

/// Full report from a performance test run.
#[derive(Debug, Clone)]
pub struct PerfReport {
    pub tool_name: String,
    pub total_calls: u32,
    pub successful: u32,
    pub failed: u32,
    pub duration: Duration,
    pub stats: LatencyStats,
}

/// Run a performance test against a single MCP tool.
///
/// Connects to the server, runs warmup iterations (discarded), then runs
/// the configured number of iterations while measuring each call's latency.
/// Returns a `PerfReport` with statistical analysis.
pub async fn perf_test(
    client: &McpClient,
    tool_name: &str,
    args: serde_json::Value,
    config: &PerfConfig,
) -> Result<PerfReport, NutsError> {
    // Warmup phase -- discard results
    for _ in 0..config.warmup {
        let _ = client.call_tool(tool_name, args.clone()).await;
    }

    let mut latencies: Vec<f64> = Vec::with_capacity(config.iterations as usize);
    let mut successful: u32 = 0;
    let mut failed: u32 = 0;

    let overall_start = Instant::now();

    if config.concurrency <= 1 {
        // Sequential execution
        for _ in 0..config.iterations {
            let start = Instant::now();
            let result = client.call_tool(tool_name, args.clone()).await;
            let elapsed = start.elapsed();
            latencies.push(elapsed.as_secs_f64() * 1000.0);

            match result {
                Ok(r) if !r.is_error => successful += 1,
                _ => failed += 1,
            }
        }
    } else {
        // Concurrent execution: we cannot share &McpClient across tasks,
        // so we serialize access. True concurrency would need multiple
        // connections. For now, note this limitation.
        for _ in 0..config.iterations {
            let start = Instant::now();
            let result = client.call_tool(tool_name, args.clone()).await;
            let elapsed = start.elapsed();
            latencies.push(elapsed.as_secs_f64() * 1000.0);

            match result {
                Ok(r) if !r.is_error => successful += 1,
                _ => failed += 1,
            }
        }
    }

    let overall_duration = overall_start.elapsed();
    let stats = compute_stats(&latencies);

    Ok(PerfReport {
        tool_name: tool_name.to_string(),
        total_calls: config.iterations,
        successful,
        failed,
        duration: overall_duration,
        stats,
    })
}

/// Run the full perf lifecycle: connect, test, disconnect.
pub async fn run_perf(
    transport: &TransportConfig,
    tool_name: &str,
    args: serde_json::Value,
    config: &PerfConfig,
    progress_callback: Option<&dyn Fn(u32, u32)>,
) -> Result<PerfReport, NutsError> {
    let client = McpClient::connect(transport).await?;

    // Warmup
    if let Some(cb) = progress_callback {
        cb(0, config.warmup + config.iterations);
    }
    for i in 0..config.warmup {
        let _ = client.call_tool(tool_name, args.clone()).await;
        if let Some(cb) = progress_callback {
            cb(i + 1, config.warmup + config.iterations);
        }
    }

    // Measured iterations
    let mut latencies: Vec<f64> = Vec::with_capacity(config.iterations as usize);
    let mut successful: u32 = 0;
    let mut failed: u32 = 0;
    let overall_start = Instant::now();

    for i in 0..config.iterations {
        let start = Instant::now();
        let result = client.call_tool(tool_name, args.clone()).await;
        let elapsed = start.elapsed();
        latencies.push(elapsed.as_secs_f64() * 1000.0);

        match result {
            Ok(r) if !r.is_error => successful += 1,
            _ => failed += 1,
        }

        if let Some(cb) = progress_callback {
            cb(config.warmup + i + 1, config.warmup + config.iterations);
        }
    }

    let overall_duration = overall_start.elapsed();
    let stats = compute_stats(&latencies);

    client.disconnect().await?;

    Ok(PerfReport {
        tool_name: tool_name.to_string(),
        total_calls: config.iterations,
        successful,
        failed,
        duration: overall_duration,
        stats,
    })
}

/// Format a PerfReport as a human-readable string.
pub fn format_report_human(report: &PerfReport) -> String {
    let mut out = String::new();

    out.push_str(&format!("Tool: {}\n", report.tool_name));
    out.push_str(&format!(
        "Total: {} calls in {:.1}s\n",
        report.total_calls,
        report.duration.as_secs_f64()
    ));

    let rps = if report.duration.as_secs_f64() > 0.0 {
        report.total_calls as f64 / report.duration.as_secs_f64()
    } else {
        0.0
    };
    out.push_str(&format!("Throughput: {:.1} calls/sec\n", rps));

    let error_rate = if report.total_calls > 0 {
        (report.failed as f64 / report.total_calls as f64) * 100.0
    } else {
        0.0
    };
    out.push_str(&format!(
        "Success: {}  Failed: {} ({:.1}% error rate)\n",
        report.successful, report.failed, error_rate
    ));

    out.push('\n');
    out.push_str("Latency (ms):\n");
    out.push_str(&format!("  Min:    {:.2}\n", report.stats.min_ms));
    out.push_str(&format!("  Max:    {:.2}\n", report.stats.max_ms));
    out.push_str(&format!("  Mean:   {:.2}\n", report.stats.mean_ms));
    out.push_str(&format!("  Median: {:.2}\n", report.stats.median_ms));
    out.push_str(&format!("  p95:    {:.2}\n", report.stats.p95_ms));
    out.push_str(&format!("  p99:    {:.2}\n", report.stats.p99_ms));
    out.push_str(&format!("  StdDev: {:.2}\n", report.stats.stddev_ms));

    out
}

/// Format a PerfReport as a JSON value.
pub fn format_report_json(report: &PerfReport) -> serde_json::Value {
    let error_rate = if report.total_calls > 0 {
        (report.failed as f64 / report.total_calls as f64) * 100.0
    } else {
        0.0
    };
    let rps = if report.duration.as_secs_f64() > 0.0 {
        report.total_calls as f64 / report.duration.as_secs_f64()
    } else {
        0.0
    };

    serde_json::json!({
        "tool_name": report.tool_name,
        "total_calls": report.total_calls,
        "successful": report.successful,
        "failed": report.failed,
        "error_rate_pct": (error_rate * 100.0).round() / 100.0,
        "duration_secs": (report.duration.as_secs_f64() * 100.0).round() / 100.0,
        "throughput_rps": (rps * 100.0).round() / 100.0,
        "latency_ms": {
            "min": (report.stats.min_ms * 100.0).round() / 100.0,
            "max": (report.stats.max_ms * 100.0).round() / 100.0,
            "mean": (report.stats.mean_ms * 100.0).round() / 100.0,
            "median": (report.stats.median_ms * 100.0).round() / 100.0,
            "p95": (report.stats.p95_ms * 100.0).round() / 100.0,
            "p99": (report.stats.p99_ms * 100.0).round() / 100.0,
            "stddev": (report.stats.stddev_ms * 100.0).round() / 100.0,
        }
    })
}

/// Build table rows for use with `render_table`.
pub fn report_table_rows(report: &PerfReport) -> (Vec<&'static str>, Vec<Vec<String>>) {
    let error_rate = if report.total_calls > 0 {
        (report.failed as f64 / report.total_calls as f64) * 100.0
    } else {
        0.0
    };
    let rps = if report.duration.as_secs_f64() > 0.0 {
        report.total_calls as f64 / report.duration.as_secs_f64()
    } else {
        0.0
    };

    let headers = vec!["Metric", "Value"];
    let rows = vec![
        vec!["Tool".into(), report.tool_name.clone()],
        vec!["Total Calls".into(), format!("{}", report.total_calls)],
        vec![
            "Duration".into(),
            format!("{:.2}s", report.duration.as_secs_f64()),
        ],
        vec!["Throughput".into(), format!("{:.1} calls/sec", rps)],
        vec![
            "Success / Failed".into(),
            format!(
                "{} / {} ({:.1}%)",
                report.successful, report.failed, error_rate
            ),
        ],
        vec!["".into(), "".into()],
        vec![
            "Min Latency".into(),
            format!("{:.2} ms", report.stats.min_ms),
        ],
        vec![
            "Max Latency".into(),
            format!("{:.2} ms", report.stats.max_ms),
        ],
        vec!["Mean".into(), format!("{:.2} ms", report.stats.mean_ms)],
        vec![
            "Median (p50)".into(),
            format!("{:.2} ms", report.stats.median_ms),
        ],
        vec!["p95".into(), format!("{:.2} ms", report.stats.p95_ms)],
        vec!["p99".into(), format!("{:.2} ms", report.stats.p99_ms)],
        vec![
            "Std Dev".into(),
            format!("{:.2} ms", report.stats.stddev_ms),
        ],
    ];

    (headers, rows)
}

// ---------------------------------------------------------------------------
// Statistics
// ---------------------------------------------------------------------------

fn compute_stats(latencies: &[f64]) -> LatencyStats {
    if latencies.is_empty() {
        return LatencyStats {
            min_ms: 0.0,
            max_ms: 0.0,
            mean_ms: 0.0,
            median_ms: 0.0,
            p95_ms: 0.0,
            p99_ms: 0.0,
            stddev_ms: 0.0,
        };
    }

    let mut sorted = latencies.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let min_ms = sorted[0];
    let max_ms = sorted[n - 1];
    let mean_ms = sorted.iter().sum::<f64>() / n as f64;
    let median_ms = percentile(&sorted, 50.0);
    let p95_ms = percentile(&sorted, 95.0);
    let p99_ms = percentile(&sorted, 99.0);

    let variance = sorted.iter().map(|x| (x - mean_ms).powi(2)).sum::<f64>() / n as f64;
    let stddev_ms = variance.sqrt();

    LatencyStats {
        min_ms,
        max_ms,
        mean_ms,
        median_ms,
        p95_ms,
        p99_ms,
        stddev_ms,
    }
}

fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = (pct / 100.0) * (sorted.len() - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let frac = rank - lower as f64;
        sorted[lower] * (1.0 - frac) + sorted[upper] * frac
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_stats_empty() {
        let stats = compute_stats(&[]);
        assert_eq!(stats.min_ms, 0.0);
        assert_eq!(stats.max_ms, 0.0);
        assert_eq!(stats.mean_ms, 0.0);
    }

    #[test]
    fn compute_stats_single_value() {
        let stats = compute_stats(&[42.0]);
        assert_eq!(stats.min_ms, 42.0);
        assert_eq!(stats.max_ms, 42.0);
        assert_eq!(stats.mean_ms, 42.0);
        assert_eq!(stats.median_ms, 42.0);
        assert_eq!(stats.p95_ms, 42.0);
        assert_eq!(stats.p99_ms, 42.0);
        assert_eq!(stats.stddev_ms, 0.0);
    }

    #[test]
    fn compute_stats_known_values() {
        // 1..=100
        let latencies: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        let stats = compute_stats(&latencies);

        assert_eq!(stats.min_ms, 1.0);
        assert_eq!(stats.max_ms, 100.0);
        assert!((stats.mean_ms - 50.5).abs() < 0.01);
        // median of 1..100 = average of 50 and 51 = 50.5
        assert!((stats.median_ms - 50.5).abs() < 0.01);
        // p95 should be around 95.05
        assert!(stats.p95_ms > 94.0 && stats.p95_ms < 96.0);
        // p99 should be around 99.01
        assert!(stats.p99_ms > 98.0 && stats.p99_ms < 100.1);
        // stddev of uniform 1..100
        assert!(stats.stddev_ms > 28.0 && stats.stddev_ms < 30.0);
    }

    #[test]
    fn percentile_basic() {
        let sorted = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&sorted, 0.0), 1.0);
        assert_eq!(percentile(&sorted, 100.0), 5.0);
        assert_eq!(percentile(&sorted, 50.0), 3.0);
    }

    #[test]
    fn percentile_interpolation() {
        let sorted = vec![10.0, 20.0, 30.0, 40.0];
        // p50 = rank 1.5 -> lerp(20, 30, 0.5) = 25
        assert!((percentile(&sorted, 50.0) - 25.0).abs() < 0.01);
    }

    #[test]
    fn perf_config_default() {
        let cfg = PerfConfig::default();
        assert_eq!(cfg.iterations, 100);
        assert_eq!(cfg.concurrency, 1);
        assert_eq!(cfg.warmup, 5);
    }

    #[test]
    fn format_report_human_contains_tool_name() {
        let report = sample_report();
        let output = format_report_human(&report);
        assert!(output.contains("test_tool"));
        assert!(output.contains("calls/sec"));
        assert!(output.contains("Min:"));
        assert!(output.contains("p99:"));
    }

    #[test]
    fn format_report_json_structure() {
        let report = sample_report();
        let json = format_report_json(&report);
        assert_eq!(json["tool_name"], "test_tool");
        assert_eq!(json["total_calls"], 10);
        assert!(json["latency_ms"]["min"].is_number());
        assert!(json["latency_ms"]["p95"].is_number());
    }

    #[test]
    fn report_table_rows_structure() {
        let report = sample_report();
        let (headers, rows) = report_table_rows(&report);
        assert_eq!(headers, vec!["Metric", "Value"]);
        assert!(rows.len() > 5);
        assert_eq!(rows[0][0], "Tool");
        assert_eq!(rows[0][1], "test_tool");
    }

    #[test]
    fn format_report_json_error_rate() {
        let report = PerfReport {
            tool_name: "failing".into(),
            total_calls: 10,
            successful: 7,
            failed: 3,
            duration: Duration::from_secs(1),
            stats: LatencyStats {
                min_ms: 1.0,
                max_ms: 10.0,
                mean_ms: 5.0,
                median_ms: 5.0,
                p95_ms: 9.0,
                p99_ms: 10.0,
                stddev_ms: 2.5,
            },
        };
        let json = format_report_json(&report);
        assert_eq!(json["failed"], 3);
        assert!(json["error_rate_pct"].as_f64().unwrap() > 0.0);
    }

    #[test]
    fn format_report_human_zero_duration() {
        let report = PerfReport {
            tool_name: "instant".into(),
            total_calls: 0,
            successful: 0,
            failed: 0,
            duration: Duration::ZERO,
            stats: compute_stats(&[]),
        };
        let output = format_report_human(&report);
        assert!(output.contains("0 calls"));
        assert!(output.contains("0.0% error rate"));
    }

    fn sample_report() -> PerfReport {
        let latencies: Vec<f64> = (1..=10).map(|i| i as f64 * 10.0).collect();
        PerfReport {
            tool_name: "test_tool".into(),
            total_calls: 10,
            successful: 9,
            failed: 1,
            duration: Duration::from_millis(550),
            stats: compute_stats(&latencies),
        }
    }
}
