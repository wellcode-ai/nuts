mod commands;
mod shell;
mod completer;
mod models;
mod config;
mod collections;
mod story;
use shell::NutsShell;
use clap::{Command, Arg};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("nuts")
        .version("0.1.0")
        .author("WellCode AI")
        .about("Network Universal Testing Suite")
        .disable_version_flag(true)
        .arg(Arg::new("version")
            .short('v')
            .long("version")
            .help("Print version info")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    if matches.get_flag("version") {
        println!("NUTS v0.1.0");
        return Ok(());
    }

    let mut shell = NutsShell::new();
    shell.run()
}