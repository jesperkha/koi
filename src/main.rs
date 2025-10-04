use koi::driver::{Config, Target, compile};

fn main() {
    let config = Config {
        outdir: ".".to_string(),
        outfile: "main".to_string(),
        srcdir: ".".to_string(),
        target: Target::X86_64,
    };

    if let Err(err) = compile(config) {
        println!("{}", err);
    }
}
