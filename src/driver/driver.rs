use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use tracing::info;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

use crate::{
    ast::{File, FileSet},
    build::x86,
    config::{Config, PathManager, Project, ProjectType, Target, load_config_file},
    error::{ErrorSet, error_str},
    ir::{Ir, Unit, emit_ir},
    module::{Module, ModuleGraph, ModulePath, create_header_file},
    parser::{parse, sort_by_dependency_graph},
    token::{Source, scan},
    typecheck::check_filesets,
    types::TypeContext,
    util::{create_dir_if_not_exist, get_root_dir, write_file},
};

/// Result type shorthand used in this file.
type Res<T> = Result<T, String>;

/// Compile the project using the given global config and build configuration.
pub fn compile() -> Res<()> {
    let (project, options, config) = load_config_file()?;
    let pm = PathManager::new(get_root_dir());

    init_logger(options.debug_mode);

    create_dir_if_not_exist(&project.bin)?;

    // Recursively search the given source directory for files and
    // return a list of FileSet of all source files found.
    let filesets = find_and_parse_all_source_files(&project.src, &config)?;

    // Create a dependency graph and sort it, returning a list of
    // filesets in correct type checking order. FileSets are sorted
    // based on their imports.
    let sorted_filesets = sort_by_dependency_graph(filesets)?;

    // Type check all file sets, turning them into Modules, and put
    // them in a ModuleGraph. The generated TypeContext containing all
    // type information is also returned.
    let (module_graph, ctx) =
        check_filesets(sorted_filesets, &config).map_err(|err| err.to_string())?;

    // High level passes are checks done after the main parsing and type checking
    // steps and are instead performed on the project as a whole.
    do_high_level_passes(&module_graph, &ctx, &project, &config)?;

    // Create header files for package
    if matches!(project.project_type, ProjectType::Package) {
        let empty = Vec::new();
        let includes = project.includes.as_ref().unwrap_or(&empty);
        create_package_headers(&module_graph, &ctx, &includes, &project)?;
    }

    // Emit the intermediate representation for all modules
    let units = module_graph
        .modules()
        .iter()
        .filter(|module| module.should_be_built())
        .map(|module| emit_module_ir(module, &ctx, &config))
        .collect::<Result<Vec<Unit>, String>>()?;

    // Build the final executable/libary file
    build(Ir::new(units), &config, &project, &pm)
}

/// Create header files for main module and all modules listed in project include list.
fn create_package_headers(
    modgraph: &ModuleGraph,
    ctx: &TypeContext,
    includes: &[String],
    project: &Project,
) -> Res<()> {
    let exported_modules = modgraph
        .modules()
        .iter()
        .filter(|m| m.is_main() || includes.iter().any(|include| include == m.modpath.path()))
        .collect::<Vec<&Module>>();

    for module in exported_modules {
        let filename = format!(
            "{}.mod",
            if module.is_main() {
                &project.out
            } else {
                module.modpath.path()
            }
        );

        let content = create_header_file(module, &ctx)?;
        write_file(&filename, &content)?;
    }

    Ok(())
}

/// Recursively search the given source directory for files and return a list of FileSet of
/// all source files found.
fn find_and_parse_all_source_files(source_dir: &str, config: &Config) -> Res<Vec<FileSet>> {
    info!("Collecting source files in {}", source_dir);
    let mut filesets = Vec::new();

    for dir in &list_source_directories(source_dir)? {
        let sources = collect_files_in_directory(dir)?;
        if sources.is_empty() {
            continue;
        }

        let mut module_path = pathbuf_to_module_path(&dir, source_dir);
        if module_path.is_empty() {
            module_path = String::from("main");
        }

        info!("Parsing module: {}", module_path);
        let files = parse_files_in_directory(sources, config)?;

        if files.is_empty() {
            info!("No input files");
            continue;
        }

        filesets.push(FileSet::new(ModulePath::new(module_path), files));
    }

    if filesets.len() == 0 {
        return Err(format!("no source files in '{}'", source_dir));
    }

    Ok(filesets)
}

/// Parse a list of Sources into AST Files, collecting errors into a single string.
fn parse_files_in_directory(sources: Vec<Source>, config: &Config) -> Res<Vec<File>> {
    let mut errs = ErrorSet::new();
    let mut files = Vec::new();

    for src in sources {
        if src.size == 0 {
            continue;
        }

        scan(&src, config)
            .and_then(|toks| parse(src, toks, config))
            .map_or_else(|err| errs.join(err), |file| files.push(file));
    }

    if errs.len() > 0 {
        Err(errs.to_string())
    } else {
        Ok(files)
    }
}

/// Shorthand for emitting a module to IR and converting error to string.
fn emit_module_ir(m: &Module, ctx: &TypeContext, config: &Config) -> Res<Unit> {
    emit_ir(m, ctx, config).map_err(|errs| errs.to_string())
}

/// Shorthand for assembling an IR unit and converting error to string.
fn build(ir: Ir, config: &Config, build_cfg: &Project, pm: &PathManager) -> Res<()> {
    match build_cfg.target {
        Target::X86_64 => x86::build(
            ir,
            x86::BuildConfig {
                linkmode: proj_type_to_link_mode(&build_cfg.project_type),
                tmpdir: build_cfg.bin.clone(),
                outfile: build_cfg.out.clone(),
            },
            config,
            pm,
        ),
    }
}

/// Report which x86 link mode to use for which compilation mode.
fn proj_type_to_link_mode(mode: &ProjectType) -> x86::LinkMode {
    match mode {
        ProjectType::App => x86::LinkMode::Exectuable,
        ProjectType::Package => x86::LinkMode::SharedObject,
    }
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

/// List all subdirectories in path, including path. Ignores hidden
/// directories and ones listed in other ignore lists (config or .gitignore).
fn list_source_directories(path: &str) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let mut errors = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
    {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() {
                    dirs.push(e.path().to_path_buf());
                }
            }
            Err(err) => {
                errors.push(format!(
                    "failed to read directory: {}",
                    err.path().unwrap_or(Path::new("")).display()
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(dirs)
    } else {
        Err(errors.join("\n"))
    }
}

/// Collects all koi files in given directory and returns as a list of sources.
fn collect_files_in_directory(dir: &PathBuf) -> Res<Vec<Source>> {
    let mut files = Vec::new();

    let dirents = match fs::read_dir(dir) {
        Err(_) => return Err(format!("failed to read directory: '{}'", dir.display())),
        Ok(ents) => ents,
    };

    for entry in dirents {
        let path = match entry {
            Err(err) => return Err(format!("failed to read file: {}", err)),
            Ok(ent) => ent.path(),
        };

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension() {
            if ext == "koi" {
                files.push(path.display().to_string());
            }
        }
    }

    let mut set = Vec::new();
    for file in files {
        match fs::read(&file) {
            Err(err) => return Err(format!("failed to read file: {}", err)),
            Ok(src) => set.push(Source::new(file, src)),
        }
    }

    Ok(set)
}

/// Convert foo/bar/faz to foo.bar.faz
fn pathbuf_to_module_path(path: &PathBuf, source_dir: &str) -> String {
    path.display()
        .to_string()
        .trim_start_matches(source_dir)
        .trim_start_matches("/")
        .trim_end_matches("/")
        .replace("/", ".")
}

/// High level passes are checks done after the main parsing and type checking
/// steps and are instead performed on the project as a whole.
fn do_high_level_passes(
    modgraph: &ModuleGraph,
    ctx: &TypeContext,
    project: &Project,
    config: &Config,
) -> Result<(), String> {
    // Check if main function is present and if it should be
    let has_main = modgraph
        .main()
        .map(|m| m.symbols.get("main"))
        .map_or(false, |_| true);

    if !has_main && matches!(project.project_type, ProjectType::App) {
        return error_str("main module has no main function");
    }
    if has_main && matches!(project.project_type, ProjectType::Package) {
        return error_str("package project cannot have a main function");
    }

    if config.dump_type_context {
        let path = format!("{}/types.txt", project.bin);
        info!("Writing type info to {}", path);
        write_file(&path, &ctx.dump_context_string())?;
    }

    if config.print_symbol_tables {
        let mut s = String::new();
        for module in modgraph.modules() {
            s += &module.symbols.dump(module.modpath.path());
        }

        let path = format!("{}/symbols.txt", project.bin);
        info!("Writing symbol info to {}", path);
        write_file(&path, &s)?;
    }

    Ok(())
}

fn init_logger(debug_mode: bool) {
    let env_filter = EnvFilter::builder()
        // Set default level based on debug_mode
        .with_default_directive(if debug_mode {
            tracing_subscriber::filter::LevelFilter::INFO.into()
        } else {
            tracing_subscriber::filter::LevelFilter::WARN.into()
        })
        // Merge with RUST_LOG if present
        .from_env_lossy(); // reads RUST_LOG if set, otherwise uses default

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(false)
        .with_line_number(false)
        .without_time()
        .compact()
        .init();
}
