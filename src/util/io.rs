use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use tracing::{debug, info};

/// Write file at given filepath with content.
pub fn write_file(filepath: &str, content: &str) -> Result<(), String> {
    debug!("Writing file: {}", filepath);
    let path = Path::new(filepath);
    if let Err(_) = fs::write(&path, content) {
        return Err(format!("error: failed to write file {}", filepath));
    };

    Ok(())
}

/// Run shell command
pub fn cmd(command: &str, args: &[String]) -> Result<(), String> {
    info!("Cmd: {} {}", command, args.join(" "));

    let status = Command::new(command)
        .args(args)
        .status()
        .or_else(|_| Err(format!("failed to run command: {}", command)))?;

    if !status.success() {
        Err(format!(
            "command '{}' exited with a non-success code",
            command,
        ))
    } else {
        Ok(())
    }
}

pub fn create_dir_if_not_exist(dir: &str) -> Result<(), String> {
    if !fs::exists(dir).unwrap_or(false) {
        info!("Creating directory: {}", dir);
        if let Err(_) = fs::create_dir(dir) {
            return Err(format!("failed to create directory: {}", dir));
        }
    }
    Ok(())
}

/// Get the directory of the executable. This is the base installation
/// directory and all config/runtime files are relative to this.
pub fn get_root_dir() -> PathBuf {
    let exec_path = env::current_exe().unwrap();

    let rootdir = if exec_path.ends_with("target/debug/koi") {
        Path::new(".") // for debug/testing using cargo run
    } else {
        exec_path.parent().unwrap().parent().unwrap()
    };

    rootdir.to_owned()
}
