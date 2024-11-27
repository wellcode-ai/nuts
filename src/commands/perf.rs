use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use std::sync::atomic::{AtomicBool, Ordering};
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

    pub async fn run(&self, url: &str, users: usize, duration: Duration) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting performance test");
        println!("URL: {}", url);
        println!("Users: {}", users);
        println!("Duration: {:?}", duration);
        
        let metrics = Arc::new(Metrics::new());
        let start_time = Instant::now();
        let stop_signal = Arc::new(AtomicBool::new(false));
        
        // Progress bar
        let pb = ProgressBar::new(duration.as_secs());
        pb.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap());

        // Spawn user simulations
        let mut handles = vec![];
        for user_id in 0..users {
            let metrics: Arc<Metrics> = Arc::clone(&metrics);
            let stop_signal: Arc<AtomicBool> = Arc::clone(&stop_signal);
            let url = url.to_string();
            let client = self.client.clone();
            
            handles.push(tokio::spawn(async move {
                while !stop_signal.load(Ordering::Relaxed) {
                    match Self::make_request(&client, &url).await {
                        Ok(metric) => metrics.record(metric),
                        Err(e) => metrics.record_error(format!("User {}: {}", user_id, e)),
                    };
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }));
        }

        // Update progress
        while start_time.elapsed() < duration {
            pb.set_position(start_time.elapsed().as_secs());
            self.display_live_stats(&metrics);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Signal completion and wait for tasks
        stop_signal.store(true, Ordering::Relaxed);
        for handle in handles {
            handle.await?;
        }

        pb.finish_with_message("Test completed");
        self.display_final_report(&metrics);
        
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
