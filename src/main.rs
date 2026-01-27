use std::{fs, process::exit};

use clap::{CommandFactory, Parser, Subcommand};
use koi::{driver::compile, util::write_file};
use tracing::Span;
use tracing_subscriber::EnvFilter;

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

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .compact()
        .init();

    let cli = Cli::parse();

    let Some(command) = cli.command else {
        Cli::command().print_help().unwrap();
        return;
    };

    let res = match command {
        Command::Init => koi_init(),
        Command::Build => compile(),
        Command::Run => todo!(),
    };

    if let Err(err) = res {
        println!("{}", err);
        exit(1);
    }
}

fn koi_init() -> Result<(), String> {
    if fs::exists("koi.toml").unwrap_or(false) {
        println!("File koi.toml already exists");
        return Ok(());
    }

    let content = r#"# Koi project configuration

[project]
type = "app"      # Project type (app|package) 
src = "_test"     # Source code directory
out = "main"      # Filepath of output file
bin = "bin"       # Output directory for temporary files
target = "x86-64" # Target arch (x86-64)

[options]
debug-mode = false
"#;

    write_file("koi.toml", content)?;
    println!("Created koi.toml");
    Ok(())
}
