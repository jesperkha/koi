use crate::{
    ast::{Ast, BlockNode, FuncNode, Node, ReturnNode, Visitable, Visitor},
    error::Error,
    token::{File, Token, TokenKind},
    types::{PrimitiveType, TypeContext, TypeId, no_type, void_type},
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
            if let Err(err) = s.eval(node) {
                s.errs.push(err);
            }
        }

        if s.errs.is_empty() {
            Ok(s.ctx)
        } else {
            Err(s.errs)
        }
    }

    /// Type check a Node. If it evaluates to a type it is internalized.
    fn eval<N: Visitable + Node>(&mut self, node: &N) -> EvalResult {
        node.accept(self).map(|ty| {
            if ty != no_type() {
                // Has to be internalized here since Node methods are
                // not impemented for node types, just their overarching
                // kind (stmt, decl etc).
                self.ctx.intern_node(node, ty);
            }
            ty
        })
    }

    fn error(&self, msg: &str, node: &dyn Node) -> Error {
        Error::range(msg, node.pos(), node.end(), self.file)
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        Error::new(msg, tok, tok, self.file)
    }

    fn error_expected_token(&self, msg: &str, expect: TypeId, tok: &Token) -> Error {
        self.error_token(
            format!("{}: expected {}", msg, self.ctx.to_string(expect),).as_str(),
            tok,
        )
    }

    fn error_expected_got(&self, msg: &str, expect: TypeId, got: TypeId, node: &dyn Node) -> Error {
        self.error(
            format!(
                "{}: expected {}, got {}",
                msg,
                self.ctx.to_string(expect),
                self.ctx.to_string(got)
            )
            .as_str(),
            node,
        )
    }
}

type EvalResult = Result<TypeId, Error>;

impl<'a> Visitor<EvalResult> for Checker<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> EvalResult {
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
            return Err(self.error_token(
                format!("missing return in function {}", node.name.kind).as_str(),
                &node.body.rbrace,
            ));
        }

        Ok(no_type())
    }

    fn visit_block(&mut self, node: &BlockNode) -> EvalResult {
        for stmt in &node.stmts {
            if let Err(err) = self.eval(stmt) {
                return Err(err);
            }
        }
        Ok(no_type())
    }

    fn visit_return(&mut self, node: &ReturnNode) -> EvalResult {
        self.has_returned = true;

        // If there is a return expression
        // Evaluate it and compare with current scopes return type
        if let Some(expr) = &node.expr {
            let ty = match self.eval(expr) {
                Ok(ty) => ty,
                Err(err) => return Err(err),
            };

            if ty != self.rtype {
                Err(self.error_expected_got("incorrect return type", self.rtype, ty, expr))
            } else {
                Ok(ty)
            }

        // If there is no return expression
        // Check if current scope has no return type
        } else {
            if self.rtype != void_type() {
                Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
            } else {
                Ok(void_type())
            }
        }
    }

    fn visit_literal(&mut self, node: &Token) -> EvalResult {
        match &node.kind {
            TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::I64)),
            TokenKind::True | TokenKind::False => Ok(self.ctx.primitive(PrimitiveType::Bool)),
            _ => todo!(),
        }
    }
}
