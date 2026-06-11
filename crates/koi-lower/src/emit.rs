use std::collections::{HashMap, HashSet};

use koi_common::error::{self, Diagnostics, Report};
use koi_common::util::VarTable;
use koi_ir::{
    AssignIns, BinaryIns, Block, CallIns, CastIns, CondIns, ConstId, Data, DataIndex, Decl,
    ElseIf, ExternDecl, FuncDecl, IRBinaryOp, IRCondOp, IRType, IRTypeId, IRTypeInterner,
    IRUnaryOp, IfIns, Ins, LValue, ParamId, Primitive, RValue, StoreIns, UnaryIns, Unit, WhileIns,
};
use koi_sema::{
    AssignOp, BinaryOp, CastKind, Context, Expr, LiteralKind, Module, ModuleId, ModuleKind,
    ModuleSourceFile, NamespaceList, PrimitiveType, Symbol, SymbolId, SymbolKind, SymbolList,
    SymbolOrigin, TypeId, TypeKind, TypedAst, TypedNode, UnaryOp,
};

// ----------------------- Free conversion functions ----------------------- //

/// Convert a semantic TypeId to an IR type id.
pub fn to_ir(ctx: &Context, ty: TypeId, types: &mut IRTypeInterner) -> IRTypeId {
    let kind = &ctx.types.lookup(ty).kind;
    let ir_type = match kind {
        TypeKind::Primitive(p) => IRType::Primitive(primitive_to_ir(p)),
        TypeKind::Function(f) => {
            let params = f
                .params
                .iter()
                .map(|&p| {
                    let id = to_ir(ctx, p, types);
                    ctx.types // get the IRType by dereferencing
                        .lookup(p); // unused - just for closure capture
                    // We need the actual IRType value for the Function variant
                    // Re-resolve to get IRType
                    types.get(id).clone()
                })
                .collect();
            let ret_id = to_ir(ctx, f.ret, types);
            let ret = types.get(ret_id).clone();
            IRType::Function(params, Box::new(ret))
        }
        TypeKind::Alias(inner) | TypeKind::Unique(_, inner) => {
            return to_ir(ctx, *inner, types);
        }
        TypeKind::Array(_) | TypeKind::Pointer(_) => {
            // Treat as pointer (u64)
            IRType::Primitive(Primitive::U64)
        }
    };
    types.get_or_intern(ir_type)
}

/// Convert a slice of semantic TypeIds to a Vec of IR type ids.
pub fn to_ir_type_list(
    ctx: &Context,
    params: &[TypeId],
    types: &mut IRTypeInterner,
) -> Vec<IRTypeId> {
    params.iter().map(|&p| to_ir(ctx, p, types)).collect()
}

fn primitive_to_ir(p: &PrimitiveType) -> Primitive {
    match p {
        PrimitiveType::Void => Primitive::Void,
        PrimitiveType::I8 => Primitive::I8,
        PrimitiveType::I16 => Primitive::I16,
        PrimitiveType::I32 => Primitive::I32,
        PrimitiveType::I64 => Primitive::I64,
        PrimitiveType::U8 | PrimitiveType::Byte => Primitive::U8,
        PrimitiveType::U16 => Primitive::U16,
        PrimitiveType::U32 => Primitive::U32,
        PrimitiveType::U64 => Primitive::U64,
        PrimitiveType::F32 => Primitive::F32,
        PrimitiveType::F64 => Primitive::F64,
        PrimitiveType::Bool => Primitive::U8,
        PrimitiveType::String => Primitive::String,
    }
}

// ----------------------- Public API ----------------------- //

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

        // Remove type imports as they are purely semantic and should not be lowered.
        externs.extend(
            self.module
                .imports()
                .iter()
                .filter(|id| !matches!(self.ctx.symbols.get(**id).kind, SymbolKind::Type)),
        );

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
            params: to_ir_type_list(self.ctx, &func.params.clone(), &mut self.types),
            ret: to_ir(self.ctx, func.ret, &mut self.types),
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
                koi_sema::Decl::Func(node) => self.emit_func(node),

                // extern symbols are declared at top using modules symbols list
                koi_sema::Decl::Extern(_) => continue,
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

    fn emit_func(&mut self, node: &koi_sema::FuncNode) -> Res<Decl> {
        self.push_scope();

        // Bind parameters to local ids
        for (i, param) in node.params.iter().enumerate() {
            self.params.bind(param.clone(), i);
        }

        let body = self.emit_func_block(&node.body.stmts)?;
        self.pop_scope();

        // Get function type
        let func = self.ctx.types.try_function(node.ty).unwrap();
        let params = to_ir_type_list(self.ctx, &func.params.clone(), self.types);
        let ret = to_ir(self.ctx, func.ret, self.types);

        Ok(Decl::Func(FuncDecl {
            public: node.public,
            name: self.to_mangled_name(&node.name),
            stacksize: self.stacksize,
            body,
            params,
            ret,
        }))
    }

    fn emit_func_block(&mut self, nodes: &Vec<koi_sema::Stmt>) -> Res<Block> {
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

    fn emit_stmt(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::Stmt) -> Res<()> {
        match node {
            koi_sema::Stmt::Return(node) => self.emit_return(ins, node)?,
            koi_sema::Stmt::ExprStmt(node) => {
                let _ = self.expr_to_rval(ins, node)?;
            }
            koi_sema::Stmt::VarDecl(node) => self.emit_var_decl(ins, node)?,
            koi_sema::Stmt::VarAssign(node) => self.emit_var_assign(ins, node)?,
            koi_sema::Stmt::If(node) => self.emit_if(ins, node)?,
            koi_sema::Stmt::While(node) => self.emit_while(ins, node)?,
            koi_sema::Stmt::For(node) => self.emit_for(ins, node)?,
            koi_sema::Stmt::Break(_) => ins.push(Ins::Break),
            koi_sema::Stmt::Continue(_) => ins.push(Ins::Continue),
            koi_sema::Stmt::OpAssign(node) => self.emit_op_assign(ins, node)?,
        };
        Ok(())
    }

    fn emit_op_assign(
        &mut self,
        ins: &mut Vec<Ins>,
        node: &koi_sema::OpAssignNode,
    ) -> Res<()> {
        let ty = to_ir(self.ctx, node.ty, self.types);
        let lhs = self.expr_to_rval(ins, &node.lval)?;
        let rhs = self.expr_to_rval(ins, &node.rval)?;
        let result = self.next_id();
        ins.push(Ins::Binary(BinaryIns {
            ty,
            op: assign_op_to_ir(node.op.clone()),
            lhs,
            rhs,
            result,
        }));
        let lval = self.expr_to_lval(ins, &node.lval)?;
        ins.push(Ins::Assign(AssignIns {
            ty,
            lval,
            rval: RValue::Const(result),
        }));
        Ok(())
    }

    fn emit_block(&mut self, node: &koi_sema::BlockNode) -> Res<Block> {
        let mut ins = Vec::new();
        for stmt in &node.stmts {
            self.emit_stmt(&mut ins, stmt)?;
        }
        Ok(Block { ins })
    }

    fn emit_for(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::ForNode) -> Res<()> {
        self.emit_stmt(ins, &node.initializer)?;

        let mut cond_ins = Vec::new();
        let cond = self.expr_to_rval(&mut cond_ins, &node.condition)?;

        let block = self.emit_block(&node.block)?;

        let mut post = Vec::new();
        self.emit_stmt(&mut post, &node.increment)?;

        ins.push(Ins::While(WhileIns {
            cond_ins,
            cond,
            block,
            post: Some(post),
        }));
        Ok(())
    }

    fn emit_while(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::WhileNode) -> Res<()> {
        let mut cond_ins = Vec::new();
        let cond = self.expr_to_rval(&mut cond_ins, &node.expr)?;
        let block = self.emit_block(&node.block)?;
        ins.push(Ins::While(WhileIns {
            cond_ins,
            cond,
            block,
            post: None,
        }));
        Ok(())
    }

    fn emit_if(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::IfNode) -> Res<()> {
        let cond = self.expr_to_rval(ins, &node.expr)?;
        let block = self.emit_block(&node.block)?;

        let mut elseifs = Vec::new();
        let mut elseif = &*node.elseif;
        let mut elseblock: Option<Block> = None;

        loop {
            match elseif {
                koi_sema::ElseBlock::ElseIf(node) => {
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
                koi_sema::ElseBlock::Else(node) => {
                    let block = self.emit_block(node)?;
                    elseblock = Some(block);
                    break;
                }
                koi_sema::ElseBlock::None => break,
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

    fn emit_var_assign(
        &mut self,
        ins: &mut Vec<Ins>,
        node: &koi_sema::VarAssignNode,
    ) -> Res<()> {
        let ty = to_ir(self.ctx, node.ty, self.types);
        let rval = self.expr_to_rval(ins, &node.rval)?;
        let lval = self.expr_to_lval(ins, &node.lval)?;
        ins.push(Ins::Assign(AssignIns { ty, lval, rval }));
        Ok(())
    }

    fn emit_var_decl(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::VarDeclNode) -> Res<()> {
        let ty = to_ir(self.ctx, node.ty, self.types);
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

    fn emit_return(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::ReturnNode) -> Res<()> {
        let ty = to_ir(self.ctx, node.ty, self.types);
        let rval = match &node.expr {
            None => RValue::Void,
            Some(expr) => self.expr_to_rval(ins, expr)?,
        };

        ins.push(Ins::Return(ty, rval));
        Ok(())
    }

    fn expr_to_rval(&mut self, ins: &mut Vec<Ins>, expr: &koi_sema::Expr) -> Res<RValue> {
        match expr {
            Expr::Literal(node) => self.lit_to_rval(node),
            Expr::Call(node) => self.call_to_rval(ins, node),
            Expr::NamespaceMember(node) => self.namespace_to_rval(node),
            Expr::Member(_) => todo!(),
            Expr::Binary(node) => self.binary_to_rval(ins, node),
            Expr::Unary(node) => self.unary_to_rval(ins, node),
            Expr::Cast(node) => self.cast_to_rval(ins, node),
        }
    }

    fn cast_to_rval(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::CastNode) -> Res<RValue> {
        let rval = self.expr_to_rval(ins, &node.expr)?;

        if matches!(node.cast_kind, CastKind::Identity) {
            return Ok(rval);
        }

        let from_ty = to_ir(self.ctx, node.expr.type_id(), self.types);
        let to_ty = to_ir(self.ctx, node.ty, self.types);
        let result = self.next_id();
        ins.push(Ins::Cast(CastIns {
            from_ty,
            to_ty,
            rval,
            result,
        }));
        Ok(RValue::Const(result))
    }

    fn unary_to_rval(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::UnaryNode) -> Res<RValue> {
        let ty = to_ir(self.ctx, node.ty, self.types);
        let rhs = self.expr_to_rval(ins, &node.rhs)?;
        let result = self.next_id();
        ins.push(Ins::Unary(UnaryIns {
            ty,
            op: unary_op_to_ir(node.op.clone()),
            rhs,
            result,
        }));
        Ok(RValue::Const(result))
    }

    fn binary_to_rval(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::BinaryNode) -> Res<RValue> {
        if matches!(
            node.op,
            koi_sema::BinaryOp::LogicAnd | koi_sema::BinaryOp::LogicOr
        ) {
            return self.conditional_to_rval(ins, node);
        }

        let ty = to_ir(self.ctx, node.ty, self.types);
        let lhs = self.expr_to_rval(ins, &node.lhs)?;
        let rhs = self.expr_to_rval(ins, &node.rhs)?;
        let result = self.next_id();
        ins.push(Ins::Binary(BinaryIns {
            ty,
            op: binary_op_to_ir(node.op.clone()),
            lhs,
            rhs,
            result,
        }));
        Ok(RValue::Const(result))
    }

    fn conditional_to_rval(
        &mut self,
        ins: &mut Vec<Ins>,
        node: &koi_sema::BinaryNode,
    ) -> Res<RValue> {
        let mut lhs_ins = Vec::new();
        let lhs = self.expr_to_rval(&mut lhs_ins, &node.lhs)?;

        let mut rhs_ins = Vec::new();
        let rhs = self.expr_to_rval(&mut rhs_ins, &node.rhs)?;

        let op = match &node.op {
            koi_sema::BinaryOp::LogicAnd => IRCondOp::And,
            koi_sema::BinaryOp::LogicOr => IRCondOp::Or,
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

    fn lit_to_rval(&mut self, node: &koi_sema::LiteralNode) -> Res<RValue> {
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

    fn namespace_to_rval(&mut self, node: &koi_sema::NamespaceMemberNode) -> Res<RValue> {
        // Both unwraps are guaranteed by type checker
        let symbol_id = self.nsl.get(&node.name).unwrap().get(&node.field).unwrap();
        let symbol = self.ctx.symbols.get(symbol_id);
        let mangled_name = mangle_symbol_name(self.ctx, symbol);
        self.externs.insert(symbol_id);

        Ok(RValue::Function(mangled_name))
    }

    fn call_to_rval(&mut self, ins: &mut Vec<Ins>, node: &koi_sema::CallNode) -> Res<RValue> {
        let args = node
            .args
            .iter()
            .map(|expr| {
                let ty = to_ir(self.ctx, expr.type_id(), self.types);
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
            ty: to_ir(self.ctx, node.ty, self.types),
            result: LValue::Const(result_id),
            callee,
            args,
        }));

        Ok(RValue::Const(result_id))
    }

    // Conversions to L-Value
    // ----------------------

    fn expr_to_lval(&mut self, _ins: &mut [Ins], expr: &koi_sema::Expr) -> Res<LValue> {
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

fn binary_op_to_ir(op: koi_sema::BinaryOp) -> IRBinaryOp {
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
        // AND and OR handled separately
        _ => unreachable!(),
    }
}

fn assign_op_to_ir(op: koi_sema::AssignOp) -> IRBinaryOp {
    match op {
        AssignOp::Plus => IRBinaryOp::Add,
        AssignOp::Minus => IRBinaryOp::Sub,
        AssignOp::Mult => IRBinaryOp::Mul,
        AssignOp::Div => IRBinaryOp::Div,
    }
}

fn unary_op_to_ir(op: koi_sema::UnaryOp) -> IRUnaryOp {
    match op {
        UnaryOp::Minus => IRUnaryOp::Neg,
        UnaryOp::LogicNot => IRUnaryOp::Not,
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
