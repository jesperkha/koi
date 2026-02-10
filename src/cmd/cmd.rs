use std::{fs, process::exit};

use crate::{config::load_config_file, driver::compile, util::write_file};
use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new project
    Init,
    /// Build and run project
    Run,
    /// Build project
    Build,
}

pub fn run() {
    let cli = Cli::parse();

    let Some(command) = cli.command else {
        Cli::command().print_help().unwrap();
        return;
    };

    if let Err(err) = run_command(command) {
        println!("{}", err);
        exit(1);
    }
}

fn run_command(command: Command) -> Result<(), String> {
    match command {
        Command::Init => koi_init(),
        Command::Build => {
            let (project, options, config) = load_config_file()?;
            compile(project, options, config)
        }
        Command::Run => todo!(),
    }
}

static DEFAULT_KOI_TOML: &str = r#"# Koi project configuration

[project]
type = "app"      # Project type (app|package)
src = "src"       # Source code directory
out = "main"      # Filepath of output file
bin = "bin"       # Output directory for temporary files
target = "x86-64" # Target arch (x86-64)
ignore-dirs = []  # Source directories to ignore

[options]
debug-mode = false
"#;

fn koi_init() -> Result<(), String> {
    if fs::exists("koi.toml").unwrap_or(false) {
        println!("File koi.toml already exists");
        return Ok(());
    }

    write_file("koi.toml", DEFAULT_KOI_TOML)?;
    println!("Created koi.toml");
    Ok(())
}
