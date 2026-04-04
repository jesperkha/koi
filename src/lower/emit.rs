use std::collections::{HashMap, HashSet};

use crate::{
    context::Context,
    error::{self, Diagnostics, Report},
    ir::{
        AssignIns, BinaryIns, Block, CallIns, ConstId, Data, DataIndex, Decl, ElseIf, ExternDecl,
        FuncDecl, IRBinaryOp, IRType, IRTypeInterner, IRUnaryOp, IfIns, Ins, LValue, ParamId,
        Primitive, RValue, StoreIns, UnaryIns, Unit,
    },
    module::{
        Module, ModuleId, ModuleKind, ModuleSourceFile, NamespaceList, Symbol, SymbolId,
        SymbolList, SymbolOrigin,
    },
    types::{self, BinaryOp, Expr, LiteralKind, TypedAst, TypedNode, UnaryOp},
    util::VarTable,
};

/// Emit standalone module IR unit for this module.
pub fn emit_ir(ctx: &Context, id: ModuleId) -> error::Res<Unit> {
    let module = ctx.modules.get(id);
    let emitter = ModuleEmitter::new(ctx, module);
    let unit = emitter.emit()?;
    Ok(unit)
}

type Res<T> = Result<T, Report>;

/// Utility type for storing module data segments.
struct DataInterner {
    data: Vec<Data>,
    string_map: HashMap<String, DataIndex>,
}

impl DataInterner {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            string_map: HashMap::new(),
        }
    }

    fn into_data(self) -> Vec<Data> {
        self.data
    }

    /// Intern new data segment, returns the unique DataIndex.
    fn intern(&mut self, data: Data) -> DataIndex {
        let index = self.data.len();
        self.data.push(data);
        index
    }

    /// Either get the string data index by cache or intern it.
    fn get_or_intern_string(&mut self, s: String) -> DataIndex {
        if let Some(index) = self.string_map.get(&s) {
            *index
        } else {
            let id = self.intern(Data::String(s.clone()));
            self.string_map.insert(s, id);
            id
        }
    }
}

/// ModuleEmitter handles module-level IR emission and
/// bundles the necessary metadata in the Unit.
struct ModuleEmitter<'a> {
    ctx: &'a Context,
    module: &'a Module,
    types: IRTypeInterner,
}

impl<'a> ModuleEmitter<'a> {
    fn new(ctx: &'a Context, module: &'a Module) -> Self {
        Self {
            ctx,
            module,
            types: IRTypeInterner::new(),
        }
    }

    /// Emit IR for all module files and create bundled IR unit.
    fn emit(mut self) -> error::Res<Unit> {
        let ModuleKind::Source { files, .. } = &self.module.kind else {
            unreachable!();
        };

        let mut data = DataInterner::new();
        let mut externs = HashSet::new();
        let mut decls = Vec::new();

        // Emit IR for each file in the module
        for file in files {
            let emitter = FileEmitter::new(
                self.ctx,
                &self.module.symbols,
                &mut self.types,
                file,
                &mut data,
            );
            let result = emitter.emit()?;
            decls.extend(result.decls);
            externs.extend(result.externs);
        }

        // Declare all imported symbols as extern
        let mut extern_decls = Vec::new();
        externs.extend(self.module.imports());
        for id in externs {
            let mut diag = Diagnostics::new();

            match self.emit_extern(id) {
                Ok(decl) => extern_decls.push(decl),
                Err(report) => diag.add(report),
            }

            if !diag.is_empty() {
                return Err(diag);
            }
        }

        // TODO: tidy up list stuff
        extern_decls.extend(decls);

        Ok(Unit {
            name: self.module.modpath.to_underscore(),
            data: data.into_data(),
            types: self.types,
            decls: extern_decls,
        })
    }

    fn emit_extern(&mut self, id: SymbolId) -> Res<Decl> {
        let symbol = self.ctx.symbols.get(id);
        let func = self.ctx.types.try_function(symbol.ty).unwrap();

        Ok(Decl::Extern(ExternDecl {
            name: mangle_symbol_name(self.ctx, symbol),
            params: self.types.to_ir_type_list(self.ctx, &func.params),
            ret: self.types.to_ir(self.ctx, func.ret),
        }))
    }
}

/// FileEmitter handles file-level emission and writes
/// to the parent modules shared state.
pub struct FileEmitter<'a> {
    ctx: &'a Context,
    symbols: &'a SymbolList,
    types: &'a mut IRTypeInterner,
    nsl: &'a NamespaceList,
    ast: &'a TypedAst,
    data: &'a mut DataInterner,

    const_id: ConstId,
    vars: VarTable<ConstId>,
    params: VarTable<ParamId>,
    stacksize: usize,

    /// Cache of already declared extern symbols
    externs: HashSet<SymbolId>,
}

struct EmitResult {
    /// Symbol ids of all extern functions used.
    externs: HashSet<SymbolId>,
    /// Top-level declarations.
    decls: Vec<Decl>,
}

impl<'a> FileEmitter<'a> {
    fn new(
        ctx: &'a Context,
        symbols: &'a SymbolList,
        types: &'a mut IRTypeInterner,
        file: &'a ModuleSourceFile,
        data: &'a mut DataInterner,
    ) -> Self {
        Self {
            ctx,
            symbols,
            data,
            types,
            nsl: &file.namespaces,
            ast: &file.ast,
            const_id: 0,
            stacksize: 0,
            vars: VarTable::new(),
            params: VarTable::new(),
            externs: HashSet::new(),
        }
    }

    /// Emit IR for this file. Mutates shared module state.
    fn emit(mut self) -> error::Res<EmitResult> {
        let mut diag = Diagnostics::new();
        let mut decls = Vec::new();

        for decl in &self.ast.decls {
            let res = match decl {
                types::Decl::Func(node) => self.emit_func(node),

                // extern symbols are declared at top using modules symbols list
                types::Decl::Extern(_) => continue,
            };

            match res {
                Ok(decl) => decls.push(decl),
                Err(report) => diag.add(report),
            }
        }

        Ok(EmitResult {
            decls,
            externs: self.externs,
        })
    }

    /// Get next unique constant id to be used in this scope.
    fn next_id(&mut self) -> ConstId {
        let id = self.const_id;
        self.const_id += 1;
        id
    }

    fn emit_func(&mut self, node: &types::FuncNode) -> Res<Decl> {
        self.push_scope();

        // Bind parameters to local ids
        for (i, param) in node.params.iter().enumerate() {
            self.params.bind(param.clone(), i);
        }

        let body = self.emit_func_block(&node.body.stmts)?;
        self.pop_scope();

        // Get function type
        let func = self.ctx.types.try_function(node.ty).unwrap();
        let params = self.types.to_ir_type_list(self.ctx, &func.params);
        let ret = self.types.to_ir(self.ctx, func.ret);

        Ok(Decl::Func(FuncDecl {
            public: node.public,
            name: self.to_mangled_name(&node.name),
            stacksize: self.stacksize,
            body,
            params,
            ret,
        }))
    }

    fn emit_func_block(&mut self, nodes: &Vec<types::Stmt>) -> Res<Block> {
        let mut ins = Vec::new();

        for node in nodes {
            self.emit_stmt(&mut ins, node)?;
        }

        // Add explicit return statement if function has no return value
        if !ins
            .last()
            .is_some_and(|ins| matches!(ins, Ins::Return(_, _)))
        {
            ins.push(Ins::Return(
                self.types.get_or_intern(IRType::Primitive(Primitive::Void)),
                RValue::Void,
            ));
        }

        Ok(Block { ins })
    }

    // The methods below all emit a variable number of instructions and therefore return no value.
    // -------------------------------------------------------------------------------------------

    fn emit_stmt(&mut self, ins: &mut Vec<Ins>, node: &types::Stmt) -> Res<()> {
        match node {
            types::Stmt::Return(node) => self.emit_return(ins, node)?,
            types::Stmt::ExprStmt(node) => {
                let _ = self.expr_to_rval(ins, node)?;
            }
            types::Stmt::VarDecl(node) => self.emit_var_decl(ins, node)?,
            types::Stmt::VarAssign(node) => self.emit_var_assign(ins, node)?,
            types::Stmt::If(node) => self.emit_if(ins, node)?,
            types::Stmt::While(node) => todo!(),
        };
        Ok(())
    }

    fn emit_block(&mut self, node: &types::BlockNode) -> Res<Block> {
        let mut ins = Vec::new();
        for stmt in &node.stmts {
            self.emit_stmt(&mut ins, stmt)?;
        }
        Ok(Block { ins })
    }

    fn emit_if(&mut self, ins: &mut Vec<Ins>, node: &types::IfNode) -> Res<()> {
        let cond = self.expr_to_rval(ins, &node.expr)?;
        let block = self.emit_block(&node.block)?;

        let mut elseifs = Vec::new();
        let mut elseif = &*node.elseif;
        let mut elseblock: Option<Block> = None;

        loop {
            match elseif {
                types::ElseBlock::ElseIf(node) => {
                    let mut cond_ins = Vec::new();
                    let cond = self.expr_to_rval(&mut cond_ins, &node.expr)?;
                    let block = self.emit_block(&node.block)?;
                    elseifs.push(ElseIf {
                        cond_ins,
                        cond,
                        block,
                    });
                    elseif = &node.elseif;
                }
                types::ElseBlock::Else(node) => {
                    let block = self.emit_block(node)?;
                    elseblock = Some(block);
                    break;
                }
                types::ElseBlock::None => break,
            };
        }

        ins.push(Ins::If(IfIns {
            cond,
            block,
            elseif: elseifs,
            elseblock,
        }));
        Ok(())
    }

    fn emit_var_assign(&mut self, ins: &mut Vec<Ins>, node: &types::VarAssignNode) -> Res<()> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let rval = self.expr_to_rval(ins, &node.rval)?;
        let lval = self.expr_to_lval(ins, &node.lval)?;
        ins.push(Ins::Assign(AssignIns { ty, lval, rval }));
        Ok(())
    }

    fn emit_var_decl(&mut self, ins: &mut Vec<Ins>, node: &types::VarDeclNode) -> Res<()> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let rval = self.expr_to_rval(ins, &node.value)?;
        let const_id = self.next_id();
        ins.push(Ins::Store(StoreIns { ty, const_id, rval }));

        self.stacksize += self.types.sizeof(ty);

        // Bind locally to look up its ConstId later
        self.vars.bind(node.name.clone(), const_id);
        Ok(())
    }

    // Conversions to R-Value
    // ----------------------

    fn emit_return(&mut self, ins: &mut Vec<Ins>, node: &types::ReturnNode) -> Res<()> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let rval = match &node.expr {
            None => RValue::Void,
            Some(expr) => self.expr_to_rval(ins, expr)?,
        };

        ins.push(Ins::Return(ty, rval));
        Ok(())
    }

    fn expr_to_rval(&mut self, ins: &mut Vec<Ins>, expr: &types::Expr) -> Res<RValue> {
        match expr {
            Expr::Literal(node) => self.lit_to_rval(node),
            Expr::Call(node) => self.call_to_rval(ins, node),
            Expr::NamespaceMember(node) => self.namespace_to_rval(node),
            Expr::Member(_) => todo!(),
            Expr::Binary(node) => self.binary_to_rval(ins, node),
            Expr::Unary(node) => self.unary_to_rval(ins, node),
        }
    }

    fn unary_to_rval(&mut self, ins: &mut Vec<Ins>, node: &types::UnaryNode) -> Res<RValue> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let rhs = self.expr_to_rval(ins, &node.rhs)?;
        let result = self.next_id();
        ins.push(Ins::Unary(UnaryIns {
            ty,
            op: node.op.clone().into(),
            rhs,
            result,
        }));
        Ok(RValue::Const(result))
    }

    fn binary_to_rval(&mut self, ins: &mut Vec<Ins>, node: &types::BinaryNode) -> Res<RValue> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let lhs = self.expr_to_rval(ins, &node.lhs)?;
        let rhs = self.expr_to_rval(ins, &node.rhs)?;
        let result = self.next_id();
        ins.push(Ins::Binary(BinaryIns {
            ty,
            op: node.op.clone().into(),
            lhs,
            rhs,
            result,
        }));
        Ok(RValue::Const(result))
    }

    fn lit_to_rval(&mut self, node: &types::LiteralNode) -> Res<RValue> {
        Ok(match &node.kind {
            LiteralKind::Int(n) => RValue::Int(*n),
            LiteralKind::String(s) => RValue::Data(self.data.get_or_intern_string(s.to_owned())),
            LiteralKind::Ident(name) => self.get_variable_rval(name),
            LiteralKind::Uint(n) => RValue::Uint(*n),
            LiteralKind::Float(n) => RValue::Float(*n),
            LiteralKind::Bool(n) => RValue::Uint(if *n { 1 } else { 0 }),
            LiteralKind::Char(n) => RValue::Uint(*n as u64),
        })
    }

    fn namespace_to_rval(&mut self, node: &types::NamespaceMemberNode) -> Res<RValue> {
        // Both unwraps are guaranteed by type checker
        let symbol_id = self.nsl.get(&node.name).unwrap().get(&node.field).unwrap();
        let symbol = self.ctx.symbols.get(symbol_id);
        let mangled_name = mangle_symbol_name(self.ctx, symbol);
        self.externs.insert(symbol_id);

        Ok(RValue::Function(mangled_name))
    }

    fn call_to_rval(&mut self, ins: &mut Vec<Ins>, node: &types::CallNode) -> Res<RValue> {
        let args = node
            .args
            .iter()
            .map(|expr| {
                let ty = self.types.to_ir(self.ctx, expr.type_id());
                let rval = self.expr_to_rval(ins, expr)?;
                Ok((ty, rval))
            })
            .collect::<Result<Vec<_>, Report>>()?;

        let callee = match node.callee.try_identifier() {
            Some(func_name) => {
                let mangled_name = self.to_mangled_name(func_name);
                RValue::Function(mangled_name)
            }
            None => self.expr_to_rval(ins, &node.callee)?,
        };

        let result_id = self.next_id();

        ins.push(Ins::Call(CallIns {
            ty: self.types.to_ir(self.ctx, node.ty),
            result: LValue::Const(result_id),
            callee,
            args,
        }));

        Ok(RValue::Const(result_id))
    }

    // Conversions to L-Value
    // ----------------------

    fn expr_to_lval(&mut self, _ins: &mut [Ins], expr: &types::Expr) -> Res<LValue> {
        if let Some(name) = expr.try_identifier() {
            return Ok(self.get_variable_lval(name));
        };
        todo!()
    }

    // Helper methods
    // --------------

    fn push_scope(&mut self) {
        self.vars.push_scope();
        self.params.push_scope();
        self.const_id = 0;
        self.stacksize = 0;
    }

    fn pop_scope(&mut self) {
        self.vars.pop_scope();
        self.params.pop_scope();
    }

    /// Get the RValue of a named value (variable or parameter).
    fn get_variable_rval(&self, name: &str) -> RValue {
        if let Some(id) = self.vars.get(name) {
            RValue::Const(*id)
        } else {
            RValue::Param(*self.params.get(name).expect("not an assigned name"))
        }
    }

    /// Get the LValue of a named value (variable or parameter).
    fn get_variable_lval(&self, name: &str) -> LValue {
        if let Some(id) = self.vars.get(name) {
            LValue::Const(*id)
        } else {
            LValue::Param(*self.params.get(name).expect("not an assigned name"))
        }
    }

    /// Get the mangled name of the given local symbol.
    fn to_mangled_name(&self, name: &str) -> String {
        let id = self.symbols.get(name).unwrap().id;
        let symbol = self.ctx.symbols.get(id);
        mangle_symbol_name(self.ctx, symbol)
    }
}

impl From<BinaryOp> for IRBinaryOp {
    fn from(op: BinaryOp) -> Self {
        match op {
            BinaryOp::Plus => IRBinaryOp::Add,
            BinaryOp::Minus => IRBinaryOp::Sub,
            BinaryOp::Mult => IRBinaryOp::Mul,
            BinaryOp::Divide => IRBinaryOp::Div,
            BinaryOp::Modulo => IRBinaryOp::Mod,
            BinaryOp::Equal => IRBinaryOp::Eq,
            BinaryOp::NotEqual => IRBinaryOp::Ne,
            BinaryOp::Greater => IRBinaryOp::Gt,
            BinaryOp::GreaterEq => IRBinaryOp::Ge,
            BinaryOp::Less => IRBinaryOp::Lt,
            BinaryOp::LessEq => IRBinaryOp::Le,
            BinaryOp::LogicAnd => IRBinaryOp::And,
            BinaryOp::LogicOr => IRBinaryOp::Or,
        }
    }
}

impl From<UnaryOp> for IRUnaryOp {
    fn from(op: UnaryOp) -> Self {
        match op {
            UnaryOp::Minus => IRUnaryOp::Neg,
            UnaryOp::LogicNot => IRUnaryOp::Not,
        }
    }
}

/// Get the mangled version of a symbol name.
fn mangle_symbol_name(ctx: &Context, symbol: &Symbol) -> String {
    if ctx.config.no_mangle_names || symbol.no_mangle || symbol.is_extern() || symbol.name == "main"
    {
        return symbol.name.clone();
    }

    let modpath = match &symbol.origin {
        SymbolOrigin::Module { modpath, .. } => modpath,
        SymbolOrigin::Library(modpath) => modpath,
        _ => unreachable!(),
    };

    format!("_{}_{}", modpath.to_underscore(), symbol.name)
}

// static STRING_ID: AtomicUsize = AtomicUsize::new(0);

// fn next_id() -> usize {
//     STRING_ID.fetch_add(1, Ordering::Relaxed)
// }

// struct ModuleFile<'a> {
//     modpath: &'a ModulePath,
//     nsl: &'a NamespaceList,
//     nodes: &'a [Decl],
//     syms: &'a SymbolList,
// }

// struct Emitter<'a> {
//     ctx: &'a Context,
//     file: ModuleFile<'a>,

//     sym: SymTracker,
//     ins: Vec<Vec<Ins>>,

//     // Track if void functions have returned or not to add explicit return
//     has_returned: bool,
//     stack_size: usize, // Cumulative stack size from declarations
// }

// impl<'a> Emitter<'a> {
//     fn new(ctx: &'a Context, file: ModuleFile<'a>) -> Self {
//         Self {
//             ctx,
//             file,
//             sym: SymTracker::new(),
//             has_returned: false,
//             ins: vec![Vec::new()],
//             stack_size: 0,
//         }
//     }

//     fn emit(mut self) -> Res<Vec<Ins>> {
//         info!("Emitting IR for module: {}", self.file.modpath.path());
//         let mut diag = Diagnostics::new();

//         for decl in self.file.nodes {
//             match decl.accept(&mut self) {
//                 Ok(_) => {}
//                 Err(err) => diag.add(err),
//             }
//         }

//         if diag.num_errors() == 0 {
//             debug!("Success: {} instructions", self.ins.len());
//             Ok(mem::take(&mut self.ins[0]))
//         } else {
//             info!("Fail: finished with {} errors", diag.num_errors());
//             Err(diag)
//         }
//     }

//     fn node_to_ir_type(&self, node: &dyn TypedNode) -> IRType {
//         self.to_ir_type(node.type_id())
//     }

//     /// Convert semantic type to IR type, lowering to primitive or union type.
//     fn to_ir_type(&self, id: TypeId) -> IRType {
//         let id = self.ctx.types.deep_resolve(id);
//         let ty = self.ctx.types.lookup(id);

//         match &ty.kind {
//             TypeKind::Primitive(p) => IRType::Primitive(p.clone().into()),
//             TypeKind::Function(f) => IRType::Function(
//                 f.params.iter().map(|p| self.to_ir_type(*p)).collect(),
//                 Box::new(self.to_ir_type(f.ret)),
//             ),
//             _ => panic!("unhandled kind {:?}", ty.kind),
//         }
//     }

//     fn push_scope(&mut self) {
//         self.stack_size = 0;
//         self.ins.push(Vec::new());
//     }

//     fn pop_scope(&mut self) -> (Vec<Ins>, usize) {
//         (
//             self.ins.pop().expect("scope list is empty"),
//             self.stack_size,
//         )
//     }

//     fn push(&mut self, ins: Ins) {
//         self.ins.last_mut().expect("scope list is empty").push(ins);
//     }

//     fn next_string_name(&mut self) -> String {
//         format!("S{}", next_id())
//     }

//     fn mangle_symbol_name(&self, sym: &Symbol) -> String {
//         if self.ctx.config.no_mangle_names || sym.no_mangle || sym.is_extern() || sym.name == "main"
//         {
//             sym.name.clone()
//         } else {
//             let modpath = match &sym.origin {
//                 SymbolOrigin::Module { modpath, .. } => modpath,
//                 SymbolOrigin::Library(modpath) => modpath,
//                 _ => unreachable!(),
//             };
//             format!("_{}_{}", modpath.to_underscore(), sym.name)
//         }
//     }

//     fn get_symbol(&self, name: &str) -> &Symbol {
//         self.ctx
//             .symbols
//             .get(self.file.syms.get(name).expect("not a symbol").id)
//     }

//     fn get_namespace_symbol(&self, namespace: &str, name: &str) -> &Symbol {
//         self.ctx.symbols.get(
//             self.file
//                 .nsl
//                 .get(namespace)
//                 .expect("not a namespace")
//                 .get(name)
//                 .expect("not a symbol"),
//         )
//     }
// }

// impl<'a> Visitor<Result<RValue, Report>> for Emitter<'a> {
//     fn visit_func(&mut self, node: &types::FuncNode) -> Result<RValue, Report> {
//         let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
//             panic!("expected func to be function type, was {:?}", &node.ty);
//         };

//         self.sym.new_function_context();

//         // Declare param indecies
//         for p in &node.params {
//             self.sym.set_param(p.clone());
//         }

//         // Generate function body IR
//         self.has_returned = false;
//         self.push_scope();

//         for stmt in &node.body {
//             stmt.accept(self)?;
//         }

//         let (mut body, mut stacksize) = self.pop_scope();

//         // Add param sizes to total stack size
//         for p in &params {
//             stacksize += p.size();
//         }

//         // Add explicit void return for non-returing functions
//         if !self.has_returned {
//             body.push(Ins::Return(
//                 IRType::Primitive(ir::Primitive::Void),
//                 RValue::Void,
//             ));
//         }

//         let sym = self.get_symbol(&node.name);

//         self.push(Ins::Func(FuncDecl {
//             name: self.mangle_symbol_name(sym),
//             public: node.public,
//             params,
//             ret: *ret,
//             body,
//             stacksize,
//         }));

//         Ok(RValue::Void)
//     }

//     fn visit_return(&mut self, node: &types::ReturnNode) -> Result<RValue, Report> {
//         let ty = self.node_to_ir_type(node);
//         let val = node
//             .expr
//             .as_ref()
//             .map_or(Ok(RValue::Void), |expr| expr.accept(self))?;

//         self.has_returned = true;
//         self.push(Ins::Return(ty, val));

//         Ok(RValue::Void)
//     }

//     fn visit_var_assign(&mut self, node: &types::VarAssignNode) -> Result<RValue, Report> {
//         let lval = match node.lval.accept(self)? {
//             RValue::Const(id) => LValue::Const(id),
//             RValue::Param(id) => LValue::Param(id),
//             _ => panic!("illegal lvalue"),
//         };

//         let value = node.rval.accept(self)?;
//         let ty = self.node_to_ir_type(&node.rval);

//         self.push(Ins::Assign(AssignIns { lval, ty, value }));
//         Ok(RValue::Void)
//     }

//     fn visit_var_decl(&mut self, node: &types::VarDeclNode) -> Result<RValue, Report> {
//         let value = node.value.accept(self)?;
//         let ty = self.node_to_ir_type(&node.value);
//         let id = self.sym.set(node.name.to_string());
//         self.stack_size += ty.size();
//         self.push(Ins::Store(StoreIns { id, ty, value }));
//         Ok(RValue::Void)
//     }

//     fn visit_literal(&mut self, node: &types::LiteralNode) -> Result<RValue, Report> {
//         Ok(match &node.kind {
//             LiteralKind::Ident(name) => self.sym.get(name),
//             LiteralKind::String(s) => {
//                 let name = self.next_string_name();
//                 self.push(Ins::StringData(StringDecl {
//                     name: name.to_owned(),
//                     length: s.len(),
//                     value: s.clone(),
//                 }));
//                 RValue::Data(name.to_owned())
//             }
//             LiteralKind::Int(n) => RValue::Int(*n),
//             LiteralKind::Uint(n) => RValue::Int(*n as i64),
//             LiteralKind::Float(f) => RValue::Float(*f),
//             LiteralKind::Bool(b) => RValue::Int(if *b { 1 } else { 0 }),
//             LiteralKind::Char(c) => RValue::Int((*c).into()),
//         })
//     }

//     fn visit_extern(&mut self, node: &types::ExternNode) -> Result<RValue, Report> {
//         let IRType::Function(params, ret) = self.node_to_ir_type(node) else {
//             panic!("expected func to be function type, was {:?}", &node.ty);
//         };

//         self.push(Ins::Extern(ExternDecl {
//             name: node.name.clone(),
//             ret: *ret,
//             params,
//         }));
//         Ok(RValue::Void)
//     }

//     fn visit_call(&mut self, node: &types::CallNode) -> Result<RValue, Report> {
//         let callee = match &*node.callee {
//             Expr::Literal(t) => match &t.kind {
//                 LiteralKind::Ident(name) => {
//                     // Get the function symbol and generate the correct link name
//                     let sym = self.get_symbol(name);
//                     RValue::Function(self.mangle_symbol_name(sym))
//                 }
//                 _ => panic!("unchecked invalid function call"),
//             },
//             e => e.accept(self)?,
//         };

//         let args = node
//             .args
//             .iter()
//             .map(|arg| {
//                 let value = arg.accept(self)?;
//                 let ty = self.node_to_ir_type(arg);
//                 Ok((ty, value))
//             })
//             .collect::<Result<Vec<_>, _>>()?;

//         let ty = self.node_to_ir_type(node);
//         let result = self.sym.next(); // declare after evaluating args to avoid incorrect id order

//         self.push(Ins::Call(ir::CallIns {
//             callee,
//             ty,
//             args,
//             result,
//         }));

//         Ok(RValue::Const(result))
//     }

//     fn visit_member(&mut self, _node: &types::MemberNode) -> Result<RValue, Report> {
//         todo!()
//     }

//     fn visit_namespace_member(
//         &mut self,
//         node: &types::NamespaceMemberNode,
//     ) -> Result<RValue, Report> {
//         let sym = self.get_namespace_symbol(&node.name, &node.field);
//         let linkname = self.mangle_symbol_name(sym);

//         // Declare as extern function
//         let f = self.node_to_ir_type(node);
//         let IRType::Function(params, ret) = f else {
//             panic!("not a function");
//         };

//         self.push(Ins::Extern(ExternDecl {
//             name: linkname.clone(),
//             params,
//             ret: *ret,
//         }));

//         Ok(RValue::Function(linkname))
//     }
// }
