use std::process::{Command, Stdio};

pub mod c;
pub mod x86;

pub enum LinkMode {
    /// Link as executable ELF file
    Executable,
    /// Link to static library file (.a)
    Library,
}

pub struct BuildConfig {
    pub linkmode: LinkMode,
    /// Where to output temp files (.s .o)
    pub tmpdir: String,
    /// Filepath out output executable/object file
    pub target_name: String,
    /// Directory to output target file(s)
    pub outdir: String,
    /// List of libraries to link with
    pub additional_libraries: Vec<String>,
}

pub(crate) fn gcc_available() -> bool {
    Command::new("gcc")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
