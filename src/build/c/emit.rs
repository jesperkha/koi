use std::{collections::HashMap, mem};

use crate::{
    build::c::ast::{Ast, BinaryOp, Decl, Expr, Stmt, Type, UnaryOp},
    config::{Config, PathManager},
    ir::{self, ConstId, IRTypeId, IRTypeInterner, ParamId, Unit},
};

pub fn emit(unit: Unit, _config: &Config, pm: &PathManager) -> Ast {
    let mut decls = Vec::new();

    decls.push(Decl::Include(pm.include_path().join("koi.h").to_string()));

    for decl in unit.decls {
        let decl = match decl {
            crate::ir::Decl::Extern(ext) => Decl::ExternFunc {
                name: ext.name,
                params: ext
                    .params
                    .iter()
                    .map(|&ty| ctype(&unit.types, ty))
                    .collect(),
                ret: ctype(&unit.types, ext.ret),
            },
            crate::ir::Decl::Func(func) => FuncEmitter::new(func, &unit.types, &unit.data).emit(),
        };

        decls.push(decl);
    }

    Ast { decls }
}

fn ctype(types: &IRTypeInterner, typeid: IRTypeId) -> Type {
    match types.get(typeid) {
        ir::IRType::Primitive(primitive) => primitive.into(),
        ir::IRType::Function(_, _) => todo!(),
    }
}

struct FuncEmitter<'a> {
    decl: ir::FuncDecl,
    types: &'a IRTypeInterner,
    data: &'a [ir::Data],
    param_count: usize,
    stmts: Vec<Stmt>,
    /// Remaps a branch-local ConstId to the canonical ConstId from the first
    /// branch when mutually exclusive blocks share a logical variable slot.
    var_remap: HashMap<ConstId, ConstId>,
    /// ConstIds already declared in an enclosing scope; Store emits VarAssign
    /// instead of VarDecl for these.
    predeclared: HashMap<ConstId, IRTypeId>,
}

impl<'a> FuncEmitter<'a> {
    fn new(decl: ir::FuncDecl, types: &'a IRTypeInterner, data: &'a [ir::Data]) -> Self {
        Self {
            param_count: decl.params.len(),
            stmts: Vec::new(),
            decl,
            types,
            data,
            var_remap: HashMap::new(),
            predeclared: HashMap::new(),
        }
    }

    fn emit(mut self) -> Decl {
        mem::take(&mut self.decl.body.ins)
            .iter()
            .for_each(|ins| self.emit_ins(ins));

        let params = mem::take(&mut self.decl.params)
            .iter()
            .enumerate()
            .map(|(i, ty)| (i, self.to_ctype(*ty)))
            .collect::<Vec<_>>();

        let ret = self.to_ctype(self.decl.ret);

        Decl::Function {
            name: self.decl.name,
            body: self.stmts,
            params,
            ret,
        }
    }

    fn to_ctype(&self, typeid: usize) -> Type {
        ctype(self.types, typeid)
    }

    fn emit_ins(&mut self, ins: &ir::Ins) {
        match ins {
            ir::Ins::Return(_, rvalue) => {
                let s = Stmt::Return(if matches!(rvalue, ir::RValue::Void) {
                    None
                } else {
                    Some(self.rval_to_expr(rvalue))
                });
                self.push(s);
            }
            ir::Ins::Store(ins) => {
                let actual_id = self
                    .var_remap
                    .get(&ins.const_id)
                    .copied()
                    .unwrap_or(ins.const_id);
                let value = Box::new(self.rval_to_expr(&ins.rval));
                let s = if self.predeclared.contains_key(&actual_id) {
                    Stmt::VarAssign {
                        lhs: self.var_id(actual_id),
                        rhs: value,
                    }
                } else {
                    Stmt::VarDecl {
                        ty: self.to_ctype(ins.ty),
                        id: self.var_id(actual_id),
                        value,
                    }
                };
                self.push(s);
            }
            ir::Ins::Assign(ins) => {
                let s = Stmt::VarAssign {
                    lhs: self.lval_to_id(&ins.lval),
                    rhs: Box::new(self.rval_to_expr(&ins.rval)),
                };
                self.push(s);
            }
            ir::Ins::Call(ins) => match &ins.callee {
                ir::RValue::Function(name) => {
                    let s = Stmt::Call {
                        ty: self.to_ctype(ins.ty),
                        callee: name.clone(),
                        dest: self.lval_to_id(&ins.result),
                        args: ins
                            .args
                            .iter()
                            .map(|(_, rval)| self.rval_to_expr(rval))
                            .collect(),
                    };
                    self.push(s);
                }
                _ => todo!("non-function callee not implemented"),
            },
            ir::Ins::Binary(ins) => {
                let s = Stmt::Binary {
                    result: self.var_id(ins.result),
                    ty: self.to_ctype(ins.ty),
                    op: (&ins.op).into(),
                    left: Box::new(self.rval_to_expr(&ins.lhs)),
                    right: Box::new(self.rval_to_expr(&ins.rhs)),
                };
                self.push(s);
            }
            ir::Ins::Unary(ins) => {
                let s = Stmt::Unary {
                    ty: self.to_ctype(ins.ty),
                    result: self.var_id(ins.result),
                    op: (&ins.op).into(),
                    expr: Box::new(self.rval_to_expr(&ins.rhs)),
                };
                self.push(s);
            }
            ir::Ins::Cast(ins) => {
                let rval = self.rval_to_expr(&ins.rval);
                let s = Stmt::VarDecl {
                    ty: self.to_ctype(ins.to_ty),
                    id: self.var_id(ins.result),
                    value: Box::new(Expr::Cast(self.to_ctype(ins.to_ty), Box::new(rval))),
                };
                self.push(s);
            }
            ir::Ins::Break => self.push(Stmt::Break),
            ir::Ins::Continue => self.push(Stmt::Continue),
            ir::Ins::If(if_ins) => self.emit_if(if_ins),
            ir::Ins::While(while_ins) => self.emit_while(while_ins),
            ir::Ins::Conditional(cond_ins) => self.emit_conditional(cond_ins),
            ir::Ins::Intrinsic(_) => todo!(),
        };
    }

    fn emit_if(&mut self, ins: &ir::IfIns) {
        // Collect Store ConstIds from the first branch. In the x86 backend,
        // all branches reset the stack offset to the same base, so they share
        // the same stack slots. In C we replicate this by pre-declaring those
        // variables in the enclosing scope and remapping subsequent branches'
        // Store ConstIds (and Const reads) to the first branch's ConstIds.
        let first_stores = block_stores(&ins.block);

        for &(cid, ty) in &first_stores {
            self.push(Stmt::VarDecl {
                ty: self.to_ctype(ty),
                id: self.var_id(cid),
                value: Box::new(Expr::IntLit(0)),
            });
            self.predeclared.insert(cid, ty);
        }

        let add_remap = |remap: &mut HashMap<ConstId, ConstId>, block: &ir::Block| {
            for (i, &(new_cid, _)) in block_stores(block).iter().enumerate() {
                if let Some(&(first_cid, _)) = first_stores.get(i) {
                    remap.insert(new_cid, first_cid);
                }
            }
        };

        for elseif in &ins.elseif {
            add_remap(&mut self.var_remap, &elseif.block);
        }
        if let Some(ref elseblock) = ins.elseblock {
            add_remap(&mut self.var_remap, elseblock);
        }

        let cond = self.rval_to_expr(&ins.cond);
        let body = self.collect_block(&ins.block);

        // Build the else chain back-to-front: innermost else block first,
        // then each else-if wraps it.
        let mut else_: Option<Vec<Stmt>> = ins.elseblock.as_ref().map(|b| self.collect_block(b));

        for elseif in ins.elseif.iter().rev() {
            let mut stmts = self.collect_ins_vec(&elseif.cond_ins);
            let elif_cond = self.rval_to_expr(&elseif.cond);
            let elif_body = self.collect_block(&elseif.block);
            stmts.push(Stmt::If {
                cond: elif_cond,
                body: elif_body,
                else_,
            });
            else_ = Some(stmts);
        }

        // Clean up remap and predeclared entries added for this if.
        for elseif in &ins.elseif {
            for (new_cid, _) in block_stores(&elseif.block) {
                self.var_remap.remove(&new_cid);
            }
        }
        if let Some(ref elseblock) = ins.elseblock {
            for (new_cid, _) in block_stores(elseblock) {
                self.var_remap.remove(&new_cid);
            }
        }
        for &(cid, _) in &first_stores {
            self.predeclared.remove(&cid);
        }

        self.push(Stmt::If { cond, body, else_ });
    }

    fn emit_while(&mut self, ins: &ir::WhileIns) {
        // Emit as `while (1)` so that:
        //  - `cond_ins` re-run on every iteration (correct for complex conditions)
        //  - `continue` skips back to the top and re-evaluates the condition
        let mut body = self.collect_ins_vec(&ins.cond_ins);

        let cond = self.rval_to_expr(&ins.cond);
        body.push(Stmt::If {
            cond: Expr::Not(Box::new(cond)),
            body: vec![Stmt::Break],
            else_: None,
        });

        body.extend(self.collect_block(&ins.block));

        if let Some(post) = &ins.post {
            body.extend(self.collect_ins_vec(post));
        }

        self.push(Stmt::While { body });
    }

    fn emit_conditional(&mut self, ins: &ir::CondIns) {
        // Emit lhs computation into the current scope; the result is captured
        // in ins.lhs (an RValue::Const pointing to the last instruction's result).
        for i in &ins.lhs_ins {
            self.emit_ins(i);
        }

        let lhs_expr = self.rval_to_expr(&ins.lhs);
        let result_id = self.var_id(ins.result);

        // AND: default false, enter if lhs is true  → evaluate rhs
        // OR:  default true,  enter if lhs is false → evaluate rhs
        let (initial, if_cond) = match ins.op {
            ir::IRCondOp::And => (Expr::IntLit(0), lhs_expr),
            ir::IRCondOp::Or => (Expr::IntLit(1), Expr::Not(Box::new(lhs_expr))),
        };

        self.push(Stmt::VarDecl {
            ty: Type::Int8,
            id: result_id,
            value: Box::new(initial),
        });

        let mut rhs_body = self.collect_ins_vec(&ins.rhs_ins);
        let rhs_expr = self.rval_to_expr(&ins.rhs);
        rhs_body.push(Stmt::VarAssign {
            lhs: result_id,
            rhs: Box::new(rhs_expr),
        });

        self.push(Stmt::If {
            cond: if_cond,
            body: rhs_body,
            else_: None,
        });
    }

    /// Temporarily swap out `self.stmts`, run `ins` through `emit_ins`, then
    /// return the collected statements and restore the original list.
    fn collect_ins_vec(&mut self, ins: &[ir::Ins]) -> Vec<Stmt> {
        let saved = std::mem::take(&mut self.stmts);
        for i in ins {
            self.emit_ins(i);
        }
        std::mem::replace(&mut self.stmts, saved)
    }

    fn collect_block(&mut self, block: &ir::Block) -> Vec<Stmt> {
        self.collect_ins_vec(&block.ins)
    }

    fn rval_to_expr(&mut self, rval: &ir::RValue) -> Expr {
        match rval {
            ir::RValue::Const(i) => {
                let actual = self.var_remap.get(i).copied().unwrap_or(*i);
                Expr::VarLit(self.var_id(actual))
            }
            ir::RValue::Param(i) => Expr::VarLit(self.param_id(*i)),
            ir::RValue::Int(n) => Expr::IntLit(*n),
            ir::RValue::Float(n) => Expr::FloatLit(*n),
            ir::RValue::Uint(n) => Expr::UintLit(*n),
            ir::RValue::Data(idx) => match &self.data[*idx] {
                ir::Data::String(s) => Expr::StrLit(s.clone()),
            },
            ir::RValue::Function(_) => todo!(),
            ir::RValue::Void => panic!("void value should always be checked"),
        }
    }

    fn lval_to_id(&mut self, lval: &ir::LValue) -> usize {
        match lval {
            ir::LValue::Const(i) => self.var_id(*i),
            ir::LValue::Param(i) => self.param_id(*i),
        }
    }

    fn param_id(&self, id: ParamId) -> usize {
        id
    }

    fn var_id(&self, id: ConstId) -> usize {
        id + self.param_count
    }

    fn push(&mut self, stmt: Stmt) {
        self.stmts.push(stmt);
    }
}

/// Collect the (ConstId, type) pairs for Store instructions at the TOP level
/// of a block (not recursing into nested if/while). Used by emit_if to set up
/// cross-branch variable sharing.
fn block_stores(block: &ir::Block) -> Vec<(ConstId, IRTypeId)> {
    block
        .ins
        .iter()
        .filter_map(|ins| {
            if let ir::Ins::Store(s) = ins {
                Some((s.const_id, s.ty))
            } else {
                None
            }
        })
        .collect()
}

impl From<&ir::Primitive> for Type {
    fn from(value: &ir::Primitive) -> Self {
        match value {
            ir::Primitive::Void => Self::Void,
            ir::Primitive::F32 => Self::Float,
            ir::Primitive::F64 => Self::Float,
            ir::Primitive::U8 => Self::Uint8,
            ir::Primitive::U16 => Self::Uint16,
            ir::Primitive::U32 => Self::Uint32,
            ir::Primitive::U64 => Self::Uint64,
            ir::Primitive::I8 => Self::Int8,
            ir::Primitive::I16 => Self::Int16,
            ir::Primitive::I32 => Self::Int32,
            ir::Primitive::I64 => Self::Int64,
            ir::Primitive::String => Self::Pointer(Box::new(Self::Uint8)),
        }
    }
}

impl From<&ir::IRBinaryOp> for BinaryOp {
    fn from(value: &ir::IRBinaryOp) -> Self {
        match value {
            ir::IRBinaryOp::Add => Self::Plus,
            ir::IRBinaryOp::Sub => Self::Minus,
            ir::IRBinaryOp::Mul => Self::Mult,
            ir::IRBinaryOp::Div => Self::Div,
            ir::IRBinaryOp::Eq => Self::Equal,
            ir::IRBinaryOp::Ne => Self::NotEqual,
            ir::IRBinaryOp::Gt => Self::Greater,
            ir::IRBinaryOp::Ge => Self::GreaterEqual,
            ir::IRBinaryOp::Lt => Self::Less,
            ir::IRBinaryOp::Le => Self::LessEqual,
            ir::IRBinaryOp::Mod => Self::Modulo,
        }
    }
}

impl From<&ir::IRUnaryOp> for UnaryOp {
    fn from(value: &ir::IRUnaryOp) -> Self {
        match value {
            ir::IRUnaryOp::Neg => Self::Minus,
            ir::IRUnaryOp::Not => Self::Not,
        }
    }
}
