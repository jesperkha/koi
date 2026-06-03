mod ast;
mod emit;

use emit::*;
use tracing::info;

use crate::{
    build::{BuildConfig, gcc_available},
    config::{Config, DriverPhase, PathManager},
    imports::LibrarySet,
    ir::ProgramIR,
    util::{FilePath, cmd, write_file},
};

pub fn build(
    ir: ProgramIR,
    buildcfg: BuildConfig,
    config: &Config,
    pm: &PathManager,
    libset: &LibrarySet,
) -> Result<(), String> {
    info!("Building for C. Output: {}", buildcfg.target_name);

    if !gcc_available() {
        return Err("Failed to run gcc. Make sure it's installed and in PATH.".into());
    }

    let mut files = Vec::new();

    for unit in ir.units {
        info!("Emitting module {}", unit.name);
        let filepath = format!("{}/{}.c", buildcfg.tmpdir, unit.name);
        let source = emit(unit, config, pm);

        if matches!(config.driver_phase, DriverPhase::Build) {
            println!("{}", source);
        } else {
            info!("Writing file {}", filepath);
            write_file(&filepath.as_str().into(), source.to_string())?;
            files.push(filepath);
        }
    }

    if matches!(config.driver_phase, DriverPhase::Build) {
        return Ok(());
    }

    let mut linker_flags = vec![];
    for lib in libset.archives() {
        linker_flags.push(format!("{}", lib));
    }

    for lib in buildcfg.additional_libraries {
        linker_flags.push(lib);
    }

    info!("Compiling executable");

    let mut args = files;
    let target_path = FilePath::from(&buildcfg.outdir).join(&buildcfg.target_name);
    args.push(format!("-o{}", target_path));
    args.extend_from_slice(&linker_flags);
    args.push("-lm".into()); // After libraries
    cmd("gcc", &args)?;

    Ok(())
}
