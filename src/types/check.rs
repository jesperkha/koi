use crate::{
    ast::{FileSet, Node},
    config::Config,
    error::{Error, ErrorSet, Res},
    types::{Checker, Exports, Package, TypeContext, TypedAst},
};
use tracing::info;

pub fn type_check(fs: FileSet, config: &Config) -> Res<Package> {
    let mut ctx = TypeContext::new();
    let pkgname = fs.package_id.0.clone();

    // Passes
    check_package_names_equal(&fs, config)?;
    check_one_file_in_main_package(&fs)?;
    resolve_imports(&fs, &mut ctx, config)?;
    global_pass(&fs, &mut ctx, config)?;

    let exports = collect_exports(&ctx);
    let tree = emit_typed_ast(fs, ctx, config)?;

    if config.dump_type_context {
        tree.ctx.dump_context_string();
    }

    Ok(Package::new(pkgname, tree, exports))
}

fn check_one_file_in_main_package(fs: &FileSet) -> Result<(), ErrorSet> {
    if !(&fs.package_id.0 == "main" && fs.files.len() > 1) {
        return Ok(());
    }

    let msg = &format!(
        "at most one file can be part of package 'main', found {}",
        fs.files.len()
    );

    let file = &fs.files[0];
    let pkg = &file.ast.package;
    Err(ErrorSet::new_from(Error::range(
        msg,
        pkg.pos(),
        pkg.end(),
        &file.src,
    )))
}

/// Assert that all package names in the file set are the same.
fn check_package_names_equal(fs: &FileSet, config: &Config) -> Result<(), ErrorSet> {
    if config.anon_packages {
        return Ok(());
    }

    let name = &fs.package_id.0;
    let mut errs = ErrorSet::new();

    fs.files
        .iter()
        .filter(|f| &f.package != name)
        .for_each(|f| {
            errs.add(Error::range(
                &format!("expected package name '{}'", name),
                f.ast.package.pos(),
                f.ast.package.end(),
                &f.src,
            ));
        });

    errs.err_or(())
}

/// Add all global declarations to context.
fn global_pass(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    let mut errs = ErrorSet::new();
    for file in &fs.files {
        Checker::new(&file.src, fs.package_id.clone(), ctx, config)
            .global_pass(&file.ast.decls)
            .map_or_else(|e| errs.join(e), |_| {});
    }

    errs.err_or(())
}

/// Collect all export from TypeContext into an Exports object.
fn collect_exports(ctx: &TypeContext) -> Exports {
    let mut exports = Exports::new();
    ctx.exported_symbols()
        .into_iter()
        .for_each(|s| exports.add(s.0, s.1));

    exports
}

/// Resolve all imported types and symbols.
fn resolve_imports(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    Ok(())
}

/// Emit combined typed AST for all files in set.
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
        Checker::new(&file.src, fs.package_id.clone(), &mut ctx, config)
            .emit_ast(file.ast.decls)
            .map_or_else(|e| errs.join(e), |d| decls.extend(d));
    }

    errs.err_or(TypedAst { ctx, decls })
}
