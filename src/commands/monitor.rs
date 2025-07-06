use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;
use crate::commands::call::CallCommand;
use std::time::{Duration, SystemTime};
use serde_json::json;
use tokio::time::{sleep, interval};

pub struct MonitorCommand {
    config: Config,
}

#[derive(Debug)]
pub struct MonitorResult {
    pub url: String,
    pub status: String,
    pub response_time: Duration,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

impl MonitorCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Smart API monitoring with AI insights
    pub async fn monitor(&self, url: &str, smart: bool) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 Starting {} monitoring for: {}", 
            if smart { "smart AI" } else { "basic" }, url);
        
        let mut interval = interval(Duration::from_secs(30));
        let mut check_count = 0;
        let mut historical_data = Vec::new();
        
        loop {
            check_count += 1;
            println!("\n🔍 Health check #{}", check_count);
            
            let result = self.perform_health_check(url).await?;
            historical_data.push(result);
            
            if smart && check_count % 3 == 0 {
                // Every 3rd check, do AI analysis
                self.ai_analysis(&historical_data).await?;
            }
            
            // Keep only last 10 results
            if historical_data.len() > 10 {
                historical_data.drain(0..1);
            }
            
            interval.tick().await;
            
            // For demo purposes, break after 5 checks
            if check_count >= 5 {
                break;
            }
        }
        
        println!("\n✅ Monitoring session complete!");
        Ok(())
    }
    
    async fn perform_health_check(&self, url: &str) -> Result<MonitorResult, Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        let call_command = CallCommand::new();
        
        // Try to make the request
        let mut status = "healthy".to_string();
        let mut issues = Vec::new();
        
        match call_command.execute_with_response(&["GET", url]).await {
            Ok(response) => {
                let response_time = start_time.elapsed()?;
                
                // Check response time
                if response_time > Duration::from_millis(1000) {
                    status = "slow".to_string();
                    issues.push(format!("Slow response: {}ms", response_time.as_millis()));
                }
                
                // Check response content
                if response.contains("error") || response.contains("Error") {
                    status = "warning".to_string();
                    issues.push("Response contains error messages".to_string());
                }
                
                if response.len() == 0 {
                    status = "warning".to_string();
                    issues.push("Empty response body".to_string());
                }
                
                let result = MonitorResult {
                    url: url.to_string(),
                    status,
                    response_time,
                    issues,
                    recommendations: vec![],
                };
                
                self.print_health_status(&result);
                Ok(result)
            }
            Err(e) => {
                let result = MonitorResult {
                    url: url.to_string(),
                    status: "error".to_string(),
                    response_time: Duration::from_millis(0),
                    issues: vec![format!("Request failed: {}", e)],
                    recommendations: vec![],
                };
                
                self.print_health_status(&result);
                Ok(result)
            }
        }
    }
    
    fn print_health_status(&self, result: &MonitorResult) {
        let emoji = match result.status.as_str() {
            "healthy" => "💚",
            "warning" => "🟡",
            "slow" => "🟠",
            "error" => "🔴",
            _ => "⚪",
        };
        
        println!("{} Status: {} ({}ms)", 
            emoji, result.status, result.response_time.as_millis());
        
        if !result.issues.is_empty() {
            println!("  Issues:");
            for issue in &result.issues {
                println!("    • {}", issue);
            }
        }
    }
    
    async fn ai_analysis(&self, historical_data: &[MonitorResult]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🤖 AI Analysis of monitoring data...");
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured for AI analysis")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let analysis_data = json!({
            "monitoring_results": historical_data.iter().map(|r| {
                json!({
                    "status": r.status,
                    "response_time_ms": r.response_time.as_millis(),
                    "issues": r.issues,
                    "url": r.url
                })
            }).collect::<Vec<_>>()
        });

        let prompt = format!(
            "Analyze this API monitoring data and provide insights:\n\n\
            {}\n\n\
            Provide:\n\
            1. TREND ANALYSIS: What trends do you see in performance?\n\
            2. ISSUE PATTERNS: Are there recurring issues?\n\
            3. PREDICTIONS: What might happen next?\n\
            4. RECOMMENDATIONS: Specific actions to take\n\
            5. ALERTS: Any immediate concerns?\n\n\
            Be specific and actionable.",
            serde_json::to_string_pretty(&analysis_data)?
        );

        let response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1000_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = response.content.first() {
            println!("📈 AI Insights:");
            println!("{}", text);
        }

        Ok(())
    }
}