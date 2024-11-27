use console::style;
use reqwest::{blocking::Client, header};
use serde_json::Value;
use std::error::Error;

pub struct CallCommand {
    client: Client,
}

impl CallCommand {
    pub fn new() -> Self {
        CallCommand {
            client: Client::new(),
        }
    }

    pub fn execute(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
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
        let response = request.send()?;
        
        // Print status code
        println!("📡 Status: {}", style(response.status()).yellow());
        
        // Print headers
        println!("\n📋 Headers:");
        for (key, value) in response.headers() {
            println!("  {}: {}", style(key).dim(), value.to_str().unwrap_or(""));
        }
        
        // Print response body
        if let Ok(text) = response.text() {
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
        }

        Ok(())
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
}
