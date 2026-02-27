use std::{
    env,
    fmt::Display,
    fs::{self, read_dir},
    path::PathBuf,
    process::Command,
};

use tracing::{debug, info};

/// Write file at given filepath with content.
pub fn write_file<C>(filepath: &FilePath, content: C) -> Result<(), String>
where
    C: AsRef<[u8]>,
{
    debug!("Writing file: {}", filepath);
    if let Err(_) = fs::write(filepath.path_buf(), content) {
        return Err(format!("error: failed to write file {}", filepath));
    };

    Ok(())
}

pub fn list_dir(dir: &FilePath) -> Result<Vec<String>, String> {
    let entries = read_dir(dir.path_buf())
        .map_err(|_| format!("error: failed to read directory {:?}", dir))?;
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
pub fn get_root_dir() -> FilePath {
    #[cfg(debug_assertions)]
    {
        FilePath::from(env!("CARGO_MANIFEST_DIR")).join("koi")
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FilePath {
    pb: PathBuf,
}

impl FilePath {
    pub fn filename(&self) -> Option<String> {
        self.pb.file_name().map(|f| f.to_string_lossy().to_string())
    }

    pub fn join(&self, path: &str) -> FilePath {
        FilePath {
            pb: self.pb.join(path),
        }
    }

    pub fn path_buf(&self) -> &PathBuf {
        &self.pb
    }
}

impl From<PathBuf> for FilePath {
    fn from(pb: PathBuf) -> Self {
        FilePath { pb }
    }
}

impl From<&str> for FilePath {
    fn from(s: &str) -> Self {
        FilePath {
            pb: PathBuf::from(s),
        }
    }
}

impl From<&String> for FilePath {
    fn from(s: &String) -> Self {
        FilePath {
            pb: PathBuf::from(s),
        }
    }
}

impl From<String> for FilePath {
    fn from(s: String) -> Self {
        FilePath {
            pb: PathBuf::from(s),
        }
    }
}

impl Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pb.to_string_lossy())
    }
}
