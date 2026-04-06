use crate::util::FilePath;
use serde::Deserialize;
use std::{fs, path::Path};

pub static DEFAULT_KOI_TOML: &str = r#"# Koi project configuration

[project]
name = "myApp"    # Project name
type = "app"      # Project type (app|package)
src = "src"       # Source code directory
bin = "bin"       # Output directory for temporary files
out = "."         # Output directory of targets
target = "x86-64" # Target arch (x86-64)
ignore-dirs = []  # Source directories to ignore
link-with=[]      # Additional libraries to link with

[options]
debug-mode = false
"#;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ConfigFile {
    pub project: Project,
    pub options: Options,
}

/// The target specifies what the output assembly (or bytecode) will look
/// like. Different builders are used for different targets.
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Target {
    /// Target CPUs with the x86_64 instruction set.
    X86_64,
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectType {
    /// In app mode the source directory is compiled to a single
    /// executable put in a specified location.
    App,

    /// In package mode the specified source directory is compiled to
    /// a shared object file (.so) and a header file is generated for
    /// each exported module. This is used for compiling libraries and
    /// shared code.
    Package,
}

/// BuildConfig contains details on the general build process. Where output
/// files should go, where the source is located, what target is used, etc.
/// Compiler specific config can be found in [src/config/config.rs].
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Project {
    /// Project name.
    /// When compiling an executable this is the output filename.
    /// When compiling a library this is the library name and prefix.
    pub name: String,
    /// Directory for assembly and object file output
    pub bin: String,
    /// Root directory of Koi project
    pub src: String,
    /// Output dir for target
    pub out: String,
    /// Target architecture
    pub target: Target,
    /// Project type determines which steps are done and/or excluded
    /// in the compilation process.
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    /// Additional include paths for package exports
    pub includes: Option<Vec<String>>,
    /// Directories to ignore when searching for source files
    #[serde(default)]
    pub ignore_dirs: Vec<String>,
    /// Additional libraries to link with, full paths.
    pub link_with: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Options {
    /// Run in debug mode. Print logs and debug info.
    pub debug_mode: bool,
    /// Custom path to installation directory.
    pub install_dir: Option<String>,
}

/// DriverPhase tells the driver at which phase compilation should be terminated.
/// This is purely a debug/dev tools for inspecting the source code at each stage.
#[derive(Debug, Clone)]
pub enum DriverPhase {
    /// Fully compile the source code.
    Full,
    /// Only parse the source and print the reconstructed ASTs.
    Parse,
    /// Parse and type check. In debug mode this will output type files.
    TypeCheck,
    /// Emit and print IR units.
    Ir,
    /// Assemble and print source but do not compile with gcc/as.
    Asm,
}

/// Internal compiler configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Print type info after type checking.
    pub dump_types: bool,
    /// Print symbol tables after type checking.
    pub print_symbol_tables: bool,
    /// Dont mangle any symbol names, used primarily for testing.
    pub no_mangle_names: bool,
    /// Add comments to assembly showing source IR code.
    pub comment_assembly: bool,
    /// Which phase of compilation to terminate at.
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

/// Load koi.toml file and parse as BuildConfig.
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

/// PathManager manages paths for Koi installation. Everything is relative to
/// the koi executable, which is assumed to be in the root directory.
///
/// Koi installation layout:
///
/// ```txt
/// :root:/
///     lib/       # Compiled shared libraries
///     include/   # Module header files
///     koi        # Koi executable
/// ```
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

    /// Path to library directory containing koi builtin libraries.
    pub fn library_path(&self) -> FilePath {
        self.root().join("lib")
    }

    /// Path to library directory containing external libraries.
    pub fn external_library_path(&self) -> FilePath {
        self.root().join("external")
    }
}
