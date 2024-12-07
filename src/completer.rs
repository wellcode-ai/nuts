use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper, Result};

pub struct NutsCompleter;

impl NutsCompleter {
    pub fn new() -> Self {
        Self
    }

    fn get_command_completions(&self, line: &str) -> Vec<String> {
        let mut completions = Vec::new();

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