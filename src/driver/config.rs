pub enum Target {
    X86_64,
}

pub struct Config {
    pub outdir: String,
    pub outfile: String,
    pub srcdir: String,
    pub target: Target,
}
