use crate::{
    ast::{
        Ast, Decl, Expr, FuncNode, PrimitiveType, ReturnNode, TypeId, TypeKind, TypeNode, no_type,
        void_type,
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
}

type CheckResult = Result<TypeContext, Vec<Error>>;

impl<'a> Checker<'a> {
    pub fn check(ast: &'a Ast, file: &'a File) -> CheckResult {
        let mut s = Self {
            ast,
            file,
            ctx: TypeContext::new(),
            errs: Vec::new(),
        };

        for node in &s.ast.nodes {
            let err = match node {
                Decl::Func(fnode) => s.check_func(fnode),
            };

            if let Some(err) = err {
                s.errs.push(err);
            }
        }

        if s.errs.is_empty() {
            Ok(s.ctx)
        } else {
            Err(s.errs)
        }
    }

    fn check_func(&mut self, node: &FuncNode) -> Option<Error> {
        /*
            skjekke om navn er declared
            declare return type i dette scopet
                skjekke om de matcher

            om det er main så
                1. skjekke at ingen args
                2. return type int
                3. er public
        */

        let mut rtype = void_type();
        if let Some(t) = &node.ret_type {
            match self.eval_syntactic_type(&t) {
                Ok(id) => rtype = id,
                Err(e) => return Some(e),
            };
            assert_ne!(rtype, no_type(), "must be valid type or error");
        }

        None
    }

    fn check_return(&mut self, node: &ReturnNode) -> TypeId {
        node.expr
            .as_ref()
            .map_or(no_type(), |node| self.eval_expr(&node))
    }

    fn eval_expr(&mut self, node: &Expr) -> TypeId {
        match node {
            Expr::Literal(token) => self.check_lit(token),
        }
    }

    fn check_lit(&mut self, node: &Token) -> TypeId {
        match &node.kind {
            TokenKind::IntLit(_) => self
                .ctx
                .get_or_intern(TypeKind::Primitive(PrimitiveType::Int64)),
            _ => no_type(),
        }
    }

    /// Evaluates a syntactic type to its semantic counterpart.
    fn eval_syntactic_type(&mut self, node: &TypeNode) -> Result<TypeId, Error> {
        match node {
            TypeNode::Ident(tok) => self
                .ctx
                .get_declared(tok.to_string())
                .map_or(Err(self.error_token("not a type", tok)), |k| Ok(k.id)),

            TypeNode::Primitive(tok) => match tok.kind {
                // TODO: lage getters for primitive typer
                TokenKind::Int => Ok(self
                    .ctx
                    .get_or_intern(TypeKind::Primitive(PrimitiveType::Int64))),
                _ => panic!("unhandled token kind in eval"),
            },
        }
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        Error::new(msg, tok, tok, self.file)
    }
}
