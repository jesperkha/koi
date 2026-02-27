use std::{
    env,
    fs::{self, read_dir},
    path::{Path, PathBuf},
    process::Command,
};

use tracing::{debug, info};

/// Write file at given filepath with content.
pub fn write_file<C>(filepath: &str, content: C) -> Result<(), String>
where
    C: AsRef<[u8]>,
{
    debug!("Writing file: {}", filepath);
    let path = Path::new(filepath);
    if let Err(_) = fs::write(&path, content) {
        return Err(format!("error: failed to write file {}", filepath));
    };

    Ok(())
}

pub fn list_dir(dir: &PathBuf) -> Result<Vec<String>, String> {
    let entries =
        read_dir(dir).map_err(|_| format!("error: failed to read directory {:?}", dir))?;
    Ok(entries
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().into_string())
        .filter_map(Result::ok)
        .collect())
}

/// Run shell command
pub fn cmd(command: &str, args: &[String]) -> Result<String, String> {
    info!("Cmd: {} {}", command, args.join(" "));

    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|_| format!("failed to run command: {}", command))?;

    if !output.status.success() {
        return Err(format!(
            "command '{}' exited with a non-success code",
            command
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if stdout == "" {
        return Ok(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(stdout)
}

pub fn create_dir_if_not_exist(dir: &str) -> Result<(), String> {
    if !fs::exists(dir).unwrap_or(false) {
        info!("Creating directory: {}", dir);
        if let Err(err) = fs::create_dir(dir) {
            println!("mkdir: {}", err);
            return Err(format!("failed to create directory: {}", dir));
        }
    }
    Ok(())
}

/// Get the directory of the executable. This is the base installation
/// directory and all config/runtime files are relative to this.
pub fn get_root_dir() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("koi")
    }

    #[cfg(not(debug_assertions))]
    {
        // TODO: check if this is correct
        let exec_path = env::current_exe().unwrap();
        let rootdir = exec_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();

        rootdir.to_owned()
    }
}
