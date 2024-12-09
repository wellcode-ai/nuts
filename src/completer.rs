use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};
use std::collections::HashMap;

pub struct NutsCompleter {
    commands: HashMap<String, String>,
    aliases: HashMap<String, String>,
}

impl NutsCompleter {
    pub fn new() -> Self {
        let mut commands = HashMap::new();
        
        // Core API Testing
        commands.insert("call".to_string(), "Examples:\n  call GET https://api.example.com/users\n  call POST https://api.example.com/users '{\"name\":\"test\"}'".to_string());
        commands.insert("perf".to_string(), "Examples:\n  perf GET https://api.example.com/users --users 100 --duration 30s".to_string());
        commands.insert("security".to_string(), "Security analysis: security <URL> [OPTIONS]".to_string());
        
        // Collection Management
        commands.insert("collection new".to_string(), "Create new collection: collection new <name>".to_string());
        commands.insert("collection add".to_string(), "Add endpoint: collection add <name> <METHOD> <path>".to_string());
        commands.insert("collection run".to_string(), "Run endpoint: collection run <name> <endpoint>".to_string());
        commands.insert("collection docs".to_string(), "Generate docs: collection docs <name> [format]".to_string());
        commands.insert("collection mock".to_string(), "Start mock server: collection mock <name> [port]".to_string());
        commands.insert("collection list".to_string(), "List all collections".to_string());
        commands.insert("collection configure_mock_data".to_string(), "Configure mock data: collection configure_mock_data <name> <endpoint>".to_string());
        commands.insert("collection story".to_string(), "Start AI-guided API workflow: collection story <name>".to_string());
        commands.insert("collection s".to_string(), "Quick story mode alias: collection s <name>".to_string());
        commands.insert("save".to_string(), "Save last request: save <collection> <name>".to_string());
        
        // Configuration
        commands.insert("config api-key".to_string(), "Configure API key".to_string());
        commands.insert("config show".to_string(), "Show current configuration".to_string());
        commands.insert("help".to_string(), "Show this help message".to_string());
        commands.insert("exit".to_string(), "Exit NUTS".to_string());

        // Add aliases
        let mut aliases = HashMap::new();
        aliases.insert("c".to_string(), "call".to_string());
        aliases.insert("p".to_string(), "perf".to_string());
        aliases.insert("s".to_string(), "collection story".to_string());
        aliases.insert("h".to_string(), "help".to_string());
        aliases.insert("q".to_string(), "quit".to_string());

        Self { commands, aliases }
    }

    fn get_command_completions(&self, line: &str) -> Vec<String> {
        let mut completions = Vec::new();
        
        // Check aliases first
        if let Some(expanded) = self.aliases.get(line) {
            completions.push(expanded.clone());
        }

        // Base commands
        let base_commands = vec![
            "call", "perf", "mock", "security", "collection", "configure", "help", "exit"
        ];

        // HTTP methods
        let http_methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH"];

        // Collection subcommands
        let collection_commands = vec![
            "collection new", "collection add", "collection run",
            "collection mock", "collection perf", "collection docs",
            "collection list"
        ];

        // Options
        let options = vec!["--analyze", "--users", "--duration", "--deep"];

        // Add base commands
        completions.extend(base_commands.iter().map(|&cmd| {
            if cmd.starts_with(line) {
                Some(cmd.to_string())
            } else {
                None
            }
        }).flatten());

        // Add collection commands
        completions.extend(collection_commands.iter().map(|&cmd| {
            if cmd.starts_with(line) {
                Some(cmd.to_string())
            } else {
                None
            }
        }).flatten());

        // Add HTTP methods for relevant commands
        if line.starts_with("call ") || line.starts_with("perf ") {
            let method_part = &line[line.find(' ').unwrap_or(0) + 1..];
            completions.extend(http_methods.iter()
                .filter(|&m| m.starts_with(method_part))
                .map(|m| format!("{} {}", line.split_whitespace().next().unwrap_or(""), m)));
        }

        // Add options where relevant
        if line.contains("perf") || line.contains("security") {
            completions.extend(options.iter()
                .filter(|&opt| opt.starts_with(line.split_whitespace().last().unwrap_or("")))
                .map(|&s| s.to_string()));
        }

        completions
    }
}

impl Completer for NutsCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let line_up_to_pos = &line[..pos];
        let completions = self.get_command_completions(line_up_to_pos);

        let start_pos = line_up_to_pos.rfind(' ').map_or(0, |i| i + 1);

        Ok((
            start_pos,
            completions
                .into_iter()
                .map(|s| Pair {
                    display: s.clone(),
                    replacement: s,
                })
                .collect(),
        ))
    }
}

impl Helper for NutsCompleter {}
impl Hinter for NutsCompleter {
    type Hint = String;
}
impl Highlighter for NutsCompleter {}
impl Validator for NutsCompleter {}