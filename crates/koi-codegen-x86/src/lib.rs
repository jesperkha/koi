mod assemble;
mod assembly;

use assemble::assemble;
use tracing::info;

use koi_common::{
    config::{BuildConfig, Config, DriverPhase, LinkMode, PathManager, gcc_available},
    util::{FilePath, cmd, write_file},
};
use koi_ir::ProgramIR;
use koi_sema::LibrarySet;

/// Build and compile an x86-64 executable or shared object file.
pub fn build(
    ir: ProgramIR,
    buildcfg: BuildConfig,
    config: &Config,
    pm: &PathManager,
    libset: &LibrarySet,
) -> Result<(), String> {
    info!("Building for x86-64. Output: {}", buildcfg.target_name);

    if !gcc_available() {
        return Err("Failed to run gcc. Make sure it's installed and in PATH.".into());
    }

    let mut asm_files = Vec::new();

    for unit in ir.units {
        info!("Assembling module {}", unit.name);
        let filepath = format!("{}/{}.s", buildcfg.tmpdir, unit.name);
        let source = assemble(unit, config);

        if matches!(config.driver_phase, DriverPhase::Build) {
            println!("{}", source);
        } else {
            info!("Writing file {}", filepath);
            write_file(&filepath.as_str().into(), source.to_string())?;
            asm_files.push(filepath);
        }
    }

    // Finished assembly phase, exit early if specified.
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

    match buildcfg.linkmode {
        LinkMode::Executable => {
            info!("Compiling executable");

            let mut args = asm_files;
            args.push("-nostartfiles".into());

            let entry_file = pm.library_path().join("entry.s");
            args.push(entry_file.to_string());
            let target_path = FilePath::from(&buildcfg.outdir).join(&buildcfg.target_name);
            args.push(format!("-o{}", target_path));
            args.extend_from_slice(&linker_flags);
            args.push("-lm".into()); // After libraries
            cmd("gcc", &args)?;
        }
        LinkMode::Library => {
            info!("Compiling static library");

            let mut objfiles = Vec::new();
            for asmfile in &asm_files {
                let objfile = asmfile.replace(".s", ".o");
                cmd(
                    "gcc",
                    &[
                        "-nostartfiles".into(),
                        "-c".into(),
                        asmfile.into(),
                        format!("-o{}", objfile),
                    ],
                )?;
                objfiles.push(objfile);
            }
            let target_path =
                FilePath::from(&buildcfg.outdir).join(&format!("lib{}.a", buildcfg.target_name));

            let mut args = vec!["rcs".into(), target_path.to_string()];
            args.extend_from_slice(&objfiles);
            args.extend_from_slice(&linker_flags);
            cmd("ar", &args)?;
        }
    }

    Ok(())
}
