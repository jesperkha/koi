use std::{
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
};

use tracing::info;
use walkdir::WalkDir;

use crate::{
    ast::File,
    build::{TransUnit, X86Builder, assemble},
    config::Config,
    error::ErrorSet,
    ir::{IRUnit, emit_ir},
    parser::parse,
    scanner::scan,
    token::Source,
    types::{Package, check},
};

type Res<T> = Result<T, String>;

pub enum Target {
    X86_64,
}

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

pub struct Driver<'a> {
    config: &'a Config,
}

impl<'a> Driver<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn compile(&mut self, config: BuildConfig) -> Res<()> {
        if !fs::exists(&config.bindir).unwrap_or(false) {
            if let Err(_) = fs::create_dir(&config.bindir) {
                return Err(format!("failed to create directory: {}", config.bindir));
            }
        }

        let source_dirs = list_source_directories(&config.srcdir)?;
        let mut asm_files = Vec::new();

        for dir in &source_dirs {
            let sources = collect_files_in_directory(dir)?;
            let file = self.parse_files(sources)?;
            let pkg = self.type_check_and_create_package(file)?;
            let ir_unit = self.emit_package_ir(&pkg)?;
            let asm = self.assemble_ir_unit(ir_unit, &config.target)?;

            let outfile = write_output_file(&config.bindir, &pkg.name, &asm.source)?;
            info!("output assembly file: {}", outfile.display());
            asm_files.push(outfile);
        }

        for file in &asm_files {
            info!("assembling: {}", file.display());
            let src = file.to_string_lossy();
            let out = file.with_extension("o");
            cmd("as", &["-o", &out.to_string_lossy(), &src])?;
        }

        let entry_o = format!("{}/entry.o", config.bindir);
        cmd("as", &["-o", &entry_o, "lib/compile/entry.s"])?;

        let mut objectfiles = vec![entry_o];
        for file in asm_files {
            objectfiles.push(file.with_extension("o").to_string_lossy().to_string());
        }

        let mut args = vec!["-o", &config.outfile, "-nostdlib"];
        args.extend_from_slice(
            &objectfiles
                .iter()
                .map(|f| f.as_str())
                .collect::<Vec<&str>>(),
        );

        cmd("ld", &args)?;

        Ok(())
    }

    fn parse_files(&self, sources: Vec<Source>) -> Res<Vec<File>> {
        let mut errs = ErrorSet::new();
        let mut files = Vec::new();

        for src in sources {
            scan(&src, self.config)
                .and_then(|toks| parse(src, toks, self.config))
                .map_or_else(|err| errs.join(err), |file| files.push(file));
        }

        if errs.len() > 0 {
            Err(errs.to_string())
        } else {
            Ok(files)
        }
    }

    fn type_check_and_create_package(&self, files: Vec<File>) -> Res<Package> {
        check(files, self.config).map_err(|errs| errs.to_string())
    }

    fn emit_package_ir(&self, pkg: &Package) -> Res<IRUnit> {
        emit_ir(pkg, self.config).map_err(|errs| errs.to_string())
    }

    fn assemble_ir_unit(&self, unit: IRUnit, target: &Target) -> Res<TransUnit> {
        match target {
            Target::X86_64 => assemble::<X86Builder>(self.config, unit),
        }
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

/// Writes output assembly file to given directory with given package name.
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

// fn write_output(config: &Config, pkg: &Package, unit: TransUnit) -> Res<()> {
//     if !fs::exists(&config.bindir).unwrap_or(false) {
//         if let Err(_) = fs::create_dir(&config.bindir) {
//             return Err(format!("failed to create directory: {}", config.bindir));
//         }
//     }

//     if let Err(_) = fs::write(pkg.name_as(&config.bindir, "s"), unit.source) {
//         return Err("failed to write output".to_string());
//     };

//     Ok(())
// }

// fn compile_and_link(packages: Vec<&Package>, config: &Config) -> Res<()> {
//     for pkg in &packages {
//         cmd(
//             "as",
//             &[
//                 "-o",
//                 &pkg.name_as(&config.bindir, "o"),
//                 &pkg.name_as(&config.bindir, "s"),
//             ],
//         )?;
//     }

//     let entry_out = &format!("{}/{}", config.bindir, "_entry.o");
//     cmd("as", &["-o", entry_out, "lib/entry.s"])?;

//     let names = packages
//         .iter()
//         .map(|pkg| pkg.name_as(&config.bindir, "o"))
//         .collect::<Vec<String>>();

//     let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

//     let mut args = vec!["-o", &config.outfile, entry_out];
//     args.extend_from_slice(&name_refs);
//     cmd("ld", &args)
// }
