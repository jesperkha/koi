use crate::{
    ast::FileSet,
    config::Config,
    error::{ErrorSet, Res},
    types::{Checker, Package, TypeContext},
};
use tracing::info;

pub fn type_check(fs: FileSet, config: &Config) -> Res<Package> {
    let mut ctx = TypeContext::new();
    let mut errs = ErrorSet::new();

    let passes = vec![resolve_imports, global_pass, check_fileset];

    passes.iter().for_each(|p| {
        let _ = p(&fs, &mut ctx, config).map_err(|e| errs.join(e));
    });

    if errs.len() > 0 {
        return Err(errs);
    }

    Ok(Package::new(fs.package_id.to_string(), fs, ctx))
}

/// Resolve all imported types and symbols.
fn resolve_imports(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    Ok(())
}

/// Add all global declarations to context.
fn global_pass(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    Ok(())
}

/// Type check files.
fn check_fileset(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    info!("checking {} files", fs.files.len());
    assert!(fs.files.len() > 0, "no files to type check");

    let mut errs = ErrorSet::new();

    for file in &fs.files {
        let checker = Checker::new(file, ctx, config);
        errs.join(checker.check());
    }

    if errs.len() > 0 {
        info!("fail, finished all with {} errors", errs.len());
        return Err(errs);
    }

    // TODO: assert all pkg names equal
    Ok(())
}
