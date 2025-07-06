use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use crate::commands::call::CallCommand;
use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use std::collections::HashMap;
use crate::flows::{OpenAPISpec, PathItem, Operation, RequestBody, Response, MediaType, Schema};
use url::Url;

#[allow(dead_code)]
pub struct StoryMode {
    flow: String,
    api_key: String,
}

#[allow(dead_code)]
impl StoryMode {
    pub fn new(flow: String, api_key: String) -> Self {
        Self { flow, api_key }
    }

    pub async fn start(&self, editor: &mut rustyline::Editor<crate::completer::NutsCompleter, rustyline::history::DefaultHistory>) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🎬 {}", style("Story Mode").cyan().bold());
        println!("AI-guided API workflow for flow: {}", style(&self.flow).yellow());
        println!("Type 'exit' to quit story mode\n");

        loop {
            let goal = editor.readline("🎯 What would you like to achieve? ");
            match goal {
                Ok(line) => {
                    if line.trim().eq_ignore_ascii_case("exit") {
                        break;
                    }

                    let spinner = self.show_thinking_spinner();
                    
                    if let Some(suggestion) = self.get_suggestion(&line).await {
                        spinner.finish_with_message("Got it! 🚀");
                        
                        println!("\n📝 {}", style("Suggested workflow:").blue());
                        println!("{}", suggestion);
                        
                        let execute = editor.readline("\n🚀 Execute this workflow? (y/n): ");
                        if let Ok(response) = execute {
                            if response.trim().eq_ignore_ascii_case("y") {
                                println!("\n🏃 Executing workflow...");
                                if let Err(e) = self.execute_flow(&suggestion).await {
                                    println!("❌ Error executing workflow: {}", e);
                                }
                            }
                        }
                    } else {
                        spinner.finish_with_message("Failed to get suggestion");
                        println!("❌ Unable to get AI suggestion. Check your API key.");
                    }
                }
                Err(_) => break,
            }
        }

        Ok(())
    }

    fn show_thinking_spinner(&self) -> ProgressBar {
        let spinner = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_spinner()
                .template("{spinner} Thinking...").unwrap());
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner
    }

    async fn get_suggestion(&self, goal: &str) -> Option<String> {
        let ai_client = ClientBuilder::default()
            .api_key(self.api_key.clone())
            .build()
            .ok()?;

        let prompt = format!(
            "You are an API workflow assistant. Help the user achieve their goal:\n\
            Flow: {}\n\
            User goal: {}\n\n\
            Suggest a sequence of API calls to achieve this goal. For each step:\n\
            1. Provide a brief description\n\
            2. Show the exact HTTP request to execute\n\
            3. Use http://localhost:3000 as the base URL\n\
            4. Format request bodies as valid JSON\n\
            5. Show expected response format\n\n\
            Example format:\n\
            1. Create user account\n\
            POST http://localhost:3000/users\n\
            {{\n  \"name\": \"test\",\n  \"email\": \"test@example.com\"\n}}\n\n\
            2. Get user details\n\
            GET http://localhost:3000/users/123\n\n\
            Keep responses concise and executable. Use only localhost URLs.",
            self.flow, goal
        );

        match ai_client.messages(MessagesRequestBuilder::default()
            .messages(vec![Message {
                role: Role::User,
                content: vec![ContentBlock::Text { text: prompt }],
            }])
            .model("claude-3-sonnet-20240229".to_string())
            .max_tokens(2000_usize)
            .build()
            .ok()?
        ).await {
            Ok(response) => response.content.first()
                .and_then(|block| {
                    if let ContentBlock::Text { text } = block {
                        Some(text.clone())
                    } else {
                        None
                    }
                }),
            Err(_) => None
        }
    }

    async fn execute_flow(&self, flow: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Skip if input is just "y" (from previous prompt)
        if flow.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }

        let steps: Vec<&str> = flow.lines()
            .filter(|line| line.contains("curl") || line.contains("http"))
            .collect();

        if steps.is_empty() {
            println!("No executable steps found in the flow");
            return Ok(());
        }

        for (i, step) in steps.iter().enumerate() {
            println!("\n📍 Step {}/{}", i + 1, steps.len());
            
            if let Some(url) = step.find("http") {
                let url_end = step[url..].find(' ').unwrap_or(step.len() - url);
                let url = &step[url..url + url_end];
                
                let method = if step.contains("POST") {
                    "POST"
                } else if step.contains("PUT") {
                    "PUT"
                } else if step.contains("DELETE") {
                    "DELETE"
                } else {
                    "GET"
                };

                let body = if step.contains("'{") {
                    step.rfind("'{").map(|i| &step[i + 1..step.len() - 1])
                } else {
                    None
                };

                println!("Executing {} {}", style(method).cyan(), style(url).green());
                CallCommand::new().execute(&[method, url, body.unwrap_or("")]).await?;
            }
        }

        self.save_story(&flow).await?;
        Ok(())
    }

    async fn save_story(&self, flow: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut paths = HashMap::new();
        let mut current_path = None;
        let mut current_method = None;
        let mut description = String::new();

        for line in flow.lines() {
            if line.starts_with(|c: char| c.is_digit(10)) {
                // Start of new step - capture description
                description = line.splitn(2, '.').nth(1)
                    .unwrap_or("").trim().to_string();
            } else if line.contains("http") {
                // Parse method and path
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    current_method = Some(parts[0].to_uppercase());
                    if let Ok(url) = Url::parse(parts[1]) {
                        current_path = Some(url.path().to_string());
                    }
                }
            } else if line.starts_with('{') && current_path.is_some() && current_method.is_some() {
                // Found request body - create operation
                let path = current_path.take().unwrap();
                let method = current_method.take().unwrap();
                
                let path_item = paths.entry(path).or_insert(PathItem::new());
                let operation = Operation {
                    summary: Some(description.clone()),
                    description: Some("Generated from Story Mode".to_string()),
                    parameters: None,
                    request_body: if line.trim().is_empty() {
                        None
                    } else {
                        Some(RequestBody {
                            description: Some("Request payload".to_string()),
                            required: Some(true),
                            content: {
                                let mut content = HashMap::new();
                                content.insert("application/json".to_string(), MediaType {
                                    schema: Schema {
                                        schema_type: "object".to_string(),
                                        format: None,
                                        properties: None,
                                        items: None,
                                    },
                                    example: serde_json::from_str(line).ok(),
                                });
                                content
                            },
                        })
                    },
                    responses: {
                        let mut responses = HashMap::new();
                        responses.insert("200".to_string(), Response {
                            description: "Successful response".to_string(),
                            content: None,
                        });
                        responses
                    },
                    ..Default::default()
                };

                match method.as_str() {
                    "GET" => path_item.get = Some(operation),
                    "POST" => path_item.post = Some(operation),
                    "PUT" => path_item.put = Some(operation),
                    "DELETE" => path_item.delete = Some(operation),
                    "PATCH" => path_item.patch = Some(operation),
                    _ => {}
                }
            }
        }

        // Save to flow file
        let spec_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".nuts")
            .join("flows")
            .join(format!("{}.yaml", self.flow));

        let mut spec = OpenAPISpec::load(&spec_path)?;
        spec.paths.extend(paths);
        spec.save(&spec_path)?;

        println!("\n✅ Saved API flow to flow {}", style(&self.flow).green());
        Ok(())
    }
} 
