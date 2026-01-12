use crate::{
    ast::{File, FileSet},
    config::Config,
    error::{Error, ErrorSet, Res},
    module::{
        CreateModule, Module, ModuleGraph, ModuleKind, ModulePath, Namespace, NamespaceList,
        SymbolList, invalid_mod_id,
    },
    types::{Checker, TypeContext, TypedAst},
};
use tracing::info;

/// Type check a fileset and produce a typed module.
pub fn type_check<'a>(
    fs: FileSet,
    mg: &'a mut ModuleGraph,
    ctx: &mut TypeContext,
    config: &Config,
) -> Res<&'a Module> {
    let mut syms = SymbolList::new();
    let mut nsl = NamespaceList::new();

    // Perform import resolution and global declaration pass
    resolve_imports(&fs, &mut syms, &mut nsl, mg)?;
    global_pass(&fs, ctx, &mut syms, &mut nsl, config)?;

    // Emit typed AST
    let typed_ast = emit_typed_ast(&fs.modpath, fs.files, ctx, &mut syms, &mut nsl, config)?;

    if config.print_symbol_tables {
        syms.print(fs.modpath.name());
    }

    let create_mod = CreateModule {
        namespaces: nsl,
        symbols: syms,
        modpath: fs.modpath,
        filepath: fs.path,
        ast: typed_ast,
        kind: ModuleKind::User,
    };

    Ok(mg.add(create_mod, invalid_mod_id()))
}

/// The global pass collects all global declarations and registers them
/// in the type context and symbol table.
fn global_pass(
    fs: &FileSet,
    ctx: &mut TypeContext,
    syms: &mut SymbolList,
    nsl: &mut NamespaceList,
    config: &Config,
) -> Result<(), ErrorSet> {
    let mut errs = ErrorSet::new();
    for file in &fs.files {
        Checker::new(&fs.modpath, &file.src, ctx, syms, nsl, config)
            .global_pass(&file.ast.decls)
            .map_or_else(|e| errs.join(e), |_| {});
    }

    errs.err_or(())
}

/// Resolve imports for all files in set, adding namespaces and symbols
/// to the provided lists.
fn resolve_imports(
    fs: &FileSet,
    syms: &mut SymbolList,
    nsl: &mut NamespaceList,
    mg: &ModuleGraph,
) -> Result<(), ErrorSet> {
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
            let ns = Namespace::new(name, module);
            let _ = nsl.add(ns).map_err(|err| {
                errs.add(Error::range(&err, range.0, range.1, &file.src));
            });

            // Put symbols imported by name directly into symbol list
            for tok in &import.imports {
                let symbol_name = tok.to_string();

                // If the symbol exists we add it to the modules symbol list
                if let Some(export_sym) = module.exports().get(&symbol_name) {
                    let _ = syms.add((*export_sym).clone()).map_err(|err| {
                        errs.add(Error::new(&err, tok, tok, &file.src));
                    });
                } else {
                    errs.add(Error::range(
                        &format!("module '{}' has no export '{}'", module.name(), symbol_name),
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
    ctx: &mut TypeContext,
    syms: &mut SymbolList,
    nsl: &mut NamespaceList,
    config: &Config,
) -> Result<TypedAst, ErrorSet> {
    info!("checking {} files", files.len());
    assert!(files.len() > 0, "no files to type check");

    let mut errs = ErrorSet::new();
    let mut decls = Vec::new();

    for file in files {
        Checker::new(modpath, &file.src, ctx, syms, nsl, config)
            .emit_ast(file.ast.decls)
            .map_or_else(|e| errs.join(e), |d| decls.extend(d));
    }

    errs.err_or(TypedAst { decls })
}
