use koi::{
    config::Config,
    driver::{BuildConfig, CompilationMode, Target, compile},
};
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), String> {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::debug();

    let build_config = BuildConfig {
        mode: CompilationMode::Normal,
        target: Target::X86_64,
        bindir: "bin".to_string(),
        outfile: "main".to_string(),
        srcdir: "_test".to_string(),
    };

    compile(&config, &build_config)
}
