use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Helper};
use rustyline::Result;

#[derive(Default)]
pub struct NutsCompleter {
    commands: Vec<String>,
}

impl NutsCompleter {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "call".to_string(),
                "collection new".to_string(),
                "collection run".to_string(),
                "collection mock".to_string(),
                "collection perf".to_string(),
                "collection configure_mock_data".to_string(),
                "save".to_string(),
                "perf".to_string(),
                "security".to_string(),
                "configure".to_string(),
                "help".to_string(),
                "exit".to_string(),
            ],
        }
    }
}

// Implement required traits for Helper
impl Completer for NutsCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        let mut matches: Vec<Pair> = Vec::new();
        
        for cmd in &self.commands {
            if cmd.starts_with(line) {
                matches.push(Pair {
                    display: cmd.clone(),
                    replacement: cmd.clone(),
                });
            }
        }

        Ok((0, matches))
    }
}

impl Helper for NutsCompleter {}
impl Hinter for NutsCompleter {
    type Hint = String;
}
impl Highlighter for NutsCompleter {}
impl Validator for NutsCompleter {}