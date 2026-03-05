use tracing::{debug, info};

use crate::{
    ast::{self, Node},
    context::{Context, CreateModule, CreateSymbol},
    error::{Diagnostics, Report, Res},
    module::{
        ImportPath, ModuleKind, ModulePath, ModuleSourceFile, ModuleSymbol, Namespace,
        NamespaceList, Symbol, SymbolId, SymbolKind, SymbolList, SymbolOrigin,
    },
    typecheck::file_check::FileChecker,
    types::{FunctionType, PrimitiveType, TypeId, TypeKind, TypedAst},
};

/// Performs module-level checks: import resolution, global symbol declaration,
/// and orchestrates per-file type checking.
pub(crate) struct ModuleChecker<'a> {
    ctx: &'a mut Context,

    /// Is this the main module?
    is_main: bool,
    /// Module-level symbol cache
    symbols: SymbolList,
    /// Per-file namespace lists, indexed by file order.
    file_namespaces: Vec<NamespaceList>,
}

impl<'a> ModuleChecker<'a> {
    pub(crate) fn new(ctx: &'a mut Context) -> Self {
        Self {
            ctx,
            symbols: SymbolList::new(),
            is_main: false,
            file_namespaces: Vec::new(),
        }
    }

    pub(crate) fn check(mut self, fs: ast::FileSet) -> Res<CreateModule> {
        self.is_main = fs.modpath.is_main();

        // The first step of type checking is to resolve all imports in this module.
        // Each import is resolved to a source or external module and it is added as
        // a namespace in this module.
        self.resolve_all_imports(&fs)?;

        // The second step is to do a global pass and pre-declare all top-level function- and type
        // definitions. This is to ensure that function bodies can reference symbols declared later
        // in the same file or another file in the same module.
        self.global_pass(&fs)?;

        // The third step is to perform the actual type checking on the function bodies. This
        // generates a typed AST along with additional semantic information.
        let files = self.emit_module_files(fs.files)?;

        Ok(CreateModule {
            modpath: fs.modpath,
            kind: ModuleKind::Source {
                filepath: fs.filepath,
                files,
            },
            symbols: self.symbols,
        })
    }

    // ----------------------- Import resolution ----------------------- //

    fn resolve_all_imports(&mut self, fs: &ast::FileSet) -> Res<()> {
        let mut diag = Diagnostics::new();

        for file in &fs.files {
            info!("Resolving imports for file {}", file.filepath);
            let mut nsl = NamespaceList::new();
            for import in &file.ast.imports {
                self.resolve_import(import, &mut nsl, &mut diag);
            }
            self.file_namespaces.push(nsl);
        }

        if !diag.is_empty() {
            return Err(diag);
        }

        Ok(())
    }

    fn resolve_import(
        &mut self,
        import: &ast::ImportNode,
        nsl: &mut NamespaceList,
        diag: &mut Diagnostics,
    ) {
        let impath = ImportPath::from(import);

        // Try to get module
        let module = match self.ctx.modules.resolve(&impath) {
            Ok(module) => module,
            Err(err) => {
                assert!(import.names.len() > 0, "unchecked missing import name");
                diag.add(Report::code_error(
                    &err,
                    &import.names[0].pos,
                    &import.names.last().unwrap().end_pos,
                ));
                return;
            }
        };

        // Get namespace name and which token to highlight when reporting
        // duplicate definition error.
        let (namespace_name, range) = if let Some(alias) = &import.alias {
            (alias.to_string(), (&alias.pos, &alias.end_pos))
        } else {
            (
                impath.name().to_owned(),
                (&import.names[0].pos, &import.names.last().unwrap().end_pos),
            )
        };

        // Add module as namespace
        let ns = Namespace::new(namespace_name, module);
        let _ = nsl.add(ns).map_err(|err| {
            diag.add(Report::code_error(&err, range.0, range.1));
        });

        let module_exports = module.exports();

        // Go through each named imported symbol and add it to the cache
        for tok in &import.imports {
            let symbol_name = tok.to_string();

            // Check if the symbol exists
            let Some(id) = module_exports.get(&symbol_name) else {
                // Module did not contain the symbol
                diag.add(Report::code_error(
                    &format!("module '{}' has no export '{}'", module.name(), symbol_name),
                    &tok.pos,
                    &tok.end_pos,
                ));
                continue;
            };

            let modsym = ModuleSymbol {
                id: *id,
                exported: false, // Imported symbols should not be re-exported
            };

            // If it was already declared, add error
            if let Err(err) = self.symbols.add(symbol_name, modsym) {
                diag.add(Report::code_error(&err, &tok.pos, &tok.end_pos));
            }
        }
    }

    // ----------------------- Global pass ----------------------- //

    fn global_pass(&mut self, fs: &ast::FileSet) -> Res<()> {
        let mut diag = Diagnostics::new();

        for file in &fs.files {
            for decl in &file.ast.decls {
                if let Err(err) = self.check_global_decl(&fs.modpath, &file.filename, decl) {
                    diag.add(err);
                }
            }
        }

        if !diag.is_empty() {
            return Err(diag);
        }

        Ok(())
    }

    fn check_global_decl(
        &mut self,
        modpath: &ModulePath,
        filename: &str,
        decl: &ast::Decl,
    ) -> Result<(), Report> {
        match decl {
            ast::Decl::FuncDecl(node) => {
                let origin = SymbolOrigin::Module {
                    modpath: modpath.clone(),
                    pos: node.pos().clone(),
                    filename: filename.into(),
                };
                self.declare_function_definition(node, origin)
            }
            ast::Decl::Func(node) => {
                let origin = SymbolOrigin::Module {
                    modpath: modpath.clone(),
                    pos: node.pos().clone(),
                    filename: filename.into(),
                };
                self.declare_function_definition(&node.clone().into(), origin)
            }
            ast::Decl::Extern(node) => {
                let origin = SymbolOrigin::Extern;
                self.declare_function_definition(node, origin)
            }
            _ => Ok(()),
        }
    }

    fn declare_function_definition(
        &mut self,
        node: &ast::FuncDeclNode,
        origin: SymbolOrigin,
    ) -> Result<(), Report> {
        // Evaluate return type if any
        let ret = self.eval_optional_type(&node.ret_type)?;

        // Get parameter types
        let param_ids = &node
            .params
            .iter()
            .map(|f| self.eval_type(&f.typ).map(|id| (&f.name, id)))
            .collect::<Result<Vec<_>, _>>()?;

        // Declare function in context
        let ty = self
            .ctx
            .types
            .get_or_intern(TypeKind::Function(FunctionType {
                params: param_ids.iter().map(|v| v.1).collect(),
                ret,
            }));

        let is_extern = matches!(origin, SymbolOrigin::Extern);
        let no_mangle = is_extern;

        let symbol = CreateSymbol {
            name: node.name.to_string(),
            kind: SymbolKind::Function {
                is_inline: false,
                is_naked: false,
            },
            no_mangle,
            ty,
            origin,
            is_exported: node.public,
        };

        debug!("declaring function: {:?}", symbol);

        // If symbol already exists, return error
        if let Ok(sym) = self.get_symbol(&symbol.name) {
            let mut report =
                Report::code_error("already declared", &node.name.pos, &node.name.end_pos);

            if let SymbolOrigin::Module { pos, filename, .. } = &sym.origin {
                report = report.with_info(&format!(
                    "previously declared in {}, line {}",
                    filename,
                    pos.row + 1
                ));
            }

            return Err(report);
        };

        let _ = self.create_symbol(symbol);
        Ok(())
    }

    // ----------------------- File-level emission ----------------------- //

    fn emit_module_files(
        &mut self,
        ast_files: Vec<ast::File>,
    ) -> Result<Vec<ModuleSourceFile>, Diagnostics> {
        let mut files = Vec::new();

        // Take file_namespaces out of self so we can iterate while borrowing self mutably
        let all_nsl = std::mem::take(&mut self.file_namespaces);

        for (file, nsl) in ast_files.into_iter().zip(all_nsl.into_iter()) {
            info!("Type check: {}", file.filepath);

            let mut file_checker = FileChecker::new(self.ctx, &self.symbols, nsl, self.is_main);

            let decls = file_checker.emit_ast(file.ast)?;
            let nsl = file_checker.into_namespaces();
            let ast = TypedAst { decls };

            files.push(ModuleSourceFile {
                filename: file.filename,
                namespaces: nsl,
                ast,
            });
        }

        Ok(files)
    }

    // ----------------------- Shared helpers ----------------------- //

    /// Evaluate an AST type node to its semantic type id.
    fn eval_type(&self, node: &ast::TypeNode) -> Result<TypeId, Report> {
        match node {
            ast::TypeNode::Primitive(token) => {
                let prim = PrimitiveType::from(&token.kind);
                Ok(self.ctx.types.primitive(prim))
            }
            ast::TypeNode::Ident(token) => self.get_symbol_type_id(token).map_or(
                Err(Report::code_error("not a type", &token.pos, &token.end_pos)),
                Ok,
            ),
        }
    }

    /// Evaluate an option of a type node. Defaults to void type if not present.
    fn eval_optional_type(&mut self, v: &Option<ast::TypeNode>) -> Result<TypeId, Report> {
        v.as_ref()
            .map_or(Ok(self.ctx.types.void()), |r| self.eval_type(r))
    }

    fn get_symbol_type_id(&self, name: &ast::Token) -> Option<TypeId> {
        let name_str = name.to_string();
        self.get_symbol(&name_str).ok().map(|sym| sym.ty)
    }

    /// Get a Symbol by name. The name is local to this module only.
    /// Returns "not declared" on error.
    fn get_symbol(&self, name: &str) -> Result<&Symbol, String> {
        self.symbols
            .get(name)
            .map_or(Err("not declared".to_string()), |sym| {
                Ok(self.ctx.symbols.get(sym.id))
            })
    }

    /// Create a new symbol and add it to the local cache. Returns "already declared" on error.
    fn create_symbol(&mut self, symbol: CreateSymbol) -> Result<SymbolId, String> {
        if self.symbols.get(&symbol.name).is_ok() {
            return Err("already declared".to_string());
        }
        let name = symbol.name.clone();
        let exported = symbol.is_exported;
        let id = self.ctx.symbols.add(symbol);
        let _ = self.symbols.add(name, ModuleSymbol { id, exported }); // Checked earlier
        Ok(id)
    }
}
