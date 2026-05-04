use crate::{
    build::c::c::{BlockNode, CType, Decl, Expr, ExternNode, FunctionNode, Stmt},
    config::Config,
    ir,
};

pub fn emit(unit: ir::Unit, config: &Config) -> Vec<Decl> {
    let mut nodes = Vec::new();
    for decl in unit.decls {
        nodes.push(match decl {
            ir::Decl::Extern(decl) => emit_extern(decl, &unit.types),
            ir::Decl::Func(decl) => emit_function(decl, &unit.types),
        });
    }

    nodes
}

fn emit_extern(decl: ir::ExternDecl, types: &ir::IRTypeInterner) -> Decl {
    Decl::Extern(ExternNode {
        name: decl.name,
        args: decl
            .params
            .iter()
            .map(|id| types.get(*id).into())
            .collect::<Vec<CType>>(),
        ret: types.get(decl.ret).into(),
    })
}

fn emit_function(decl: ir::FuncDecl, types: &ir::IRTypeInterner) -> Decl {
    Decl::Function(FunctionNode {
        name: decl.name,
        args: decl
            .params
            .iter()
            .map(|id| types.get(*id).into())
            .collect::<Vec<CType>>(),
        ret: types.get(decl.ret).into(),
        body: emit_block(decl.body, types),
    })
}

fn emit_block(block: ir::Block, types: &ir::IRTypeInterner) -> BlockNode {
    let mut stmts = Vec::new();
    for ins in block.ins {
        let stmt = match ins {
            ir::Ins::Return(_, rvalue) => Stmt::Return(Some(emit_rvalue(rvalue, types))),
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
        };

        stmts.push(stmt);
    }

    BlockNode { stmts }
}

fn emit_rvalue(rval: ir::RValue, types: &ir::IRTypeInterner) -> Expr {
    match rval {
        ir::RValue::Void => Expr::Void,
        ir::RValue::Int(n) => Expr::Int(n),

        ir::RValue::Float(_) => todo!(),
        ir::RValue::Uint(_) => todo!(),
        ir::RValue::Const(_) => todo!(),
        ir::RValue::Param(_) => todo!(),
        ir::RValue::Function(_) => todo!(),
        ir::RValue::Data(_) => todo!(),
    }
}
