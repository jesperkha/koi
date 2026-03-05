mod file_check;
mod module_check;

#[cfg(test)]
mod tests;

use module_check::ModuleChecker;

use crate::{
    ast::FileSet,
    context::{Context, CreateModule},
    error::Res,
};

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
