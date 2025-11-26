use tracing::info;

use crate::{
    ast::{self, Expr, Field, Node, PackageID, TypeNode},
    config::Config,
    error::{Error, ErrorSet},
    token::{Pos, Source, Token, TokenKind},
    types::{
        self, NodeMeta, PrimitiveType, Type, TypeContext, TypeId, TypeKind, TypedNode,
        ast_node_to_meta, no_type, symtable::SymTable,
    },
};

struct Value {
    ty: TypeId,
    constant: bool,
}

pub struct Checker<'a> {
    pkg: PackageID,
    ctx: &'a mut TypeContext,
    vars: SymTable<Value>,
    src: &'a Source,
    _config: &'a Config,

    /// Return type in current scope
    rtype: TypeId,

    /// Has returned in the base function scope
    /// Not counting nested scopes as returning there is optional
    has_returned: bool,
}

impl<'a> Checker<'a> {
    // pub fn new(file: &'a File, ctx: &'a mut TypeContext, config: &'a Config) -> Self {
    //     Self {
    //         _config: config,
    //         file,
    //         ctx,
    //         vars: SymTable::new(),
    //         rtype: no_type(),
    //         has_returned: false,
    //     }
    // }

    pub fn new(
        src: &'a Source,
        pkg: PackageID,
        ctx: &'a mut TypeContext,
        config: &'a Config,
    ) -> Self {
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

    // /// Iterates over each node in the file and type checks. Populates
    // /// TypeContext with files types. Collects errors.
    // pub fn check(mut self) -> ErrorSet {
    //     let mut errs = ErrorSet::new();
    //     info!("file '{}'", self.file.src.filepath);

    //     for n in &self.file.ast.decls {
    //         let _ = self.eval(n).map_err(|e| errs.add(e));
    //     }

    //     if errs.len() > 0 {
    //         info!("fail, finished with {} errors", errs.len());
    //     }
    //     errs
    // }

    // /// Type check a Node. If it evaluates to a type it is internalized.
    // fn eval<N: Visitable + Node>(&mut self, node: &N) -> EvalResult {
    //     node.accept(self).map(|ty| {
    //         if ty != no_type() {
    //             // Has to be internalized here since Node methods are not
    //             // impemented for all node types, just their overarching
    //             // kind (stmt, decl etc).
    //             self.ctx.intern_node(node, ty);
    //         }
    //         ty
    //     })
    // }

    // /// Evaluate an option of a node. Defaults to void type if not present.
    // fn eval_optional<V: Visitable + Node>(&mut self, v: &Option<V>) -> Result<TypeId, Error> {
    //     v.as_ref().map_or(Ok(self.ctx.void()), |r| self.eval(r))
    // }

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

    /// Collect a list of type ids for each field in the slice.
    fn collect_field_types(&mut self, fields: &[Field]) -> Result<Vec<TypeId>, Error> {
        fields.iter().map(|f| self.eval_type(&f.typ)).collect()
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

    fn eval_type(&mut self, node: &TypeNode) -> Result<TypeId, Error> {
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

    /// Evaluate an option of a type node. Defaults to void type if not present.
    fn eval_optional_type(&mut self, v: &Option<TypeNode>) -> Result<TypeId, Error> {
        v.as_ref()
            .map_or(Ok(self.ctx.void()), |r| self.eval_type(r))
    }

    // ---------------------------- Generate AST ---------------------------- //

    pub fn emit_ast(&mut self, decls: Vec<ast::Decl>) -> Result<Vec<types::Decl>, ErrorSet> {
        let mut errs = ErrorSet::new();
        info!("file '{}'", self.src.filepath);

        let mut typed_decls = Vec::new();
        for n in decls {
            match self.emit_decl(n) {
                Ok(d) => typed_decls.push(d),
                Err(e) => errs.add(e),
            };
        }

        if errs.len() > 0 {
            info!("fail, finished with {} errors", errs.len());
            Err(errs)
        } else {
            Ok(typed_decls)
        }
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
            Expr::Literal(tok) => self.emit_literal(tok),
            Expr::Group(node) => self.emit_expr(*node.inner),
            Expr::Call(node) => self.emit_call(node),
        }
    }

    fn emit_func(&mut self, node: ast::FuncNode) -> Result<types::Decl, Error> {
        let meta = ast_node_to_meta(&node);

        // Evaluate return type if any
        let ret_id = self.eval_optional_type(&node.ret_type)?;

        // Get parameter types
        let param_ids = &node
            .params
            .iter()
            .map(|f| self.eval_type(&f.typ).map(|id| (&f.name, id)))
            .collect::<Result<Vec<_>, _>>()?;

        // If this is the main function we do additional checks
        if node.name.to_string() == "main" {
            let int_id = self.ctx.primitive(PrimitiveType::I64);

            // Must be package main
            if !self.pkg.0.is_empty() && self.pkg.0 != "main" {
                info!("package name expected to be main, is {}", self.pkg);
                return Err(self.error("main function can only be declared in main package", &node));
            }

            // If return type is not int
            if !self.ctx.equivalent(ret_id, int_id) {
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

        // Declare function while still in global scope
        let func_id = self.ctx.get_or_intern(TypeKind::Function(
            param_ids.iter().map(|v| v.1).collect(),
            ret_id,
        ));
        self.bind(&node.name, func_id, true)?;

        // Set up function body
        self.vars.push_scope();
        self.rtype = ret_id;
        self.has_returned = false;

        // Declare params in function body
        for p in param_ids {
            self.bind(p.0, p.1, false)?;
        }

        let body = node
            .body
            .stmts
            .into_iter()
            .map(|s| self.emit_stmt(s))
            .collect::<Result<Vec<types::Stmt>, Error>>()?;

        self.vars.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && ret_id != self.ctx.void() {
            return Err(self.error_token(
                format!("missing return in function '{}'", node.name.kind).as_str(),
                &node.body.rbrace,
            ));
        }

        Ok(types::Decl::Func(types::FuncNode {
            meta,
            name: node.name.to_string(),
            public: node.public,
            ty: self.ctx.lookup(func_id).clone(),
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
            TokenKind::IdentLit(name) => self
                .vars
                .get(name)
                .map_or(Err(self.error_token("not declared", &tok)), |t| {
                    Ok(self.ctx.lookup(t.ty))
                })?,
            _ => todo!(),
        };

        Ok(types::Expr::Literal(types::LiteralNode {
            meta: NodeMeta {
                id: tok.id,
                pos: tok.pos,
                end: tok.end_pos,
            },
            ty: ty.clone(),
            tok: tok.kind,
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
}

type EvalResult = Result<TypeId, Error>;

// impl<'a> Visitor<EvalResult> for Checker<'a> {
//     fn visit_func(&mut self, node: &FuncNode) -> EvalResult {
//         // Evaluate return type if any
//         let ret_type = self.eval_optional(&node.ret_type)?;

//         // Get parameter types
//         let params = &node
//             .params
//             .iter()
//             .map(|f| self.eval(&f.typ).map(|id| (&f.name, id)))
//             .collect::<Result<Vec<_>, _>>()?;

//         // If this is the main function we do additional checks
//         if node.name.to_string() == "main" {
//             let int_id = self.ctx.primitive(PrimitiveType::I64);

//             // Must be package main
//             if !self.file.package.is_empty() && self.file.package != "main" {
//                 info!("package name expected to be main, is {}", self.file.package);
//                 return Err(self.error("main function can only be declared in main package", node));
//             }

//             // If return type is not int
//             if !self.ctx.equivalent(ret_type, int_id) {
//                 let msg = "main function must return 'i64'";
//                 return Err(node
//                     .ret_type
//                     .as_ref()
//                     .map_or(self.error_token(msg, &node.rparen), |ty_node| {
//                         self.error(msg, ty_node)
//                     }));
//             }

//             // No parameters allowed
//             if params.len() > 0 {
//                 return Err(self.error("main function must not take any arguments", node));
//             }
//         }

//         // Declare function while still in global scope
//         let func_id = self.ctx.get_or_intern(TypeKind::Function(
//             params.iter().map(|v| v.1).collect(),
//             ret_type,
//         ));
//         self.bind(&node.name, func_id, true)?;

//         // Set up function body
//         self.vars.push_scope();
//         self.rtype = ret_type;
//         self.has_returned = false;

//         // Declare params in function body
//         for p in params {
//             self.bind(p.0, p.1, false)?;
//         }

//         if let Err(err) = self.visit_block(&node.body) {
//             return Err(err);
//         }

//         self.vars.pop_scope();

//         // There was no return when there should have been
//         if !self.has_returned && ret_type != self.ctx.void() {
//             return Err(self.error_token(
//                 format!("missing return in function '{}'", node.name.kind).as_str(),
//                 &node.body.rbrace,
//             ));
//         }

//         Ok(func_id)
//     }

//     fn visit_block(&mut self, node: &BlockNode) -> EvalResult {
//         for stmt in &node.stmts {
//             self.eval(stmt)?;
//         }
//         Ok(no_type())
//     }

//     fn visit_return(&mut self, node: &ReturnNode) -> EvalResult {
//         self.has_returned = true;

//         // If there is a return expression
//         // Evaluate it and compare with current scopes return type
//         if let Some(expr) = &node.expr {
//             let ty = self.eval(expr)?;

//             return if ty != self.rtype {
//                 Err(self.error_expected_got("incorrect return type", self.rtype, ty, expr))
//             } else {
//                 Ok(ty)
//             };
//         }

//         // If there is no return expression
//         // Check if current scope has no return type
//         if self.rtype != self.ctx.void() {
//             Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
//         } else {
//             Ok(self.ctx.void())
//         }
//     }

//     fn visit_literal(&mut self, node: &Token) -> EvalResult {
//         match &node.kind {
//             TokenKind::IntLit(_) => Ok(self.ctx.primitive(PrimitiveType::I64)),
//             TokenKind::FloatLit(_) => Ok(self.ctx.primitive(PrimitiveType::F64)),
//             TokenKind::StringLit(_) => Ok(self.ctx.primitive(PrimitiveType::String)),
//             TokenKind::True | TokenKind::False => Ok(self.ctx.primitive(PrimitiveType::Bool)),
//             TokenKind::IdentLit(name) => self
//                 .vars
//                 .get(name)
//                 .map_or(Err(self.error_token("not declared", node)), |t| Ok(t.ty)),
//             _ => todo!(),
//         }
//     }

//     fn visit_type(&mut self, node: &TypeNode) -> EvalResult {
//         match node {
//             TypeNode::Primitive(token) => {
//                 let prim = token_to_primitive_type(token);
//                 Ok(self.ctx.primitive(prim))
//             }
//             TypeNode::Ident(token) => self
//                 .ctx
//                 .get_symbol(token.to_string())
//                 .map_or(Err(self.error_token("not a type", token)), |ty| Ok(ty)),
//         }
//     }

//     fn visit_package(&mut self, node: &Token) -> EvalResult {
//         Err(self.error_token("package already declared earlier in file", node))
//     }

//     fn visit_call(&mut self, node: &CallExpr) -> EvalResult {
//         let callee_id = self.eval(node.callee.as_ref())?;
//         let callee_kind = self.ctx.lookup(callee_id).kind.clone();

//         if let TypeKind::Function(params, ret_id) = &callee_kind {
//             // Check if number of arguments matches
//             let (n_params, n_args) = (params.len(), node.args.len());
//             if n_params != n_args {
//                 let msg = format!("function takes {} arguments, got {}", n_params, n_args,);
//                 return Err(self
//                     .error_from_to(&msg, node.callee.pos(), &node.rparen.pos)
//                     .with_info(&format!("definition: {}", self.ctx.to_string(callee_id))));
//             }

//             // Check if each argument type matches the param type
//             for (i, param_id) in params.iter().enumerate() {
//                 let arg_id = self.eval(&node.args[i])?;
//                 if *param_id != arg_id {
//                     let msg = format!(
//                         "mismatched types in function call. expected '{}', got '{}'",
//                         self.ctx.to_string(*param_id),
//                         self.ctx.to_string(arg_id)
//                     );
//                     return Err(self.error(&msg, &node.args[i]));
//                 }
//             }

//             return Ok(*ret_id);
//         }

//         info!("callee type is actually: {:?}", callee_kind);
//         Err(self.error("not a function", &*node.callee))
//     }

//     fn visit_group(&mut self, node: &GroupExpr) -> EvalResult {
//         node.inner.accept(self)
//     }

//     fn visit_extern(&mut self, node: &crate::ast::FuncDeclNode) -> EvalResult {
//         let ret = self.eval_optional(&node.ret_type)?;
//         let params = self.collect_field_types(&node.params)?;
//         let id = self.ctx.get_or_intern(TypeKind::Function(params, ret));
//         self.bind(&node.name, id, true)
//     }

//     fn visit_var_decl(&mut self, node: &crate::ast::VarDeclNode) -> EvalResult {
//         let id = self.eval(&node.expr)?;
//         if id == self.ctx.void() {
//             return Err(self.error("cannot assign void type to variable", node));
//         }
//         self.bind(&node.name, id, node.constant)
//     }

//     fn visit_var_assign(&mut self, node: &crate::ast::VarAssignNode) -> EvalResult {
//         let lval_id = self.eval(&node.lval)?;
//         let rval_id = self.eval(&node.expr)?;

//         if lval_id != rval_id {
//             return Err(self.error(
//                 &format!(
//                     "mismatched types in assignment. expected '{}', got '{}'",
//                     self.ctx.to_string(lval_id),
//                     self.ctx.to_string(rval_id)
//                 ),
//                 &node.expr,
//             ));
//         }

//         if self.is_constant(&node.lval) {
//             return Err(self.error("cannot assign new value to a constant", &node.lval));
//         }

//         Ok(no_type())
//     }

//     fn visit_import(&mut self, node: &crate::ast::ImportNode) -> EvalResult {
//         todo!()
//     }
// }

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
