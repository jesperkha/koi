use std::fs::read_to_string;

use koi::util::debug_print_all_steps;
use tracing_subscriber::EnvFilter;

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    debug_print_all_steps(&read_to_string("main.koi").unwrap());

    // let config = Config::default();
    // let mut driver = Driver::new(&config);
    // let _ = driver
    //     .compile(BuildConfig {
    //         bindir: "bin".to_string(),
    //         outfile: "main".to_string(),
    //         srcdir: "_test".to_string(),
    //         target: Target::X86_64,
    //     })
    //     .map_err(|e| println!("{}", e));
}
