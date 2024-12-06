use console::{style, Term};
use std::error::Error;
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};

use reqwest::header;
use reqwest::Client;
use std::fmt::Write;

pub struct SecurityCommand {
    api_key: String,
    deep_scan: bool,
    auth_token: Option<String>,
    save_file: Option<String>,
    http_client: Client,
    ai_client: AnthropicClient,
}

impl SecurityCommand {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            deep_scan: false,
            auth_token: None,
            save_file: None,
            http_client: Client::new(),
            ai_client: ClientBuilder::default().api_key(api_key.to_string()).build().unwrap(),
        }
    }

    pub fn with_deep_scan(mut self, deep_scan: bool) -> Self {
        self.deep_scan = deep_scan;
        self
    }

    pub fn with_auth(mut self, auth_token: Option<String>) -> Self {
        self.auth_token = auth_token;
        self
    }

    pub fn with_save_file(mut self, save_file: Option<String>) -> Self {
        self.save_file = save_file;
        self
    }

    async fn display_security_analysis(&self, analysis: &str) {
        let term = Term::stdout();
        let width = term.size().1 as usize;
        
        println!("\n{}", style("📊 Security Analysis").bold().cyan());
        println!("{}\n", style("═".repeat(width.min(80))).cyan());

        // Split analysis into sections based on numbered items
        let sections: Vec<&str> = analysis.split("\n\n").collect();
        
        for section in sections {
            if section.starts_with(|c: char| c.is_digit(10)) {
                // Main section headers
                let (header, content) = section.split_once(":\n").unwrap_or((section, ""));
                println!("{}", style(header).yellow().bold());
                
                // Process bullet points and sub-sections
                for line in content.lines() {
                    if line.trim().is_empty() { continue; }
                    
                    if line.starts_with("- ") {
                        println!("  {} {}", 
                            style("•").cyan(),
                            style(&line[2..]).white()
                        );
                    } else if line.starts_with("`") {
                        // Format code/technical items
                        println!("    {}", style(line).blue());
                    } else {
                        println!("  {}", style(line).white());
                    }
                }
                println!(); // Add spacing between sections
            }
        }

        // Add a summary box at the end
        println!("{}", style("─".repeat(width.min(80))).cyan());
        let summary = style("ℹ️  This analysis is based on the API response only. A comprehensive security audit would require additional context.").dim();
        println!("{}\n", summary);
    }

    pub async fn execute(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        println!("{}", style("🔒 Starting security scan...").bold());
        
        if self.deep_scan {
            println!("{}", style("📋 Deep scan enabled - this may take a few minutes").yellow());
        }

        if args.len() < 2 {
            println!("❌ Usage: security <url>");
            println!("Example: security api.example.com/v1/users");
            return Ok(());
        }

        let url = if args[1].starts_with("http") {
            args[1].to_string()
        } else {
            format!("http://{}", args[1])
        };

        println!("🔒 Running security analysis on {}", style(&url).cyan());
        
        let mut analysis_data = Vec::new();

        // Basic scan - check main endpoint
        let response = self.http_client.get(&url).send().await?;
        analysis_data.push(self.analyze_response(response).await?);

        // Deep scan - additional checks
        if self.deep_scan {
            // Check common security endpoints
            for endpoint in ["/security.txt", "/.well-known/security.txt", "/robots.txt"] {
                let sec_url = format!("{}{}", url, endpoint);
                if let Ok(resp) = self.http_client.get(&sec_url).send().await {
                    analysis_data.push(self.analyze_response(resp).await?);
                }
            }

            // Check HTTP methods
            for method in ["HEAD", "OPTIONS", "TRACE"] {
                if let Ok(resp) = self.http_client
                    .request(reqwest::Method::from_bytes(method.as_bytes()).unwrap(), &url)
                    .send()
                    .await 
                {
                    analysis_data.push(self.analyze_response(resp).await?);
                }
            }
        }

        // Combine all analyses for AI processing
        let analysis_prompt = if self.deep_scan {
            format!(
                "Perform a deep security analysis of these API responses, including main endpoint and additional security checks.\n\n\
                Main endpoint response:\n{}\n\n\
                Additional endpoints and methods tested:\n{}\n\n\
                Provide a comprehensive security analysis focusing on:\n\
                1. Response headers security and variations across endpoints\n\
                2. Data exposure risks and information disclosure patterns\n\
                3. Authentication/Authorization mechanisms and consistency\n\
                4. Security headers and configurations across endpoints\n\
                5. Detailed security recommendations based on all findings",
                analysis_data[0],
                analysis_data[1..].join("\n---\n")
            )
        } else {
            format!(
                "Analyze this API response for security issues. Consider OWASP top 10 and best practices.\n\n{}\n\
                Provide a security analysis focusing on:\n\
                1. Response headers security\n\
                2. Data exposure risks\n\
                3. Authentication/Authorization concerns\n\
                4. Sensitive information disclosure\n\
                5. Security recommendations",
                analysis_data[0]
            )
        };

        println!("🤖 Analyzing response with Claude AI...\n");

        // Get AI analysis
        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: analysis_prompt.into() }]
        }];

        let messages_request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1000_usize)
            .build()?;

        let messages_response = self.ai_client.messages(messages_request).await?;

        // Print the analysis
        if let Some(ContentBlock::Text { text }) = messages_response.content.first() {
            self.display_security_analysis(text).await;
        } else {
            println!("❌ Error: Could not parse AI response");
        }

        Ok(())
    }

    async fn analyze_response(&self, response: reqwest::Response) -> Result<String, Box<dyn std::error::Error>> {
        let url = response.url().to_string();
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        Ok(format!(
            "URL: {}\nStatus: {}\nHeaders:\n{}\nBody:\n{}\n",
            url,
            status,
            self.format_headers(&headers),
            body
        ))
    }

    fn format_headers(&self, headers: &header::HeaderMap) -> String {
        headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
            .collect::<Vec<String>>()
            .join("\n")
    }
} 