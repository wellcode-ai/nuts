use console::style;
use reqwest::{header, Client, Method};
use serde_json::Value;
use std::error::Error;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fs;
use crate::models::analysis::{ApiAnalysis, CacheAnalysis};
use crate::commands::CommandResult;

#[derive(Debug)]
pub struct CallOptions {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub follow_redirects: bool,
    pub timeout: Option<Duration>,
    pub verbose: bool,
    pub include_headers: bool,
    pub output_file: Option<String>,
    pub user_agent: Option<String>,
    pub auth: Option<(String, String)>,
    pub bearer_token: Option<String>,
    pub insecure: bool,
    pub max_retries: u32,
    pub form_data: HashMap<String, String>,
}

impl Default for CallOptions {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            url: String::new(),
            headers: HashMap::new(),
            body: None,
            follow_redirects: false,
            timeout: Some(Duration::from_secs(30)),
            verbose: false,
            include_headers: false,
            output_file: None,
            user_agent: Some("NUTS/0.1.0 (AI-Powered CURL Killer)".to_string()),
            auth: None,
            bearer_token: None,
            insecure: false,
            max_retries: 0,
            form_data: HashMap::new(),
        }
    }
}

pub struct CallCommand {
    client: Client,
}

impl CallCommand {
    pub fn new() -> Self {
        CallCommand {
            client: Client::builder()
                .user_agent("NUTS/0.1.0 (AI-Powered CURL Killer)")
                .build()
                .unwrap(),
        }
    }

    pub async fn execute(&self, args: &[&str]) -> CommandResult {
        let options = self.parse_advanced_args(args)?;
        self.execute_with_options(options).await
    }

    pub async fn execute_with_options(&self, options: CallOptions) -> CommandResult {
        if options.verbose {
            println!("🔍 Verbose mode enabled");
            self.print_request_info(&options);
        }

        let start_time = Instant::now();
        let mut attempts = 0;
        let max_attempts = options.max_retries + 1;

        loop {
            attempts += 1;
            
            if options.verbose && attempts > 1 {
                println!("🔄 Retry attempt {} of {}", attempts, max_attempts);
            }

            match self.make_request(&options).await {
                Ok(response) => {
                    let elapsed = start_time.elapsed();
                    self.handle_response(response, &options, elapsed).await?;
                    break;
                }
                Err(e) if attempts < max_attempts => {
                    if options.verbose {
                        println!("❌ Attempt {} failed: {}", attempts, e);
                        println!("⏳ Waiting before retry...");
                    }
                    tokio::time::sleep(Duration::from_millis(1000 * attempts as u64)).await;
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn print_request_info(&self, options: &CallOptions) {
        println!("🌐 {} {}", style(&options.method).cyan(), style(&options.url).cyan());
        
        if !options.headers.is_empty() {
            println!("📋 Request Headers:");
            for (key, value) in &options.headers {
                println!("  {}: {}", style(key).dim(), value);
            }
        }

        if let Some(body) = &options.body {
            println!("📝 Request Body:");
            println!("{}", style(body).blue());
        }

        if !options.form_data.is_empty() {
            println!("📊 Form Data:");
            for (key, value) in &options.form_data {
                println!("  {}: {}", style(key).dim(), value);
            }
        }
    }

    async fn make_request(&self, options: &CallOptions) -> Result<reqwest::Response, Box<dyn Error>> {
        let mut client_builder = Client::builder();

        // Configure client based on options
        if let Some(timeout) = options.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        if options.insecure {
            client_builder = client_builder.danger_accept_invalid_certs(true);
        }

        if !options.follow_redirects {
            client_builder = client_builder.redirect(reqwest::redirect::Policy::none());
        }

        let client = client_builder.build()?;
        let method: Method = options.method.parse()?;
        let mut request = client.request(method, &options.url);

        // Add headers
        for (key, value) in &options.headers {
            request = request.header(key, value);
        }

        // Add user agent
        if let Some(ua) = &options.user_agent {
            request = request.header("User-Agent", ua);
        }

        // Add authentication
        if let Some((username, password)) = &options.auth {
            request = request.basic_auth(username, Some(password));
        }

        if let Some(token) = &options.bearer_token {
            request = request.bearer_auth(token);
        }

        // Add body or form data
        if !options.form_data.is_empty() {
            request = request.form(&options.form_data);
        } else if let Some(body) = &options.body {
            // Try to parse as JSON first
            if let Ok(json_value) = serde_json::from_str::<Value>(body) {
                request = request.json(&json_value);
            } else {
                request = request.body(body.clone());
            }
        }

        Ok(request.send().await?)
    }

    async fn handle_response(&self, response: reqwest::Response, options: &CallOptions, elapsed: Duration) -> CommandResult {
        let status = response.status();
        let headers = response.headers().clone();
        
        println!("📡 Status: {} ({}ms)", 
            style(status).yellow(), 
            style(elapsed.as_millis()).dim()
        );

        if options.include_headers || options.verbose {
            println!("\n📋 Response Headers:");
            for (key, value) in &headers {
                println!("  {}: {}", style(key).dim(), value.to_str().unwrap_or(""));
            }
        }

        // Get response body
        let text = response.text().await?;

        // Save to file if specified
        if let Some(output_file) = &options.output_file {
            fs::write(output_file, &text)?;
            println!("💾 Response saved to: {}", style(output_file).green());
        } else {
            // Print response
            println!("\n📦 Response:");
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                println!("{}", style(serde_json::to_string_pretty(&json)?).green());
            } else {
                println!("{}", style(text.trim()).green());
            }
        }

        // Performance metrics
        if options.verbose {
            println!("\n⚡ Performance:");
            println!("  Response time: {}ms", elapsed.as_millis());
            println!("  Response size: {} bytes", text.len());
        }

        Ok(())
    }

    fn parse_advanced_args(&self, args: &[&str]) -> Result<CallOptions, Box<dyn Error>> {
        if args.len() < 2 {
            return Err("Usage: call [OPTIONS] [METHOD] URL [BODY]".into());
        }

        let mut options = CallOptions::default();
        let mut i = 1; // Skip "call"
        let mut url_found = false;

        while i < args.len() {
            match args[i] {
                // Headers
                "-H" | "--header" => {
                    if i + 1 >= args.len() {
                        return Err("Header value required after -H/--header".into());
                    }
                    let header = args[i + 1];
                    if let Some((key, value)) = header.split_once(':') {
                        options.headers.insert(key.trim().to_string(), value.trim().to_string());
                    } else {
                        return Err("Header must be in format 'Key: Value'".into());
                    }
                    i += 2;
                }
                
                // Authentication
                "-u" | "--user" => {
                    if i + 1 >= args.len() {
                        return Err("Username:password required after -u/--user".into());
                    }
                    let auth_str = args[i + 1];
                    if let Some((username, password)) = auth_str.split_once(':') {
                        options.auth = Some((username.to_string(), password.to_string()));
                    } else {
                        return Err("Auth must be in format 'username:password'".into());
                    }
                    i += 2;
                }

                "--bearer" => {
                    if i + 1 >= args.len() {
                        return Err("Bearer token required after --bearer".into());
                    }
                    options.bearer_token = Some(args[i + 1].to_string());
                    i += 2;
                }

                // Request options
                "-X" | "--request" => {
                    if i + 1 >= args.len() {
                        return Err("HTTP method required after -X/--request".into());
                    }
                    options.method = args[i + 1].to_uppercase();
                    i += 2;
                }

                "-d" | "--data" => {
                    if i + 1 >= args.len() {
                        return Err("Data required after -d/--data".into());
                    }
                    options.body = Some(args[i + 1].to_string());
                    if options.method == "GET" {
                        options.method = "POST".to_string();
                    }
                    i += 2;
                }

                "-F" | "--form" => {
                    if i + 1 >= args.len() {
                        return Err("Form data required after -F/--form".into());
                    }
                    let form_data = args[i + 1];
                    if let Some((key, value)) = form_data.split_once('=') {
                        options.form_data.insert(key.to_string(), value.to_string());
                    } else {
                        return Err("Form data must be in format 'key=value'".into());
                    }
                    if options.method == "GET" {
                        options.method = "POST".to_string();
                    }
                    i += 2;
                }

                // Output options
                "-v" | "--verbose" => {
                    options.verbose = true;
                    i += 1;
                }

                "-i" | "--include" => {
                    options.include_headers = true;
                    i += 1;
                }

                "-o" | "--output" => {
                    if i + 1 >= args.len() {
                        return Err("Output file required after -o/--output".into());
                    }
                    options.output_file = Some(args[i + 1].to_string());
                    i += 2;
                }

                // Network options
                "-L" | "--location" => {
                    options.follow_redirects = true;
                    i += 1;
                }

                "--timeout" => {
                    if i + 1 >= args.len() {
                        return Err("Timeout value required after --timeout".into());
                    }
                    let timeout_secs: u64 = args[i + 1].parse()
                        .map_err(|_| "Invalid timeout value")?;
                    options.timeout = Some(Duration::from_secs(timeout_secs));
                    i += 2;
                }

                "--retry" => {
                    if i + 1 >= args.len() {
                        return Err("Retry count required after --retry".into());
                    }
                    options.max_retries = args[i + 1].parse()
                        .map_err(|_| "Invalid retry count")?;
                    i += 2;
                }

                "-A" | "--user-agent" => {
                    if i + 1 >= args.len() {
                        return Err("User agent required after -A/--user-agent".into());
                    }
                    options.user_agent = Some(args[i + 1].to_string());
                    i += 2;
                }

                "-k" | "--insecure" => {
                    options.insecure = true;
                    i += 1;
                }

                // If it starts with -, it's an unknown option
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg).into());
                }

                // HTTP methods
                "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" => {
                    if url_found {
                        // This is probably body data after URL
                        options.body = Some(args[i..].join(" "));
                        break;
                    } else {
                        options.method = args[i].to_uppercase();
                        i += 1;
                    }
                }

                // URL or body data
                _ => {
                    if !url_found {
                        // First non-option argument is the URL
                        let url_candidate = args[i];
                        if url_candidate.starts_with("http") {
                            options.url = url_candidate.to_string();
                        } else {
                            options.url = format!("https://{}", url_candidate);
                        }
                        url_found = true;
                        i += 1;
                    } else {
                        // Everything else is body data
                        options.body = Some(args[i..].join(" "));
                        break;
                    }
                }
            }
        }

        if options.url.is_empty() {
            return Err("URL is required".into());
        }

        Ok(options)
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
