use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize, AtomicU64};
use std::error::Error;
use std::io::Write;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Chart, Dataset, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Color, Style},
    Terminal,
    prelude::Marker,
};
use statistical::{mean, standard_deviation, median};
use std::collections::HashMap;
use std::sync::Mutex;
use crossterm::{
    terminal::{Clear, ClearType},
    ExecutableCommand,
};

use crate::models::metrics::{Metrics, RequestMetric};

#[derive(Default)]
struct PerfMetrics {
    requests: Vec<usize>,
    latencies: Vec<f64>,
    timestamps: Vec<f64>,
    errors: Vec<usize>,
    status_codes: HashMap<u16, usize>,
}

pub struct PerfCommand {
    client: Client,
}

impl PerfCommand {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn run(&self, url: &str, users: u32, duration: Duration, method: &str, body: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting performance test");
        println!("URL: {}", url);
        println!("Method: {}", method);
        println!("Users: {}", users);
        println!("Duration: {}s", duration.as_secs());
        if let Some(body) = body {
            println!("Body: {}", body);
        }

        let client = reqwest::Client::new();
        let start_time = std::time::Instant::now();
        let request_count = Arc::new(AtomicUsize::new(0));
        let total_latency = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicUsize::new(0));

        // Progress reporting task
        let progress_count = request_count.clone();
        let progress_errors = errors.clone();
        let progress_latency = total_latency.clone();
        let progress_handle = tokio::spawn(async move {
            while start_time.elapsed() < duration {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let reqs = progress_count.load(Ordering::Relaxed);
                let errs = progress_errors.load(Ordering::Relaxed);
                let avg_latency = if reqs > 0 {
                    progress_latency.load(Ordering::Relaxed) as f64 / reqs as f64
                } else {
                    0.0
                };
                let rps = reqs as f64 / start_time.elapsed().as_secs_f64();
                
                print!("\rRequests: {} | Errors: {} | Avg Latency: {:.1}ms | RPS: {:.1}", 
                      reqs, errs, avg_latency, rps);
                std::io::stdout().flush().unwrap();
            }
        });

        // Worker tasks
        let mut handles = vec![];
        for _ in 0..users {
            let client = client.clone();
            let url = url.to_string();
            let request_count = request_count.clone();
            let total_latency = total_latency.clone();
            let errors = errors.clone();
            let method = method.to_string();
            let body = body.map(String::from);

            let handle = tokio::spawn(async move {
                while start_time.elapsed() < duration {
                    let start = std::time::Instant::now();
                    
                    let result = match method.as_str() {
                        "POST" => {
                            let req = client.post(&url);
                            if let Some(body_content) = &body {
                                req.header("Content-Type", "application/json")
                                   .body(body_content.clone())
                                   .send()
                                   .await
                            } else {
                                req.send().await
                            }
                        },
                        _ => client.get(&url).send().await,
                    };

                    match result {
                        Ok(_) => {
                            request_count.fetch_add(1, Ordering::Relaxed);
                            let latency = start.elapsed().as_millis() as u64;
                            total_latency.fetch_add(latency, Ordering::Relaxed);
                        },
                        Err(_) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all handles to complete
        for handle in handles {
            handle.await?;
        }

        let total_requests = request_count.load(Ordering::Relaxed);
        let total_errors = errors.load(Ordering::Relaxed);
        let avg_latency = if total_requests > 0 {
            total_latency.load(Ordering::Relaxed) as f64 / total_requests as f64
        } else {
            0.0
        };

        println!("\nResults:");
        println!("Total Requests: {}", total_requests);
        println!("Successful Requests: {}", total_requests - total_errors);
        println!("Failed Requests: {}", total_errors);
        println!("Average Latency: {:.2}ms", avg_latency);
        println!("Requests/second: {:.2}", total_requests as f64 / duration.as_secs_f64());

        Ok(())
    }

    pub async fn run_with_variations(&self, url: &str, users: u32, duration: Duration, 
        method: &str, variations: &[String]) -> Result<(), Box<dyn std::error::Error>> 
    {
        let metrics = Arc::new(Mutex::new(PerfMetrics::default()));
        let start_time = Instant::now();

        println!("🚀 Starting performance test with variations");
        println!("URL: {}", url);
        println!("Method: {}", method);
        println!("Users: {}", users);
        println!("Duration: {}s", duration.as_secs());
        println!("Body variations: {}", variations.len());

        println!("\n📝 Test Variations:");
        for (i, variation) in variations.iter().enumerate() {
            println!("  {}. {}", i + 1, variation);
        }
        println!("\n🚀 Starting test...\n");

        let client = reqwest::Client::new();
        let start_time = std::time::Instant::now();
        let request_count = Arc::new(AtomicUsize::new(0));
        let total_latency = Arc::new(AtomicU64::new(0));
        let errors = Arc::new(AtomicUsize::new(0));

        let variations = Arc::new(variations.to_vec());

        // Progress reporting
        let progress_count = request_count.clone();
        let progress_errors = errors.clone();
        let progress_latency = total_latency.clone();
        
        // Replace the metrics_handle spawn with a non-async display update
        let metrics_clone = metrics.clone();
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Spawn worker tasks first
        let mut handles = vec![];
        for _ in 0..users {
            let metrics_clone = Arc::clone(&metrics);  // Clone the Arc before the worker tasks
            let client = client.clone();
            let url = url.to_string();
            let request_count = request_count.clone();
            let total_latency = total_latency.clone();
            let errors = errors.clone();
            let variations = variations.clone();

            let handle = tokio::spawn(async move {
                while start_time.elapsed() < duration {
                    let start = std::time::Instant::now();
                    
                    // Rotate through variations
                    let variation_index = request_count.load(Ordering::Relaxed) % variations.len();
                    let body = &variations[variation_index];

                    let result = client.post(&url)
                        .header("Content-Type", "application/json")
                        .body(body.clone())
                        .send()
                        .await;

                    let elapsed = start.elapsed();
                    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
                    
                    // Update metrics
                    let mut metrics = metrics_clone.lock().unwrap();
                    metrics.timestamps.push(start_time.elapsed().as_secs_f64());
                    metrics.latencies.push(elapsed_ms);
                    
                    match result {
                        Ok(response) => {
                            request_count.fetch_add(1, Ordering::Relaxed);
                            let status = response.status().as_u16();
                            *metrics.status_codes.entry(status).or_insert(0) += 1;
                            metrics.requests.push(1);
                        },
                        Err(_) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                            metrics.errors.push(1);
                            metrics.requests.push(0);
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Handle display updates on the main thread
        let stdout = std::io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 1. First, clear the terminal before starting TUI
        std::io::stdout().execute(Clear(ClearType::All))?;

        while start_time.elapsed() < duration {
            let current_metrics = metrics_clone.lock().unwrap();
            let stats = PerfStats::calculate(&current_metrics);
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(30),
                        Constraint::Percentage(30),
                        Constraint::Percentage(40),
                    ].as_ref())
                    .split(f.size());

                // Calculate RPS data points
                let window_size = 1.0; // 1-second window for RPS calculation
                let rps_data: Vec<(f64, f64)> = current_metrics.timestamps.windows(2)
                    .enumerate()
                    .map(|(i, window)| {
                        let time = window[0];
                        let requests = current_metrics.requests[i..i+1].iter().sum::<usize>() as f64;
                        let window_duration = (window[1] - window[0]).max(window_size);
                        (time, requests / window_duration)
                    })
                    .collect();

                // Collect latency data points
                let latency_data: Vec<(f64, f64)> = current_metrics.timestamps.iter()
                    .zip(current_metrics.latencies.iter())
                    .map(|(&time, &latency)| (time, latency))
                    .collect();

                // Calculate bounds for better visualization
                let max_rps = rps_data.iter().map(|(_, rps)| *rps).fold(0.0, f64::max);
                let max_latency = latency_data.iter().map(|(_, lat)| *lat).fold(0.0, f64::max);

                let rps_dataset = Dataset::default()
                    .name("RPS")
                    .marker(Marker::Dot)
                    .style(Style::default().fg(Color::Cyan))
                    .data(&rps_data);

                let latency_dataset = Dataset::default()
                    .name("Latency")
                    .marker(Marker::Dot)
                    .style(Style::default().fg(Color::Yellow))
                    .data(&latency_data);

                // RPS Chart with proper bounds
                let rps_chart = Chart::new(vec![rps_dataset])
                    .block(Block::default()
                        .title(" ⚡ Requests/sec ")
                        .borders(Borders::ALL))
                    .x_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, duration.as_secs_f64()])
                        .title("Time (s)"))
                    .y_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, max_rps * 1.2])
                        .title("RPS"));

                // Latency Chart with proper bounds
                let latency_chart = Chart::new(vec![latency_dataset])
                    .block(Block::default()
                        .title(" ⏱️  Latency (ms) ")
                        .borders(Borders::ALL))
                    .x_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, duration.as_secs_f64()])
                        .title("Time (s)"))
                    .y_axis(ratatui::widgets::Axis::default()
                        .bounds([0.0, max_latency * 1.2])
                        .title("ms"));

                f.render_widget(rps_chart, chunks[0]);
                f.render_widget(latency_chart, chunks[1]);

                // Stats Summary
                let summary = Paragraph::new(format!(
                    "\n█ METRICS\n\
                     ├────────────────\n\
                     │ Total: {} requests\n\
                     │ Rate: {:.1} req/s (avg)\n\
                     │ Peak: {:.1} req/s\n\
                     │ Latency (p95): {:.1}ms\n\
                     │ Latency (p99): {:.1}ms\n\
                     │ Error Rate: {:.1}%\n\
                     ├────────────────\n\
                     │ Status Codes:\n\
                     {}",
                    stats.total_requests,
                    stats.avg_rps,
                    stats.max_rps,
                    stats.p95_latency,
                    stats.p99_latency,
                    stats.error_rate * 100.0,
                    PerfStats::format_status_codes(&stats.status_codes)
                ))
                .block(Block::default()
                    .title("🔥 LIVE STATS")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta)))
                .style(Style::default().fg(Color::Green));

                f.render_widget(summary, chunks[2]);
            })?;
            std::thread::sleep(Duration::from_millis(100));
        }
        running.store(false, Ordering::SeqCst);

        // Wait for all handles to complete
        for handle in handles {
            handle.await?;
        }

        // Print final summary
        let final_metrics = metrics.lock().unwrap();
        let stats = PerfStats::calculate(&final_metrics);
        self.print_final_report(&stats)?;

        Ok(())
    }

    fn calculate_stats(&self, metrics: &PerfMetrics) -> PerfStats {
        let total_requests = metrics.requests.iter().sum::<usize>();
        let duration = metrics.timestamps.last().unwrap_or(&0.0) - metrics.timestamps.first().unwrap_or(&0.0);
        
        let mut sorted_latencies = metrics.latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        PerfStats {
            total_requests,
            avg_rps: total_requests as f64 / duration,
            max_rps: *metrics.requests.iter().max().unwrap_or(&0) as f64,
            min_rps: *metrics.requests.iter().min().unwrap_or(&0) as f64,
            p95_latency: percentile(&sorted_latencies, 95.0),
            p99_latency: percentile(&sorted_latencies, 99.0),
            error_rate: metrics.errors.iter().sum::<usize>() as f64 / total_requests as f64,
            status_codes: metrics.status_codes.clone(),
            std_dev_latency: standard_deviation(metrics.latencies.as_slice(), None),
            median_latency: median(metrics.latencies.as_slice()),
        }
    }

    fn print_final_report(&self, stats: &PerfStats) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n📊 Performance Test Results");
        println!("═══════════════════════\n");

        println!("🔢 Request Statistics");
        println!("  • Total Requests: {}", stats.total_requests);
        println!("  • Average RPS: {:.2}", stats.avg_rps);
        println!("  • Peak RPS: {:.2}", stats.max_rps);
        println!("  • Minimum RPS: {:.2}", stats.min_rps);

        println!("\n⏱️  Latency Statistics");
        println!("  • Median: {:.2}ms", stats.median_latency);
        println!("  • P95: {:.2}ms", stats.p95_latency);
        println!("  • P99: {:.2}ms", stats.p99_latency);
        println!("  • Standard Deviation: {:.2}ms", stats.std_dev_latency);

        println!("\n📈 Response Codes");
        for (status, count) in &stats.status_codes {
            let percentage = (*count as f64 / stats.total_requests as f64) * 100.0;
            println!("  • {}: {} ({:.1}%)", status, count, percentage);
        }

        println!("\n❌ Error Rate: {:.2}%", stats.error_rate * 100.0);

        Ok(())
    }
}

#[derive(Debug)]
struct PerfStats {
    total_requests: usize,
    avg_rps: f64,
    max_rps: f64,
    min_rps: f64,
    p95_latency: f64,
    p99_latency: f64,
    error_rate: f64,
    status_codes: HashMap<u16, usize>,
    std_dev_latency: f64,
    median_latency: f64,
}

fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    let len = sorted_data.len();
    if len == 0 {
        return 0.0;
    }
    let index = (p / 100.0 * (len - 1) as f64).round() as usize;
    sorted_data[index]
}

impl PerfStats {
    fn calculate(metrics: &PerfMetrics) -> Self {
        let total_requests = metrics.requests.iter().sum::<usize>();
        let duration = metrics.timestamps.last().unwrap_or(&0.0) - metrics.timestamps.first().unwrap_or(&0.0);
        
        let mut sorted_latencies = metrics.latencies.clone();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // Add safety checks for statistical calculations
        let std_dev = if metrics.latencies.len() >= 2 {
            standard_deviation(metrics.latencies.as_slice(), None)
        } else {
            0.0
        };

        let median_lat = if !metrics.latencies.is_empty() {
            median(metrics.latencies.as_slice())
        } else {
            0.0
        };
        
        Self {
            total_requests,
            avg_rps: total_requests as f64 / duration.max(1.0), // Prevent division by zero
            max_rps: *metrics.requests.iter().max().unwrap_or(&0) as f64,
            min_rps: *metrics.requests.iter().min().unwrap_or(&0) as f64,
            p95_latency: percentile(&sorted_latencies, 95.0),
            p99_latency: percentile(&sorted_latencies, 99.0),
            error_rate: metrics.errors.iter().sum::<usize>() as f64 / total_requests.max(1) as f64,
            status_codes: metrics.status_codes.clone(),
            std_dev_latency: std_dev,
            median_latency: median_lat,
        }
    }
    fn format_status_codes(codes: &HashMap<u16, usize>) -> String {
        codes.iter()
            .map(|(code, count)| format!("│  {} : {}", code, count))
            .collect::<Vec<_>>()
            .join("\n")
    }    
}

