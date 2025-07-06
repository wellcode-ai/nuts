use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;
use crate::commands::call::CallCommand;
use crate::commands::generate::GenerateCommand;
use serde_json::Value;
use std::collections::HashMap;

pub struct AskCommand {
    config: Config,
}

impl AskCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// AI-Powered Natural Language API Interface
    /// This is the revolutionary CURL killer - just ask in plain English!
    pub async fn execute(&self, request: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 AI Understanding: {}", request);
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let prompt = format!(
            "You are NUTS AI, a revolutionary API testing assistant. The user wants to perform this task:\n\n\
            '{}'\n\n\
            Based on this request, determine what API actions to perform and respond with JSON:\n\n\
            {{\n\
              \"action\": \"call|generate|test|monitor\",\n\
              \"method\": \"GET|POST|PUT|DELETE|PATCH\",\n\
              \"url\": \"inferred or ask user\",\n\
              \"body\": {{...}} or null,\n\
              \"headers\": {{...}} or null,\n\
              \"explanation\": \"what you're doing and why\",\n\
              \"follow_up\": \"suggested next steps\"\n\
            }}\n\n\
            If the request is about generating test data, set action to 'generate'.\n\
            If the request is about monitoring, set action to 'monitor'.\n\
            If the request is about testing workflows, set action to 'test'.\n\
            Otherwise, set action to 'call' for API requests.\n\n\
            Be smart about inferring common API patterns and realistic data.",
            request
        );

        let response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1500_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = response.content.first() {
            println!("\n🧠 AI Analysis:");
            
            // Try to parse as JSON
            if let Ok(ai_response) = serde_json::from_str::<Value>(text) {
                let action = ai_response.get("action").and_then(|v| v.as_str()).unwrap_or("call");
                let explanation = ai_response.get("explanation").and_then(|v| v.as_str()).unwrap_or("Processing your request");
                let follow_up = ai_response.get("follow_up").and_then(|v| v.as_str()).unwrap_or("What would you like to do next?");
                
                println!("📋 {}", explanation);
                
                match action {
                    "call" => {
                        self.execute_api_call(&ai_response).await?;
                    }
                    "generate" => {
                        self.execute_generate_data(&ai_response).await?;
                    }
                    "test" => {
                        println!("🧪 Executing intelligent test workflow...");
                        // Could integrate with test command
                    }
                    "monitor" => {
                        println!("📊 Setting up smart monitoring...");
                        // Could integrate with monitor command
                    }
                    _ => {
                        println!("🤷 I'm not sure how to handle that request yet.");
                    }
                }
                
                println!("\n💡 Next: {}", follow_up);
                
            } else {
                // Fallback to showing AI response as text
                println!("{}", text);
            }
        }

        Ok(())
    }

    async fn execute_api_call(&self, ai_response: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let method = ai_response.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
        let url = ai_response.get("url").and_then(|v| v.as_str());
        
        if let Some(url) = url {
            println!("🚀 Making {} request to {}", method, url);
            
            let mut args = vec![method, url];
            
            let call_command = CallCommand::new();
            
            // Add body if present and execute
            if let Some(body) = ai_response.get("body") {
                if !body.is_null() {
                    let body_str = serde_json::to_string(body)?;
                    args.push(&body_str);
                    call_command.execute(&args).await?;
                } else {
                    call_command.execute(&args).await?;
                }
            } else {
                call_command.execute(&args).await?;
            }
        } else {
            println!("❓ I need more information. What URL should I call?");
        }
        
        Ok(())
    }

    async fn execute_generate_data(&self, ai_response: &Value) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎲 Generating intelligent test data...");
        
        // Extract generation parameters
        let data_type = ai_response.get("data_type").and_then(|v| v.as_str()).unwrap_or("users");
        let count = ai_response.get("count").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        
        // Use the generate command
        let generate_command = GenerateCommand::new(self.config.clone());
        generate_command.generate(data_type, count).await?;
        
        Ok(())
    }
}