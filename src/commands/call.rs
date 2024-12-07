use console::style;
use reqwest::{header, Client};
use serde_json::Value;
use std::error::Error;
use crate::models::analysis::{ApiAnalysis, CacheAnalysis};
use crate::commands::CommandResult;

pub struct CallCommand {
    client: Client,
}

impl CallCommand {
    pub fn new() -> Self {
        CallCommand {
            client: Client::new(),
        }
    }

    pub async fn execute(&self, args: &[&str]) -> CommandResult {
        println!("🚀 Executing request...");
        
        let (method, url, body) = self.parse_args(args)?;
        
        // Convert url to proper URL type
        let url = if !url.starts_with("http") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };
        
        let response = self.client.request(method.parse()?, &url)
            .body(body.map(|b| b.to_string()).unwrap_or_default())
            .send()
            .await?;
            
        self.print_response(response).await?;
        
        Ok(())
    }

    async fn print_response(&self, response: reqwest::Response) -> CommandResult {
        println!("📡 Status: {}", style(response.status()).yellow());
        
        // Print headers
        println!("\n📋 Headers:");
        for (key, value) in response.headers() {
            println!("  {}: {}", style(key).dim(), value.to_str().unwrap_or(""));
        }
        
        // Print response body
        let text = response.text().await?;
        println!("\n📦 Response:");
        
        if let Ok(json) = serde_json::from_str::<Value>(&text) {
            println!("{}", style(serde_json::to_string_pretty(&json)?).green());
        } else {
            println!("{}", style(text.trim()).green());
        }

        Ok(())
    }

    pub async fn execute_with_response(&self, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
        // Parse arguments
        let (method, url, body) = self.parse_args(args)?;
        
        // Add http:// if not present
        let full_url = if !url.starts_with("http") {
            format!("http://{}", url)
        } else {
            url.to_string()
        };

        println!("🌐 {} {}", style(&method).cyan(), style(&full_url).cyan());

        // Build the request
        let mut request = self.client.request(
            method.parse()?,
            &full_url
        );

        // Add JSON body if provided
        if let Some(json_body) = body {
            println!("📝 Request Body:");
            println!("{}", style(&json_body).blue());
            request = request.header(header::CONTENT_TYPE, "application/json")
                           .body(json_body.to_string());
        }

        // Send request
        let response = request.send().await?;
        
        // Print status code
        println!("📡 Status: {}", style(response.status()).yellow());
        
        // Print headers
        println!("\n📋 Headers:");
        for (key, value) in response.headers() {
            println!("  {}: {}", style(key).dim(), value.to_str().unwrap_or(""));
        }
        
        // Store headers before consuming response
        let headers = response.headers().clone();
        
        // Print response body
        let text = response.text().await?;
        println!("\n📦 Response:");
        // Try to pretty print if it's JSON
        match serde_json::from_str::<Value>(&text) {
            Ok(json) => {
                println!("{}", style(serde_json::to_string_pretty(&json)?).green());
            },
            Err(_) => {
                // If it's not JSON, just print as plain text
                println!("{}", style(text.trim()).green());
            }
        }

        if args.contains(&"--analyze") {
            let _ = self.handle_analyze(&headers, &text).await?;
        }

        Ok(text)  // Return the response body
    }

    fn parse_args<'a>(&self, args: &[&'a str]) -> Result<(String, &'a str, Option<Value>), Box<dyn Error>> {
        if args.len() < 2 {
            return Err("Usage: call [METHOD] URL [JSON_BODY]".into());
        }

        let (method, url, body_start) = if args[1].eq_ignore_ascii_case("get") 
            || args[1].eq_ignore_ascii_case("post")
            || args[1].eq_ignore_ascii_case("put")
            || args[1].eq_ignore_ascii_case("delete")
            || args[1].eq_ignore_ascii_case("patch") {
            // Method specified
            (args[1].to_uppercase(), args[2], 3)
        } else {
            // No method specified, default to GET
            ("GET".to_string(), args[1], 2)
        };

        // Validate we have enough arguments when method is specified
        if body_start == 3 && args.len() < 3 {
            return Err("URL is required after HTTP method".into());
        }

        // Parse JSON body if provided and method is not GET
        let body = if args.len() > body_start && method != "GET" {
            let body_str = args[body_start..].join(" ");
            Some(serde_json::from_str(&body_str)?)
        } else {
            None
        };

        Ok((method, url, body))
    }
    async fn handle_analyze(&self, headers: &header::HeaderMap, body: &str) -> Result<ApiAnalysis, Box<dyn Error>> {
        let analysis = ApiAnalysis {
            auth_type: self.detect_auth_type(headers),
            rate_limit: self.detect_rate_limit(headers),
            cache_status: self.analyze_cache(headers),
            recommendations: self.generate_recommendations(headers, body).await,
        };
    
        println!("\n🤖 Analyzing API patterns...");
        if let Some(auth) = &analysis.auth_type {
            println!("✓ Authentication: {}", auth);
        }
        if let Some(rate) = analysis.rate_limit {
            println!("✓ Rate limiting: {} req/min", rate);
        }
        if analysis.cache_status.cacheable {
            println!("✓ Caching opportunity identified");
        }
        
        if !analysis.recommendations.is_empty() {
            println!("\n📝 Recommendations:");
            for rec in &analysis.recommendations {
                println!("• {}", rec);
            }
        }
    
        Ok(analysis)
    }
    
    fn detect_auth_type(&self, headers: &reqwest::header::HeaderMap) -> Option<String> {
        if headers.contains_key("www-authenticate") {
            Some("Basic".to_string())
        } else if let Some(auth) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
            if auth.starts_with("Bearer") {
                Some("JWT/Bearer".to_string())
            } else if auth.starts_with("Basic") {
                Some("Basic".to_string())
            } else {
                Some("Custom".to_string())
            }
        } else {
            None
        }
    }
    
    fn detect_rate_limit(&self, headers: &reqwest::header::HeaderMap) -> Option<u32> {
        // Check multiple common rate limit headers
        headers.get("x-ratelimit-limit")
            .or(headers.get("ratelimit-limit"))
            .or(headers.get("x-rate-limit"))
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
    }
    
    fn analyze_cache(&self, headers: &reqwest::header::HeaderMap) -> CacheAnalysis {
        let cache_control = headers
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        
        let etag = headers.contains_key("etag");
        let last_modified = headers.contains_key("last-modified");
        
        let mut reason = Vec::new();
        if etag { reason.push("ETag header present"); }
        if last_modified { reason.push("Last-Modified header present"); }
        if !cache_control.is_empty() { reason.push("Cache-Control directive found"); }
        
        CacheAnalysis {
            cacheable: (!cache_control.contains("no-cache") 
                       && !cache_control.contains("private"))
                       || etag 
                       || last_modified,
            suggested_ttl: if cache_control.contains("max-age=") {
                cache_control
                    .split("max-age=")
                    .nth(1)
                    .and_then(|s| s.split(',').next())
                    .and_then(|s| s.parse().ok())
            } else {
                Some(3600) // Default 1 hour if no max-age specified
            },
            reason: reason.join(", "),
        }
    }

    async fn generate_recommendations(&self, headers: &reqwest::header::HeaderMap, body: &str) -> Vec<String> {
        let mut recommendations = self.generate_basic_recommendations(headers);
        
        // Add AI recommendations
        if let Ok(ai_recommendations) = self.get_ai_recommendations(headers, body).await {
            recommendations.extend(ai_recommendations);
        }
        
        recommendations
    }

    // Rename existing recommendations to basic
    fn generate_basic_recommendations(&self, headers: &reqwest::header::HeaderMap) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // Rate limiting recommendations
        if headers.get("x-ratelimit-limit").is_none() {
            recommendations.push("Consider implementing rate limiting".to_string());
        }
        
        // Security recommendations
        if !headers.contains_key("x-content-type-options") {
            recommendations.push("Add X-Content-Type-Options: nosniff header".to_string());
        }
        if !headers.contains_key("x-frame-options") {
            recommendations.push("Consider adding X-Frame-Options header".to_string());
        }
        
        // Cache recommendations
        if !headers.contains_key("cache-control") {
            recommendations.push("Add explicit Cache-Control directives".to_string());
        }
        
        // CORS recommendations
        if headers.contains_key("access-control-allow-origin") {
            if headers.get("access-control-allow-origin")
                     .and_then(|v| v.to_str().ok())
                     .map_or(false, |v| v == "*") {
                recommendations.push("Consider restricting CORS Access-Control-Allow-Origin".to_string());
            }
        }
        
        recommendations
    }

    async fn get_ai_recommendations(&self, headers: &reqwest::header::HeaderMap, body: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let prompt = format!(
            "Analyze this API response and provide specific recommendations for improvement. \
            Headers: {:?}\nBody preview: {}", 
            headers,
            &body[..body.len().min(500)] // First 500 chars of body
        );

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", std::env::var("ANTHROPIC_API_KEY")?)
            .header("anthropic-version", "2023-06-01")
            .json(&serde_json::json!({
                "model": "claude-3-sonnet-20240229",
                "max_tokens": 1000,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .send()
            .await?;

        let ai_response: Value = response.json().await?;
        let content = ai_response["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Split response into individual recommendations
        Ok(content
            .lines()
            .filter(|line| line.trim().starts_with("-"))
            .map(|line| line.trim_start_matches('-').trim().to_string())
            .collect())
    }
}
