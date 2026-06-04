use std::mem;

use crate::{
    build::c::ast::{Ast, BinaryOp, Decl, Expr, Stmt, Type, UnaryOp},
    config::{Config, PathManager},
    ir::{self, ConstId, IRTypeInterner, ParamId, Unit},
};

pub fn emit(unit: Unit, config: &Config, pm: &PathManager) -> Ast {
    let mut decls = Vec::new();

    decls.push(Decl::Include(pm.include_path().join("koi.h").to_string()));

    for decl in unit.decls {
        let decl = match decl {
            crate::ir::Decl::Extern(extern_decl) => todo!(),
            crate::ir::Decl::Func(func) => FuncEmitter::new(func, &unit.types).emit(),
        };

        decls.push(decl);
    }

    Ast { decls }
}

struct FuncEmitter<'a> {
    decl: ir::FuncDecl,
    types: &'a IRTypeInterner,
    param_count: usize,
    stmts: Vec<Stmt>,
}

impl<'a> FuncEmitter<'a> {
    fn new(decl: ir::FuncDecl, types: &'a IRTypeInterner) -> Self {
        Self {
            param_count: decl.params.len(),
            stmts: Vec::new(),
            decl,
            types,
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
        match self.types.get(typeid) {
            ir::IRType::Primitive(primitive) => primitive.into(),
            ir::IRType::Function(irtypes, irtype) => todo!(),
        }
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
                let s = Stmt::VarDecl {
                    ty: self.to_ctype(ins.ty),
                    id: self.var_id(ins.const_id),
                    value: Box::new(self.rval_to_expr(&ins.rval)),
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
            ir::Ins::Break => self.push(Stmt::Break),
            ir::Ins::Continue => self.push(Stmt::Continue),
            ir::Ins::If(if_ins) => todo!(),
            ir::Ins::While(while_ins) => todo!(),
            ir::Ins::Conditional(cond_ins) => todo!(),
            ir::Ins::Intrinsic(intrinsic_ins) => todo!(),
        };
    }

    fn rval_to_expr(&mut self, rval: &ir::RValue) -> Expr {
        match rval {
            ir::RValue::Const(i) => Expr::VarLit(self.var_id(*i)),
            ir::RValue::Param(i) => Expr::VarLit(self.param_id(*i)),
            ir::RValue::Int(n) => Expr::IntLit(*n),
            ir::RValue::Float(n) => Expr::FloatLit(*n),
            ir::RValue::Uint(n) => Expr::UintLit(*n),
            ir::RValue::Data(_) => todo!(),
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
            ir::Primitive::String => todo!("implement sized String type in C"),
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
