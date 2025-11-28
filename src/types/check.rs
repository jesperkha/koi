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
    if &fs.package_id.0 == "main" && fs.files.len() > 1 {
        let f = &fs.files[0];
        Err(ErrorSet::new_from(Error::range(
            &format!(
                "at most one file can be part of package 'main', found {}",
                fs.files.len()
            ),
            f.ast.package.pos(),
            f.ast.package.end(),
            &f.src,
        )))
    } else {
        Ok(())
    }
}

/// Assert that all package names in the file set are the same.
fn check_package_names_equal(fs: &FileSet, config: &Config) -> Result<(), ErrorSet> {
    if config.anon_packages {
        return Ok(());
    }

    let name = &fs.package_id.0;
    let mut errs = ErrorSet::new();

    for file in &fs.files {
        if &file.package != name {
            errs.add(Error::range(
                &format!("expected package name '{}'", name),
                file.ast.package.pos(),
                file.ast.package.end(),
                &file.src,
            ));
        }
    }

    errs.err_or(())
}

/// Add all global declarations to context.
fn global_pass(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    let mut errs = ErrorSet::new();
    for file in &fs.files {
        let _ = Checker::new(&file.src, fs.package_id.clone(), ctx, config)
            .global_pass(&file.ast.decls)
            .map_err(|e| errs.join(e));
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
        let mut checker = Checker::new(&file.src, fs.package_id.clone(), &mut ctx, config);
        match checker.emit_ast(file.ast.decls) {
            Ok(d) => decls.extend(d),
            Err(e) => errs.join(e),
        };
    }

    errs.err_or(TypedAst { ctx, decls })
}
