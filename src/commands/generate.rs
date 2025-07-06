use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;
use serde_json::Value;
use rand::Rng;

pub struct GenerateCommand {
    config: Config,
}

impl GenerateCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Generate realistic test data with AI
    pub async fn generate(&self, data_type: &str, count: usize) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎲 Generating {} realistic {} records...", count, data_type);
        
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let prompt = format!(
            "Generate {} realistic {} records for API testing. Make the data diverse and realistic.\n\n\
            Return as a JSON array with these requirements:\n\
            - Use realistic names, emails, addresses, etc.\n\
            - Include edge cases (empty strings, special characters, long values)\n\
            - Make data suitable for testing APIs\n\
            - Include different data types (strings, numbers, booleans, dates)\n\
            - Ensure data is valid but diverse\n\n\
            For users: include id, name, email, age, address, phone, registration_date\n\
            For products: include id, name, price, category, description, in_stock, created_at\n\
            For orders: include id, user_id, products, total, status, order_date\n\n\
            Return only the JSON array, no other text.",
            count, data_type
        );

        let response = ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(2000_usize)
            .build()?
        ).await?;

        if let Some(ContentBlock::Text { text }) = response.content.first() {
            // Try to parse as JSON
            if let Ok(data) = serde_json::from_str::<Value>(text) {
                println!("\n✅ Generated test data:");
                println!("{}", serde_json::to_string_pretty(&data)?);
                
                // Save to file for reuse
                let filename = format!("nuts_generated_{}_{}.json", data_type, count);
                std::fs::write(&filename, serde_json::to_string_pretty(&data)?)?;
                println!("\n💾 Saved to: {}", filename);
                
                // Show usage examples
                println!("\n🚀 Usage examples:");
                println!("  call POST https://api.example.com/{} @{}", data_type, filename);
                println!("  cat {} | jq '.[0]'", filename);
                
            } else {
                // Fallback - show as text
                println!("📄 Generated data:\n{}", text);
            }
        }

        Ok(())
    }

    /// Generate data for specific API endpoint testing
    pub async fn generate_for_endpoint(&self, endpoint: &str, method: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let prompt = format!(
            "Generate realistic test data for this API endpoint:\n\n\
            Method: {}\n\
            Endpoint: {}\n\n\
            Based on the endpoint path and method, generate appropriate test data:\n\
            - For POST/PUT: Generate request body data\n\
            - For GET: Generate query parameters if needed\n\
            - Make the data realistic and suitable for testing\n\
            - Include edge cases and variations\n\n\
            Return as JSON object with the test data.",
            method, endpoint
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
            if let Ok(data) = serde_json::from_str::<Value>(text) {
                return Ok(data);
            }
        }

        // Fallback to basic data generation
        Ok(serde_json::json!({
            "test_data": "generated",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}