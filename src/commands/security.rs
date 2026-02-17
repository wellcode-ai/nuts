use crate::config::Config;
use crate::output::{colors, renderer};
use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use reqwest::header;
use reqwest::Client;

pub struct SecurityCommand {
    #[allow(dead_code)]
    config: Config,
    deep_scan: bool,
    auth_token: Option<String>,
    save_file: Option<String>,
    http_client: Client,
    ai_client: AnthropicClient,
}

impl SecurityCommand {
    pub fn new(config: Config) -> Self {
        let api_key = config.anthropic_api_key.clone().unwrap_or_default();

        Self {
            config,
            deep_scan: false,
            auth_token: None,
            save_file: None,
            http_client: Client::new(),
            ai_client: ClientBuilder::default().api_key(api_key).build().unwrap(),
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
        renderer::render_ai_insight("Security Analysis", analysis);

        println!(
            "\n  {}",
            colors::muted().apply_to(
                "This analysis is based on the API response only. A comprehensive security audit would require additional context."
            )
        );
        println!();
    }

    pub async fn execute(&self, args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 2 {
            renderer::render_error(
                "Missing URL",
                "security <url>",
                "security https://api.example.com",
            );
            return Ok(());
        }

        let url = if args[1].starts_with("http") {
            args[1].to_string()
        } else {
            format!("http://{}", args[1])
        };

        println!(
            "\n  {} {}",
            colors::accent().apply_to("Scanning:"),
            colors::muted().apply_to(&url),
        );
        if self.deep_scan {
            println!(
                "  {}",
                colors::muted().apply_to("Deep scan enabled -- this may take a few minutes")
            );
        }

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
                if let Ok(resp) = self
                    .http_client
                    .request(
                        reqwest::Method::from_bytes(method.as_bytes()).unwrap(),
                        &url,
                    )
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
                "You are the best security architect on the world and you will perform a deep security analysis of these API responses, including main endpoint and additional security checks.\n\n\
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
                "You are the best security architect on the world and you will analyze this API response for security issues. Consider OWASP top 10 and best practices.\n\n{}\n\
                Provide a security analysis focusing on:\n\
                1. Response headers security\n\
                2. Data exposure risks\n\
                3. Authentication/Authorization concerns\n\
                4. Sensitive information disclosure\n\
                5. Security recommendations",
                analysis_data[0]
            )
        };

        println!("  {}", colors::muted().apply_to("Analyzing with AI..."));

        // Get AI analysis
        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: analysis_prompt,
            }],
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
            renderer::render_error("Could not parse AI response", "", "");
        }

        Ok(())
    }

    async fn analyze_response(
        &self,
        response: reqwest::Response,
    ) -> Result<String, Box<dyn std::error::Error>> {
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
