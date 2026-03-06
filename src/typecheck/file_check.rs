use tracing::info;

use crate::{
    ast::{self, Ast, Node, Pos, Token, TokenKind},
    context::Context,
    error::{Diagnostics, Report, Res},
    module::{NamespaceList, Symbol, SymbolList},
    types::{
        self, FunctionType, NO_TYPE, NodeMeta, PrimitiveType, Type, TypeId, TypeKind, TypedNode,
        ast_node_to_meta,
    },
    util::VarTable,
};

/// A Binding is either a declared variable or function parameter. Bindings
/// shadow global symbols like functions and types.
struct Binding {
    ty: TypeId,
    is_const: bool,
    pos: Pos,
}

/// Performs type checking on a single source file AST, producing a typed AST.
/// Borrows the module-level symbol table immutably and owns per-file state
/// (namespaces, variable bindings, return context).
pub(crate) struct FileChecker<'a> {
    ctx: &'a mut Context,
    /// Module-level symbols (read-only during file checking).
    symbols: &'a SymbolList,
    /// Per-file namespaces from import resolution.
    nsl: NamespaceList,
    /// Locally declared variables.
    vars: VarTable<Binding>,
    /// Return type in current function scope.
    rtype: TypeId,
    /// Whether the current function has returned.
    has_returned: bool,
    /// Whether we are in the main module.
    is_main: bool,
}

impl<'a> FileChecker<'a> {
    pub(crate) fn new(
        ctx: &'a mut Context,
        symbols: &'a SymbolList,
        nsl: NamespaceList,
        is_main: bool,
    ) -> Self {
        Self {
            ctx,
            symbols,
            nsl,
            vars: VarTable::new(),
            rtype: NO_TYPE,
            has_returned: false,
            is_main,
        }
    }

    /// Consume the file checker and return the namespace list for storage.
    pub(crate) fn into_namespaces(self) -> NamespaceList {
        self.nsl
    }

    pub(crate) fn emit_ast(&mut self, ast: Ast) -> Res<Vec<types::Decl>> {
        let mut diag = Diagnostics::new();

        let typed_decls = ast
            .decls
            .into_iter()
            .map(|d| self.emit_decl(d))
            .map(|s| s.map_err(|e| diag.add(e)))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if diag.num_errors() > 0 {
            info!("Fail: finished with {} errors", diag.num_errors());
            return Err(diag);
        }

        Ok(typed_decls)
    }

    fn emit_decl(&mut self, decl: ast::Decl) -> Result<types::Decl, Report> {
        match decl {
            ast::Decl::Func(node) => self.emit_func(node),
            ast::Decl::Extern(node) => self.emit_extern(node),
            _ => panic!("unexpected decl node in ast: {:?}", decl),
        }
    }

    fn emit_stmt(&mut self, stmt: ast::Stmt) -> Result<types::Stmt, Report> {
        match stmt {
            ast::Stmt::ExprStmt(node) => Ok(types::Stmt::ExprStmt(self.emit_expr(node)?)),
            ast::Stmt::Return(node) => self.emit_return(node),
            ast::Stmt::VarDecl(node) => self.emit_var_decl(node),
            ast::Stmt::VarAssign(node) => self.emit_var_assign(node),
            ast::Stmt::Block(_) => panic!("block should be handled manually as list of stmt"),
        }
    }

    fn emit_expr(&mut self, expr: ast::Expr) -> Result<types::Expr, Report> {
        match expr {
            ast::Expr::Literal(tok) => self.emit_literal(tok),
            ast::Expr::Group(node) => self.emit_expr(*node.inner),
            ast::Expr::Call(node) => self.emit_call(node),
            ast::Expr::Member(node) => self.emit_member(node),
        }
    }

    fn emit_func(&mut self, node: ast::FuncNode) -> Result<types::Decl, Report> {
        let meta = ast_node_to_meta(&node);

        // Get declared function
        let func_type = self.get_symbol_type(&node.name)?.clone(); // moved later
        let TypeKind::Function(f) = &func_type.kind else {
            panic!("function was not declared properly");
        };

        // If this is the main function we do additional checks
        if node.name.to_string() == "main" {
            self.check_main_function(f, &node)?;
        }

        // Set up function body
        self.vars.push_scope();
        self.rtype = f.ret;
        self.has_returned = false;

        // Declare params in function body
        for (i, ty) in f.params.iter().enumerate() {
            let name = &node.params[i].name;
            self.bind(name, *ty, false)?;
        }

        let body = node
            .body
            .stmts
            .into_iter()
            .map(|s| self.emit_stmt(s))
            .collect::<Result<Vec<types::Stmt>, Report>>()?;

        self.vars.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && f.ret != self.ctx.types.void() {
            return Err(self.error_token(
                &format!("missing return in function '{}'", node.name.kind),
                &node.body.rbrace,
            ));
        }

        Ok(types::Decl::Func(types::FuncNode {
            meta,
            name: node.name.to_string(),
            public: node.public,
            ty: func_type.id,
            params: node.params.iter().map(|p| p.name.to_string()).collect(),
            body,
        }))
    }

    fn check_main_function(&self, f: &FunctionType, node: &ast::FuncNode) -> Result<(), Report> {
        // Must be main module
        if !self.is_main {
            return Err(self.error("main function can only be declared in main module", node));
        }

        // If return type is not int
        let return_type = self.ctx.types.primitive(PrimitiveType::I64);

        if !self.ctx.types.equivalent(f.ret, return_type) {
            let msg = format!(
                "main function must return '{}'",
                self.ctx.types.type_to_string(return_type)
            );
            return Err(node
                .ret_type
                .as_ref()
                .map_or(self.error_token(&msg, &node.rparen), |ty_node| {
                    self.error(&msg, ty_node)
                }));
        }

        // No parameters allowed
        if f.params.len() > 0 {
            return Err(self.error("main function must not take any arguments", node));
        }

        Ok(())
    }

    fn emit_extern(&mut self, node: ast::FuncDeclNode) -> Result<types::Decl, Report> {
        let meta = ast_node_to_meta(&node);
        let name = node.name.to_string();
        let sym = self
            .get_symbol(&name)
            .expect("should have been declared in global pass");

        Ok(types::Decl::Extern(types::ExternNode {
            ty: sym.ty,
            meta,
            name,
        }))
    }

    fn emit_var_assign(&mut self, node: ast::VarAssignNode) -> Result<types::Stmt, Report> {
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
                    self.ctx.types.type_to_string(lval.type_id()),
                    self.ctx.types.type_to_string(rval.type_id())
                ),
                &rval,
            ));
        }

        Ok(types::Stmt::VarAssign(types::VarAssignNode {
            meta,
            ty: rval.type_id(),
            lval,
            rval,
        }))
    }

    fn emit_var_decl(&mut self, node: ast::VarDeclNode) -> Result<types::Stmt, Report> {
        let meta = ast_node_to_meta(&node);
        let typed_expr = self.emit_expr(node.expr)?;

        if typed_expr.type_id() == self.ctx.types.void() {
            return Err(self.error("cannot assign void type to variable", &typed_expr));
        }

        if self.nsl.get(&node.name.to_string()).is_ok() {
            return Err(self.error_token("shadowing a namespace is not allowed", &node.name));
        }

        let ty = self.bind(&node.name, typed_expr.type_id(), node.constant)?;
        Ok(types::Stmt::VarDecl(types::VarDeclNode {
            meta,
            ty,
            name: node.name.to_string(),
            value: typed_expr,
        }))
    }

    fn emit_return(&mut self, node: ast::ReturnNode) -> Result<types::Stmt, Report> {
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
                    ty: typed_expr.type_id(),
                    expr: Some(typed_expr),
                }))
            };
        }

        // If there is no return expression
        // Check if current scope has no return type
        if self.rtype != self.ctx.types.void() {
            Err(self.error_expected_token("incorrect return type", self.rtype, &node.kw))
        } else {
            Ok(types::Stmt::Return(types::ReturnNode {
                meta,
                expr: None,
                ty: self.ctx.types.void(),
            }))
        }
    }

    fn emit_literal(&mut self, tok: Token) -> Result<types::Expr, Report> {
        let ty = match &tok.kind {
            TokenKind::IntLit(_) => self.ctx.types.primitive_type(PrimitiveType::I64),
            TokenKind::FloatLit(_) => self.ctx.types.primitive_type(PrimitiveType::F64),
            TokenKind::StringLit(_) => self.ctx.types.primitive_type(PrimitiveType::String),
            TokenKind::True | TokenKind::False => {
                self.ctx.types.primitive_type(PrimitiveType::Bool)
            }
            TokenKind::IdentLit(name) => {
                let ty_id = match self.get(&tok) {
                    Err(err) => {
                        if self.nsl.get(name).is_ok() {
                            return Err(
                                self.error_token("namespace cannot be used as a value", &tok)
                            );
                        }
                        return Err(err);
                    }
                    Ok(id) => id,
                };
                self.ctx.types.lookup(ty_id)
            }
            _ => todo!(),
        };

        // TODO: token to type??

        Ok(types::Expr::Literal(types::LiteralNode {
            meta: NodeMeta {
                id: tok.id,
                pos: tok.pos,
                end: tok.end_pos,
            },
            ty: ty.id,
            kind: tok.kind.into(),
        }))
    }

    fn emit_call(&mut self, node: ast::CallExpr) -> Result<types::Expr, Report> {
        let meta = ast_node_to_meta(&node);
        let callee = self.emit_expr(*node.callee)?;

        let (params, ret) = match self.ctx.types.try_function(callee.type_id()) {
            Some(f) => (f.params.clone(), f.ret), // Copy to not use mut ref later
            None => return Err(self.error("not a function", &callee)),
        };

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
                    self.ctx.types.type_to_string(callee.type_id())
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
                    self.ctx.types.type_to_string(param_id),
                    self.ctx.types.type_to_string(arg_id)
                );
                return Err(self.error(&msg, &typed_arg));
            }

            args.push(typed_arg);
        }

        Ok(types::Expr::Call(types::CallNode {
            meta,
            ty: ret,
            callee: Box::new(callee),
            args,
        }))
    }

    fn emit_member(&mut self, node: ast::MemberNode) -> Result<types::Expr, Report> {
        let meta = ast_node_to_meta(&node);
        let field = node.field.to_string();

        // First check if the left hand value is a namespace
        if let Some(name) = self.if_identifier_get_name(&*node.expr) {
            if let Ok(ns) = self.nsl.get(name) {
                // Get symbol from field
                let Some(id) = ns.get(&field) else {
                    return Err(self.error_token(
                        &format!("namespace '{}' has no member '{}'", ns.name(), &field),
                        &node.field,
                    ));
                };

                let symbol = self.ctx.symbols.get(id);

                return Ok(types::Expr::NamespaceMember(types::NamespaceMemberNode {
                    ty: symbol.ty,
                    name: name.to_owned(),
                    meta,
                    field,
                }));
            }
        }

        // Otherwise this is a normal member getter and we treat lval as
        // a normal expression.
        let expr = self.emit_expr(*node.expr)?;

        Err(self.error(
            &format!(
                "type '{}' has no fields",
                self.ctx.types.type_to_string(expr.type_id())
            ),
            &expr,
        ))
    }

    // ----------------------- Utility methods ----------------------- //

    fn error(&self, msg: &str, node: &dyn Node) -> Report {
        Report::code_error(msg, node.pos(), node.end())
    }

    fn error_token(&self, msg: &str, tok: &Token) -> Report {
        Report::code_error(msg, &tok.pos, &tok.end_pos)
    }

    fn error_from_to(&self, msg: &str, from: &Pos, to: &Pos) -> Report {
        Report::code_error(msg, from, to)
    }

    fn error_expected_token(&self, msg: &str, expect: TypeId, tok: &Token) -> Report {
        self.error_token(
            &format!(
                "{}: expected '{}'",
                msg,
                self.ctx.types.type_to_string(expect),
            ),
            tok,
        )
    }

    fn error_expected_got(
        &self,
        msg: &str,
        expect: TypeId,
        got: TypeId,
        node: &dyn Node,
    ) -> Report {
        self.error(
            &format!(
                "{}: expected '{}', got '{}'",
                msg,
                self.ctx.types.type_to_string(expect),
                self.ctx.types.type_to_string(got)
            ),
            node,
        )
    }

    /// Bind a name (token) to a type. Returns same type id or error if already defined.
    fn bind(&mut self, name: &Token, id: TypeId, constant: bool) -> Result<TypeId, Report> {
        if !self.vars.bind(
            name.to_string(),
            Binding {
                ty: id,
                is_const: constant,
                pos: name.pos.clone(),
            },
        ) {
            Err(self
                .error_token("already declared", name)
                .with_info(&format!(
                    "previously declared on line {}", // always local to this file
                    self.vars.get(&name.to_string()).unwrap().pos.row + 1
                )))
        } else {
            Ok(id)
        }
    }

    /// Get a declared symbol by a token identifier. Returns "not declared" error if not found.
    fn get(&self, name: &Token) -> Result<TypeId, Report> {
        let name_str = name.to_string();
        if let Some(var) = self.vars.get(&name_str) {
            return Ok(var.ty);
        }
        if let Ok(sym) = self.get_symbol(&name_str) {
            return Ok(sym.ty);
        }
        Err(self.error_token("not declared", name))
    }

    /// Get the type of a declared symbol
    fn get_symbol_type(&self, name: &Token) -> Result<&Type, Report> {
        let id = self.get(name)?;
        Ok(self.ctx.types.lookup(id))
    }

    /// Report whether the given l-value is constant or not.
    fn is_constant(&self, lval: &ast::Expr) -> bool {
        match lval {
            ast::Expr::Literal(token) => match &token.kind {
                TokenKind::IdentLit(name) => self.vars.get(name).map_or(false, |sym| sym.is_const),
                _ => false,
            },
            ast::Expr::Group(_) | ast::Expr::Call(_) => true,
            ast::Expr::Member(node) => self.is_constant(&node.expr),
        }
    }

    /// If the given expression is a Token::Ident kind, it returns the identifier name.
    fn if_identifier_get_name<'b>(&self, expr: &'b ast::Expr) -> Option<&'b str> {
        if let ast::Expr::Literal(token) = expr {
            if let TokenKind::IdentLit(name) = &token.kind {
                return Some(name);
            }
        }
        None
    }

    /// Get a Symbol by name. The name is local to this module only.
    /// Returns "not declared" on error.
    fn get_symbol(&self, name: &str) -> Result<&Symbol, String> {
        self.symbols
            .get(name)
            .map_or(Err("not declared".to_string()), |sym| {
                Ok(self.ctx.symbols.get(sym.id))
            })
    }
}
