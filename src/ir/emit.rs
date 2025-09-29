use crate::{
    ast::{Ast, BlockNode, FuncNode, ReturnNode, TypeNode, Visitor},
    error::Error,
    ir::{Instruction, Value},
    token::Token,
    types::TypeContext,
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
}

impl<'a> Visitor<Instruction> for IR<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Instruction {
        todo!()
    }

    fn visit_block(&mut self, node: &BlockNode) -> Instruction {
        todo!()
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Instruction {
        todo!()
    }

    fn visit_literal(&mut self, _: &Token) -> Instruction {
        panic!("unused method")
    }

    fn visit_type(&mut self, _: &TypeNode) -> Instruction {
        panic!("unused method")
    }
}
