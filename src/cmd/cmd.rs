use std::{
    fs::{self, read},
    process::exit,
};

use crate::{
    config::load_config_file, driver::compile, imports::read_header_file, types::TypeContext,
    util::write_file,
};
use clap::{CommandFactory, Parser, Subcommand};
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
    /// Read the contents of a header file
    Read { filename: String },
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
            init_logger(options.debug_mode);
            compile(project, options, config)
        }
        Command::Run => todo!(),
        Command::Read { filename } => {
            let contents = read(&filename).map_err(|e| e.to_string())?;
            let mut ctx = TypeContext::new();
            let create_mod = read_header_file(
                filename.trim_end_matches(".koi.h").into(),
                &contents,
                &mut ctx,
            )?;
            println!("{}", create_mod.symbols.dump(&filename));
            Ok(())
        }
    }
}

static DEFAULT_KOI_TOML: &str = r#"# Koi project configuration

[project]
name = "myApp"    # Project name
type = "app"      # Project type (app|package)
src = "src"       # Source code directory
bin = "bin"       # Output directory for temporary files
out = "."         # Output directory of targets
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

fn init_logger(debug_mode: bool) {
    let env_filter = EnvFilter::builder()
        // Set default level based on debug_mode
        .with_default_directive(if debug_mode {
            tracing_subscriber::filter::LevelFilter::INFO.into()
        } else {
            tracing_subscriber::filter::LevelFilter::WARN.into()
        })
        // Merge with RUST_LOG if present
        .from_env_lossy(); // reads RUST_LOG if set, otherwise uses default

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .compact()
        .init();
}
