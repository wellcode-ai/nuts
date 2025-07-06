use anthropic::{
    client::ClientBuilder,
    types::{Message, ContentBlock, MessagesRequestBuilder, Role},
};
use crate::config::Config;
use crate::commands::call::CallCommand;
use serde_json::Value;

pub struct FixCommand {
    config: Config,
}

impl FixCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// AI-powered API fixing - automatically detect and suggest fixes
    pub async fn auto_fix(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔧 AI-powered auto-fix starting for: {}", url);
        
        // Step 1: Diagnose the API
        println!("🔍 Step 1: Diagnosing API issues...");
        let diagnosis = self.diagnose_api(url).await?;
        
        // Step 2: Generate AI-powered fix recommendations
        println!("🧠 Step 2: AI generating fix recommendations...");
        let fixes = self.generate_fixes(&diagnosis).await?;
        
        // Step 3: Present fixes to user
        self.present_fixes(&fixes)?;
        
        // Step 4: Offer to apply automated fixes
        self.offer_automated_fixes(url, &fixes).await?;
        
        Ok(())
    }

    async fn diagnose_api(&self, url: &str) -> Result<ApiDiagnosis, Box<dyn std::error::Error>> {
        let mut diagnosis = ApiDiagnosis {
            url: url.to_string(),
            connectivity_issues: Vec::new(),
            performance_issues: Vec::new(),
            security_issues: Vec::new(),
            response_issues: Vec::new(),
            status_code: 200,
            response_time_ms: 0,
        };

        // Test basic connectivity
        let call_command = CallCommand::new();
        let start_time = std::time::SystemTime::now();
        
        match call_command.execute_with_response(&["GET", url]).await {
            Ok(response) => {
                let response_time = start_time.elapsed()?.as_millis();
                diagnosis.response_time_ms = response_time;
                
                // Check performance
                if response_time > 2000 {
                    diagnosis.performance_issues.push("Very slow response time".to_string());
                } else if response_time > 1000 {
                    diagnosis.performance_issues.push("Slow response time".to_string());
                }
                
                // Check response content
                if response.is_empty() {
                    diagnosis.response_issues.push("Empty response body".to_string());
                }
                
                if response.contains("error") || response.contains("Error") {
                    diagnosis.response_issues.push("Response contains error messages".to_string());
                }
                
                // Try to parse as JSON
                if let Err(_) = serde_json::from_str::<Value>(&response) {
                    if !response.trim().starts_with('<') { // Not HTML
                        diagnosis.response_issues.push("Invalid JSON response".to_string());
                    }
                }
            }
            Err(e) => {
                diagnosis.connectivity_issues.push(format!("Connection failed: {}", e));
            }
        }

        // Check security (simplified)
        if !url.starts_with("https://") {
            diagnosis.security_issues.push("Not using HTTPS".to_string());
        }

        // Test common problematic endpoints
        for test_path in &["/admin", "/.env", "/debug", "/test"] {
            let test_url = format!("{}{}", url.trim_end_matches('/'), test_path);
            if let Ok(_) = call_command.execute_with_response(&["GET", &test_url]).await {
                diagnosis.security_issues.push(format!("Exposed sensitive endpoint: {}", test_path));
            }
        }

        Ok(diagnosis)
    }

    async fn generate_fixes(&self, diagnosis: &ApiDiagnosis) -> Result<Vec<Fix>, Box<dyn std::error::Error>> {
        let api_key = self.config.anthropic_api_key.as_ref()
            .ok_or("API key not configured for AI fixes")?;

        let ai_client = ClientBuilder::default()
            .api_key(api_key.clone())
            .build()?;

        let diagnosis_json = serde_json::json!({
            "url": diagnosis.url,
            "connectivity_issues": diagnosis.connectivity_issues,
            "performance_issues": diagnosis.performance_issues,
            "security_issues": diagnosis.security_issues,
            "response_issues": diagnosis.response_issues,
            "response_time_ms": diagnosis.response_time_ms
        });

        let prompt = format!(
            "You are an expert API troubleshooter. Based on this diagnosis, provide specific fixes:\n\n\
            Diagnosis:\n{}\n\n\
            For each issue found, provide:\n\
            1. ISSUE: Clear description of the problem\n\
            2. SEVERITY: critical|high|medium|low\n\
            3. FIX: Specific steps to resolve it\n\
            4. AUTOMATED: Can this be auto-fixed? (true/false)\n\
            5. CODE: Example code or configuration changes needed\n\
            6. IMPACT: What happens if not fixed?\n\n\
            Return as JSON array of fix objects.",
            serde_json::to_string_pretty(&diagnosis_json)?
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

        let mut fixes = Vec::new();

        if let Some(ContentBlock::Text { text }) = response.content.first() {
            // Try to parse AI response as JSON
            if let Ok(ai_fixes) = serde_json::from_str::<Value>(text) {
                if let Some(fixes_array) = ai_fixes.as_array() {
                    for fix_value in fixes_array {
                        let fix = Fix {
                            issue: fix_value.get("issue")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown issue")
                                .to_string(),
                            severity: fix_value.get("severity")
                                .and_then(|v| v.as_str())
                                .unwrap_or("medium")
                                .to_string(),
                            solution: fix_value.get("fix")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Manual investigation needed")
                                .to_string(),
                            automated: fix_value.get("automated")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            code_example: fix_value.get("code")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            impact: fix_value.get("impact")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown impact")
                                .to_string(),
                        };
                        fixes.push(fix);
                    }
                }
            }
        }

        // Add some fallback fixes if AI parsing failed
        if fixes.is_empty() {
            if !diagnosis.connectivity_issues.is_empty() {
                fixes.push(Fix {
                    issue: "Connectivity problems detected".to_string(),
                    severity: "high".to_string(),
                    solution: "Check network connectivity and DNS resolution".to_string(),
                    automated: false,
                    code_example: None,
                    impact: "API is unreachable".to_string(),
                });
            }
            
            if !diagnosis.security_issues.is_empty() {
                fixes.push(Fix {
                    issue: "Security vulnerabilities found".to_string(),
                    severity: "critical".to_string(),
                    solution: "Implement HTTPS and secure endpoint configurations".to_string(),
                    automated: false,
                    code_example: Some("Use https:// URLs and disable debug endpoints".to_string()),
                    impact: "Data exposure and security breaches".to_string(),
                });
            }
        }

        Ok(fixes)
    }

    fn present_fixes(&self, fixes: &[Fix]) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n🔧 AI DIAGNOSTIC RESULTS");
        println!("═══════════════════════════");
        
        for (i, fix) in fixes.iter().enumerate() {
            let severity_emoji = match fix.severity.as_str() {
                "critical" => "🚨",
                "high" => "⚠️",
                "medium" => "🟡",
                "low" => "ℹ️",
                _ => "🔍",
            };
            
            println!("\n{} {}. {} ({})", severity_emoji, i + 1, fix.issue, fix.severity.to_uppercase());
            println!("   💡 Solution: {}", fix.solution);
            println!("   📈 Impact: {}", fix.impact);
            
            if let Some(code) = &fix.code_example {
                println!("   📝 Example: {}", code);
            }
            
            if fix.automated {
                println!("   🤖 Can be auto-fixed: Yes");
            }
        }
        
        Ok(())
    }

    async fn offer_automated_fixes(&self, url: &str, fixes: &[Fix]) -> Result<(), Box<dyn std::error::Error>> {
        let automated_fixes: Vec<&Fix> = fixes.iter().filter(|f| f.automated).collect();
        
        if !automated_fixes.is_empty() {
            println!("\n🤖 Available automated fixes:");
            for fix in &automated_fixes {
                println!("   • {}", fix.issue);
            }
            
            println!("\n💡 Manual fixes required for other issues.");
            println!("🚀 Consider using 'security {}' for detailed security analysis.", url);
        } else {
            println!("\n📋 All fixes require manual intervention.");
            println!("💡 Use the provided solutions and code examples above.");
        }
        
        Ok(())
    }
}

#[derive(Debug)]
struct ApiDiagnosis {
    url: String,
    connectivity_issues: Vec<String>,
    performance_issues: Vec<String>,
    security_issues: Vec<String>,
    response_issues: Vec<String>,
    #[allow(dead_code)]
    status_code: u16,
    response_time_ms: u128,
}

#[derive(Debug)]
struct Fix {
    issue: String,
    severity: String,
    solution: String,
    automated: bool,
    code_example: Option<String>,
    impact: String,
}