use crate::commands::CommandResult;
use console::style;
use crate::config::Config;

pub struct ConfigCommand {
    config: Config,
}

impl ConfigCommand {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn execute(&self, args: &[&str]) -> CommandResult {
        match args.get(1).copied() {
            Some("api-key") => {
                println!("Enter your Anthropic API key:");
                let key = dialoguer::Input::<String>::new()
                    .with_prompt("API Key")
                    .interact()?;
                
                let mut config = self.config.clone();
                config.anthropic_api_key = Some(key);
                config.save()?;
                
                // Verify the save worked
                match Config::load() {
                    Ok(loaded) => {
                        if loaded.anthropic_api_key.is_some() {
                            println!("✅ {}", style("API key configured successfully").green());
                        } else {
                            println!("❌ Failed to verify saved API key");
                        }
                    },
                    Err(e) => println!("❌ Error verifying config: {}", e),
                }
            }
            Some("show") => {
                // Load fresh config to ensure we show current state
                let config = Config::load()?;
                println!("Current Configuration:");
                println!("  API Key: {}", config.anthropic_api_key
                    .as_ref()
                    .map(|_| "********")
                    .unwrap_or("Not set"));
            }
            _ => {
                println!("Available config commands:");
                println!("  {} - Configure Anthropic API key", style("config api-key").green());
                println!("  {} - Show current configuration", style("config show").green());
            }
        }
        Ok(())
    }
} 