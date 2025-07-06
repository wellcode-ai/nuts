use std::sync::Arc;

pub mod call;
pub mod security;
pub mod perf;
pub mod mock;
pub mod config;
pub mod test;
pub mod discover;
pub mod predict;
pub mod ask;
pub mod generate;
pub mod monitor;
pub mod explain;
pub mod fix;

// Add shared command result type
pub type CommandResult = Result<(), Box<dyn std::error::Error>>;

// Add shared command context
#[derive(Clone)]
#[allow(dead_code)]
pub struct CommandContext {
    pub flows: Arc<crate::flows::CollectionManager>,
}

// Add shared command traits
#[allow(dead_code)]
pub trait Command {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    
    fn execute(&self, ctx: &CommandContext, args: &[String]) -> CommandResult;
}

// Re-export (commented out unused imports)
// pub use config::ConfigCommand;
// pub use test::TestCommand;
// pub use discover::DiscoverCommand;
// pub use predict::PredictCommand;
// pub use ask::AskCommand;
// pub use generate::GenerateCommand;
// pub use monitor::MonitorCommand;
// pub use explain::ExplainCommand;
// pub use fix::FixCommand;
