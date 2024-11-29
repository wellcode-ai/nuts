use console::style;
use serde_json::Value;
use std::error::Error;
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use anthropic::{AI_PROMPT, HUMAN_PROMPT};
use reqwest::header;

pub struct SecurityCommand {
    http_client: reqwest::Client,
    ai_client: AnthropicClient,
}

impl SecurityCommand {
    pub fn new(anthropic_api_key: &str) -> Self {
        SecurityCommand {
            http_client: reqwest::Client::new(),
            ai_client: ClientBuilder::default().api_key(anthropic_api_key.to_string()).build().unwrap(),
        }
    }

    pub async fn execute(&self, args: &[&str]) -> Result<(), Box<dyn Error>> {
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
        
        // Make the HTTP request
        let response = self.http_client.get(&url).send().await?;
        
        // Gather information for analysis
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await?;

        // Prepare context for Claude
        let analysis_prompt = format!(
            "Analyze this API response for security issues. Consider OWASP top 10 and best practices.\n\n\
            Status: {}\n\
            Headers:\n{}\n\
            Body:\n{}\n\n\
            Provide a security analysis focusing on:\n\
            1. Response headers security\n\
            2. Data exposure risks\n\
            3. Authentication/Authorization concerns\n\
            4. Sensitive information disclosure\n\
            5. Security recommendations",
            status,
            self.format_headers(&headers),
            body
        );

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
        println!("📊 Security Analysis:");
        if let ContentBlock::Text { text } = &messages_response.content[0] {
            println!("{}", style(text).green());
        }

        Ok(())
    }

    fn format_headers(&self, headers: &header::HeaderMap) -> String {
        headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
            .collect::<Vec<String>>()
            .join("\n")
    }
} 