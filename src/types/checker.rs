use std::collections::HashMap;

use tracing::{error, info};

use crate::{
    ast::{BlockNode, File, FuncNode, Node, ReturnNode, TypeNode, Visitable, Visitor},
    config::Config,
    error::{Error, ErrorSet, Res},
    token::{Token, TokenKind},
    types::{Package, PrimitiveType, SymTable, TypeContext, TypeId, TypeKind, no_type},
};

pub fn check(files: Vec<File>, config: &Config) -> Res<Package> {
    let mut ctx = TypeContext::new();
    let mut errs = ErrorSet::new();

    info!("checking {} files", files.len());

    // TODO: remove this check and handle empty packages properly
    assert!(files.len() > 0, "no files to type check");

    for file in &files {
        let checker = Checker::new(&file, &mut ctx, config);
        errs.join(checker.check());
    }

    if errs.len() > 0 {
        info!("fail, finished all with {} errors", errs.len());
        return Err(errs);
    }

    // TODO: assert all pkg names equal

    info!("success, all files");
    Ok(Package::new(
        files[0].pkgname.clone(),
        // TODO: filepath in packages, copy from file
        "".to_string(),
        files,
        ctx,
    ))
}

struct Checker<'a> {
    ctx: &'a mut TypeContext,
    sym: SymTable<TypeId>,
    file: &'a File,
    config: &'a Config,

    /// Map of global type declarations.
    type_decls: HashMap<String, TypeId>,

    /// Return type in current scope
    rtype: TypeId,

    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
}

impl<'a> Checker<'a> {
    fn new(file: &'a File, ctx: &'a mut TypeContext, config: &'a Config) -> Self {
        Self {
            config,
            file,
            ctx,
            sym: SymTable::new(),
            rtype: no_type(),
            has_returned: false,
            type_decls: HashMap::new(),
        }
    }

    fn check(mut self) -> ErrorSet {
        let mut errs = ErrorSet::new();
        info!("file '{}', pkg '{}'", self.file.src.name, self.file.pkgname);

        for node in &self.file.nodes {
            if let Err(err) = self.eval(node) {
                errs.add(err);
            }
        }

        if errs.len() > 0 {
            info!("fail, finished with {} errors", errs.len());
        }
        errs
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
        error!("{}", msg);
        Error::range(msg, node.pos(), node.end(), &self.file.src)
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        error!("{}", msg);
        Error::new(msg, tok, tok, &self.file.src)
    }

    fn error_expected_token(&self, msg: &str, expect: TypeId, tok: &Token) -> Error {
        self.error_token(
            format!("{}: expected '{}'", msg, self.ctx.to_string(expect),).as_str(),
            tok,
        )
    }

    fn error_expected_got(&self, msg: &str, expect: TypeId, got: TypeId, node: &dyn Node) -> Error {
        self.error(
            format!(
                "{}: expected '{}', got '{}'",
                msg,
                self.ctx.to_string(expect),
                self.ctx.to_string(got)
            )
            .as_str(),
            node,
        )
    }

    /// Get a declared type.
    pub fn get_type(&self, name: &Token) -> Option<TypeId> {
        self.type_decls.get(&name.to_string()).copied()
    }
}

type EvalResult = Result<TypeId, Error>;

impl<'a> Visitor<EvalResult> for Checker<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> EvalResult {
        let mut ret_type = self.ctx.void();

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

        // If this is the main function we do additional checks
        if node.name.to_string() == "main" {
            let int_id = self.ctx.primitive(PrimitiveType::I64);

            // If return type is not int
            if !self.ctx.equivalent(ret_type, int_id) {
                let msg = "main function must return 'i64'";
                return Err(node
                    .ret_type
                    .as_ref()
                    .map_or(self.error_token(msg, &node.rparen), |ty_node| {
                        self.error(msg, ty_node)
                    }));
            }

            // No parameters allowed
            if params.len() > 0 {
                return Err(self.error("main function must not take any arguments", node));
            }

            // Must be package main
            if self.file.pkgname != "main" {
                return Err(self.error("main function can only be declared in main package", node));
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

        self.ctx.intern_node(node, func_id);

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
        if !self.has_returned && ret_type != self.ctx.void() {
            return Err(self.error_token(
                format!("missing return in function '{}'", node.name.kind).as_str(),
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
            if self.rtype != self.ctx.void() {
                Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
            } else {
                Ok(self.ctx.void())
            }
        }
    }

    fn visit_literal(&mut self, node: &Token) -> EvalResult {
        match &node.kind {
            TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::I64)),
            TokenKind::FloatLit(_) => Ok(self.ctx.primitive(PrimitiveType::F64)),
            TokenKind::True | TokenKind::False => Ok(self.ctx.primitive(PrimitiveType::Bool)),
            TokenKind::IdentLit(name) => self
                .sym
                .get_symbol(name)
                .map_or(Err(self.error_token("not declared", node)), |t| Ok(*t)),
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
                .get_type(token)
                .map_or(Err(self.error_token("not a type", token)), |ty| Ok(ty)),
        }
    }

    fn visit_package(&mut self, node: &Token) -> EvalResult {
        Err(self.error_token("package already declared earlier in file", node))
    }

    fn visit_call(&mut self, node: &crate::ast::CallExpr) -> EvalResult {
        todo!()
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
