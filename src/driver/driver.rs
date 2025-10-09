use std::{fs, process::Command, vec};

use crate::{
    ast::TreeSet,
    build::{Builder, TransUnit, X86Builder},
    config,
    driver::{Config, Target},
    error::ErrorSet,
    ir::{IR, IRUnit},
    parser::Parser,
    pkg::Package,
    scanner::Scanner,
    token::{FileSet, Source},
    types::Checker,
};

type Res<T> = Result<T, String>;

/// Compiler entry point and main driver
pub fn compile(config: Config) -> Res<()> {
    let fileset = collect_files_in_directory(&config.srcdir)?;
    let treeset = parse_files(&fileset)?;
    let pkg = type_check_and_create_package(&config.srcdir, fileset, treeset)?;
    let ir_unit = generate_ir_unit(&pkg)?;
    let trans_unit = assemble_ir_unit(&config, ir_unit)?;

    write_output(&config, &pkg, trans_unit)?;
    compile_and_link(vec![&pkg], &config)?;

    Ok(())
}

fn collect_files_in_directory(dir: &str) -> Res<FileSet> {
    let mut files = Vec::new();

    let dirents = match fs::read_dir(&dir) {
        Err(_) => return Err(format!("failed to read directory: '{}'", dir)),
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

    let mut set = FileSet::new();
    for file in files {
        match fs::read(&file) {
            Err(err) => return Err(format!("failed to read file: {}", err)),
            Ok(src) => set.add(Source::new(file, src)),
        }
    }

    Ok(set)
}

fn parse_files(fs: &FileSet) -> Res<TreeSet> {
    let mut errs = ErrorSet::new();
    let mut ts = TreeSet::new();

    // TODO: driver impl
    let c = config::Config::default();

    for file in &fs.files {
        Scanner::scan(file)
            .and_then(|toks| Parser::parse(file, toks, &c))
            .map_or_else(|err| errs.join(err), |ast| ts.add(ast));
    }

    if errs.size() > 0 {
        Err(errs.to_string())
    } else {
        Ok(ts)
    }
}

fn type_check_and_create_package(dir: &str, fs: FileSet, ts: TreeSet) -> Res<Package> {
    let ctx = Checker::check_set(&fs, &ts).map_err(|err| err.to_string())?;
    let ast = TreeSet::join(ts);

    Ok(Package::new(
        "main".to_string(), // TODO: get pkg name from ast
        dir.to_string(),
        fs,
        ast,
        ctx,
    ))
}

fn generate_ir_unit(pkg: &Package) -> Res<IRUnit> {
    IR::emit(&pkg.ast, &pkg.ctx).map_or_else(|err| Err(err.to_string()), |ins| Ok(IRUnit::new(ins)))
}

fn assemble_ir_unit(config: &Config, unit: IRUnit) -> Res<TransUnit> {
    let builder = match config.target {
        Target::X86_64 => X86Builder::new(),
    };

    builder.assemble(unit)
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

fn write_output(config: &Config, pkg: &Package, unit: TransUnit) -> Res<()> {
    if !fs::exists(&config.bindir).unwrap_or(false) {
        if let Err(_) = fs::create_dir(&config.bindir) {
            return Err(format!("failed to create directory: {}", config.bindir));
        }
    }

    if let Err(_) = fs::write(pkg.name_as(&config.bindir, "s"), unit.source) {
        return Err("failed to write output".to_string());
    };

    Ok(())
}

fn compile_and_link(packages: Vec<&Package>, config: &Config) -> Res<()> {
    for pkg in &packages {
        cmd(
            "as",
            &[
                "-o",
                &pkg.name_as(&config.bindir, "o"),
                &pkg.name_as(&config.bindir, "s"),
            ],
        )?;
    }

    let entry_out = &format!("{}/{}", config.bindir, "_entry.o");
    cmd("as", &["-o", entry_out, "lib/entry.s"])?;

    let names = packages
        .iter()
        .map(|pkg| pkg.name_as(&config.bindir, "o"))
        .collect::<Vec<String>>();

    let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();

    let mut args = vec!["-o", &config.outfile, entry_out];
    args.extend_from_slice(&name_refs);
    cmd("ld", &args)
}
