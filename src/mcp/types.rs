use serde::{Deserialize, Serialize};

/// Capabilities discovered from an MCP server via the initialize handshake
/// followed by tools/list, resources/list, and prompts/list calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Server name as reported during initialization.
    pub server_name: String,
    /// Server version as reported during initialization.
    pub server_version: String,
    /// MCP protocol version negotiated during initialization.
    pub protocol_version: String,
    /// Tools exposed by the server.
    pub tools: Vec<Tool>,
    /// Resources exposed by the server.
    pub resources: Vec<Resource>,
    /// Resource templates exposed by the server.
    pub resource_templates: Vec<ResourceTemplate>,
    /// Prompts exposed by the server.
    pub prompts: Vec<Prompt>,
}

/// An MCP tool with its name, description, and JSON Schema for input parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema describing the tool's expected input parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// An MCP resource identified by a URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// An MCP resource template with a URI pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// An MCP prompt with optional arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<PromptArgument>,
}

/// A single argument for an MCP prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptArgument {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
}

/// The result of calling an MCP tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool call was an error.
    #[serde(default)]
    pub is_error: bool,
    /// Content items returned by the tool.
    pub content: Vec<ContentItem>,
}

/// A single content item in a tool result or resource read.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ContentItem {
    Text { text: String },
    Image { data: String, mime_type: String },
    Audio { data: String, mime_type: String },
    Resource { uri: String, text: String },
}

impl ContentItem {
    /// Extract the text content if this is a text item.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentItem::Text { text } => Some(text),
            _ => None,
        }
    }
}

/// The content of a resource after reading it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    pub uri: String,
    pub contents: Vec<ContentItem>,
}

/// The result of fetching a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub messages: Vec<PromptMessage>,
}

/// A single message in a prompt result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: ContentItem,
}

/// Which transport mechanism to use when connecting to an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// Spawn a child process and communicate over stdin/stdout.
    Stdio {
        command: String,
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        env: Vec<(String, String)>,
    },
    /// Connect via Server-Sent Events (legacy SSE transport).
    Sse {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bearer: Option<String>,
    },
    /// Connect via Streamable HTTP (the newest MCP transport).
    Http {
        url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bearer: Option<String>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_serializes_to_json() {
        let tool = Tool {
            name: "search".into(),
            description: Some("Search documents".into()),
            input_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            })),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("search"));
        assert!(json.contains("query"));
    }

    #[test]
    fn content_item_as_text() {
        let item = ContentItem::Text {
            text: "hello".into(),
        };
        assert_eq!(item.as_text(), Some("hello"));

        let img = ContentItem::Image {
            data: "abc".into(),
            mime_type: "image/png".into(),
        };
        assert_eq!(img.as_text(), None);
    }

    #[test]
    fn transport_config_roundtrips() {
        let cfg = TransportConfig::Stdio {
            command: "node".into(),
            args: vec!["server.js".into()],
            env: vec![],
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: TransportConfig = serde_json::from_str(&json).unwrap();
        match parsed {
            TransportConfig::Stdio { command, args, .. } => {
                assert_eq!(command, "node");
                assert_eq!(args, vec!["server.js"]);
            }
            _ => panic!("expected Stdio"),
        }
    }

    #[test]
    fn server_capabilities_serializes() {
        let caps = ServerCapabilities {
            server_name: "test-server".into(),
            server_version: "1.0.0".into(),
            protocol_version: "2025-03-26".into(),
            tools: vec![Tool {
                name: "echo".into(),
                description: Some("Echo input".into()),
                input_schema: None,
            }],
            resources: vec![],
            resource_templates: vec![],
            prompts: vec![],
        };
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json["server_name"], "test-server");
        assert_eq!(json["tools"].as_array().unwrap().len(), 1);
    }
}
