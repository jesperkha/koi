use koi::{config::Config, driver::compile};
use tracing_subscriber::EnvFilter;

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::debug();
    let _ = compile(&config).map_err(|e| println!("{}", e));
}
