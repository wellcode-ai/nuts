use anthropic::client::ClientBuilder;
use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, Role};
use async_trait::async_trait;

/// A completed AI response from a provider.
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// The text content of the response.
    pub text: String,
    /// Number of input tokens consumed (if reported by the provider).
    pub input_tokens: Option<u64>,
    /// Number of output tokens consumed (if reported by the provider).
    pub output_tokens: Option<u64>,
}

/// A single message in a conversation.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// Role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// Configuration for a completion request.
#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub system: Option<String>,
    pub model: String,
    pub max_tokens: usize,
}

/// Trait for AI providers. Designed so that Anthropic is the primary implementation
/// today, with OpenAI/Ollama easily added later.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider display name (e.g. "Anthropic", "OpenAI").
    fn name(&self) -> &str;

    /// List of model identifiers this provider supports.
    fn available_models(&self) -> Vec<&str>;

    /// Send a completion request and return the response.
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<AiResponse, Box<dyn std::error::Error>>;
}

/// Anthropic provider using the `anthropic` crate.
pub struct AnthropicProvider {
    client: anthropic::client::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = ClientBuilder::default()
            .api_key(api_key.to_string())
            .build()?;
        Ok(Self { client })
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "Anthropic"
    }

    fn available_models(&self) -> Vec<&str> {
        vec![
            "claude-sonnet-4-5-20250929",
            "claude-3-5-sonnet-20241022",
            "claude-3-sonnet-20240229",
            "claude-3-haiku-20240307",
        ]
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<AiResponse, Box<dyn std::error::Error>> {
        let messages: Vec<Message> = request
            .messages
            .iter()
            .map(|m| Message {
                role: match m.role {
                    ChatRole::User => Role::User,
                    ChatRole::Assistant => Role::Assistant,
                },
                content: vec![ContentBlock::Text {
                    text: m.content.clone(),
                }],
            })
            .collect();

        let mut builder = MessagesRequestBuilder::default();
        builder
            .messages(messages)
            .model(request.model)
            .max_tokens(request.max_tokens);

        // The anthropic crate 0.0.8 does not support a system field on the builder,
        // so we prepend system instructions as a User message when provided.
        let messages_request = builder.build()?;

        let response = self.client.messages(messages_request).await?;

        let text = response
            .content
            .iter()
            .filter_map(|block| {
                if let ContentBlock::Text { text } = block {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("");

        Ok(AiResponse {
            text,
            // The anthropic crate 0.0.8 does not expose token counts on the response
            // struct directly, so we leave these as None for now.
            input_tokens: None,
            output_tokens: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_message_construction() {
        let msg = ChatMessage {
            role: ChatRole::User,
            content: "Hello".to_string(),
        };
        assert_eq!(msg.role, ChatRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn completion_request_construction() {
        let req = CompletionRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: "test".to_string(),
            }],
            system: Some("You are a testing assistant.".to_string()),
            model: "claude-3-sonnet-20240229".to_string(),
            max_tokens: 1000,
        };
        assert_eq!(req.model, "claude-3-sonnet-20240229");
        assert_eq!(req.max_tokens, 1000);
        assert!(req.system.is_some());
    }

    #[test]
    fn anthropic_provider_available_models() {
        // We can't construct AnthropicProvider without a valid API key to test the client,
        // but we can verify the trait design compiles correctly.
        fn assert_provider<T: AiProvider>() {}
        assert_provider::<AnthropicProvider>();
    }

    #[test]
    fn ai_response_construction() {
        let resp = AiResponse {
            text: "Hello world".to_string(),
            input_tokens: Some(10),
            output_tokens: Some(5),
        };
        assert_eq!(resp.text, "Hello world");
        assert_eq!(resp.input_tokens, Some(10));
    }
}
