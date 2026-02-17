use std::sync::Arc;

pub mod ask;
pub mod call;
pub mod config;
pub mod discover;
pub mod explain;
pub mod fix;
pub mod generate;
pub mod mock;
pub mod monitor;
pub mod perf;
pub mod predict;
pub mod security;
pub mod test;

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
