use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

use tracing::info;
use walkdir::WalkDir;

use crate::{
    ast::{File, FileSet},
    build::{TransUnit, X86Builder, assemble},
    config::Config,
    error::ErrorSet,
    ir::{IRUnit, emit_ir},
    module::{Module, ModuleGraph, ModulePath, create_header_file},
    parser::{parse, sort_by_dependency_graph},
    token::{Source, scan},
    types::{TypeContext, type_check},
};

/// Result type shorthand used in this file.
type Res<T> = Result<T, String>;

/// The target specifies what the output assembly (or bytecode) will look
/// like. Different builders are used for different targets.
pub enum Target {
    /// Target CPUs with the x86_64 instruction set.
    X86_64,
}

/// Compilation mode determines which steps are done and/or excluded
/// in the compilation process.
pub enum CompilationMode {
    /// In normal mode the source directory is compiled to a single
    /// executable put in a specified location.
    Normal,

    /// In module mode the source directory is compiled to a shared
    /// object file (.so) and a header file is generated along with it.
    /// This is used for compiling libraries and shared code.
    Module,

    /// Only compile the source to assembly and output it to the
    /// specified directory.
    CompileOnly,
}

/// BuildConfig contains details on the general build process. Where output
/// files should go, where the source is located, what target is used, etc.
/// Compiler specific config can be found in [src/config/config.rs].
pub struct BuildConfig {
    /// Directory for assembly and object file output
    pub bindir: String,
    /// Name of target executable
    pub outfile: String,
    /// Root directory of Koi project
    pub srcdir: String,
    /// Target architecture
    pub target: Target,
}

/// Compile the project using the given global config and build configuration.
pub fn compile(cfg: &Config, build_cfg: &BuildConfig) -> Res<()> {
    create_dir_if_not_exist(&build_cfg.bindir)?;

    // Parse all files and store as Filesets
    let mut filesets = Vec::new();
    for dir in &list_source_directories(&build_cfg.srcdir)? {
        let sources = collect_files_in_directory(dir)?;
        if sources.is_empty() {
            continue;
        }

        let mut module_path = pathbuf_to_module_path(&dir, &build_cfg.srcdir);
        if module_path.is_empty() {
            module_path = String::from("main");
        }

        info!("parsing module: {}", module_path);
        let files = parse_files(sources, cfg)?;

        if files.is_empty() {
            info!("no files to parse");
            continue;
        }

        filesets.push(FileSet::new(ModulePath::new(module_path), files));
    }

    // Create and sort dependency graph, returning a list of
    // filesets in correct type checking order.
    let sorted_filesets = sort_by_dependency_graph(filesets)?;

    // Global state
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();

    // Type check, convert to IR, and emit assembly
    let mut asm_files = Vec::new();
    for fs in sorted_filesets {
        let module = type_check_and_create_module(fs, &mut mg, &mut ctx, cfg)?;

        // Write header file
        // create_header_file_for_module(&build_cfg.bindir, module, &ctx)?;

        let ir_unit = emit_module_ir(module, &ctx, cfg)?;
        let asm = assemble_ir_unit(ir_unit, &build_cfg.target, cfg)?;

        let outfile = write_output_file(&build_cfg.bindir, module.name(), &asm.source)?;
        info!("output assembly file: {}", outfile.display());
        asm_files.push(outfile);
    }

    if cfg.dump_type_context {
        ctx.dump_context_string();
    }

    // Assemble all source files
    for file in &asm_files {
        info!("assembling: {}", file.display());
        let src = file.to_string_lossy();
        let out = file.with_extension("o");
        cmd("as", &["-o", &out.to_string_lossy(), &src])?;
    }

    let mut objectfiles = vec![];
    for file in asm_files {
        objectfiles.push(file.with_extension("o").to_string_lossy().to_string());
    }

    // TODO: rewrite this mess

    // let entry_o = format!("{}/entry.o", build_cfg.bindir);
    // cmd("as", &["-o", &entry_o, "lib/compile/entry.s"])?;

    let mut args = vec!["-o", &build_cfg.outfile];
    args.extend_from_slice(
        &objectfiles
            .iter()
            .map(|f| f.as_str())
            .collect::<Vec<&str>>(),
    );

    cmd("gcc", &args)?;

    Ok(())
}

/// Parse a list of Sources into AST Files, collecting errors into a single string.
fn parse_files(sources: Vec<Source>, config: &Config) -> Res<Vec<File>> {
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

/// Shorthand for type checking a fileset and converting error to string.
fn type_check_and_create_module<'m>(
    fs: FileSet,
    mg: &'m mut ModuleGraph,
    ctx: &mut TypeContext,
    config: &Config,
) -> Res<&'m Module> {
    type_check(fs, mg, ctx, config).map_err(|errs| errs.to_string())
}

/// Shorthand for emitting a module to IR and converting error to string.
fn emit_module_ir(m: &Module, ctx: &TypeContext, config: &Config) -> Res<IRUnit> {
    emit_ir(m, ctx, config).map_err(|errs| errs.to_string())
}

/// Shorthand for assembling an IR unit and converting error to string.
fn assemble_ir_unit(unit: IRUnit, target: &Target, config: &Config) -> Res<TransUnit> {
    match target {
        Target::X86_64 => assemble::<X86Builder>(config, unit),
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

/// Write file at given filepath with content.
fn write_file(filepath: &str, content: &str) -> Res<()> {
    let path = Path::new(filepath);
    if let Err(_) = fs::write(&path, content) {
        return Err("failed to write output".to_string());
    };

    Ok(())
}

/// Writes output assembly file to given directory with given module name.
/// Returns path to written file.
fn write_output_file(dir: &str, pkgname: &str, content: &str) -> Res<PathBuf> {
    let fmtpath = &format!("{}/{}.s", dir, pkgname);
    let path = Path::new(fmtpath);
    if let Err(_) = fs::write(&path, content) {
        return Err("failed to write output".to_string());
    };

    Ok(path.to_path_buf())
}

fn cmd(command: &str, args: &[&str]) -> Res<()> {
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

fn create_dir_if_not_exist(dir: &str) -> Res<()> {
    if !fs::exists(dir).unwrap_or(false) {
        info!("creating directory:{}", dir);
        if let Err(_) = fs::create_dir(dir) {
            return Err(format!("failed to create directory: {}", dir));
        }
    }
    Ok(())
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

/// Create header file for module in given output dir.
fn create_header_file_for_module(outdir: &str, module: &Module, ctx: &TypeContext) -> Res<()> {
    let header = create_header_file(module, &ctx)?;
    write_file(
        &format!("{}/{}.head", outdir, module.modpath.path_underscore()),
        &header,
    )
}
