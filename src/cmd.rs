use std::{
    fs::{self},
    process::exit,
};

use crate::{
    config::{DEFAULT_KOI_TOML, ProjectType, load_config_file},
    driver::compile,
    imports::dump_header_symbols,
    util::{exec, write_file},
};
use clap::{CommandFactory, Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

const VERSION: &str = concat!("v", env!("CARGO_PKG_VERSION"));

#[derive(Parser)]
#[command(author, about, long_about = None, version = VERSION, disable_version_flag = true)]
struct Cli {
    /// Print version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    version: (),

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Args, Default)]
struct ProjectOverrides {
    /// Override project name
    #[arg(long)]
    name: Option<String>,
    /// Override binary/temp output directory
    #[arg(long)]
    bin: Option<String>,
    /// Override output directory
    #[arg(long)]
    out: Option<String>,
    /// Override source directory
    #[arg(long)]
    src: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new project
    Init,
    /// Build and run project
    Run {
        #[command(flatten)]
        overrides: ProjectOverrides,
    },
    /// Build project
    Build {
        #[command(flatten)]
        overrides: ProjectOverrides,
    },
    /// Read the contents of a header file
    Read { filename: String },
    /// Print version
    Version,
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

fn apply_overrides(
    mut project: crate::config::Project,
    o: ProjectOverrides,
) -> crate::config::Project {
    if let Some(v) = o.name {
        project.name = v;
    }
    if let Some(v) = o.bin {
        project.bin = v;
    }
    if let Some(v) = o.out {
        project.out = v;
    }
    if let Some(v) = o.src {
        project.src = v;
    }
    project
}

fn run_command(command: Command) -> Result<(), String> {
    match command {
        Command::Init => koi_init(),
        Command::Build { overrides } => {
            let (project, options, config) = load_config_file()?;
            let project = apply_overrides(project, overrides);
            info!("Building project: {}", project.name);
            init_logger(options.debug_mode);
            compile(project, options, config)
        }
        Command::Run { overrides } => {
            let (project, options, config) = load_config_file()?;
            let project = apply_overrides(project, overrides);
            info!("Building and running project: {}", project.name);

            // Check if project can be run
            if matches!(project.project_type, ProjectType::Package) {
                return Err("error: run command only works for app projects".into());
            }

            init_logger(options.debug_mode);
            let binary = format!("./{}/{}", project.out, project.name);
            compile(project, options, config)?;
            exec(&binary, &[])
        }
        Command::Read { filename } => {
            let s = dump_header_symbols(&filename).unwrap();
            println!("{}", s);
            Ok(())
        }
        Command::Version => {
            println!("koi {}", VERSION);
            Ok(())
        }
    }
}

fn koi_init() -> Result<(), String> {
    if fs::exists("koi.toml").unwrap_or(false) {
        println!("File koi.toml already exists");
        return Ok(());
    }

    write_file(&"koi.toml".into(), DEFAULT_KOI_TOML)?;
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
