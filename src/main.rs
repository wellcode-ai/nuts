mod commands;
mod shell;
mod completer;

use shell::NutsShell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut shell = NutsShell::new();
    shell.run()
}