use std::fs;

use crate::{
    ast::TreeSet,
    build::TransUnit,
    driver::Config,
    error::ErrorSet,
    ir::IRUnit,
    parser::Parser,
    pkg::Package,
    scanner::Scanner,
    token::{File, FileSet},
};

type Res<T> = Result<T, String>;

/// Compiler entry point and main driver
pub fn compile(config: Config) -> Res<()> {
    let fileset = collect_files_in_directory(config.srcdir)?;
    let treeset = parse_files(&fileset)?;
    let pkg = type_check_and_create_package(fileset, treeset)?;

    Ok(())
}

fn collect_files_in_directory(dir: String) -> Res<FileSet> {
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
            Ok(src) => set.add(File::new(file, src)),
        }
    }

    Ok(set)
}

fn parse_files(fs: &FileSet) -> Res<TreeSet> {
    let mut errs = ErrorSet::new();
    let mut ts = TreeSet::new();

    for file in &fs.files {
        match Scanner::scan(file).and_then(|toks| Parser::parse(file, toks)) {
            Ok(ast) => ts.add(ast),
            Err(err) => errs.join(err),
        }
    }

    if errs.size() > 0 {
        Err(errs.to_string())
    } else {
        Ok(ts)
    }
}

fn type_check_and_create_package(fs: FileSet, ts: TreeSet) -> Res<Package> {
    // TODO: implement checking of multiple trees together
    todo!()
}

fn generate_ir_unit(pkg: Package) -> Res<IRUnit> {
    todo!()
}

fn assemble_ir_unit(unit: IRUnit) -> Res<TransUnit> {
    todo!()
}

fn compile_all(config: Config, units: Vec<TransUnit>) -> Res<String> {
    todo!()
}
