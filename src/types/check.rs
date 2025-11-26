use crate::{
    ast::FileSet,
    config::Config,
    error::{ErrorSet, Res},
    types::{Checker, Package, TypeContext, TypedAst},
};
use tracing::info;

pub fn type_check(fs: FileSet, config: &Config) -> Res<Package> {
    let mut ctx = TypeContext::new();
    let pkgname = fs.package_id.0.clone();

    // Passes
    resolve_imports(&fs, &mut ctx, config)?;
    global_pass(&fs, &mut ctx, config)?;

    // Final tree emition
    let tree = emit_typed_ast(fs, ctx, config)?;
    Ok(Package::new(pkgname, tree))
}

/// Resolve all imported types and symbols.
fn resolve_imports(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    Ok(())
}

/// Add all global declarations to context.
fn global_pass(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    Ok(())
}

fn emit_typed_ast(
    fs: FileSet,
    mut ctx: TypeContext,
    config: &Config,
) -> Result<TypedAst, ErrorSet> {
    info!("checking {} files", fs.files.len());
    assert!(fs.files.len() > 0, "no files to type check");

    let mut errs = ErrorSet::new();
    let mut decls = Vec::new();

    for file in fs.files {
        let mut checker = Checker::new(&file.src, fs.package_id.clone(), &mut ctx, config);
        match checker.emit_ast(file.ast.decls) {
            Ok(d) => decls.extend(d),
            Err(e) => errs.join(e),
        };
    }

    if errs.len() > 0 {
        info!("fail, finished all with {} errors", errs.len());
        return Err(errs);
    }

    // TODO: assert all pkg names equal
    Ok(TypedAst { ctx, decls })
}
