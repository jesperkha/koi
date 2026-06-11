use crate::util::FilePath;
use serde::Deserialize;
use std::{
    fs, path::Path,
    process::{Command, Stdio},
};

pub static DEFAULT_KOI_TOML: &str = r#"# Koi project configuration

[project]
name = "myApp"    # Project name
type = "app"      # Project type (app|package)
src = "src"       # Source code directory
bin = "bin"       # Output directory for temporary files
out = "."         # Output directory of targets
ignore-dirs = []  # Source directories to ignore
link-with=[]      # Additional libraries to link with

[options]
debug-mode = false
codegen = "c"
"#;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    pub project: Project,
    pub options: Options,
}

#[derive(Deserialize, Clone, strum_macros::EnumIter, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Codegen {
    X86_64,
    #[default]
    C,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectType {
    App,
    Package,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    pub name: String,
    pub bin: String,
    pub src: String,
    pub out: String,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    pub includes: Option<Vec<String>>,
    #[serde(default)]
    pub ignore_dirs: Vec<String>,
    pub link_with: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Options {
    pub debug_mode: bool,
    pub install_dir: Option<String>,
    #[serde(default)]
    pub codegen: Codegen,
}

#[derive(Debug, Clone)]
pub enum DriverPhase {
    Full,
    Parse,
    TypeCheck,
    Ir,
    Build,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub dump_types: bool,
    pub print_symbol_tables: bool,
    pub no_mangle_names: bool,
    pub comment_assembly: bool,
    pub driver_phase: DriverPhase,
}

impl Config {
    pub fn normal() -> Self {
        Self {
            dump_types: false,
            no_mangle_names: false,
            print_symbol_tables: false,
            comment_assembly: true,
            driver_phase: DriverPhase::Full,
        }
    }

    pub fn test() -> Self {
        Self {
            dump_types: false,
            no_mangle_names: true,
            print_symbol_tables: false,
            comment_assembly: false,
            driver_phase: DriverPhase::Full,
        }
    }

    pub fn debug() -> Self {
        Self {
            dump_types: true,
            no_mangle_names: false,
            print_symbol_tables: true,
            comment_assembly: true,
            driver_phase: DriverPhase::Full,
        }
    }
}

pub fn load_config_file() -> Result<(Project, Options, Config), String> {
    load_config_file_ex(".")
}

pub fn load_config_file_ex(path: &str) -> Result<(Project, Options, Config), String> {
    let filepath = Path::new(path).join("koi.toml");
    let src = fs::read_to_string(filepath)
        .map_err(|_| "Failed to open koi.toml. Run `koi init` if missing.".to_string())?;
    let config_file: ConfigFile = toml::from_str(&src).map_err(|e| e.to_string())?;

    let config = if config_file.options.debug_mode {
        Config::debug()
    } else {
        Config::normal()
    };

    Ok((config_file.project, config_file.options, config))
}

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
    /// Filepath of output executable/object file
    pub target_name: String,
    /// Directory to output target file(s)
    pub outdir: String,
    /// List of libraries to link with
    pub additional_libraries: Vec<String>,
}

pub fn gcc_available() -> bool {
    Command::new("gcc")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub struct PathManager {
    root: FilePath,
}

impl PathManager {
    pub fn new(root: FilePath) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &FilePath {
        &self.root
    }

    pub fn include_path(&self) -> FilePath {
        self.root().join("include")
    }

    pub fn library_path(&self) -> FilePath {
        self.root().join("lib")
    }

    pub fn external_library_path(&self) -> FilePath {
        self.root().join("external")
    }
}
