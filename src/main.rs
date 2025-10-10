use std::fs::read_to_string;

use koi::{
    driver::{Config, Target, compile},
    ir::print_ir,
    util::emit_string,
};
use tracing_subscriber::EnvFilter;

fn main() {
    // Configure global subscriber for tracing
    // Run with RUST_LOG=<level> (trace, debug, info, warn, error)
    // Defaults to error
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // let config = Config {
    //     bindir: "bin".to_string(),
    //     outfile: "main".to_string(),
    //     srcdir: ".".to_string(),
    //     target: Target::X86_64,
    // };

    // if let Err(err) = compile(config) {
    //     println!("{}", err);
    // }

    let _ = emit_string(&read_to_string("main.koi").unwrap())
        .and_then(|ir| Ok(print_ir(ir.ins)))
        .map_err(|err| println!("{}", err));
}
