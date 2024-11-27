use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering, AtomicUsize, AtomicU64};
use std::error::Error;
use std::io::Write;

use crate::models::metrics::{Metrics, RequestMetric};

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
                        Ok(response) => {
                            request_count.fetch_add(1, Ordering::Relaxed);
                            let latency = start.elapsed().as_millis() as u64;
                            total_latency.fetch_add(latency, Ordering::Relaxed);
                            println!("Response status: {}", response.status());
                        },
                        Err(e) => {
                            errors.fetch_add(1, Ordering::Relaxed);
                            println!("Error: {}", e);
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

    fn display_live_stats(&self, metrics: &Metrics) {
        let summary = metrics.summary();
        print!("\r\x1B[K"); // Clear line
        print!(
            "Requests: {} | Avg: {:.2}ms | P95: {:.2}ms | Errors: {:.1}%",
            summary.total_requests,
            summary.avg_latency.as_millis(),
            summary.p95_latency.as_millis(),
            summary.error_rate * 100.0,
        );
        std::io::stdout().flush().unwrap();
    }

    async fn make_request(client: &reqwest::Client, url: &str) -> Result<RequestMetric, Box<dyn Error>> {
        let start = Instant::now();
        let response = client.get(url).send().await?;
        let duration = start.elapsed();
        
        Ok(RequestMetric {
            duration,
            status: response.status().as_u16(),
            timestamp: SystemTime::now(),
        })
    }

    fn display_final_report(&self, metrics: &Metrics) {
        let summary = metrics.summary();
        println!("\n📊 Final Report");
        println!("Total Requests: {}", summary.total_requests);
        println!("Average Latency: {:.2}ms", summary.avg_latency.as_millis());
        println!("P95 Latency: {:.2}ms", summary.p95_latency.as_millis());
        println!("Error Rate: {:.2}%", summary.error_rate * 100.0);
    }
}
