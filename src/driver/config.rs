pub enum Target {
    X86_64,
}

pub struct Config {
    /// Directory for assembly and object file output
    pub bindir: String,
    /// Name of target executable
    pub outfile: String,
    /// Root directory of Koi project
    pub srcdir: String,
    /// Target architecture
    pub target: Target,
}
