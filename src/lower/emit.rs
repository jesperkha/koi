use std::collections::{HashMap, HashSet};

use crate::{
    context::Context,
    error::{self, Diagnostics, Report},
    ir::{
        AssignIns, BinaryIns, Block, CallIns, CondIns, ConstDecl, ConstId, Data, DataIndex, Decl,
        ElseIf, ExternDecl, FuncDecl, IRBinaryOp, IRCondOp, IRType, IRTypeInterner, IRUnaryOp,
        IfIns, Ins, LValue, ParamId, Primitive, RValue, StoreIns, UnaryIns, Unit, WhileIns,
    },
    module::{
        ConstValue, Module, ModuleId, ModuleKind, ModuleSourceFile, NamespaceList, Symbol,
        SymbolId, SymbolKind, SymbolList, SymbolOrigin,
    },
    types::{self, Expr, LiteralKind, TypedAst, TypedNode},
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
    data: DataInterner,
}

impl<'a> ModuleEmitter<'a> {
    fn new(ctx: &'a Context, module: &'a Module) -> Self {
        Self {
            ctx,
            module,
            types: IRTypeInterner::new(),
            data: DataInterner::new(),
        }
    }

    /// Emit IR for all module files and create bundled IR unit.
    fn emit(mut self) -> error::Res<Unit> {
        let ModuleKind::Source { files, .. } = &self.module.kind else {
            unreachable!();
        };

        let mut externs = HashSet::new();
        let mut decls = Vec::new();

        // Emit IR for each file in the module
        for file in files {
            let emitter = FileEmitter::new(
                self.ctx,
                &self.module.symbols,
                &mut self.types,
                file,
                &mut self.data,
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

        extern_decls.extend(decls);

        Ok(Unit {
            name: self.module.modpath.to_underscore(),
            data: self.data.into_data(),
            types: self.types,
            decls: extern_decls,
        })
    }

    fn emit_extern(&mut self, id: SymbolId) -> Res<Decl> {
        // Clone everything we need from ctx before any mutable operations
        let (mangled_name, symbol_ty, symbol_kind, func_info) = {
            let symbol = self.ctx.symbols.get(id);
            let mangled = mangle_symbol_name(self.ctx, symbol);
            let ty = symbol.ty;
            let kind = symbol.kind.clone();
            let func = self
                .ctx
                .types
                .try_function(ty)
                .map(|f| (f.params.clone(), f.ret));
            (mangled, ty, kind, func)
        };

        if let Some((params, ret)) = func_info {
            Ok(Decl::Extern(ExternDecl {
                name: mangled_name,
                params: self.types.to_ir_type_list(self.ctx, &params),
                ret: self.types.to_ir(self.ctx, ret),
            }))
        } else if let SymbolKind::Const(const_value) = symbol_kind {
            // Emit a local const declaration so the assembler can inline the value.
            let ty = self.types.to_ir(self.ctx, symbol_ty);
            let value = match const_value {
                ConstValue::Int(n) => RValue::Int(n),
                ConstValue::Uint(n) => RValue::Uint(n),
                ConstValue::Float(n) => RValue::Float(n),
                ConstValue::String(s) => RValue::Data(self.data.get_or_intern_string(s)),
            };
            Ok(Decl::Const(ConstDecl {
                name: mangled_name,
                ty,
                value,
                public: false,
            }))
        } else {
            panic!(
                "cannot emit extern for non-function, non-const symbol (id={})",
                id
            )
        }
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
                types::Decl::Const(node) => self.emit_const(node),

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

    fn emit_const(&mut self, node: &types::ConstNode) -> Res<Decl> {
        let ty = self.types.to_ir(self.ctx, node.ty);
        let value = self.expr_to_rval(&mut Vec::new(), &node.value)?;
        let name = self.to_mangled_name(&node.name);
        Ok(Decl::Const(ConstDecl {
            name,
            ty,
            value,
            public: node.public,
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
            types::Stmt::While(node) => self.emit_while(ins, node)?,
            types::Stmt::Break(_) => ins.push(Ins::Break),
            types::Stmt::Continue(_) => ins.push(Ins::Continue),
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

    fn emit_while(&mut self, ins: &mut Vec<Ins>, node: &types::WhileNode) -> Res<()> {
        let mut cond_ins = Vec::new();
        let cond = self.expr_to_rval(&mut cond_ins, &node.expr)?;
        let block = self.emit_block(&node.block)?;
        ins.push(Ins::While(WhileIns {
            cond_ins,
            cond,
            block,
        }));
        Ok(())
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
        if matches!(
            node.op,
            types::BinaryOp::LogicAnd | types::BinaryOp::LogicOr
        ) {
            return self.conditional_to_rval(ins, node);
        }

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

    fn conditional_to_rval(&mut self, ins: &mut Vec<Ins>, node: &types::BinaryNode) -> Res<RValue> {
        let mut lhs_ins = Vec::new();
        let lhs = self.expr_to_rval(&mut lhs_ins, &node.lhs)?;

        let mut rhs_ins = Vec::new();
        let rhs = self.expr_to_rval(&mut rhs_ins, &node.rhs)?;

        let op = match &node.op {
            types::BinaryOp::LogicAnd => IRCondOp::And,
            types::BinaryOp::LogicOr => IRCondOp::Or,
            _ => unreachable!(),
        };

        let result = self.next_id();
        ins.push(Ins::Conditional(CondIns {
            op,
            lhs_ins,
            lhs,
            rhs_ins,
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
        let symbol_kind = symbol.kind.clone();
        self.externs.insert(symbol_id);

        match symbol_kind {
            SymbolKind::Const(_) => Ok(RValue::GlobalConst(mangled_name)),
            SymbolKind::Function { .. } => Ok(RValue::Function(mangled_name)),
        }
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

    /// Get the RValue of a named value (variable, parameter, or global constant).
    fn get_variable_rval(&self, name: &str) -> RValue {
        if let Some(id) = self.vars.get(name) {
            RValue::Const(*id)
        } else if let Some(id) = self.params.get(name) {
            RValue::Param(*id)
        } else {
            // Must be a global constant
            let mangled = self.to_mangled_name(name);
            RValue::GlobalConst(mangled)
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

impl From<types::BinaryOp> for IRBinaryOp {
    fn from(op: types::BinaryOp) -> Self {
        match op {
            types::BinaryOp::Plus => IRBinaryOp::Add,
            types::BinaryOp::Minus => IRBinaryOp::Sub,
            types::BinaryOp::Mult => IRBinaryOp::Mul,
            types::BinaryOp::Divide => IRBinaryOp::Div,
            types::BinaryOp::Modulo => IRBinaryOp::Mod,
            types::BinaryOp::Equal => IRBinaryOp::Eq,
            types::BinaryOp::NotEqual => IRBinaryOp::Ne,
            types::BinaryOp::Greater => IRBinaryOp::Gt,
            types::BinaryOp::GreaterEq => IRBinaryOp::Ge,
            types::BinaryOp::Less => IRBinaryOp::Lt,
            types::BinaryOp::LessEq => IRBinaryOp::Le,
            // AND and OR handled separately
            _ => unreachable!(),
        }
    }
}

impl From<types::UnaryOp> for IRUnaryOp {
    fn from(op: types::UnaryOp) -> Self {
        match op {
            types::UnaryOp::Minus => IRUnaryOp::Neg,
            types::UnaryOp::LogicNot => IRUnaryOp::Not,
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
