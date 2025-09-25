use crate::{
    ast::{Ast, BlockNode, FuncNode, Node, ReturnNode, TypeNode, Visitable, Visitor},
    error::Error,
    token::{File, Token, TokenKind},
    types::{PrimitiveType, SymTable, TypeContext, TypeId, TypeKind, no_type, void_type},
};

pub struct Checker<'a> {
    ctx: TypeContext,
    sym: SymTable,
    ast: &'a Ast,
    file: &'a File,
    errs: Vec<Error>,

    /// Return type in current scope
    rtype: TypeId,
    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
}

pub type CheckResult = Result<TypeContext, Vec<Error>>;

impl<'a> Checker<'a> {
    pub fn check(ast: &'a Ast, file: &'a File) -> CheckResult {
        let mut s = Self {
            ast,
            file,
            ctx: TypeContext::new(),
            sym: SymTable::new(),
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
            match self.eval(t) {
                Ok(id) => ret_type = id,
                Err(err) => return Err(err),
            };
            assert_ne!(ret_type, no_type(), "must be valid type or error");
        }

        // Evaluate parameter types
        let mut params = Vec::new();
        if let Some(pms) = &node.params {
            for p in pms {
                match self.eval(&p.typ) {
                    Ok(id) => params.push((&p.name, id)),
                    Err(err) => return Err(err),
                }
            }
        }

        // If this is main() assert return type is int and no params
        // TODO: make actual none and void types and add getters to context
        if node.name.kind.to_string() == "main" {
            let int_id = self.ctx.primitive(PrimitiveType::I64);
            if ret_type == void_type() || !self.ctx.equivalent(ret_type, int_id) {
                return Err(self.error("main function must return i64", node));
            }

            if params.len() > 0 {
                return Err(self.error("main function must not take any argument", node));
            }
        }

        // Declare function while still in global scope
        let func_id = self.ctx.get_or_intern(TypeKind::Function(
            params.iter().map(|v| v.1).collect(),
            ret_type,
        ));
        if !self.sym.bind(&node.name, func_id) {
            return Err(self.error_token("already declared", &node.name));
        }

        // Set up function body
        self.sym.push_scope();
        self.rtype = ret_type;
        self.has_returned = false;

        // Declare params in function body
        for p in params {
            self.sym.bind(p.0, p.1);
        }

        if let Err(err) = self.visit_block(&node.body) {
            return Err(err);
        }

        self.sym.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && ret_type != void_type() {
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

        // TODO: warn 'unreachable' for any successive statements in this block

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
            TokenKind::IdentLit(name) => self
                .sym
                .get_symbol(name)
                .map_or(Err(self.error_token("not declared", node)), Ok),
            _ => todo!(),
        }
    }

    fn visit_type(&mut self, node: &TypeNode) -> EvalResult {
        match node {
            TypeNode::Primitive(token) => {
                let prim = token_to_primitive_type(token);
                Ok(self.ctx.primitive(prim))
            }
            TypeNode::Ident(token) => self
                .sym
                .get_type(token)
                .map_or(Err(self.error_token("not a type", token)), |ty| Ok(ty)),
        }
    }
}

fn token_to_primitive_type(tok: &Token) -> PrimitiveType {
    match tok.kind {
        TokenKind::BoolType => PrimitiveType::Bool,
        TokenKind::ByteType => PrimitiveType::Byte,

        // Builtin 'aliases'
        TokenKind::IntType => PrimitiveType::I64,
        TokenKind::FloatType => PrimitiveType::F64,

        _ => panic!("unknown TypeNode::Primitive kind: {}", tok.kind),
    }
}
