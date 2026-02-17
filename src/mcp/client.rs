use rmcp::{
    model::{
        CallToolRequestParam, ClientCapabilities, ClientInfo, GetPromptRequestParam,
        Implementation, ReadResourceRequestParam,
    },
    service::{RunningService, ServiceExt},
    transport::{
        streamable_http_client::StreamableHttpClientTransportConfig, ConfigureCommandExt,
        SseClientTransport, StreamableHttpClientTransport, TokioChildProcess,
    },
    RoleClient,
};
use tokio::process::Command;

use crate::error::NutsError;
use crate::mcp::types::{
    ContentItem, Prompt, PromptArgument, PromptMessage, PromptResult, Resource, ResourceContent,
    ResourceTemplate, ServerCapabilities, Tool, ToolResult, TransportConfig,
};

/// MCP client that wraps the rmcp SDK and provides a high-level interface
/// for connecting to MCP servers, discovering capabilities, and invoking
/// tools, resources, and prompts.
pub struct McpClient {
    service: RunningService<RoleClient, ClientInfo>,
}

impl McpClient {
    // ------------------------------------------------------------------
    // Connection constructors
    // ------------------------------------------------------------------

    /// Connect to an MCP server by spawning a child process and
    /// communicating over stdin/stdout.
    pub async fn connect_stdio(
        command: &str,
        args: &[&str],
        env: &[(String, String)],
    ) -> Result<Self, NutsError> {
        let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let env_owned: Vec<(String, String)> = env.to_vec();
        let transport = TokioChildProcess::new(Command::new(command).configure(|c| {
            for arg in &args_owned {
                c.arg(arg);
            }
            for (key, value) in &env_owned {
                c.env(key, value);
            }
        }))
        .map_err(|e| NutsError::Mcp {
            message: format!("failed to spawn MCP server process: {e}"),
        })?;

        let client_info = Self::client_info();
        let service = client_info
            .serve(transport)
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("MCP handshake failed: {e}"),
            })?;

        Ok(Self { service })
    }

    /// Connect to an MCP server via Server-Sent Events (legacy SSE transport).
    ///
    /// Note: The SSE transport in rmcp 0.8.5 does not support bearer
    /// authentication. If a bearer token is provided, a warning is printed
    /// and the connection proceeds without auth. Use `--http` for
    /// authenticated endpoints instead.
    pub async fn connect_sse(url: &str, bearer: Option<&str>) -> Result<Self, NutsError> {
        if bearer.is_some() {
            eprintln!(
                "Warning: SSE transport does not support bearer auth. \
                 Use --http for authenticated endpoints."
            );
        }

        let transport = SseClientTransport::start(url)
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("SSE connection failed: {e}"),
            })?;

        let client_info = Self::client_info();
        let service = client_info
            .serve(transport)
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("MCP handshake over SSE failed: {e}"),
            })?;

        Ok(Self { service })
    }

    /// Connect to an MCP server via Streamable HTTP (the newest transport).
    pub async fn connect_http(url: &str, bearer: Option<&str>) -> Result<Self, NutsError> {
        let mut config = StreamableHttpClientTransportConfig::with_uri(url);
        if let Some(token) = bearer {
            config = config.auth_header(token);
        }
        let transport = StreamableHttpClientTransport::from_config(config);
        let client_info = Self::client_info();
        let service = client_info
            .serve(transport)
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("MCP handshake over HTTP failed: {e}"),
            })?;

        Ok(Self { service })
    }

    /// Connect using a `TransportConfig` enum (convenience method).
    pub async fn connect(config: &TransportConfig) -> Result<Self, NutsError> {
        match config {
            TransportConfig::Stdio { command, args, env } => {
                let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
                Self::connect_stdio(command, &arg_refs, env).await
            }
            TransportConfig::Sse { url, bearer } => {
                Self::connect_sse(url, bearer.as_deref()).await
            }
            TransportConfig::Http { url, bearer } => {
                Self::connect_http(url, bearer.as_deref()).await
            }
        }
    }

    // ------------------------------------------------------------------
    // Discovery
    // ------------------------------------------------------------------

    /// Discover the full capabilities of the connected MCP server.
    pub async fn discover(&self) -> Result<ServerCapabilities, NutsError> {
        let peer = self.service.peer_info();

        let (server_name, server_version, protocol_version) = match peer {
            Some(info) => (
                info.server_info.name.clone(),
                info.server_info.version.clone(),
                format!("{:?}", info.protocol_version),
            ),
            None => (
                "unknown".to_string(),
                "unknown".to_string(),
                "unknown".to_string(),
            ),
        };

        let tools = self.list_tools().await?;
        let resources = self.list_resources().await?;
        let resource_templates = self.list_resource_templates().await?;
        let prompts = self.list_prompts().await?;

        Ok(ServerCapabilities {
            server_name,
            server_version,
            protocol_version,
            tools,
            resources,
            resource_templates,
            prompts,
        })
    }

    // ------------------------------------------------------------------
    // Tools
    // ------------------------------------------------------------------

    /// List all tools available on the server (handles pagination internally).
    pub async fn list_tools(&self) -> Result<Vec<Tool>, NutsError> {
        let rmcp_tools = self
            .service
            .list_all_tools()
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("tools/list failed: {e}"),
            })?;

        Ok(rmcp_tools
            .into_iter()
            .map(|t| Tool {
                name: t.name.to_string(),
                description: t.description.map(|d| d.to_string()),
                input_schema: Some(serde_json::to_value(&*t.input_schema).unwrap_or_default()),
            })
            .collect())
    }

    /// Call a tool by name with the given JSON arguments.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<ToolResult, NutsError> {
        let arguments = args.as_object().cloned();
        let tool_name: String = name.to_string();
        let result = self
            .service
            .call_tool(CallToolRequestParam {
                name: tool_name.into(),
                arguments,
            })
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("tools/call '{name}' failed: {e}"),
            })?;

        Ok(ToolResult {
            is_error: result.is_error.unwrap_or(false),
            content: result
                .content
                .into_iter()
                .map(|c| convert_content(&c.raw))
                .collect(),
        })
    }

    // ------------------------------------------------------------------
    // Resources
    // ------------------------------------------------------------------

    /// List all resources available on the server (handles pagination).
    pub async fn list_resources(&self) -> Result<Vec<Resource>, NutsError> {
        let rmcp_resources =
            self.service
                .list_all_resources()
                .await
                .map_err(|e| NutsError::Mcp {
                    message: format!("resources/list failed: {e}"),
                })?;

        Ok(rmcp_resources
            .into_iter()
            .map(|r| Resource {
                uri: r.uri.clone(),
                name: r.name.clone(),
                description: r.description.clone(),
                mime_type: r.mime_type.clone(),
            })
            .collect())
    }

    /// List all resource templates available on the server.
    pub async fn list_resource_templates(&self) -> Result<Vec<ResourceTemplate>, NutsError> {
        let rmcp_templates = self
            .service
            .list_all_resource_templates()
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("resources/templates/list failed: {e}"),
            })?;

        Ok(rmcp_templates
            .into_iter()
            .map(|t| ResourceTemplate {
                uri_template: t.uri_template.clone(),
                name: t.name.clone(),
                description: t.description.clone(),
                mime_type: t.mime_type.clone(),
            })
            .collect())
    }

    /// Read a resource by URI.
    pub async fn read_resource(&self, uri: &str) -> Result<ResourceContent, NutsError> {
        let result = self
            .service
            .read_resource(ReadResourceRequestParam { uri: uri.into() })
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("resources/read '{uri}' failed: {e}"),
            })?;

        let contents = result
            .contents
            .into_iter()
            .map(|rc| match rc {
                rmcp::model::ResourceContents::TextResourceContents { text, .. } => {
                    ContentItem::Text { text }
                }
                rmcp::model::ResourceContents::BlobResourceContents {
                    blob, mime_type, ..
                } => ContentItem::Image {
                    data: blob,
                    mime_type: mime_type.unwrap_or_default(),
                },
            })
            .collect();

        Ok(ResourceContent {
            uri: uri.to_string(),
            contents,
        })
    }

    // ------------------------------------------------------------------
    // Prompts
    // ------------------------------------------------------------------

    /// List all prompts available on the server (handles pagination).
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, NutsError> {
        let rmcp_prompts = self
            .service
            .list_all_prompts()
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("prompts/list failed: {e}"),
            })?;

        Ok(rmcp_prompts
            .into_iter()
            .map(|p| Prompt {
                name: p.name.clone(),
                description: p.description.clone(),
                arguments: p
                    .arguments
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| PromptArgument {
                        name: a.name.clone(),
                        description: a.description.clone(),
                        required: a.required.unwrap_or(false),
                    })
                    .collect(),
            })
            .collect())
    }

    /// Get a prompt by name with optional arguments.
    pub async fn get_prompt(
        &self,
        name: &str,
        args: Option<serde_json::Value>,
    ) -> Result<PromptResult, NutsError> {
        let arguments = args.and_then(|v| v.as_object().cloned());
        let result = self
            .service
            .get_prompt(GetPromptRequestParam {
                name: name.into(),
                arguments,
            })
            .await
            .map_err(|e| NutsError::Mcp {
                message: format!("prompts/get '{name}' failed: {e}"),
            })?;

        Ok(PromptResult {
            description: result.description.map(|d| d.to_string()),
            messages: result
                .messages
                .into_iter()
                .map(|m| PromptMessage {
                    role: format!("{:?}", m.role),
                    content: convert_prompt_content(m.content),
                })
                .collect(),
        })
    }

    // ------------------------------------------------------------------
    // Lifecycle
    // ------------------------------------------------------------------

    /// Gracefully disconnect from the MCP server.
    pub async fn disconnect(self) -> Result<(), NutsError> {
        self.service.cancel().await.map_err(|e| NutsError::Mcp {
            message: format!("disconnect failed: {e}"),
        })?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Build the ClientInfo used for initialization handshakes.
    fn client_info() -> ClientInfo {
        ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "nuts".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                website_url: None,
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Content conversion helpers
// ---------------------------------------------------------------------------

/// Convert an rmcp `RawContent` to our `ContentItem`.
fn convert_content(raw: &rmcp::model::RawContent) -> ContentItem {
    match raw {
        rmcp::model::RawContent::Text(t) => ContentItem::Text {
            text: t.text.clone(),
        },
        rmcp::model::RawContent::Image(i) => ContentItem::Image {
            data: i.data.clone(),
            mime_type: i.mime_type.clone(),
        },
        rmcp::model::RawContent::Audio(a) => ContentItem::Audio {
            data: a.data.clone(),
            mime_type: a.mime_type.clone(),
        },
        rmcp::model::RawContent::Resource(r) => {
            let text = match &r.resource {
                rmcp::model::ResourceContents::TextResourceContents { text, .. } => text.clone(),
                rmcp::model::ResourceContents::BlobResourceContents { blob, .. } => blob.clone(),
            };
            ContentItem::Text { text }
        }
        rmcp::model::RawContent::ResourceLink(link) => ContentItem::Resource {
            uri: link.uri.clone(),
            text: link.name.clone(),
        },
    }
}

/// Convert prompt message content to our `ContentItem`.
fn convert_prompt_content(content: rmcp::model::PromptMessageContent) -> ContentItem {
    match content {
        rmcp::model::PromptMessageContent::Text { text } => ContentItem::Text { text },
        rmcp::model::PromptMessageContent::Image { image } => ContentItem::Image {
            data: image.data.clone(),
            mime_type: image.mime_type.clone(),
        },
        rmcp::model::PromptMessageContent::Resource { resource } => {
            let text = match &resource.raw.resource {
                rmcp::model::ResourceContents::TextResourceContents { text, .. } => text.clone(),
                rmcp::model::ResourceContents::BlobResourceContents { blob, .. } => blob.clone(),
            };
            ContentItem::Text { text }
        }
        rmcp::model::PromptMessageContent::ResourceLink { link } => ContentItem::Resource {
            uri: link.uri.clone(),
            text: link.name.clone(),
        },
    }
}
