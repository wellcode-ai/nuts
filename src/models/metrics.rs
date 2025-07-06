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
    pub median_latency: Duration,
    pub std_dev_latency: f64,
    pub total_requests: usize,
    pub error_rate: f64,
    pub response_time_ranges: HashMap<String, usize>,
    #[allow(dead_code)]
    pub requests_per_second: Vec<(SystemTime, usize)>,
    pub peak_rps: usize,
}

pub struct Metrics {
    latencies: Mutex<Vec<Duration>>,
    status_codes: Mutex<HashMap<u16, usize>>,
    requests_per_second: Mutex<Vec<(SystemTime, usize)>>,
    errors: Mutex<Vec<String>>,
    start_time: SystemTime,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            latencies: Mutex::new(Vec::new()),
            status_codes: Mutex::new(HashMap::new()),
            requests_per_second: Mutex::new(Vec::new()),
            errors: Mutex::new(Vec::new()),
            start_time: SystemTime::now(),
        }
    }

    pub fn record(&self, metric: RequestMetric) {
        let mut latencies = self.latencies.lock().unwrap();
        let mut status_codes = self.status_codes.lock().unwrap();
        let mut rps = self.requests_per_second.lock().unwrap();
        
        // Record basic metrics
        latencies.push(metric.duration);
        *status_codes.entry(metric.status).or_insert(0) += 1;

        // Update requests per second
        let current_second = metric.timestamp
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
            
        if let Some(last) = rps.last_mut() {
            if last.0.duration_since(self.start_time).unwrap().as_secs() == current_second {
                last.1 += 1;
            } else {
                rps.push((metric.timestamp, 1));
            }
        } else {
            rps.push((metric.timestamp, 1));
        }
    }

    pub fn summary(&self) -> MetricsSummary {
        let latencies = self.latencies.lock().unwrap();
        let rps = self.requests_per_second.lock().unwrap();
        
        MetricsSummary {
            avg_latency: self.calculate_average(&latencies),
            p95_latency: self.calculate_percentile(&latencies, 95),
            p99_latency: self.calculate_percentile(&latencies, 99),
            total_requests: latencies.len(),
            error_rate: self.calculate_error_rate(),
            response_time_ranges: self.calculate_response_time_ranges(&latencies),
            requests_per_second: rps.clone(),
            peak_rps: rps.iter().map(|(_, count)| *count).max().unwrap_or(0),
            median_latency: self.calculate_percentile(&latencies, 50),
            std_dev_latency: self.calculate_std_dev(&latencies),
        }
    }

    fn calculate_response_time_ranges(&self, latencies: &Vec<Duration>) -> HashMap<String, usize> {
        let mut ranges = HashMap::new();
        
        for &latency in latencies {
            let ms = latency.as_millis();
            let range = match ms {
                ms if ms < 800 => "<800ms",
                ms if ms < 1200 => "<1.2s",
                ms if ms < 2000 => "<2s",
                _ => ">2s",
            };
            *ranges.entry(range.to_string()).or_insert(0) += 1;
        }
        
        ranges
    }

    fn calculate_std_dev(&self, latencies: &Vec<Duration>) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }

        let mean = self.calculate_average(latencies);
        let variance: f64 = latencies.iter()
            .map(|&duration| {
                let diff = duration.as_secs_f64() - mean.as_secs_f64();
                diff * diff
            })
            .sum::<f64>() / latencies.len() as f64;

        variance.sqrt()
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
