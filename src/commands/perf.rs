use crate::models::metrics::{Metrics, RequestMetric, MetricsSummary};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering};
use std::io::Write;
use console::style;
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use crate::config::Config;

pub struct PerfCommand {
    client: Client,
    metrics: Arc<Metrics>,
    ai_client: AnthropicClient,
}

impl PerfCommand {
    pub fn new(config: &Config) -> Self {
        let api_key = config.anthropic_api_key.clone()
            .unwrap_or_default();

        Self {
            client: Client::new(),
            metrics: Arc::new(Metrics::new()),
            ai_client: ClientBuilder::default()
                .api_key(api_key)
                .build()
                .unwrap(),
        }
    }

    async fn get_performance_analysis(&self, summary: &MetricsSummary, duration: Duration) -> Result<String, Box<dyn std::error::Error>> {
        let prompt = format!(
            "Analyze these API performance metrics and provide 3 key insights or recommendations:\n\
            Total Requests: {} ({} req/s)\n\
            Success Rate: {:.1}%\n\
            Response Times:\n\
            - Average: {}ms\n\
            - p50: {}ms\n\
            - p95: {}ms\n\
            - p99: {}ms\n\
            Peak RPS: {}\n\
            \n\
            Provide concise, actionable insights focusing on:\n\
            1. Performance characteristics\n\
            2. Potential bottlenecks\n\
            3. Optimization opportunities",
            summary.total_requests,
            summary.total_requests as f64 / duration.as_secs_f64(),
            (1.0 - summary.error_rate) * 100.0,
            summary.avg_latency.as_millis(),
            summary.median_latency.as_millis(),
            summary.p95_latency.as_millis(),
            summary.p99_latency.as_millis(),
            summary.peak_rps
        );

        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt }],
        }];

        let message_request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-haiku-20240307".to_string())
            .max_tokens(300_usize)
            .build()?;

        let response = self.ai_client.messages(message_request).await?;
        
        if let Some(ContentBlock::Text { text }) = response.content.first() {
            Ok(text.trim().to_string())
        } else {
            Ok("Analysis not available.".to_string())
        }
    }

    pub async fn run(&self, url: &str, users: u32, duration: Duration, method: &str, body: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🚀 Performance Test Configuration");
        println!("═══════════════════════════════");
        println!("URL: {}", style(url).cyan());
        println!("Method: {}", style(method).cyan());
        println!("Concurrent Users: {}", style(users).cyan());
        println!("Duration: {}s", style(duration.as_secs()).cyan());
        if let Some(body) = body {
            println!("Body: {}", style(body).cyan());
        }
        println!();

        let metrics = self.metrics.clone();
        let running = Arc::new(AtomicBool::new(true));
        let mut handles = Vec::new();
        let start_time = Instant::now();

        // Spawn user tasks
        for _ in 0..users {
            let client = self.client.clone();
            let url = url.to_string();
            let metrics = metrics.clone();
            let method = method.to_string();
            let body = body.map(String::from);
            let running = running.clone();

            let handle = tokio::spawn(async move {
                while running.load(Ordering::Relaxed) && start_time.elapsed() < duration {
                    let request_start = SystemTime::now();
                    
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
                        Ok(response) => {
                            let duration = request_start.elapsed().unwrap();
                            metrics.record(RequestMetric {
                                duration,
                                status: response.status().as_u16(),
                                timestamp: request_start,
                            });
                        },
                        Err(e) => {
                            metrics.record_error(e.to_string());
                        }
                    }
                }
            });
            handles.push(handle);
        }

        // Progress reporting
        while start_time.elapsed() < duration {
            let summary = metrics.summary();
            let current_rps = summary.total_requests as f64 / start_time.elapsed().as_secs_f64();
            let ok_requests = (summary.total_requests as f64 * (1.0 - summary.error_rate)) as usize;
            let ko_requests = summary.total_requests - ok_requests;
            
            print!("\r⚡ {} req ({} ok, {} ko) | {} req/s | lat: avg {}ms p95 {}ms | {}", 
                style(summary.total_requests).magenta().bold(),
                style(ok_requests).green().bold(),
                style(ko_requests).red().bold(),
                style(format!("{:.1}", current_rps)).cyan().bold(),
                style(summary.avg_latency.as_millis()).yellow().bold(),
                style(summary.p95_latency.as_millis()).yellow().bold(),
                if summary.error_rate > 0.0 { 
                    style(format!("errors: {:.1}%", summary.error_rate * 100.0)).red().bold().to_string()
                } else {
                    style("✓").green().bold().to_string()
                }
            );
            std::io::stdout().flush()?;

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        println!();  // New line after progress
        running.store(false, Ordering::SeqCst);

        // Wait for all handles to complete
        for handle in handles {
            handle.await?;
        }

        // Print final summary
        let final_summary = metrics.summary();
        let ok_requests = (final_summary.total_requests as f64 * (1.0 - final_summary.error_rate)) as usize;
        let ko_requests = final_summary.total_requests - ok_requests;

        println!("\n{}", style("Performance Results").cyan().bold());
        println!("{}", style("═════════════════").cyan());
        
        // Request statistics
        println!("\n{}  {}", style("📊").cyan(), style("Requests").bold());
        println!("   • Total: {}", style(final_summary.total_requests).magenta().bold());
        if final_summary.error_rate == 0.0 {
            println!("   • OK: {} (100%)", style(ok_requests).green().bold());
            println!("   • KO: {}", style("0").dim());
        } else {
            println!("   • OK: {} ({:.1}%)", 
                style(ok_requests).green().bold(),
                style(format!("{:.1}", (1.0 - final_summary.error_rate) * 100.0)).green().bold().to_string()
            );
            println!("   • KO: {} ({:.1}%)", 
                style(ko_requests).red().bold(),
                style(format!("{:.1}", final_summary.error_rate * 100.0)).red().bold().to_string()
            );
        }

        // Throughput metrics
        println!("\n{}  {}", style("⚡").cyan(), style("Throughput").bold());
        println!("   • Average: {} req/s", 
            style(format!("{:.1}", final_summary.total_requests as f64 / duration.as_secs_f64())).yellow().bold()
        );
        println!("   • Peak: {} req/s", style(final_summary.peak_rps).magenta().bold());
        
        // Response time distribution
        println!("\n{}  {}", style("⏱️").cyan(), style("Response Time Distribution").bold());
        for (range, count) in &final_summary.response_time_ranges {
            let percentage = (*count as f64 / final_summary.total_requests as f64) * 100.0;
            println!("   • {}: {} ({:.1}%)", 
                style(range).dim(),
                style(count).yellow().bold(),
                style(format!("{:.1}", percentage)).yellow().bold()
            );
        }

        // Detailed latency metrics
        println!("\n{}  {}", style("📈").cyan(), style("Response Time Details").bold());
        println!("   • Min: {}ms", style(final_summary.response_time_ranges.keys().next().unwrap_or(&"N/A".to_string())).yellow().bold());
        println!("   • Average: {}ms", style(final_summary.avg_latency.as_millis()).yellow().bold());
        println!("   • Median (p50): {}ms", style(final_summary.median_latency.as_millis()).yellow().bold());
        println!("   • p95: {}ms", style(final_summary.p95_latency.as_millis()).yellow().bold());
        println!("   • p99: {}ms", style(final_summary.p99_latency.as_millis()).magenta().bold());
        println!("   • Max: {}ms", style(final_summary.response_time_ranges.keys().last().unwrap_or(&"N/A".to_string())).yellow().bold());
        println!("   • Std Dev: {}ms", style(format!("±{:.1}", final_summary.std_dev_latency)).dim());

        // Status code distribution
        if final_summary.error_rate > 0.0 {
            println!("\n{}  {}", style("🔍").cyan(), style("Status Codes").bold());
            let total = final_summary.total_requests as f64;
            let ok_perc = (ok_requests as f64 / total) * 100.0;
            let ko_perc = (ko_requests as f64 / total) * 100.0;
            println!("   • 2xx: {} ({:.1}%)", 
                style(ok_requests).green().bold(),
                style(format!("{:.1}", ok_perc)).green().bold()
            );
            if ko_requests > 0 {
                println!("   • Non-2xx: {} ({:.1}%)", 
                    style(ko_requests).red().bold(),
                    style(format!("{:.1}", ko_perc)).red().bold()
                );
            }
        }
        
        // AI Analysis
        println!("\n{}  {}", style("🤖").cyan(), style("AI Insights").bold());
        match self.get_performance_analysis(&final_summary, duration).await {
            Ok(analysis) => {
                for (_i, line) in analysis.lines().enumerate() {
                    if !line.trim().is_empty() {
                        println!("   {} {}", style("•").dim(), style(line.trim()).dim());
                    }
                }
            }
            Err(_) => println!("   {} Analysis not available", style("•").dim()),
        }

        println!();
        Ok(())
    }
}

