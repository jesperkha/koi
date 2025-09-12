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
    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
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
            has_returned: false,
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

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        Error::new(msg, tok, tok, self.file)
    }
}

impl<'a> Visitor<Result<TypeId, Error>> for Checker<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> Result<TypeId, Error> {
        let mut ret_type = void_type();

        // Evaluate return type if any
        if let Some(t) = &node.ret_type {
            match self.ctx.resolve_ast_node_type(t) {
                Some(id) => ret_type = id,
                None => return Err(self.error("not a type", t)),
            };
            assert_ne!(ret_type, no_type(), "must be valid type or error");
        }

        // Set for current scope only
        self.rtype = ret_type;
        self.has_returned = false;

        if let Err(err) = self.visit_block(&node.body) {
            return Err(err);
        }

        // There was no return when there should have been
        if !self.has_returned && ret_type != no_type() {
            return Err(self.error_token("expected return", &node.name));
        }

        Ok(no_type())
    }

    fn visit_block(&mut self, node: &BlockNode) -> Result<TypeId, Error> {
        for stmt in &node.stmts {
            if let Err(err) = stmt.accept(self) {
                return Err(err);
            }
        }
        Ok(no_type())
    }

    fn visit_return(&mut self, node: &ReturnNode) -> Result<TypeId, Error> {
        self.has_returned = true;

        // If there is a return expression
        // Evaluate it and compare with current scopes return type
        if let Some(expr) = &node.expr {
            let ty = match expr.accept(self) {
                Ok(ty) => ty,
                Err(err) => return Err(err),
            };

            if ty != self.rtype {
                Err(self.error("expected return type _ got _", expr))
            } else {
                Ok(ty)
            }

        // If there is no return expression
        // Check if current scope has no return type
        } else {
            if self.rtype != void_type() {
                Err(self.error_token("expected return type _", &node.kw))
            } else {
                Ok(void_type())
            }
        }
    }

    fn visit_literal(&mut self, node: &Token) -> Result<TypeId, Error> {
        match &node.kind {
            TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::Int64)),
            TokenKind::True | TokenKind::False => Ok(self.ctx.primitive(PrimitiveType::Bool)),
            _ => todo!(),
        }
    }
}
