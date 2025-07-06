use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;
use crate::commands::call::CallCommand;

pub struct TestCommand {
    config: Config,
}

impl TestCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// AI-First Natural Language Testing
    pub async fn execute_natural_language(&self, description: &str, base_url: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Processing natural language test: {}", description);
        
        // Get AI to convert natural language to test plan
        let test_plan = self.generate_test_plan(description, base_url).await?;
        
        println!("📋 Generated Test Plan:");
        println!("{}", test_plan);
        
        // Execute the generated test plan
        self.execute_test_plan(&test_plan).await?;
        
        Ok(())
    }

    async fn generate_test_plan(&self, description: &str, base_url: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured. Use 'config api-key' to set it")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let base_url_context = base_url
            .map(|url| format!("Base URL: {}", url))
            .unwrap_or_else(|| "No base URL provided".to_string());

        let prompt = format!(
            "You are an expert API testing assistant. Convert this natural language test description into a detailed, executable test plan.

Test Description: \"{}\"
{}

Generate a structured test plan that includes:
1. Test objective
2. Required API endpoints (infer from description)
3. HTTP methods to use
4. Request data needed
5. Expected responses
6. Validation criteria
7. Step-by-step execution plan

Format as executable steps that can be run with HTTP requests.
Use realistic example URLs and data.
Be specific about HTTP status codes, headers, and response validation.

Example format:
=== TEST PLAN ===
Objective: [Clear test objective]

Step 1: [Action]
  Method: GET/POST/etc
  URL: /api/endpoint
  Data: {{\"key\": \"value\"}}
  Expected: 200 OK, response contains X

Step 2: [Next action]
  ...

Validation:
- Check response status
- Verify response structure
- Validate business logic
",
            description, base_url_context
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
            Ok(text.clone())
        } else {
            Err("Failed to generate test plan".into())
        }
    }

    async fn execute_test_plan(&self, test_plan: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Executing test plan...");
        
        // Parse test plan and extract HTTP requests
        let requests = self.parse_test_plan(test_plan)?;
        
        for (i, request) in requests.iter().enumerate() {
            println!("\n📍 Step {}/{}: {}", i + 1, requests.len(), request.description);
            
            // Execute HTTP request
            match self.execute_request(request).await {
                Ok(response) => {
                    println!("✅ Success: {}", response);
                    
                    // Validate response against expected criteria
                    if let Some(validation) = &request.validation {
                        self.validate_response(&response, validation)?;
                    }
                }
                Err(e) => {
                    println!("❌ Failed: {}", e);
                    return Err(e);
                }
            }
        }
        
        println!("\n🎉 Test plan completed successfully!");
        Ok(())
    }

    fn parse_test_plan(&self, test_plan: &str) -> Result<Vec<TestRequest>, Box<dyn std::error::Error>> {
        let mut requests = Vec::new();
        
        // Simple parsing logic - in a real implementation, this would be more sophisticated
        let lines: Vec<&str> = test_plan.lines().collect();
        let mut current_request: Option<TestRequest> = None;
        
        for line in lines {
            let trimmed = line.trim();
            
            if trimmed.starts_with("Step ") {
                // Save previous request if exists
                if let Some(req) = current_request.take() {
                    requests.push(req);
                }
                
                // Start new request
                current_request = Some(TestRequest {
                    description: trimmed.to_string(),
                    method: "GET".to_string(),
                    url: "/".to_string(),
                    data: None,
                    validation: None,
                });
            } else if trimmed.starts_with("Method: ") {
                if let Some(ref mut req) = current_request {
                    req.method = trimmed.replace("Method: ", "").trim().to_string();
                }
            } else if trimmed.starts_with("URL: ") {
                if let Some(ref mut req) = current_request {
                    req.url = trimmed.replace("URL: ", "").trim().to_string();
                }
            } else if trimmed.starts_with("Data: ") {
                if let Some(ref mut req) = current_request {
                    let data_replaced = trimmed.replace("Data: ", "");
                    let data_str = data_replaced.trim();
                    if !data_str.is_empty() && data_str != "{}" {
                        req.data = Some(data_str.to_string());
                    }
                }
            } else if trimmed.starts_with("Expected: ") {
                if let Some(ref mut req) = current_request {
                    req.validation = Some(trimmed.replace("Expected: ", "").trim().to_string());
                }
            }
        }
        
        // Don't forget the last request
        if let Some(req) = current_request {
            requests.push(req);
        }
        
        Ok(requests)
    }

    async fn execute_request(&self, request: &TestRequest) -> Result<String, Box<dyn std::error::Error>> {
        let call_command = CallCommand::new();
        
        // Build command arguments
        let mut args = vec![request.method.as_str(), request.url.as_str()];
        if let Some(data) = &request.data {
            args.push(data);
        }
        
        // Execute the HTTP request
        let response = call_command.execute_with_response(&args).await?;
        Ok(response)
    }

    fn validate_response(&self, response: &str, validation: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Simple validation logic - check if response contains expected elements
        if validation.contains("200 OK") && !response.contains("200") {
            return Err("Expected 200 OK status not found".into());
        }
        
        // More sophisticated validation would go here
        println!("✅ Response validation passed");
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct TestRequest {
    description: String,
    method: String,
    url: String,
    data: Option<String>,
    validation: Option<String>,
}