use crate::{
    ast::{Ast, BlockNode, FuncNode, ReturnNode, TypeNode, Visitor},
    error::Error,
    ir::{FuncInst, Instruction, Type, Value},
    token::Token,
    types::{TypeContext, TypeId, TypeKind},
};

pub struct IR<'a> {
    ast: &'a Ast,
    ctx: &'a TypeContext,
}

impl<'a> IR<'a> {
    pub fn emit(ast: &'a Ast, ctx: &'a TypeContext) -> Result<Vec<Instruction>, Vec<Error>> {
        todo!()
    }

    fn visit_literal(&self) -> Value {
        todo!()
    }

    /// Convert semantic type to IR type, lowering to primitive or union type.
    fn semtype_to_irtype(&self, id: TypeId) -> Type {
        todo!()
    }
}

impl<'a> Visitor<Result<Instruction, Error>> for IR<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Result<Instruction, Error> {
        let name = node.name.kind.to_string(); // TODO: store as string??
        let func_type = self.ctx.lookup(self.ctx.get_node(node));

        let TypeKind::Function(ref param_ids, ret_id) = func_type.kind else {
            // Not implemented correctly if not function type
            panic!("function type was not TypeKind::Function")
        };

        let ret = self.semtype_to_irtype(ret_id);
        let params = param_ids
            .iter()
            .map(|ty| self.semtype_to_irtype(*ty))
            .collect();

        Ok(Instruction::Func(FuncInst { name, params, ret }))
    }

    fn visit_block(&mut self, node: &BlockNode) -> Result<Instruction, Error> {
        todo!()
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Result<Instruction, Error> {
        todo!()
    }

    fn visit_literal(&mut self, _: &Token) -> Result<Instruction, Error> {
        panic!("unused method")
    }

    fn visit_type(&mut self, _: &TypeNode) -> Result<Instruction, Error> {
        panic!("unused method")
    }
}
