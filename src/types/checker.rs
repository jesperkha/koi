use tracing::info;

use crate::{
    ast::{self, Field, Node, TypeNode},
    config::Config,
    error::{Error, ErrorSet},
    token::{Pos, Source, Token, TokenKind},
    types::{
        self, LiteralKind, NodeMeta, PrimitiveType, Type, TypeContext, TypeId, TypeKind, TypedNode,
        ast_node_to_meta, no_type, symtable::SymTable,
    },
};

struct Value {
    ty: TypeId,
    constant: bool,
}

pub struct Checker<'a> {
    pkg: String,
    ctx: &'a mut TypeContext,
    src: &'a Source,
    _config: &'a Config,

    /// Locally declared variables. Package private and global
    /// symbols are part of the TypeContext.
    vars: SymTable<Value>,

    /// Return type in current scope
    rtype: TypeId,

    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
}

impl<'a> Checker<'a> {
    pub fn new(src: &'a Source, pkg: String, ctx: &'a mut TypeContext, config: &'a Config) -> Self {
        Self {
            _config: config,
            src,
            pkg,
            ctx,
            vars: SymTable::new(),
            rtype: no_type(),
            has_returned: false,
        }
    }

    fn error(&self, msg: &str, node: &dyn Node) -> Error {
        Error::range(msg, node.pos(), node.end(), &self.src)
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Error {
        Error::new(msg, tok, tok, &self.src)
    }

    fn error_from_to(&self, msg: &str, from: &Pos, to: &Pos) -> Error {
        Error::range(msg, from, to, &self.src)
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

    /// Get a declared symbol by a token identifier. Returns "not declared" error if not found.
    fn get(&self, name: &Token) -> Result<TypeId, Error> {
        let name_str = name.to_string();
        if let Some(var) = self.vars.get(&name_str) {
            return Ok(var.ty);
        }
        if let Ok(sym) = self.ctx.get_symbol(&name_str) {
            return Ok(sym);
        }
        Err(self.error_token("not declared", name))
    }

    /// Get the type of a declared symbol
    fn get_symbol_type(&self, name: &Token) -> Result<&Type, Error> {
        let id = self.get(name)?;
        Ok(self.ctx.lookup(id))
    }

    /// Collect a list of type ids for each field in the slice.
    fn collect_field_types(&mut self, fields: &[Field]) -> Result<Vec<TypeId>, Error> {
        fields.iter().map(|f| self.eval_type(&f.typ)).collect()
    }

    /// Report whether the given l-value is constant or not.
    fn is_constant(&self, lval: &ast::Expr) -> bool {
        match lval {
            ast::Expr::Literal(token) => match &token.kind {
                TokenKind::IdentLit(name) => self.vars.get(name).map_or(false, |sym| sym.constant),
                _ => false,
            },
            ast::Expr::Group(_) | ast::Expr::Call(_) => true,
            ast::Expr::Member(node) => self.is_constant(&node.expr),
        }
    }

    /// Evaluate an AST type node to its semantic type id.
    fn eval_type(&mut self, node: &TypeNode) -> Result<TypeId, Error> {
        match node {
            TypeNode::Primitive(token) => {
                let prim = token_to_primitive_type(token);
                Ok(self.ctx.primitive(prim))
            }
            TypeNode::Ident(token) => self
                .get(token)
                .map_or(Err(self.error_token("not a type", token)), |ty| Ok(ty)),
        }
    }

    /// Evaluate an option of a type node. Defaults to void type if not present.
    fn eval_optional_type(&mut self, v: &Option<TypeNode>) -> Result<TypeId, Error> {
        v.as_ref()
            .map_or(Ok(self.ctx.void()), |r| self.eval_type(r))
    }

    /// Check if the expression is an identifier and return the corresponding type.
    fn if_identifier_get_type(&self, expr: &ast::Expr) -> Option<&Type> {
        if let ast::Expr::Literal(token) = expr {
            if let TokenKind::IdentLit(name) = &token.kind {
                if let Ok(id) = self.ctx.get_symbol(name) {
                    return Some(self.ctx.lookup(id));
                }
            }
        }
        None
    }

    // ---------------------------- Global first pass ---------------------------- //

    pub fn global_pass(&mut self, decls: &Vec<ast::Decl>) -> Result<(), ErrorSet> {
        let mut errs = ErrorSet::new();
        for d in decls {
            let _ = match d {
                ast::Decl::Func(node) => self.declare_function(node),
                ast::Decl::Extern(node) => self.declare_extern(node),
                _ => Ok(()),
            }
            .map_err(|e| errs.add(e));
        }

        errs.err_or(())
    }

    fn declare_function(&mut self, node: &ast::FuncNode) -> Result<(), Error> {
        self.declare_function_definition(&node.name, node.public, &node.params, &node.ret_type)
    }

    fn declare_extern(&mut self, node: &ast::FuncDeclNode) -> Result<(), Error> {
        self.declare_function_definition(&node.name, node.public, &node.params, &node.ret_type)
    }

    fn declare_function_definition(
        &mut self,
        name: &Token,
        public: bool,
        params: &Vec<ast::Field>,
        ret_type: &Option<ast::TypeNode>,
    ) -> Result<(), Error> {
        // Evaluate return type if any
        let ret_id = self.eval_optional_type(ret_type)?;

        // Get parameter types
        let param_ids = &params
            .iter()
            .map(|f| self.eval_type(&f.typ).map(|id| (&f.name, id)))
            .collect::<Result<Vec<_>, _>>()?;

        // Declare function in context
        let kind = TypeKind::Function(param_ids.iter().map(|v| v.1).collect(), ret_id);
        let func_id = self.ctx.get_or_intern(kind);
        let name = name.to_string();
        self.ctx.set_symbol(name, func_id, public);

        Ok(())
    }

    // ---------------------------- Generate AST ---------------------------- //

    pub fn emit_ast(&mut self, decls: Vec<ast::Decl>) -> Result<Vec<types::Decl>, ErrorSet> {
        let mut errs = ErrorSet::new();
        info!("file '{}'", self.src.filepath);

        let typed_decls = decls
            .into_iter()
            .map(|d| self.emit_decl(d))
            .map(|s| s.map_err(|e| errs.add(e)))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if errs.len() > 0 {
            info!("fail, finished with {} errors", errs.len());
        }

        errs.err_or(typed_decls)
    }

    fn emit_decl(&mut self, decl: ast::Decl) -> Result<types::Decl, Error> {
        match decl {
            ast::Decl::Func(node) => self.emit_func(node),
            ast::Decl::Extern(node) => self.emit_extern(node),
            ast::Decl::Import(_) => panic!("import statements should not be emitted"),
        }
    }

    fn emit_stmt(&mut self, stmt: ast::Stmt) -> Result<types::Stmt, Error> {
        match stmt {
            ast::Stmt::ExprStmt(node) => Ok(types::Stmt::ExprStmt(self.emit_expr(node)?)),
            ast::Stmt::Return(node) => self.emit_return(node),
            ast::Stmt::VarDecl(node) => self.emit_var_decl(node),
            ast::Stmt::VarAssign(node) => self.emit_var_assign(node),
            ast::Stmt::Block(_) => panic!("block should be handled manually as list of stmt"),
        }
    }

    fn emit_expr(&mut self, expr: ast::Expr) -> Result<types::Expr, Error> {
        match expr {
            ast::Expr::Literal(tok) => self.emit_literal(tok),
            ast::Expr::Group(node) => self.emit_expr(*node.inner),
            ast::Expr::Call(node) => self.emit_call(node),
            ast::Expr::Member(node) => self.emit_member(node),
        }
    }

    fn emit_func(&mut self, node: ast::FuncNode) -> Result<types::Decl, Error> {
        let meta = ast_node_to_meta(&node);

        // Get declared function
        let func_type = self.get_symbol_type(&node.name)?.clone(); // moved later
        let TypeKind::Function(param_ids, ret_id) = &func_type.kind else {
            panic!("function was not declared properly");
        };

        // If this is the main function we do additional checks
        if node.name.to_string() == "main" {
            let int_id = self.ctx.primitive(PrimitiveType::I64);

            // Must be package main
            if !self.pkg.is_empty() && self.pkg != "main" {
                info!("package name expected to be main, is {}", self.pkg);
                return Err(self.error("main function can only be declared in main package", &node));
            }

            // If return type is not int
            if !self.ctx.equivalent(*ret_id, int_id) {
                let msg = "main function must return 'i64'";
                return Err(node
                    .ret_type
                    .as_ref()
                    .map_or(self.error_token(msg, &node.rparen), |ty_node| {
                        self.error(msg, ty_node)
                    }));
            }

            // No parameters allowed
            if param_ids.len() > 0 {
                return Err(self.error("main function must not take any arguments", &node));
            }
        }

        // Set up function body
        self.vars.push_scope();
        self.rtype = *ret_id;
        self.has_returned = false;

        // Declare params in function body
        for (i, ty) in param_ids.iter().enumerate() {
            let name = &node.params[i].name;
            self.bind(name, *ty, false)?;
        }

        let body = node
            .body
            .stmts
            .into_iter()
            .map(|s| self.emit_stmt(s))
            .collect::<Result<Vec<types::Stmt>, Error>>()?;

        self.vars.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && *ret_id != self.ctx.void() {
            return Err(self.error_token(
                format!("missing return in function '{}'", node.name.kind).as_str(),
                &node.body.rbrace,
            ));
        }

        Ok(types::Decl::Func(types::FuncNode {
            meta,
            name: node.name.to_string(),
            public: node.public,
            ty: func_type,
            params: node.params.iter().map(|p| p.name.to_string()).collect(),
            body,
        }))
    }

    fn emit_extern(&mut self, node: ast::FuncDeclNode) -> Result<types::Decl, Error> {
        let meta = ast_node_to_meta(&node);

        let ret_id = self.eval_optional_type(&node.ret_type)?;
        let params = self.collect_field_types(&node.params)?;
        let kind = TypeKind::Function(params, ret_id);
        let id = self.ctx.get_or_intern(kind.clone());
        self.bind(&node.name, id, true)?;

        Ok(types::Decl::Extern(types::ExternNode {
            meta,
            ty: Type { kind, id },
            name: node.name.to_string(),
        }))
    }

    fn emit_var_assign(&mut self, node: ast::VarAssignNode) -> Result<types::Stmt, Error> {
        let meta = ast_node_to_meta(&node);

        if self.is_constant(&node.lval) {
            return Err(self.error("cannot assign new value to a constant", &node.lval));
        }

        let lval = self.emit_expr(node.lval)?;
        let rval = self.emit_expr(node.expr)?;

        if lval.type_id() != rval.type_id() {
            return Err(self.error(
                &format!(
                    "mismatched types in assignment. expected '{}', got '{}'",
                    self.ctx.to_string(lval.type_id()),
                    self.ctx.to_string(rval.type_id())
                ),
                &rval,
            ));
        }

        Ok(types::Stmt::VarAssign(types::VarAssignNode {
            meta,
            ty: self.ctx.void_type(),
            lval,
            rval,
        }))
    }

    fn emit_var_decl(&mut self, node: ast::VarDeclNode) -> Result<types::Stmt, Error> {
        let meta = ast_node_to_meta(&node);
        let typed_expr = self.emit_expr(node.expr)?;

        if typed_expr.type_id() == self.ctx.void() {
            return Err(self.error("cannot assign void type to variable", &typed_expr));
        }

        let id = self.bind(&node.name, typed_expr.type_id(), node.constant)?;
        Ok(types::Stmt::VarDecl(types::VarDeclNode {
            meta,
            ty: self.ctx.lookup(id).clone(),
            name: node.name.to_string(),
            value: typed_expr,
        }))
    }

    fn emit_return(&mut self, node: ast::ReturnNode) -> Result<types::Stmt, Error> {
        self.has_returned = true;
        let meta = ast_node_to_meta(&node);

        // If there is a return expression
        // Evaluate it and compare with current scopes return type
        if let Some(expr) = node.expr {
            let typed_expr = self.emit_expr(expr)?;

            return if typed_expr.type_id() != self.rtype {
                Err(self.error_expected_got(
                    "incorrect return type",
                    self.rtype,
                    typed_expr.type_id(),
                    &typed_expr,
                ))
            } else {
                Ok(types::Stmt::Return(types::ReturnNode {
                    meta,
                    ty: Type {
                        kind: typed_expr.kind().clone(),
                        id: typed_expr.type_id(),
                    },
                    expr: Some(typed_expr),
                }))
            };
        }

        // If there is no return expression
        // Check if current scope has no return type
        if self.rtype != self.ctx.void() {
            Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
        } else {
            Ok(types::Stmt::Return(types::ReturnNode {
                meta,
                expr: None,
                ty: self.ctx.void_type(),
            }))
        }
    }

    fn emit_literal(&mut self, tok: Token) -> Result<types::Expr, Error> {
        let ty = match &tok.kind {
            TokenKind::IntLit(_) => self.ctx.primitive_type(PrimitiveType::I64),
            TokenKind::FloatLit(_) => self.ctx.primitive_type(PrimitiveType::F64),
            TokenKind::StringLit(_) => self.ctx.primitive_type(PrimitiveType::String),
            TokenKind::True | TokenKind::False => self.ctx.primitive_type(PrimitiveType::Bool),
            TokenKind::IdentLit(_) => {
                let ty_id = self.get(&tok)?;
                let t = self.ctx.lookup(ty_id);
                if matches!(t.kind, TypeKind::Namespace(_)) {
                    return Err(self.error_token("namespace cannot be used as a value", &tok));
                }
                t
            }
            _ => todo!(),
        };

        Ok(types::Expr::Literal(types::LiteralNode {
            meta: NodeMeta {
                id: tok.id,
                pos: tok.pos,
                end: tok.end_pos,
            },
            ty: ty.clone(),
            kind: token_kind_to_type_literal_kind(tok.kind),
        }))
    }

    fn emit_call(&mut self, node: ast::CallExpr) -> Result<types::Expr, Error> {
        let meta = ast_node_to_meta(&node);
        let callee = self.emit_expr(*node.callee)?;

        if let TypeKind::Function(params, ret_id) = callee.kind() {
            // Check if number of arguments matches
            if params.len() != node.args.len() {
                let msg = format!(
                    "function takes {} arguments, got {}",
                    params.len(),
                    node.args.len(),
                );
                return Err(self
                    .error_from_to(&msg, callee.pos(), &node.rparen.pos)
                    .with_info(&format!(
                        "definition: {}",
                        self.ctx.to_string(callee.type_id())
                    )));
            }

            assert_eq!(
                params.len(),
                node.args.len(),
                "sanity check: args and params are same size"
            );

            let mut args = Vec::new();
            for (i, arg) in node.args.into_iter().enumerate() {
                let typed_arg = self.emit_expr(arg)?;

                // Check if each argument type matches the param type
                let (arg_id, param_id) = (typed_arg.type_id(), params[i]);
                if arg_id != param_id {
                    let msg = format!(
                        "mismatched types in function call. expected '{}', got '{}'",
                        self.ctx.to_string(param_id),
                        self.ctx.to_string(arg_id)
                    );
                    return Err(self.error(&msg, &typed_arg));
                }

                args.push(typed_arg);
            }

            return Ok(types::Expr::Call(types::CallNode {
                meta,
                ty: self.ctx.lookup(*ret_id).clone(),
                callee: Box::new(callee),
                args,
            }));
        }

        info!("callee type is actually: {:?}", callee.kind());
        Err(self.error("not a function", &callee))
    }

    fn emit_member(&mut self, node: ast::MemberNode) -> Result<types::Expr, Error> {
        let meta = ast_node_to_meta(&node);
        let field = node.field.to_string();

        // First check if the left hand value is a namespace
        if let Some(ty) = self.if_identifier_get_type(&*node.expr) {
            if let TypeKind::Namespace(ns) = &ty.kind {
                // Get symbol from field
                let Some(symbol) = ns.symbols.get(&field) else {
                    return Err(self.error_token(
                        &format!("namespace '{}' has no member '{}'", &ns.name, &field),
                        &node.field,
                    ));
                };

                return Ok(types::Expr::NamespaceMember(types::NamespaceMemberNode {
                    ty: self.ctx.lookup(*symbol).clone(),
                    namespace: ns.name.clone(),
                    meta,
                    field,
                }));
            }
        }

        // Otherwise this is a normal member getter and we treat lval as
        // a normal expression.
        let expr = self.emit_expr(*node.expr)?;

        // TODO: implement struct fields here
        return Err(self.error(
            &format!(
                "type '{}' has no fields",
                self.ctx.to_string(expr.type_id())
            ),
            &expr,
        ));
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

fn token_kind_to_type_literal_kind(kind: TokenKind) -> LiteralKind {
    match kind {
        TokenKind::IdentLit(name) => LiteralKind::Ident(name),
        TokenKind::IntLit(n) => LiteralKind::Int(n),
        TokenKind::FloatLit(n) => LiteralKind::Float(n),
        TokenKind::StringLit(s) => LiteralKind::String(s),
        TokenKind::CharLit(c) => LiteralKind::Char(c),
        TokenKind::True => LiteralKind::Bool(true),
        TokenKind::False => LiteralKind::Bool(false),
        TokenKind::Null => todo!(),
        _ => panic!("unhandled token kind in conversion, {:?}", kind),
    }
}
