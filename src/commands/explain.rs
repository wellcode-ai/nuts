use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;

pub struct ExplainCommand {
    config: Config,
}

impl ExplainCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// AI explains the last API response in human terms
    pub async fn explain_response(&self, response: &str, context: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🧠 AI explaining your API response...");
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let context_info = context.unwrap_or("No additional context provided");
        
        let prompt = format!(
            "You are an expert API response interpreter. Explain this API response in human-friendly terms:\n\n\
            Context: {}\n\n\
            Response:\n{}\n\n\
            Please provide:\n\
            1. SUMMARY: What this response means in plain English\n\
            2. STATUS: Is this a success, error, or something else?\n\
            3. DATA BREAKDOWN: Explain the key data fields\n\
            4. NEXT STEPS: What should the developer do next?\n\
            5. POTENTIAL ISSUES: Any red flags or concerns?\n\
            6. IMPROVEMENTS: How could this API response be better?\n\n\
            Make it friendly and educational for developers of all levels.",
            context_info, response
        );

        let ai_response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1500_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = ai_response.content.first() {
            println!("\n📖 AI Explanation:");
            println!("{}", text);
        }

        Ok(())
    }

    /// Explain API errors with helpful solutions
    pub async fn explain_error(&self, error: &str, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚨 AI analyzing error...");
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let prompt = format!(
            "You are an expert API troubleshooter. Help debug this API error:\n\n\
            Endpoint: {}\n\
            Error: {}\n\n\
            Provide:\n\
            1. ERROR DIAGNOSIS: What exactly went wrong?\n\
            2. ROOT CAUSE: Why did this happen?\n\
            3. SOLUTION STEPS: How to fix it (step by step)\n\
            4. PREVENTION: How to avoid this in the future\n\
            5. CODE EXAMPLES: Show corrected request examples\n\
            6. RELATED ISSUES: Other problems this might indicate\n\n\
            Be specific and actionable. Help the developer solve this quickly.",
            endpoint, error
        );

        let ai_response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1500_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = ai_response.content.first() {
            println!("\n🔧 AI Troubleshooting:");
            println!("{}", text);
        }

        Ok(())
    }

    /// Explain HTTP status codes with context
    pub async fn explain_status_code(&self, status_code: u16, context: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("📊 AI explaining status code {}...", status_code);
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let prompt = format!(
            "Explain HTTP status code {} in the context of this API interaction:\n\n\
            Status Code: {}\n\
            Context: {}\n\n\
            Provide:\n\
            1. MEANING: What this status code means\n\
            2. CONTEXT: Why this happened in this specific situation\n\
            3. EXPECTATION: Is this normal or unexpected?\n\
            4. ACTION: What should the developer do?\n\
            5. EXAMPLES: When else might you see this code?\n\n\
            Keep it educational and practical.",
            status_code, status_code, context
        );

        let ai_response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(800_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = ai_response.content.first() {
            println!("\n📚 Status Code Explanation:");
            println!("{}", text);
        }

        Ok(())
    }
}