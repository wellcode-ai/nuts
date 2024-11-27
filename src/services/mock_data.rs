use anthropic::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};

pub struct MockDataGenerator {
    client: AnthropicClient,
}

impl MockDataGenerator {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: ClientBuilder::default()
                .api_key(api_key.to_string())
                .build()
                .unwrap(),
        }
    }

    pub async fn generate_mock_data(&self, config: &MockDataConfig, count: usize) 
        -> Result<Vec<String>, Box<dyn std::error::Error>> 
    {
        let prompt = format!(
            "Generate {} unique JSON mock data items based on this description: {}. 
             {}
             {}
             Return only valid JSON array, no explanations.",
            count,
            config.description,
            config.schema.as_ref().map(|s| format!("\nSchema: {}", s)).unwrap_or_default(),
            config.examples.as_ref().map(|e| format!("\nExamples: {}", e.join("\n"))).unwrap_or_default()
        );

        let messages = vec![Message {
            role: Role::User,
            content: vec![ContentBlock::Text { text: prompt }],
        }];

        let request = MessagesRequestBuilder::default()
            .messages(messages)
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(1000_usize)
            .build()?;

        let response = self.client.messages(request).await?;
        
        if let ContentBlock::Text { text } = &response.content[0] {
            let mock_data: Vec<String> = serde_json::from_str(text)?;
            Ok(mock_data)
        } else {
            Err("Unexpected response format".into())
        }
    }
}
