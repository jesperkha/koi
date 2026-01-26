use std::{fs, process::exit};

use clap::{CommandFactory, Parser, Subcommand};
use koi::{config::Config, driver::compile, util::write_file};
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
        .init();

    let cli = Cli::parse();

    let Some(command) = cli.command else {
        Cli::command().print_help().unwrap();
        return;
    };

    let config = Config::debug();

    let res = match command {
        Command::Init => koi_init(),
        Command::Build => compile(&config),
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

bin = "bin"  # Output directory for temporary files
src = "src"  # Source code directory
out = "main" # Filepath of output file

# Compilation mode
#   normal  - Compile as an executable
#   package - Compile as a shared library
mode = "normal"

# Target architecture/format
#   x86-64 - Target unknown x86-64 gnu linux 
target = "x86-64"
"#;

    write_file("koi.toml", content)?;
    println!("Created koi.toml");
    Ok(())
}
