use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::ai::provider::{
    AiProvider, AiResponse, AnthropicProvider, ChatMessage, ChatRole, CompletionRequest,
};

/// Default model to use for AI requests.
const DEFAULT_MODEL: &str = "claude-sonnet-4-5-20250929";

/// Maximum number of recent messages to retain for conversational context.
const MAX_CONVERSATION_BUFFER: usize = 20;

/// Centralized AI service that replaces per-command Anthropic client construction.
///
/// Holds a single provider instance, tracks token usage, and maintains a
/// conversation buffer for context-aware interactions.
pub struct AiService {
    provider: Box<dyn AiProvider>,
    model: String,
    input_tokens_used: AtomicU64,
    output_tokens_used: AtomicU64,
    conversation_buffer: Mutex<VecDeque<ChatMessage>>,
}

impl AiService {
    /// Create a new AiService with an Anthropic provider.
    pub fn new(api_key: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let provider = AnthropicProvider::new(api_key)?;
        Ok(Self {
            provider: Box::new(provider),
            model: DEFAULT_MODEL.to_string(),
            input_tokens_used: AtomicU64::new(0),
            output_tokens_used: AtomicU64::new(0),
            conversation_buffer: Mutex::new(VecDeque::with_capacity(MAX_CONVERSATION_BUFFER)),
        })
    }

    /// Create an AiService with a custom provider (useful for testing or alternative backends).
    #[allow(dead_code)]
    pub fn with_provider(provider: Box<dyn AiProvider>) -> Self {
        Self {
            provider,
            model: DEFAULT_MODEL.to_string(),
            input_tokens_used: AtomicU64::new(0),
            output_tokens_used: AtomicU64::new(0),
            conversation_buffer: Mutex::new(VecDeque::with_capacity(MAX_CONVERSATION_BUFFER)),
        }
    }

    /// Override the default model.
    #[allow(dead_code)]
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Get the provider name.
    #[allow(dead_code)]
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }

    /// Get the current model identifier.
    #[allow(dead_code)]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get total input tokens used this session.
    #[allow(dead_code)]
    pub fn input_tokens_used(&self) -> u64 {
        self.input_tokens_used.load(Ordering::Relaxed)
    }

    /// Get total output tokens used this session.
    #[allow(dead_code)]
    pub fn output_tokens_used(&self) -> u64 {
        self.output_tokens_used.load(Ordering::Relaxed)
    }

    /// Send a one-shot completion request (no conversation context).
    pub async fn complete(
        &self,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<AiResponse, Box<dyn std::error::Error>> {
        let request = CompletionRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: prompt.to_string(),
            }],
            system: None,
            model: self.model.clone(),
            max_tokens,
        };

        let response = self.provider.complete(request).await?;
        self.track_tokens(&response);
        Ok(response)
    }

    /// Send a completion request with a system prompt and user message.
    pub async fn complete_with_system(
        &self,
        system: &str,
        user_message: &str,
        max_tokens: usize,
    ) -> Result<AiResponse, Box<dyn std::error::Error>> {
        // Since the anthropic crate 0.0.8 doesn't support a system field,
        // we prepend the system prompt as part of the user message.
        let combined = format!("{}\n\n{}", system, user_message);
        let request = CompletionRequest {
            messages: vec![ChatMessage {
                role: ChatRole::User,
                content: combined,
            }],
            system: Some(system.to_string()),
            model: self.model.clone(),
            max_tokens,
        };

        let response = self.provider.complete(request).await?;
        self.track_tokens(&response);
        Ok(response)
    }

    /// Send a multi-turn conversation request.
    #[allow(dead_code)]
    pub async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        max_tokens: usize,
    ) -> Result<AiResponse, Box<dyn std::error::Error>> {
        let request = CompletionRequest {
            messages,
            system: None,
            model: self.model.clone(),
            max_tokens,
        };

        let response = self.provider.complete(request).await?;
        self.track_tokens(&response);
        Ok(response)
    }

    /// Send a message in the context of the conversation buffer, then append
    /// both the user message and the AI response to the buffer.
    #[allow(dead_code)]
    pub async fn converse(
        &self,
        user_message: &str,
        max_tokens: usize,
    ) -> Result<AiResponse, Box<dyn std::error::Error>> {
        let mut messages = {
            let buffer = self.conversation_buffer.lock().unwrap();
            buffer.iter().cloned().collect::<Vec<_>>()
        };

        let user_msg = ChatMessage {
            role: ChatRole::User,
            content: user_message.to_string(),
        };
        messages.push(user_msg.clone());

        let request = CompletionRequest {
            messages,
            system: None,
            model: self.model.clone(),
            max_tokens,
        };

        let response = self.provider.complete(request).await?;
        self.track_tokens(&response);

        // Append to conversation buffer
        {
            let mut buffer = self.conversation_buffer.lock().unwrap();
            buffer.push_back(user_msg);
            buffer.push_back(ChatMessage {
                role: ChatRole::Assistant,
                content: response.text.clone(),
            });
            // Trim if over capacity
            while buffer.len() > MAX_CONVERSATION_BUFFER {
                buffer.pop_front();
            }
        }

        Ok(response)
    }

    /// Clear the conversation buffer.
    #[allow(dead_code)]
    pub fn clear_conversation(&self) {
        let mut buffer = self.conversation_buffer.lock().unwrap();
        buffer.clear();
    }

    // --- High-level convenience methods ---

    /// Generate test cases for an MCP tool using AI.
    pub async fn generate_test_cases(
        &self,
        tool_name: &str,
        tool_description: &str,
        input_schema: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use crate::ai::prompts;

        let tool_info = prompts::McpToolInfo {
            name: tool_name.to_string(),
            description: tool_description.to_string(),
            input_schema: input_schema.to_string(),
        };

        let prompt = prompts::mcp_test_generation(&tool_info);
        let response = self.complete(&prompt, 4000).await?;
        Ok(response.text)
    }

    /// Run an AI-powered security scan for an MCP tool.
    pub async fn security_scan(
        &self,
        tool_name: &str,
        tool_description: &str,
        input_schema: &str,
        previous_results: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use crate::ai::prompts;

        let input = prompts::McpSecurityScanInput {
            tool_name: tool_name.to_string(),
            tool_description: tool_description.to_string(),
            input_schema: input_schema.to_string(),
            previous_results: previous_results.map(|s| s.to_string()),
        };

        let prompt = prompts::mcp_security_scan(&input);
        let response = self.complete(&prompt, 4000).await?;
        Ok(response.text)
    }

    /// Explain an API response in human-friendly terms.
    pub async fn explain(
        &self,
        api_response: &str,
        context: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use crate::ai::prompts;

        let prompt = prompts::explain_response(api_response, context);
        let response = self.complete(&prompt, 1500).await?;
        Ok(response.text)
    }

    /// Validate MCP tool output semantically.
    #[allow(dead_code)]
    pub async fn validate_output(
        &self,
        tool_name: &str,
        tool_description: &str,
        input_sent: &str,
        output_received: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use crate::ai::prompts;

        let input = prompts::McpOutputValidationInput {
            tool_name: tool_name.to_string(),
            tool_description: tool_description.to_string(),
            input_sent: input_sent.to_string(),
            output_received: output_received.to_string(),
        };

        let prompt = prompts::mcp_output_validation(&input);
        let response = self.complete(&prompt, 1500).await?;
        Ok(response.text)
    }

    /// Suggest a command correction for invalid input.
    #[allow(dead_code)]
    pub async fn suggest_command(
        &self,
        invalid_input: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use crate::ai::prompts;

        let prompt = prompts::command_suggestion(invalid_input);
        let response = self.complete(&prompt, 100).await?;
        Ok(response.text)
    }

    fn track_tokens(&self, response: &AiResponse) {
        if let Some(input) = response.input_tokens {
            self.input_tokens_used.fetch_add(input, Ordering::Relaxed);
        }
        if let Some(output) = response.output_tokens {
            self.output_tokens_used.fetch_add(output, Ordering::Relaxed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::provider::{AiProvider, AiResponse, CompletionRequest};
    use async_trait::async_trait;

    /// A mock provider for unit testing that returns canned responses.
    struct MockProvider {
        response_text: String,
    }

    #[async_trait]
    impl AiProvider for MockProvider {
        fn name(&self) -> &str {
            "Mock"
        }

        fn available_models(&self) -> Vec<&str> {
            vec!["mock-model-v1"]
        }

        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<AiResponse, Box<dyn std::error::Error>> {
            Ok(AiResponse {
                text: self.response_text.clone(),
                input_tokens: Some(10),
                output_tokens: Some(20),
            })
        }
    }

    fn mock_service(response: &str) -> AiService {
        AiService::with_provider(Box::new(MockProvider {
            response_text: response.to_string(),
        }))
    }

    #[tokio::test]
    async fn complete_returns_text() {
        let svc = mock_service("Hello from mock");
        let response = svc.complete("test prompt", 100).await.unwrap();
        assert_eq!(response.text, "Hello from mock");
    }

    #[tokio::test]
    async fn token_tracking() {
        let svc = mock_service("response");
        assert_eq!(svc.input_tokens_used(), 0);
        assert_eq!(svc.output_tokens_used(), 0);

        svc.complete("prompt 1", 100).await.unwrap();
        assert_eq!(svc.input_tokens_used(), 10);
        assert_eq!(svc.output_tokens_used(), 20);

        svc.complete("prompt 2", 100).await.unwrap();
        assert_eq!(svc.input_tokens_used(), 20);
        assert_eq!(svc.output_tokens_used(), 40);
    }

    #[tokio::test]
    async fn conversation_buffer() {
        let svc = mock_service("AI response");

        // First turn
        let resp = svc.converse("Hello", 100).await.unwrap();
        assert_eq!(resp.text, "AI response");

        // Buffer should have 2 messages (user + assistant)
        {
            let buffer = svc.conversation_buffer.lock().unwrap();
            assert_eq!(buffer.len(), 2);
            assert_eq!(buffer[0].role, ChatRole::User);
            assert_eq!(buffer[0].content, "Hello");
            assert_eq!(buffer[1].role, ChatRole::Assistant);
            assert_eq!(buffer[1].content, "AI response");
        }

        // Clear
        svc.clear_conversation();
        {
            let buffer = svc.conversation_buffer.lock().unwrap();
            assert_eq!(buffer.len(), 0);
        }
    }

    #[tokio::test]
    async fn with_model_override() {
        let svc = mock_service("ok").with_model("custom-model");
        assert_eq!(svc.model(), "custom-model");
        assert_eq!(svc.provider_name(), "Mock");
    }

    #[tokio::test]
    async fn generate_test_cases_calls_provider() {
        let svc = mock_service(
            "- name: test\n  tool: search\n  input: {}\n  assert:\n    status: success",
        );
        let result = svc
            .generate_test_cases("search", "Search documents", "{}")
            .await
            .unwrap();
        assert!(result.contains("search"));
    }

    #[tokio::test]
    async fn security_scan_calls_provider() {
        let svc = mock_service(r#"[{"category": "injection", "name": "SQL injection"}]"#);
        let result = svc
            .security_scan("query", "Run a query", "{}", None)
            .await
            .unwrap();
        assert!(result.contains("injection"));
    }

    #[tokio::test]
    async fn security_scan_adaptive() {
        let svc = mock_service("adaptive results");
        let result = svc
            .security_scan("query", "Run a query", "{}", Some("previous error"))
            .await
            .unwrap();
        assert_eq!(result, "adaptive results");
    }

    #[tokio::test]
    async fn explain_calls_provider() {
        let svc = mock_service("This is a 200 OK response meaning success.");
        let result = svc
            .explain("{\"status\": 200}", Some("health check"))
            .await
            .unwrap();
        assert!(result.contains("200 OK"));
    }

    #[tokio::test]
    async fn validate_output_calls_provider() {
        let svc = mock_service(
            r#"{"valid": true, "confidence": 0.95, "issues": [], "summary": "Looks good"}"#,
        );
        let result = svc
            .validate_output("get_stats", "Get stats", "{}", r#"{"count": 5}"#)
            .await
            .unwrap();
        assert!(result.contains("valid"));
    }

    #[tokio::test]
    async fn suggest_command_calls_provider() {
        let svc = mock_service("call GET https://example.com");
        let result = svc
            .suggest_command("cal GET https://example.com")
            .await
            .unwrap();
        assert!(result.contains("call GET"));
    }
}
