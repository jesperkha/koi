use koi::{
    config::Config,
    driver::{BuildConfig, Driver, Target},
};
use tracing_subscriber::EnvFilter;

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    //debug_print_all_steps(&read_to_string("_test/main.koi").unwrap());
    compile();
}

fn compile() {
    let config = Config::debug();
    let mut driver = Driver::new(&config);
    let _ = driver
        .compile(BuildConfig {
            bindir: "bin".to_string(),
            outfile: "main".to_string(),
            srcdir: "_test".to_string(),
            target: Target::X86_64,
        })
        .map_err(|e| println!("{}", e));
}
