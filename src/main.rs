use std::fs::read_to_string;

use koi::util::compile_string;
use tracing_subscriber::EnvFilter;

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    compile_string(&read_to_string("main.koi").unwrap())
        .map_or_else(|err| println!("{}", err), |src| println!("{}", src))
}
