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
        let commands = vec![
            "call", "perf", "security", "mock", "collection", "configure", "help", "exit",
            "collection new", "collection add", "collection run", "collection mock",
            "collection docs", "collection list", "collection perf",
            "collection configure_mock_data", "save", "daemon",
        ];

        let http_methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        let mut completions = Vec::new();

        // Complete commands
        for cmd in commands {
            if cmd.starts_with(line) {
                completions.push(cmd.to_string());
            }
        }

        // Complete HTTP methods if line starts with supported commands
        if line.starts_with("call ") || line.starts_with("perf ") {
            for method in http_methods {
                if method.starts_with(&line[5..]) {
                    completions.push(format!("{} {}", line[..5].trim(), method));
                }
            }
        }

        completions
    }
}

// Implement the required Helper trait and its sub-traits
impl Completer for NutsCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, _pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        let completions = self.get_command_completions(line);
        Ok((
            0,
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