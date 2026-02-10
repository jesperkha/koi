use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use tracing::info;
use tracing_subscriber::EnvFilter;
use walkdir::WalkDir;

use crate::{
    ast::FileSet,
    build::x86,
    config::{Config, Options, PathManager, Project, ProjectType, Target},
    ir::{Ir, Unit},
    lower::emit_ir,
    module::{Module, ModuleGraph, ModulePath, create_header_file},
    parser::{sort_by_dependency_graph, source_map_to_fileset},
    token::{Source, SourceMap},
    typecheck::check_filesets,
    types::TypeContext,
    util::{create_dir_if_not_exist, get_root_dir, write_file},
};

/// Result type shorthand used in this file.
type Res<T> = Result<T, String>;

/// Compile the project using the given global config and build configuration.
pub fn compile(project: Project, options: Options, config: Config) -> Res<()> {
    init_logger(options.debug_mode);
    create_dir_if_not_exist(&project.bin)?;

    // Recursively search the given source directory for files and
    // return a list of SourceDir of all source files found.
    let source_dirs = collect_all_source_dirs(&project.src, &project.ignore_dirs)?;

    // Parse all of the sources and return a list of FileSet.
    let filesets = parse_source_dirs(&source_dirs, &config)?;

    // Flatten the SourceMaps for error handling.
    let source_map = source_dirs
        .into_iter()
        .fold(SourceMap::new(), |mut map, dir| {
            map.join(dir.map);
            map
        });

    // Create a dependency graph and sort it, returning a list of
    // filesets in correct type checking order. FileSets are sorted
    // based on their imports.
    let sorted_filesets = sort_by_dependency_graph(filesets)?;

    // Type check all file sets, turning them into Modules, and put
    // them in a ModuleGraph. The generated TypeContext containing all
    // type information is also returned.
    let (module_graph, ctx) = create_modules(sorted_filesets, &source_map, &config)?;

    // Do some high level passes at a module level before lowering
    check_main_function_present(&module_graph, &project)?;
    dump_debug_info(&ctx, &module_graph, &project, &config)?;

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
        .map(|module| emit_module_ir(&source_map, module, &ctx, &config))
        .collect::<Result<Vec<Unit>, String>>()?;

    // Build the final executable/libary file
    let pm = PathManager::new(get_root_dir());
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

struct SourceDir {
    modpath: ModulePath,
    map: SourceMap,
}

/// Recursively search the given source directory for files and return a list of FileSet of
/// all source files found.
fn collect_all_source_dirs(source_dir: &str, ignore_dirs: &[String]) -> Res<Vec<SourceDir>> {
    info!("Collecting source files in {}", source_dir);
    let mut dirs = Vec::new();

    for dir in &list_source_directories(source_dir, ignore_dirs)? {
        let map = dir_to_source_map(dir)?;

        if map.is_empty() {
            continue;
        }

        let mut modpath_str = pathbuf_to_module_path(&dir, source_dir);
        if modpath_str.is_empty() {
            modpath_str = String::from("main");
        }

        let dir = SourceDir {
            modpath: ModulePath::new(modpath_str),
            map,
        };

        dirs.push(dir);
    }

    if dirs.len() == 0 {
        return Err(format!("no source files in '{}'", source_dir));
    }

    Ok(dirs)
}

/// Collects all koi files in given directory and returns as a list of sources.
fn dir_to_source_map(dir: &PathBuf) -> Res<SourceMap> {
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

    let mut map = SourceMap::new();
    for file in files {
        match fs::read(&file) {
            Err(err) => return Err(format!("failed to read file: {}", err)),
            Ok(src) => map.add(Source::new(file, src)),
        }
    }

    Ok(map)
}

/// Parse all files in each source directory.
fn parse_source_dirs(dirs: &Vec<SourceDir>, config: &Config) -> Res<Vec<FileSet>> {
    let mut filesets = Vec::new();

    for dir in dirs {
        info!("Parsing module: {}", dir.modpath.path());
        let fileset = source_map_to_fileset(dir.modpath.clone(), &dir.map, config)
            .map_err(|err| err.render(&dir.map))?;

        if fileset.is_empty() {
            info!("No input files");
            continue;
        }

        filesets.push(fileset);
    }

    Ok(filesets)
}

fn create_modules(
    filesets: Vec<FileSet>,
    map: &SourceMap,
    config: &Config,
) -> Res<(ModuleGraph, TypeContext)> {
    check_filesets(filesets, &config).map_err(|err| err.render(&map))
}

/// Shorthand for emitting a module to IR and converting error to string.
fn emit_module_ir(map: &SourceMap, m: &Module, ctx: &TypeContext, config: &Config) -> Res<Unit> {
    emit_ir(m, ctx, config).map_err(|errs| errs.render(&map))
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

fn is_hidden(entry: &walkdir::DirEntry, ignore_dirs: &[String]) -> bool {
    if let Some(file_name) = entry.file_name().to_str() {
        ignore_dirs.iter().any(|ignored| ignored == file_name)
    } else {
        false
    }
}

/// List all subdirectories in path, including path. Ignores hidden
/// directories and ones listed in other ignore lists (config or .gitignore).
fn list_source_directories(path: &str, ignore_dirs: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut dirs = Vec::new();
    let mut errors = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_hidden(e, ignore_dirs))
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

/// Convert foo/bar/faz to foo.bar.faz
fn pathbuf_to_module_path(path: &PathBuf, source_dir: &str) -> String {
    path.display()
        .to_string()
        .trim_start_matches(source_dir)
        .trim_start_matches("/")
        .trim_end_matches("/")
        .replace("/", ".")
}

fn error_str(msg: &str) -> Result<(), String> {
    Err(format!("error: {}", msg))
}

/// Check if main function is present and if it should be
fn check_main_function_present(modgraph: &ModuleGraph, project: &Project) -> Res<()> {
    let has_main = modgraph
        .main()
        .map(|m| m.symbols.get("main").is_ok())
        .unwrap_or(false);

    if !has_main && matches!(project.project_type, ProjectType::App) {
        return error_str("main module has no main function");
    }
    if has_main && matches!(project.project_type, ProjectType::Package) {
        return error_str("package project cannot have a main function");
    }

    Ok(())
}

/// Print debug info if configured.
fn dump_debug_info(
    ctx: &TypeContext,
    modgraph: &ModuleGraph,
    project: &Project,
    config: &Config,
) -> Res<()> {
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
