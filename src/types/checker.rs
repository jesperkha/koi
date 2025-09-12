use crate::{
    ast::{
        Ast, BlockNode, FuncNode, Node, PrimitiveType, ReturnNode, TypeId, Visitable, Visitor,
        no_type, void_type,
    },
    error::Error,
    token::{File, Token, TokenKind},
    types::TypeContext,
};

pub struct Checker<'a> {
    ctx: TypeContext,
    ast: &'a Ast,
    file: &'a File,
    errs: Vec<Error>,

    /// Return type in current scope
    rtype: TypeId,
}

type CheckResult = Result<TypeContext, Vec<Error>>;

impl<'a> Checker<'a> {
    pub fn check(ast: &'a Ast, file: &'a File) -> CheckResult {
        let mut s = Self {
            ast,
            file,
            ctx: TypeContext::new(),
            errs: Vec::new(),
            rtype: no_type(),
        };

        for node in &s.ast.nodes {
            if let Err(err) = node.accept(&mut s) {
                s.errs.push(err);
            }
        }

        if s.errs.is_empty() {
            Ok(s.ctx)
        } else {
            Err(s.errs)
        }
    }

    fn error(&self, msg: &str, node: &dyn Node) -> Error {
        Error::range(msg, node.pos(), node.end(), self.file)
    }
}

impl<'a> Visitor<Result<TypeId, Error>> for Checker<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Result<TypeId, Error> {
        /*
            skjekke om navn er declared
            declare return type i dette scopet
                skjekke om de matcher

            om det er main så
                1. skjekke at ingen args
                2. return type int
                3. er public
        */

        let mut ret_type = void_type();

        // Evaluate return type if any
        if let Some(t) = &node.ret_type {
            match self.ctx.resolve_ast_node_type(t) {
                Some(id) => ret_type = id,
                None => return Err(self.error("not a type", t)),
            };
            assert_ne!(ret_type, no_type(), "must be valid type or error");
        }

        self.rtype = ret_type; // Set for current scope
        for stmt in &node.body.stmts {
            let _ = stmt.accept(self);
        }

        Ok(0)
    }

    fn visit_block(&mut self, node: &BlockNode) -> Result<TypeId, Error> {
        todo!()
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Result<TypeId, Error> {
        todo!()
    }

    fn visit_literal(&mut self, node: &Token) -> Result<TypeId, Error> {
        match &node.kind {
            TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::Int64)),
            _ => todo!(),
        }
    }
}
