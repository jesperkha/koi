use std::collections::HashMap;

use tracing::{debug, info};

use crate::{
    ast::{self},
    common::Span,
    context::{Context, CreateModule, CreateSymbol},
    error::{Diagnostics, Report, Res, error_span},
    module::{
        ImportPath, ModuleKind, ModulePath, ModuleSourceFile, ModuleSymbol, ModuleSymbolKind,
        Namespace, NamespaceList, Symbol, SymbolId, SymbolKind, SymbolList, SymbolOrigin,
    },
    typecheck::file_check::FileChecker,
    typecheck::helper::CheckerHelpers,
    types::{FunctionType, PrimitiveType, StructType, TypeId, TypeKind, TypedAst},
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
    /// Index of current file being checked.
    current_file: usize,
    /// Struct names successfully pre-declared in the global pre-pass,
    /// mapping to their placeholder TypeId (empty-fields type).
    predeclared_structs: HashMap<String, TypeId>,
}

impl<'a> CheckerHelpers<'a> for ModuleChecker<'a> {
    fn ctx(&self) -> &Context {
        self.ctx
    }

    fn symbols(&self) -> &SymbolList {
        &self.symbols
    }

    fn get_namespace(&self, name: &str) -> Option<&Namespace> {
        self.file_namespaces[self.current_file].get(name)
    }
}

impl<'a> ModuleChecker<'a> {
    pub(crate) fn new(ctx: &'a mut Context) -> Self {
        let mut s = Self {
            ctx,
            symbols: SymbolList::new(),
            is_main: false,
            file_namespaces: Vec::new(),
            current_file: 0,
            predeclared_structs: HashMap::new(),
        };

        s.initialize_symbol_list();
        s
    }

    /// Create all builtin symbols and types.
    fn initialize_symbol_list(&mut self) {
        self.new_builtin_type("usize", PrimitiveType::U64);
        self.new_builtin_type("u8", PrimitiveType::U8);
        self.new_builtin_type("u16", PrimitiveType::U16);
        self.new_builtin_type("u32", PrimitiveType::U32);
        self.new_builtin_type("u64", PrimitiveType::U64);

        self.new_builtin_type("int", PrimitiveType::I32);
        self.new_builtin_type("i8", PrimitiveType::I8);
        self.new_builtin_type("i16", PrimitiveType::I16);
        self.new_builtin_type("i32", PrimitiveType::I32);
        self.new_builtin_type("i64", PrimitiveType::I64);

        self.new_builtin_type("float", PrimitiveType::F32);
        self.new_builtin_type("f32", PrimitiveType::F32);
        self.new_builtin_type("f64", PrimitiveType::F64);

        self.new_builtin_type("bool", PrimitiveType::Bool);
        self.new_builtin_type("byte", PrimitiveType::U8);
        self.new_builtin_type("string", PrimitiveType::String);
    }

    fn new_builtin_type(&mut self, name: &str, primitive: PrimitiveType) {
        self.create_symbol(CreateSymbol {
            name: name.into(),
            alias: None,
            kind: SymbolKind::Type,
            ty: self.ctx.types.primitive(primitive),
            origin: SymbolOrigin::Intrinsic,
            is_exported: false,
            no_mangle: false,
        })
        .unwrap();
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

        // Go through each import of each file and create a namespace list for each file.
        for (i, file) in fs.files.iter().enumerate() {
            self.current_file = i;

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
                assert!(!import.names.is_empty(), "unchecked missing import name");
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
                diag.add(error_span(
                    &format!("module '{}' has no export '{}'", module.name(), symbol_name),
                    tok,
                ));
                continue;
            };

            let modsym = ModuleSymbol {
                id: *id,
                exported: false,
                kind: ModuleSymbolKind::Imported,
            };

            // If it was already declared, add error
            if let Err(err) = self.symbols.add(symbol_name, modsym) {
                diag.add(error_span(&err, tok));
            }
        }
    }

    // ----------------------- Global pass ----------------------- //

    fn global_pass(&mut self, fs: &ast::FileSet) -> Res<()> {
        let mut diag = Diagnostics::new();

        // Pre-pass: register all struct names so field types can forward-reference other structs.
        for (i, file) in fs.files.iter().enumerate() {
            self.current_file = i;
            for decl in &file.ast.decls {
                if let ast::Decl::Struct(node) = decl {
                    let origin = SymbolOrigin::Module {
                        modpath: fs.modpath.clone(),
                        pos: node.pos().clone(),
                        filename: file.filename.clone(),
                    };
                    if let Err(err) = self.predeclare_struct(node, origin) {
                        diag.add(err);
                    }
                }
            }
        }

        // Main pass: all declarations (structs complete their field registration here).
        for (i, file) in fs.files.iter().enumerate() {
            self.current_file = i;
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
            ast::Decl::Func(node) => {
                let origin = SymbolOrigin::Module {
                    modpath: modpath.clone(),
                    pos: node.pos().clone(),
                    filename: filename.into(),
                };
                self.declare_function_definition(&(**node).clone().into(), origin)
            }
            ast::Decl::Extern(node) => {
                let origin = SymbolOrigin::Extern;
                self.declare_function_definition(node, origin)
            }
            ast::Decl::Type(node) => {
                let origin = SymbolOrigin::Module {
                    modpath: modpath.clone(),
                    pos: node.pos().clone(),
                    filename: filename.into(),
                };
                self.declare_type(node, origin)
            }
            ast::Decl::Struct(node) => {
                // Only process structs that were successfully pre-declared.
                if let Some(placeholder_id) =
                    self.predeclared_structs.remove(&node.name.to_string())
                {
                    self.declare_struct_fields(node, placeholder_id)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn declare_type(
        &mut self,
        node: &ast::TypeDeclNode,
        origin: SymbolOrigin,
    ) -> Result<(), Report> {
        let name = node.name.to_string();

        let ty = {
            let id = self.eval_type(&node.ty)?;
            if node.unique {
                self.ctx
                    .types
                    .get_or_intern(TypeKind::Unique(name.clone(), id))
            } else {
                id
            }
        };

        let symbol = CreateSymbol {
            name,
            alias: None,
            kind: SymbolKind::Type,
            ty,
            origin,
            is_exported: node.public,
            no_mangle: false,
        };

        self.check_symbol_already_declared(&symbol.name, node)?;
        let _ = self.create_symbol(symbol);
        Ok(())
    }

    fn predeclare_struct(
        &mut self,
        node: &ast::StructDeclNode,
        origin: SymbolOrigin,
    ) -> Result<(), Report> {
        let name = node.name.to_string();
        let placeholder = self.ctx.types.get_or_intern(TypeKind::Struct(StructType {
            name: name.clone(),
            fields: vec![],
        }));
        let symbol = CreateSymbol {
            name: name.clone(),
            alias: None,
            kind: SymbolKind::Type,
            ty: placeholder,
            origin,
            is_exported: node.public,
            no_mangle: false,
        };
        self.check_symbol_already_declared(&symbol.name, &node.name)?;
        let _ = self.create_symbol(symbol);
        self.predeclared_structs.insert(name, placeholder);
        Ok(())
    }

    fn declare_struct_fields(
        &mut self,
        node: &ast::StructDeclNode,
        placeholder_id: TypeId,
    ) -> Result<(), Report> {
        let mut fields = Vec::new();
        for field in &node.fields {
            let field_type_id = self.eval_type(&field.typ)?;
            fields.push((field.name.to_string(), field_type_id));
        }

        // Cycle detection: if the placeholder appears in any field's reference graph,
        // the struct directly or transitively contains itself — infinite size.
        for (_, field_ty) in &fields {
            if self.ctx.types.get_all_references(*field_ty).contains(&placeholder_id) {
                return Err(error_span("infinite struct size", &node.name));
            }
        }

        // Update the placeholder in-place with the finalized field list.
        // The symbol already points to placeholder_id and does not need to change.
        // All code that captured placeholder_id now automatically sees the correct struct.
        self.ctx.types.update_struct(placeholder_id, fields);
        Ok(())
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
        let mut no_mangle = is_extern;
        let mut is_inline = false;
        let mut is_naked = false;
        let name = node.name.to_string();
        let mut alias = None;

        // Evaluate modifiers
        for m in &node.modifiers {
            match m.modifier.to_string().as_str() {
                // @nomangle
                // Require the symbol name not to be mangled.
                "nomangle" => {
                    if is_extern {
                        return Err(error_span(
                            "'nomangle' modifier is only allowed for local functions",
                            m,
                        ));
                    }
                    no_mangle = true;
                }
                // @inline
                // Require that the function is inlined.
                "inline" => {
                    if is_extern {
                        return Err(error_span(
                            "'inline' modifier is only allowed for local functions",
                            m,
                        ));
                    }
                    is_inline = true;
                }
                // @naked
                // Omit function entry/exit protocol.
                "naked" => {
                    if is_extern {
                        return Err(error_span(
                            "'naked' modifier is only allowed for local functions",
                            m,
                        ));
                    }
                    is_naked = true;
                }
                // @alias <name>
                // Create alias for external symbol.
                "alias" => {
                    if !is_extern {
                        return Err(error_span(
                            "'alias' modifier is only allowed for extern functions",
                            m,
                        ));
                    }
                    if m.args.len() != 1 {
                        return Err(error_span(
                            &format!(
                                "'alias' modifier expects exactly one argument, got {}",
                                m.args.len()
                            ),
                            m,
                        ));
                    }
                    let new_name = m.args[0].to_string(); // len asserted
                    self.symbols.set_alias(name.clone(), new_name.clone());
                    alias = Some(new_name);
                }
                _ => {
                    return Err(error_span("unknown modifier", m));
                }
            }
        }

        let symbol = CreateSymbol {
            name,
            alias,
            kind: SymbolKind::Function {
                is_inline,
                is_naked,
            },
            no_mangle,
            ty,
            origin,
            is_exported: node.public,
        };

        debug!("declaring function: {:?}", symbol);

        self.check_symbol_already_declared(&symbol.name, node)?;
        let _ = self.create_symbol(symbol);
        Ok(())
    }

    fn check_symbol_already_declared(&self, name: &str, node: &dyn Span) -> Result<(), Report> {
        if let Ok(sym) = self.get_symbol(name) {
            let mut report = error_span("already declared", node);

            if let SymbolOrigin::Module { pos, filename, .. } = &sym.origin {
                report = report.with_info(&format!(
                    "previously declared in {}, line {}",
                    filename,
                    pos.row + 1
                ));
            }

            return Err(report);
        };
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

        for (file, nsl) in ast_files.into_iter().zip(all_nsl) {
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

    /// Evaluate an option of a type node. Defaults to void type if not present.
    fn eval_optional_type(&mut self, v: &Option<ast::TypeNode>) -> Result<TypeId, Report> {
        v.as_ref()
            .map_or(Ok(self.ctx.types.void()), |r| self.eval_type(r))
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

        let kind = match symbol.origin {
            SymbolOrigin::Module { .. } | SymbolOrigin::Intrinsic => ModuleSymbolKind::Module,
            SymbolOrigin::Library(_) | SymbolOrigin::Extern => ModuleSymbolKind::Imported,
        };

        let exported = symbol.is_exported;
        let name = symbol
            .alias
            .as_ref()
            .map_or(symbol.name.clone(), |alias| alias.clone());
        let id = self.ctx.symbols.add(symbol);
        let _ = self.symbols.add(name, ModuleSymbol { id, kind, exported }); // Checked earlier
        Ok(id)
    }
}
