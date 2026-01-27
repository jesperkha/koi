use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use tracing::info;
use walkdir::WalkDir;

use crate::{
    ast::{File, FileSet},
    build::x86,
    config::{Config, Project, ProjectType, Target, load_config_file},
    error::{ErrorSet, error_str},
    ir::{Ir, Unit, emit_ir},
    module::{Module, ModuleGraph, ModulePath, create_header_file},
    parser::{parse, sort_by_dependency_graph},
    token::{Source, scan},
    types::{TypeContext, type_check},
    util::{create_dir_if_not_exist, write_file},
};

/// Result type shorthand used in this file.
type Res<T> = Result<T, String>;

/// Compile the project using the given global config and build configuration.
pub fn compile() -> Res<()> {
    let (project, config) = load_config_file()?;

    create_dir_if_not_exist(&project.bin)?;

    // Recursively search the given source directory for files and
    // return a list of FileSet of all source files found.
    let filesets = find_and_parse_all_source_files(&project.src, &config)?;

    if filesets.len() == 0 {
        return Err(format!("no source files in '{}'", project.src));
    }

    // Create a dependency graph and sort it, returning a list of
    // filesets in correct type checking order. FileSets are sorted
    // based on their imports.
    let sorted_filesets = sort_by_dependency_graph(filesets)?;

    // Create global type context. This stores all types created and
    // used in all modules and lets us reference them by numeric IDs.
    let mut ctx = TypeContext::new();

    // Type check all file sets, turning them into Modules, and put
    // them in a ModuleGraph.
    let module_graph = type_check_and_create_modules(sorted_filesets, &mut ctx, &config)?;

    // High level passes are checks done after the main parsing and type checking
    // steps and are instead performed on the project as a whole.
    do_high_level_passes(&module_graph, &ctx, &project, &config)?;

    // Create header file for package
    if matches!(project.project_type, ProjectType::Package) {
        let content = create_header_file(module_graph.main(), &ctx)?;
        write_file(&format!("{}.h.koi", project.out), &content)?;
    }

    // Emit the intermediate representation for all modules
    let units = module_graph
        .modules()
        .iter()
        .map(|module| emit_module_ir(module, &ctx, &config))
        .collect::<Result<Vec<Unit>, String>>()?;

    // Build the final executable/libary file
    let _ = build_ir(Ir::new(units), &config, &project)?;

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

fn type_check_and_create_modules(
    sorted_sets: Vec<FileSet>,
    ctx: &mut TypeContext,
    config: &Config,
) -> Res<ModuleGraph> {
    let mut mg = ModuleGraph::new();

    for fs in sorted_sets {
        type_check(fs, &mut mg, ctx, config).map_err(|errs| errs.to_string())?;
    }

    Ok(mg)
}

/// Shorthand for emitting a module to IR and converting error to string.
fn emit_module_ir(m: &Module, ctx: &TypeContext, config: &Config) -> Res<Unit> {
    emit_ir(m, ctx, config).map_err(|errs| errs.to_string())
}

/// Shorthand for assembling an IR unit and converting error to string.
fn build_ir(ir: Ir, config: &Config, build_cfg: &Project) -> Res<String> {
    match build_cfg.target {
        Target::X86_64 => x86::build(
            ir,
            x86::BuildConfig {
                linkmode: proj_type_to_link_mode(&build_cfg.project_type),
                tmpdir: build_cfg.bin.clone(),
                outfile: build_cfg.out.clone(),
            },
            config,
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

    // TODO: ignore based on config and .gitignore

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
    let has_main = modgraph.main().symbols.get("main").map_or(false, |_| true);
    if !has_main && matches!(project.project_type, ProjectType::App) {
        return error_str("main module has no main function");
    }
    if has_main && matches!(project.project_type, ProjectType::Package) {
        return error_str("package project cannot have a main function");
    }

    // Print debug info
    if config.dump_type_context {
        ctx.dump_context_string();
    }
    if config.print_symbol_tables {
        for module in modgraph.modules() {
            module.symbols.print(module.modpath.path());
        }
    }

    Ok(())
}
