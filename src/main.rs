use koi::{
    config::Config,
    driver::{BuildConfig, Driver, Target},
};
use tracing_subscriber::EnvFilter;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build {
        /// Name of output binary
        #[arg(short, long, default_value_t = String::from("main"))]
        outfile: String,

        /// Name of directory to output intermediate files
        #[arg(short, long, default_value_t = String::from("bin"))]
        bindir: String,

        /// Source directory for files
        #[arg(short, long, default_value_t = String::from("."))]
        source: String,

        /// Run in debug mode
        #[arg(short, default_value_t = false)]
        debug: bool,
    },
}

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Cli::parse();

    match args.command {
        Commands::Build {
            outfile,
            bindir,
            source,
            debug,
        } => {
            let config = match debug {
                true => Config::debug(),
                false => Config::default(),
            };

            let build_config = BuildConfig {
                bindir,
                outfile,
                srcdir: source,
                target: Target::X86_64,
            };

            let mut driver = Driver::new(&config);
            let _ = driver.compile(build_config).map_err(|e| println!("{}", e));
        }
    };
}
