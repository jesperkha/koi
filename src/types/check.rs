use crate::{
    ast::{File, FileSet},
    config::Config,
    error::{Error, ErrorSet, Res},
    module::{CreateModule, Exports, Module, ModuleGraph, ModuleKind, ModulePath, invalid_mod_id},
    types::{Checker, Namespace, TypeContext, TypedAst},
};
use tracing::info;

pub fn type_check<'a>(fs: FileSet, mg: &'a mut ModuleGraph, config: &Config) -> Res<&'a Module> {
    let mut ctx = TypeContext::new();

    // Passes
    resolve_imports(&fs, &mut ctx, mg)?;
    global_pass(&fs, &mut ctx, config)?;

    let exports = collect_exports(&ctx);
    let tree = emit_typed_ast(&fs.modpath, fs.files, ctx, config)?;

    if config.dump_type_context {
        tree.ctx.dump_context_string();
    }

    let create_mod = CreateModule {
        modpath: fs.modpath,
        filepath: fs.path,
        ast: tree,
        exports,
        kind: ModuleKind::User,
    };

    let module = mg.add(create_mod, invalid_mod_id());
    Ok(module)
}

/// Add all global declarations to context.
fn global_pass(fs: &FileSet, ctx: &mut TypeContext, config: &Config) -> Result<(), ErrorSet> {
    let mut errs = ErrorSet::new();
    for file in &fs.files {
        Checker::new(&fs.modpath, &file.src, ctx, config)
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
fn resolve_imports(fs: &FileSet, ctx: &mut TypeContext, mg: &ModuleGraph) -> Result<(), ErrorSet> {
    let mut errs = ErrorSet::new();

    for file in &fs.files {
        for import in &file.ast.imports {
            // Join the imported names into an import path
            let import_path = ModulePath::new(
                import
                    .names
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join("."),
            );

            // Try to get module
            let module = match mg.resolve(&import_path) {
                Ok(module) => module,
                Err(err) => {
                    assert!(import.names.len() > 0, "unchecked missing import name");
                    errs.add(Error::range(
                        &err,
                        &import.names[0].pos,
                        &import.names.last().unwrap().end_pos,
                        &file.src,
                    ));
                    continue;
                }
            };

            // Get namespace name and which token to highlight when reporting
            // duplicate definition error.
            let (name, range) = if let Some(alias) = &import.alias {
                (alias.to_string(), (&alias.pos, &alias.end_pos))
            } else {
                (
                    module.modpath.name().to_owned(),
                    (&import.names[0].pos, &import.names.last().unwrap().end_pos),
                )
            };

            // Add module as namespace
            let ns = Namespace::new(name, module.modpath.clone(), &module.exports, ctx);
            let _ = ctx.set_namespace(ns).map_err(|err| {
                errs.add(Error::range(&err, range.0, range.1, &file.src));
            });

            // Put symbols imported by name directly into context
            for tok in &import.imports {
                let sym = tok.to_string();

                if let Some(kind) = module.exports.get(&sym) {
                    let id = ctx.get_or_intern(kind.clone());
                    let _ = ctx.set_symbol(sym.clone(), id, false).map_err(|err| {
                        errs.add(Error::new(&err, tok, tok, &file.src));
                    });
                } else {
                    errs.add(Error::range(
                        &format!("module '{}' has no export '{}'", module.name(), sym),
                        &tok.pos,
                        &tok.end_pos,
                        &file.src,
                    ));
                }
            }
        }
    }

    errs.err_or(())
}

/// Emit combined typed AST for all files in set.
fn emit_typed_ast(
    modpath: &ModulePath,
    files: Vec<File>,
    mut ctx: TypeContext,
    config: &Config,
) -> Result<TypedAst, ErrorSet> {
    info!("checking {} files", files.len());
    assert!(files.len() > 0, "no files to type check");

    let mut errs = ErrorSet::new();
    let mut decls = Vec::new();

    for file in files {
        Checker::new(modpath, &file.src, &mut ctx, config)
            .emit_ast(file.ast.decls)
            .map_or_else(|e| errs.join(e), |d| decls.extend(d));
    }

    errs.err_or(TypedAst { ctx, decls })
}
