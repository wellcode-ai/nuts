use std::sync::Arc;

pub mod call;
pub mod security;
pub mod perf;
pub mod mock;
pub mod config;

// Add shared command result type
pub type CommandResult = Result<(), Box<dyn std::error::Error>>;

// Add shared command context
#[derive(Clone)]
pub struct CommandContext {
    pub flows: Arc<crate::flows::CollectionManager>,
}

// Add shared command traits
pub trait Command {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    
    fn execute(&self, ctx: &CommandContext, args: &[String]) -> CommandResult;
}

// Re-export
pub use config::ConfigCommand;
