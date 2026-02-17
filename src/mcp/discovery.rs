use crate::error::NutsError;
use crate::mcp::client::McpClient;
use crate::mcp::types::ServerCapabilities;

/// Connect to an MCP server, discover its capabilities, and disconnect.
///
/// This is a convenience function that performs the full discovery lifecycle:
/// connect -> discover -> disconnect.
pub async fn discover(client: &McpClient) -> Result<ServerCapabilities, NutsError> {
    client.discover().await
}

/// Format discovered capabilities as a human-readable string for terminal output.
pub fn format_discovery_human(caps: &ServerCapabilities) -> String {
    let mut out = String::new();

    out.push_str(&format!("MCP Server: {}\n", caps.server_name));
    out.push_str(&format!("Version:    {}\n", caps.server_version));
    out.push_str(&format!("Protocol:   {}\n", caps.protocol_version));
    out.push('\n');

    // Tools
    if caps.tools.is_empty() {
        out.push_str("Tools: (none)\n");
    } else {
        out.push_str(&format!("Tools ({}):\n", caps.tools.len()));
        for tool in &caps.tools {
            let desc = tool.description.as_deref().unwrap_or("(no description)");
            out.push_str(&format!("  {:<24}{}\n", tool.name, desc));

            // Show parameters from input_schema if available
            if let Some(schema) = &tool.input_schema {
                if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
                    let required: Vec<String> = schema
                        .get("required")
                        .and_then(|r| r.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();

                    for (param_name, param_schema) in props {
                        let param_type = param_schema
                            .get("type")
                            .and_then(|t| t.as_str())
                            .unwrap_or("any");
                        let req_str = if required.contains(param_name) {
                            "required"
                        } else {
                            "optional"
                        };
                        let param_desc = param_schema
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("");
                        out.push_str(&format!(
                            "    - {}: {} ({})  {}\n",
                            param_name, param_type, req_str, param_desc
                        ));
                    }
                }
            }
        }
    }
    out.push('\n');

    // Resources
    if caps.resources.is_empty() && caps.resource_templates.is_empty() {
        out.push_str("Resources: (none)\n");
    } else {
        let total = caps.resources.len() + caps.resource_templates.len();
        out.push_str(&format!("Resources ({}):\n", total));
        for resource in &caps.resources {
            let desc = resource
                .description
                .as_deref()
                .unwrap_or("(no description)");
            out.push_str(&format!("  {:<24}{}\n", resource.uri, desc));
        }
        for template in &caps.resource_templates {
            let desc = template
                .description
                .as_deref()
                .unwrap_or("(no description)");
            out.push_str(&format!(
                "  {:<24}{} (template)\n",
                template.uri_template, desc
            ));
        }
    }
    out.push('\n');

    // Prompts
    if caps.prompts.is_empty() {
        out.push_str("Prompts: (none)\n");
    } else {
        out.push_str(&format!("Prompts ({}):\n", caps.prompts.len()));
        for prompt in &caps.prompts {
            let desc = prompt.description.as_deref().unwrap_or("(no description)");
            out.push_str(&format!("  {:<24}{}\n", prompt.name, desc));

            for arg in &prompt.arguments {
                let req_str = if arg.required { "required" } else { "optional" };
                let arg_desc = arg.description.as_deref().unwrap_or("");
                out.push_str(&format!(
                    "    - {}: ({})  {}\n",
                    arg.name, req_str, arg_desc
                ));
            }
        }
    }

    out
}

/// Format discovered capabilities as a JSON value for machine-readable output.
pub fn format_discovery_json(caps: &ServerCapabilities) -> serde_json::Value {
    serde_json::to_value(caps).unwrap_or_else(|_| serde_json::json!({}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::*;

    fn sample_caps() -> ServerCapabilities {
        ServerCapabilities {
            server_name: "test-server".into(),
            server_version: "0.1.0".into(),
            protocol_version: "2025-03-26".into(),
            tools: vec![
                Tool {
                    name: "search_documents".into(),
                    description: Some("Search the document database".into()),
                    input_schema: Some(serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "limit": {
                                "type": "number",
                                "description": "Max results"
                            }
                        },
                        "required": ["query"]
                    })),
                },
                Tool {
                    name: "get_stats".into(),
                    description: Some("Get database statistics".into()),
                    input_schema: None,
                },
            ],
            resources: vec![Resource {
                uri: "documents://recent".into(),
                name: "recent".into(),
                description: Some("Recently modified documents".into()),
                mime_type: Some("application/json".into()),
            }],
            resource_templates: vec![ResourceTemplate {
                uri_template: "documents://{id}".into(),
                name: "document".into(),
                description: Some("A single document by ID".into()),
                mime_type: None,
            }],
            prompts: vec![Prompt {
                name: "summarize".into(),
                description: Some("Summarize a document".into()),
                arguments: vec![PromptArgument {
                    name: "document_id".into(),
                    description: Some("ID of the document to summarize".into()),
                    required: true,
                }],
            }],
        }
    }

    #[test]
    fn human_format_contains_server_info() {
        let caps = sample_caps();
        let output = format_discovery_human(&caps);
        assert!(output.contains("MCP Server: test-server"));
        assert!(output.contains("Protocol:   2025-03-26"));
    }

    #[test]
    fn human_format_lists_tools() {
        let caps = sample_caps();
        let output = format_discovery_human(&caps);
        assert!(output.contains("Tools (2):"));
        assert!(output.contains("search_documents"));
        assert!(output.contains("query: string (required)"));
        assert!(output.contains("limit: number (optional)"));
        assert!(output.contains("get_stats"));
    }

    #[test]
    fn human_format_lists_resources() {
        let caps = sample_caps();
        let output = format_discovery_human(&caps);
        assert!(output.contains("Resources (2):"));
        assert!(output.contains("documents://recent"));
        assert!(output.contains("documents://{id}"));
        assert!(output.contains("(template)"));
    }

    #[test]
    fn human_format_lists_prompts() {
        let caps = sample_caps();
        let output = format_discovery_human(&caps);
        assert!(output.contains("Prompts (1):"));
        assert!(output.contains("summarize"));
        assert!(output.contains("document_id"));
        assert!(output.contains("(required)"));
    }

    #[test]
    fn human_format_handles_empty_server() {
        let caps = ServerCapabilities {
            server_name: "empty".into(),
            server_version: "0.0.0".into(),
            protocol_version: "2025-03-26".into(),
            tools: vec![],
            resources: vec![],
            resource_templates: vec![],
            prompts: vec![],
        };
        let output = format_discovery_human(&caps);
        assert!(output.contains("Tools: (none)"));
        assert!(output.contains("Resources: (none)"));
        assert!(output.contains("Prompts: (none)"));
    }

    #[test]
    fn json_format_roundtrips() {
        let caps = sample_caps();
        let json = format_discovery_json(&caps);
        assert_eq!(json["server_name"], "test-server");
        assert_eq!(json["tools"].as_array().unwrap().len(), 2);
        assert_eq!(json["resources"].as_array().unwrap().len(), 1);
        assert_eq!(json["prompts"].as_array().unwrap().len(), 1);
    }
}
