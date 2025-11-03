use tracing::info;

use crate::{
    ast::{
        BlockNode, CallExpr, Expr, Field, File, FuncNode, GroupExpr, Node, ReturnNode, TypeNode,
        Visitable, Visitor,
    },
    config::Config,
    error::{Error, ErrorSet, Res},
    token::{Pos, Token, TokenKind},
    types::{Package, PrimitiveType, SymTable, TypeContext, TypeId, TypeKind, no_type},
};

// TODO: Complete imports
// 1. Scan each file in package and collect all exported items into Exports
// 2. Create a map of all exports in the project, including std and external imports
// 3. Type check each package using this import map
// 4. Checker now only accepts a list of Decl, typecontext

/*
    exports, ctx = collect_exports(file)
    pkg = check(export, ctx, file)
*/

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

struct Value {
    ty: TypeId,
    constant: bool,
}

struct Checker<'a> {
    ctx: &'a mut TypeContext,
    vars: SymTable<Value>,
    file: &'a File,
    _config: &'a Config,

    /// Return type in current scope
    rtype: TypeId,

    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
}

impl<'a> Checker<'a> {
    fn new(file: &'a File, ctx: &'a mut TypeContext, config: &'a Config) -> Self {
        Self {
            _config: config,
            file,
            ctx,
            vars: SymTable::new(),
            rtype: no_type(),
            has_returned: false,
        }
    }

    /// Iterates over each node in the file and type checks. Populates
    /// TypeContext with files types. Collects errors.
    fn check(mut self) -> ErrorSet {
        let mut errs = ErrorSet::new();
        info!("file '{}', pkg '{}'", self.file.src.name, self.file.pkgname);

        for n in &self.file.nodes {
            let _ = self.eval(n).map_err(|e| errs.add(e));
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
                // Has to be internalized here since Node methods are not
                // impemented for all node types, just their overarching
                // kind (stmt, decl etc).
                self.ctx.intern_node(node, ty);
            }
            ty
        })
    }

    /// Evaluate an option of a node. Defaults to void type if not present.
    fn eval_optional<V: Visitable + Node>(&mut self, v: &Option<V>) -> Result<TypeId, Error> {
        v.as_ref().map_or(Ok(self.ctx.void()), |r| self.eval(r))
    }

    fn error(&self, msg: &str, node: &dyn Node) -> Error {
        Error::range(msg, node.pos(), node.end(), &self.file.src)
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        Error::new(msg, tok, tok, &self.file.src)
    }

    fn error_from_to(&self, msg: &str, from: &Pos, to: &Pos) -> Error {
        Error::range(msg, from, to, &self.file.src)
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

    // TODO: remove constants (maybe)
    /// Bind a name (token) to a type. Returns same type id or error if already defined.
    fn bind(&mut self, name: &Token, id: TypeId, constant: bool) -> Result<TypeId, Error> {
        if !self.vars.bind(name, Value { ty: id, constant }) {
            Err(self.error_token("already declared", name))
        } else {
            Ok(id)
        }
    }

    /// Collect a list of type ids for each field in the slice.
    fn collect_field_types(&mut self, fields: &[Field]) -> Result<Vec<TypeId>, Error> {
        fields.iter().map(|f| self.eval(&f.typ)).collect()
    }

    /// Report whether the given l-value is constant or not.
    fn is_constant(&self, lval: &Expr) -> bool {
        match lval {
            Expr::Literal(token) => match &token.kind {
                TokenKind::IdentLit(name) => self.vars.get(name).map_or(false, |sym| sym.constant),
                _ => false,
            },
            Expr::Group(_) | Expr::Call(_) => true,
        }
    }
}

type EvalResult = Result<TypeId, Error>;

impl<'a> Visitor<EvalResult> for Checker<'a> {
    fn visit_func(&mut self, node: &FuncNode) -> EvalResult {
        // Evaluate return type if any
        let ret_type = self.eval_optional(&node.ret_type)?;

        // Get parameter types
        let params = &node
            .params
            .iter()
            .map(|f| self.eval(&f.typ).map(|id| (&f.name, id)))
            .collect::<Result<Vec<_>, _>>()?;

        // If this is the main function we do additional checks
        if node.name.to_string() == "main" {
            let int_id = self.ctx.primitive(PrimitiveType::I64);

            // Must be package main
            if !self.file.pkgname.is_empty() && self.file.pkgname != "main" {
                info!("package name expected to be main, is {}", self.file.pkgname);
                return Err(self.error("main function can only be declared in main package", node));
            }

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
        }

        // Declare function while still in global scope
        let func_id = self.ctx.get_or_intern(TypeKind::Function(
            params.iter().map(|v| v.1).collect(),
            ret_type,
        ));
        self.bind(&node.name, func_id, true)?;

        // Set up function body
        self.vars.push_scope();
        self.rtype = ret_type;
        self.has_returned = false;

        // Declare params in function body
        for p in params {
            self.bind(p.0, p.1, false)?;
        }

        if let Err(err) = self.visit_block(&node.body) {
            return Err(err);
        }

        self.vars.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && ret_type != self.ctx.void() {
            return Err(self.error_token(
                format!("missing return in function '{}'", node.name.kind).as_str(),
                &node.body.rbrace,
            ));
        }

        Ok(func_id)
    }

    fn visit_block(&mut self, node: &BlockNode) -> EvalResult {
        for stmt in &node.stmts {
            self.eval(stmt)?;
        }
        Ok(no_type())
    }

    fn visit_return(&mut self, node: &ReturnNode) -> EvalResult {
        self.has_returned = true;

        // If there is a return expression
        // Evaluate it and compare with current scopes return type
        if let Some(expr) = &node.expr {
            let ty = self.eval(expr)?;

            return if ty != self.rtype {
                Err(self.error_expected_got("incorrect return type", self.rtype, ty, expr))
            } else {
                Ok(ty)
            };
        }

        // If there is no return expression
        // Check if current scope has no return type
        if self.rtype != self.ctx.void() {
            Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
        } else {
            Ok(self.ctx.void())
        }
    }

    fn visit_literal(&mut self, node: &Token) -> EvalResult {
        match &node.kind {
            TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::I64)),
            TokenKind::FloatLit(_) => Ok(self.ctx.primitive(PrimitiveType::F64)),
            TokenKind::StringLit(_) => Ok(self.ctx.primitive(PrimitiveType::String)),
            TokenKind::True | TokenKind::False => Ok(self.ctx.primitive(PrimitiveType::Bool)),
            TokenKind::IdentLit(name) => self
                .vars
                .get(name)
                .map_or(Err(self.error_token("not declared", node)), |t| Ok(t.ty)),
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
                .ctx
                .get_symbol(token.to_string())
                .map_or(Err(self.error_token("not a type", token)), |ty| Ok(ty)),
        }
    }

    fn visit_package(&mut self, node: &Token) -> EvalResult {
        Err(self.error_token("package already declared earlier in file", node))
    }

    fn visit_call(&mut self, node: &CallExpr) -> EvalResult {
        let callee_id = self.eval(node.callee.as_ref())?;
        let callee_kind = self.ctx.lookup(callee_id).kind.clone();

        if let TypeKind::Function(params, ret_id) = &callee_kind {
            // Check if number of arguments matches
            let (n_params, n_args) = (params.len(), node.args.len());
            if n_params != n_args {
                let msg = format!("function takes {} arguments, got {}", n_params, n_args,);
                return Err(self
                    .error_from_to(&msg, node.callee.pos(), &node.rparen.pos)
                    .with_info(&format!("definition: {}", self.ctx.to_string(callee_id))));
            }

            // Check if each argument type matches the param type
            for (i, param_id) in params.iter().enumerate() {
                let arg_id = self.eval(&node.args[i])?;
                if *param_id != arg_id {
                    let msg = format!(
                        "mismatched types in function call. expected '{}', got '{}'",
                        self.ctx.to_string(*param_id),
                        self.ctx.to_string(arg_id)
                    );
                    return Err(self.error(&msg, &node.args[i]));
                }
            }

            return Ok(*ret_id);
        }

        info!("callee type is actually: {:?}", callee_kind);
        Err(self.error("not a function", &*node.callee))
    }

    fn visit_group(&mut self, node: &GroupExpr) -> EvalResult {
        node.inner.accept(self)
    }

    fn visit_extern(&mut self, node: &crate::ast::FuncDeclNode) -> EvalResult {
        let ret = self.eval_optional(&node.ret_type)?;
        let params = self.collect_field_types(&node.params)?;
        let id = self.ctx.get_or_intern(TypeKind::Function(params, ret));
        self.bind(&node.name, id, true)
    }

    fn visit_var_decl(&mut self, node: &crate::ast::VarDeclNode) -> EvalResult {
        let id = self.eval(&node.expr)?;
        self.bind(&node.name, id, node.constant)
    }

    fn visit_var_assign(&mut self, node: &crate::ast::VarAssignNode) -> EvalResult {
        let lval_id = self.eval(&node.lval)?;
        let rval_id = self.eval(&node.expr)?;

        if lval_id != rval_id {
            return Err(self.error(
                &format!(
                    "mismatched types in assignment. expected '{}', got '{}'",
                    self.ctx.to_string(lval_id),
                    self.ctx.to_string(rval_id)
                ),
                &node.expr,
            ));
        }

        if self.is_constant(&node.lval) {
            return Err(self.error("cannot assign new value to a constant", &node.lval));
        }

        Ok(no_type())
    }

    fn visit_import(&mut self, node: &crate::ast::ImportNode) -> EvalResult {
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

        TokenKind::StringType => PrimitiveType::String,

        _ => panic!("unknown TypeNode::Primitive kind: {}", tok.kind),
    }
}
