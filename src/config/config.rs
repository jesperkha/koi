use serde::Deserialize;
use std::{fs, path::Path};
use tracing::field;

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

#[derive(Deserialize)]
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
    /// Directory for assembly and object file output
    pub bin: String,
    /// Name of target executable
    pub out: String,
    /// Root directory of Koi project
    pub src: String,
    /// Target architecture
    pub target: Target,
    /// Project type determines which steps are done and/or excluded
    /// in the compilation process.
    #[serde(rename = "type")]
    pub project_type: ProjectType,
    /// Additional include paths for package exports
    pub includes: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Options {
    pub debug_mode: bool,
}

/// Internal compiler configuration
pub struct Config {
    /// Print TypeContext after type checking.
    pub dump_type_context: bool,
    /// Print symbol tables after type checking.
    pub print_symbol_tables: bool,
    /// Dont mangle any symbol names, used primarily for testing.
    pub no_mangle_names: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: false,
            print_symbol_tables: false,
        }
    }

    pub fn test() -> Self {
        Self {
            dump_type_context: false,
            no_mangle_names: true,
            print_symbol_tables: false,
        }
    }

    pub fn debug() -> Self {
        Self {
            dump_type_context: true,
            no_mangle_names: false,
            print_symbol_tables: true,
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
        .map_err(|_| format!("Failed to open koi.toml. Run `koi init` if missing."))?;
    let config_file: ConfigFile = toml::from_str(&src).map_err(|e| e.to_string())?;

    let config = if config_file.options.debug_mode {
        Config::debug()
    } else {
        Config::default()
    };

    Ok((config_file.project, config_file.options, config))
}
