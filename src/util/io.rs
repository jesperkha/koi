use std::{fs, path::Path, process::Command};

/// Write file at given filepath with content.
pub fn write_file(filepath: &str, content: &str) -> Result<(), String> {
    let path = Path::new(filepath);
    if let Err(_) = fs::write(&path, content) {
        return Err("failed to write output".to_string());
    };

    Ok(())
}

/// Run shell command
pub fn cmd(command: &str, args: &[String]) -> Result<(), String> {
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
