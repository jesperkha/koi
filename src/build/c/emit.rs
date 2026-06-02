use std::mem;

use crate::{
    build::c::ast::{Ast, Decl, Expr, Stmt, Type},
    config::{Config, PathManager},
    ir::{self, IRTypeInterner, Unit},
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

    /// Current ID count
    id_count: usize,
}

impl<'a> FuncEmitter<'a> {
    fn new(decl: ir::FuncDecl, types: &'a IRTypeInterner) -> Self {
        Self {
            decl,
            id_count: 0,
            types,
        }
    }

    fn emit(mut self) -> Decl {
        let body = mem::take(&mut self.decl.body.ins)
            .iter()
            .map(|ins| self.emit_ins(ins))
            .collect::<Vec<_>>();

        let params = mem::take(&mut self.decl.params)
            .iter()
            .map(|ty| (self.next_id(), self.to_ctype(*ty)))
            .collect::<Vec<_>>();

        let ret = self.to_ctype(self.decl.ret);

        Decl::Function {
            name: self.decl.name,
            params,
            ret,
            body,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_count;
        self.id_count += 1;
        id
    }

    fn to_ctype(&self, typeid: usize) -> Type {
        match self.types.get(typeid) {
            ir::IRType::Primitive(primitive) => primitive.into(),
            ir::IRType::Function(irtypes, irtype) => todo!(),
        }
    }

    fn emit_ins(&mut self, ins: &ir::Ins) -> Stmt {
        match ins {
            ir::Ins::Return(_, rvalue) => Stmt::Return(if matches!(rvalue, ir::RValue::Void) {
                None
            } else {
                Some(self.rval_to_expr(rvalue))
            }),
            ir::Ins::Store(store_ins) => todo!(),
            ir::Ins::Assign(assign_ins) => todo!(),
            ir::Ins::Call(call_ins) => todo!(),
            ir::Ins::Intrinsic(intrinsic_ins) => todo!(),
            ir::Ins::Binary(binary_ins) => todo!(),
            ir::Ins::Unary(unary_ins) => todo!(),
            ir::Ins::If(if_ins) => todo!(),
            ir::Ins::While(while_ins) => todo!(),
            ir::Ins::Conditional(cond_ins) => todo!(),
            ir::Ins::Break => todo!(),
            ir::Ins::Continue => todo!(),
        }
    }

    fn rval_to_expr(&mut self, rval: &ir::RValue) -> Expr {
        match rval {
            ir::RValue::Int(i) => Expr::IntLit(*i),
            ir::RValue::Float(_) => todo!(),
            ir::RValue::Uint(_) => todo!(),
            ir::RValue::Const(_) => todo!(),
            ir::RValue::Param(_) => todo!(),
            ir::RValue::Function(_) => todo!(),
            ir::RValue::Data(_) => todo!(),
            ir::RValue::Void => panic!("void value should always be checked"),
        }
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
