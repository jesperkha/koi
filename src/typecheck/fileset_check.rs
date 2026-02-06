use crate::{
    ast::FileSet, config::Config, error::Res, module::ModuleGraph, typecheck::FileChecker,
    types::TypeContext,
};

/// Type check a list of filesets, producing a module graph and type context.
pub fn check_filesets(filesets: Vec<FileSet>, config: &Config) -> Res<(ModuleGraph, TypeContext)> {
    let mut mg = ModuleGraph::new();
    let mut ctx = TypeContext::new();

    for fs in filesets {
        let checker = FileChecker::new(&mut ctx, config);
        let create_mod = checker.check(fs)?;
        mg.add(create_mod);
    }

    Ok((mg, ctx))
}

/// The FilesetChecker is responsible for type checking an entire fileset,
/// producing a typed module that is added to the module graph.
pub struct FilesetChecker<'a> {
    mg: &'a mut ModuleGraph,
    ctx: &'a mut TypeContext,
    config: &'a Config,
}

impl<'a> FilesetChecker<'a> {
    pub fn new(mg: &'a mut ModuleGraph, ctx: &'a mut TypeContext, config: &'a Config) -> Self {
        Self { mg, ctx, config }
    }

    // /// Type check a fileset and produce a typed module.
    // pub fn check(&mut self, fs: FileSet) -> Res<ModuleId> {
    //     info!(
    //         "Type checking {} files in module {}",
    //         fs.files.len(),
    //         fs.modpath.path()
    //     );

    //     let mut syms = SymbolList::new();
    //     let mut nsl = NamespaceList::new();

    //     // Perform import resolution and global declaration pass
    //     self.resolve_imports(&fs, &mut syms, &mut nsl)?;
    //     self.global_pass(&fs, &mut syms, &mut nsl)?;

    //     // Emit typed AST
    //     let typed_ast = self.emit_typed_ast(&fs.modpath, fs.files, &mut syms, &nsl)?;

    //     let create_mod = CreateModule {
    //         symbols: syms,
    //         modpath: fs.modpath,
    //         kind: ModuleKind::User,
    //         path: fs.path,
    //         namespaces: nsl,
    //         ast: typed_ast,
    //     };

    //     let module = self.mg.add(create_mod);
    //     Ok(module.id)
    // }

    // /// The global pass collects all global declarations and registers them
    // /// in the type context and symbol table.
    // fn global_pass(
    //     &mut self,
    //     fs: &FileSet,
    //     syms: &mut SymbolList,
    //     nsl: &mut NamespaceList,
    // ) -> Result<(), ErrorSet> {
    //     let mut errs = ErrorSet::new();
    //     for file in &fs.files {
    //         info!("Running global pass for file {}", file.src.filepath);
    //         FileChecker::new(&fs.modpath, &file.src, self.ctx, syms, nsl, self.config)
    //             .global_pass(&file.ast.decls)
    //             .map_or_else(|e| errs.join(e), |_| {});
    //     }

    //     errs.err_or(())
    // }

    // /// Resolve imports for all files in set, adding namespaces and symbols
    // /// to the provided lists.
    // fn resolve_imports(
    //     &mut self,
    //     fs: &FileSet,
    //     syms: &mut SymbolList,
    //     nsl: &mut NamespaceList,
    // ) -> Result<(), ErrorSet> {
    //     let mut errs = ErrorSet::new();

    //     for file in &fs.files {
    //         info!("Resolving imports for file {}", file.src.filepath);
    //         for import in &file.ast.imports {
    //             // Join the imported names into an import path
    //             let import_path = ModulePath::new(
    //                 import
    //                     .names
    //                     .iter()
    //                     .map(|t| t.to_string())
    //                     .collect::<Vec<_>>()
    //                     .join("."),
    //             );

    //             // Try to get module
    //             let module = match self.mg.resolve(&import_path) {
    //                 Ok(module) => module,
    //                 Err(err) => {
    //                     assert!(import.names.len() > 0, "unchecked missing import name");
    //                     errs.add(Error::range(
    //                         &err,
    //                         &import.names[0].pos,
    //                         &import.names.last().unwrap().end_pos,
    //                         &file.src,
    //                     ));
    //                     continue;
    //                 }
    //             };

    //             // Get namespace name and which token to highlight when reporting
    //             // duplicate definition error.
    //             let (name, range) = if let Some(alias) = &import.alias {
    //                 (alias.to_string(), (&alias.pos, &alias.end_pos))
    //             } else {
    //                 (
    //                     module.modpath.name().to_owned(),
    //                     (&import.names[0].pos, &import.names.last().unwrap().end_pos),
    //                 )
    //             };

    //             // Add module as namespace
    //             let ns = Namespace::new(name, module);
    //             let _ = nsl.add(ns).map_err(|err| {
    //                 errs.add(Error::range(&err, range.0, range.1, &file.src));
    //             });

    //             // Put symbols imported by name directly into symbol list
    //             for tok in &import.imports {
    //                 let symbol_name = tok.to_string();

    //                 // If the symbol exists we add it to the modules symbol list
    //                 if let Some(export_sym) = module.exports().get(&symbol_name) {
    //                     let _ = syms.add((*export_sym).clone()).map_err(|err| {
    //                         errs.add(Error::new(&err, tok, tok, &file.src));
    //                     });
    //                 } else {
    //                     errs.add(Error::range(
    //                         &format!("module '{}' has no export '{}'", module.name(), symbol_name),
    //                         &tok.pos,
    //                         &tok.end_pos,
    //                         &file.src,
    //                     ));
    //                 }
    //             }
    //         }
    //     }

    //     errs.err_or(())
    // }

    // /// Emit combined typed AST for all files in set.
    // fn emit_typed_ast(
    //     &mut self,
    //     modpath: &ModulePath,
    //     files: Vec<File>,
    //     syms: &mut SymbolList,
    //     nsl: &NamespaceList,
    // ) -> Result<TypedAst, ErrorSet> {
    //     assert!(files.len() > 0, "no files to type check");

    //     let mut errs = ErrorSet::new();
    //     let mut decls = Vec::new();

    //     for file in files {
    //         FileChecker::new(&modpath, &file.src, &mut self.ctx, syms, nsl, self.config)
    //             .emit_ast(file.ast.decls)
    //             .map_or_else(|e| errs.join(e), |d| decls.extend(d));
    //     }

    //     errs.err_or(TypedAst { decls })
    // }
}
