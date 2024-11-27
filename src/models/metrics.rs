use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use std::sync::Mutex;

#[derive(Debug)]
pub struct RequestMetric {
    pub duration: Duration,
    pub status: u16,
    pub timestamp: SystemTime,
}

#[derive(Debug)]
pub struct MetricsSummary {
    pub avg_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub total_requests: usize,
    pub error_rate: f64,
}

pub struct Metrics {
    latencies: Mutex<Vec<Duration>>,
    status_codes: Mutex<HashMap<u16, usize>>,
    requests_per_second: Mutex<Vec<(SystemTime, usize)>>,
    errors: Mutex<Vec<String>>,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            latencies: Mutex::new(Vec::new()),
            status_codes: Mutex::new(HashMap::new()),
            requests_per_second: Mutex::new(Vec::new()),
            errors: Mutex::new(Vec::new()),
        }
    }

    pub fn record(&self, metric: RequestMetric) {
        let mut latencies = self.latencies.lock().unwrap();
        let mut status_codes = self.status_codes.lock().unwrap();
        
        latencies.push(metric.duration);
        *status_codes.entry(metric.status).or_insert(0) += 1;
    }

    pub fn summary(&self) -> MetricsSummary {
        let latencies = self.latencies.lock().unwrap();
        
        MetricsSummary {
            avg_latency: self.calculate_average(&latencies),
            p95_latency: self.calculate_percentile(&latencies, 95),
            p99_latency: self.calculate_percentile(&latencies, 99),
            total_requests: latencies.len(),
            error_rate: self.calculate_error_rate(),
        }
    }

    fn calculate_average(&self, latencies: &Vec<Duration>) -> Duration {
        if latencies.is_empty() {
            return Duration::from_secs(0);
        }
        let sum: Duration = latencies.iter().sum();
        sum / latencies.len() as u32
    }

    fn calculate_percentile(&self, latencies: &Vec<Duration>, percentile: usize) -> Duration {
        if latencies.is_empty() {
            return Duration::from_secs(0);
        }
        let mut sorted = latencies.clone();
        sorted.sort();
        let index = (percentile * sorted.len() / 100).saturating_sub(1);
        sorted[index]
    }

    fn calculate_error_rate(&self) -> f64 {
        let status_codes = self.status_codes.lock().unwrap();
        let total: usize = status_codes.values().sum();
        if total == 0 {
            return 0.0;
        }
        let errors: usize = status_codes
            .iter()
            .filter(|(&code, &_)| code >= 400)
            .map(|(_, &count)| count)
            .sum();
        errors as f64 / total as f64
    }

    pub fn record_error(&self, error: String) {
        let mut errors = self.errors.lock().unwrap();
        errors.push(error);
    }
}