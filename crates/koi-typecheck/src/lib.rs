mod file_check;
mod module_check;

use module_check::ModuleChecker;

use koi_ast::FileSet;
use koi_common::error::Res;
use koi_sema::{Context, CreateModule};

/// Type check a list of filesets, producing a module graph and type context.
pub fn check_filesets(ctx: &mut Context, filesets: Vec<FileSet>) -> Res<()> {
    for fs in filesets {
        let create_mod = check_fileset(ctx, fs)?;
        ctx.modules.add(create_mod);
    }
    Ok(())
}

/// Type check single FileSet into a module.
pub fn check_fileset(ctx: &mut Context, fs: FileSet) -> Res<CreateModule> {
    let checker = ModuleChecker::new(ctx);
    checker.check(fs)
}
