mod commands;
mod shell;
mod completer;
mod models;
use shell::NutsShell;
mod collections;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = NutsShell::new();
    shell.run()
}