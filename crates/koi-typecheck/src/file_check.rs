use tracing::info;

use koi_ast::{self as ast, Ast, Node, Pos, Token, TokenKind};
use koi_common::error::{Diagnostics, Report, Res};
use koi_common::util::VarTable;
use koi_sema::{
    self as types, BinaryOp, CastKind, Context, FunctionType, LiteralKind, NO_TYPE, NamespaceList,
    NodeMeta, PrimitiveType, Symbol, SymbolKind, SymbolList, Type, TypeId, TypeKind, TypedNode,
    UnaryOp, ast_node_to_meta,
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
    /// Currently checking a loop body? Used for break/continue checks.
    in_loop: bool,
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
            in_loop: false,
        }
    }

    /// Consume the file checker and return the namespace list for storage.
    pub(crate) fn into_namespaces(self) -> NamespaceList {
        self.nsl
    }

    pub(crate) fn emit_ast(&mut self, ast: Ast) -> Res<Vec<types::Decl>> {
        let mut diag = Diagnostics::new();

        let typed_decls = self
            .emit_decls(ast)
            .into_iter()
            .map(|s| s.map_err(|e| diag.add(e)))
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        if diag.num_errors() > 0 {
            info!("Fail: finished with {} errors", diag.num_errors());
            return Err(diag);
        }

        Ok(typed_decls)
    }

    fn emit_decls(&mut self, ast: Ast) -> Vec<Result<types::Decl, Report>> {
        let mut decls = Vec::new();
        for d in ast.decls {
            match d {
                ast::Decl::Func(node) => decls.push(self.emit_func(*node)),
                ast::Decl::Extern(node) => decls.push(self.emit_extern(*node)),
                ast::Decl::Type(..) => {} // Declared in global pass
            };
        }
        decls
    }

    fn emit_stmt(&mut self, stmt: ast::Stmt) -> Result<types::Stmt, Report> {
        match stmt {
            ast::Stmt::ExprStmt(node) => Ok(types::Stmt::ExprStmt(self.emit_expr(node)?)),
            ast::Stmt::Return(node) => self.emit_return(node),
            ast::Stmt::VarDecl(node) => self.emit_var_decl(node),
            ast::Stmt::VarAssign(node) => self.emit_var_assign(node),
            ast::Stmt::While(node) => self.emit_while(node),
            ast::Stmt::For(node) => self.emit_for(node),
            ast::Stmt::If(node) => Ok(types::Stmt::If(self.emit_if(node)?)),
            ast::Stmt::Block(_) => panic!("block should be handled manually as list of stmt"),
            ast::Stmt::Break(node) => {
                if !self.in_loop {
                    return Err(self.error("break cannot be used outside a loop", &node));
                }
                let meta = ast_node_to_meta(&node);
                Ok(types::Stmt::Break(types::BreakNode { meta }))
            }
            ast::Stmt::Continue(node) => {
                if !self.in_loop {
                    return Err(self.error("continue cannot be used outside a loop", &node));
                }
                let meta = ast_node_to_meta(&node);
                Ok(types::Stmt::Continue(types::ContinueNode { meta }))
            }
            ast::Stmt::OpAssign(node) => self.emit_op_assign(node),
        }
    }

    fn emit_expr(&mut self, expr: ast::Expr) -> Result<types::Expr, Report> {
        match expr {
            ast::Expr::Literal(tok) => self.emit_literal(tok),
            ast::Expr::Group(node) => self.emit_expr(*node.inner),
            ast::Expr::Call(node) => self.emit_call(node),
            ast::Expr::Member(node) => self.emit_member(node),
            ast::Expr::Binary(node) => self.emit_binary(node),
            ast::Expr::Unary(node) => self.emit_unary(node),
            ast::Expr::Cast(node) => self.emit_cast(node),
        }
    }

    fn emit_func(&mut self, node: ast::FuncNode) -> Result<types::Decl, Report> {
        let meta = ast_node_to_meta(&node);
        self.vars.clear(); // Make sure table is clean

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

        let body = self.emit_block(node.body)?;
        self.vars.pop_scope();

        // There was no return when there should have been
        if !self.has_returned && f.ret != self.ctx.types.void() {
            return Err(self.error_token(
                &format!("missing return in function '{}'", node.name.kind),
                &node.name,
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
        let return_type = self.ctx.types.primitive(PrimitiveType::I32);

        if !self.ctx.types.equivalent(f.ret, return_type) {
            let msg = format!(
                "main function must return '{}', got '{}'",
                self.ctx.types.type_to_string(return_type),
                self.ctx.types.type_to_string(f.ret),
            );
            return Err(node
                .ret_type
                .as_ref()
                .map_or(self.error_token(&msg, &node.rparen), |ty_node| {
                    self.error(&msg, ty_node)
                }));
        }

        // No parameters allowed
        if !f.params.is_empty() {
            return Err(self.error("main function must not take any arguments", node));
        }

        Ok(())
    }

    fn emit_extern(&mut self, node: ast::FuncDeclNode) -> Result<types::Decl, Report> {
        let meta = ast_node_to_meta(&node);

        // If symbol has an alias, get in before fetching symbol
        let real_name = node.name.to_string();
        let name = if let Some(alias) = self.symbols.get_alias(&real_name) {
            alias.clone()
        } else {
            real_name
        };

        let sym = self
            .get_symbol(&name)
            .expect("should have been declared in global pass");

        Ok(types::Decl::Extern(types::ExternNode {
            ty: sym.ty,
            meta,
            name,
        }))
    }

    fn emit_block(&mut self, node: ast::BlockNode) -> Result<types::BlockNode, Report> {
        self.vars.push_scope();
        let stmts = node
            .stmts
            .into_iter()
            .map(|s| self.emit_stmt(s))
            .collect::<Result<Vec<types::Stmt>, Report>>()?;

        self.vars.pop_scope();
        Ok(types::BlockNode { stmts })
    }

    fn emit_for(&mut self, node: ast::ForNode) -> Result<types::Stmt, Report> {
        let meta = ast_node_to_meta(&node);

        let initializer = Box::new(self.emit_stmt(*node.initializer)?);
        let condition = Box::new(self.emit_expr(*node.condition)?);
        self.assert_expr_is_type(PrimitiveType::Bool, &condition)?;

        let increment = Box::new(self.emit_stmt(*node.increment)?);
        let block = self.emit_loop_block(node.block)?;

        Ok(types::Stmt::For(types::ForNode {
            meta,
            initializer,
            condition,
            increment,
            block,
        }))
    }

    fn emit_while(&mut self, node: ast::WhileNode) -> Result<types::Stmt, Report> {
        let meta = ast_node_to_meta(&node);
        let expr = self.emit_expr(node.expr)?;
        self.assert_expr_is_type(PrimitiveType::Bool, &expr)?;
        let block = self.emit_loop_block(node.block)?;

        Ok(types::Stmt::While(types::WhileNode { meta, expr, block }))
    }

    /// Emit block node while preserving return status and setting the loop flag.
    fn emit_loop_block(&mut self, block: ast::BlockNode) -> Result<types::BlockNode, Report> {
        let prev_in_loop = self.in_loop;
        let has_returned = self.has_returned;
        self.in_loop = true;
        let block = self.emit_block(block)?;
        self.in_loop = prev_in_loop;
        self.has_returned = has_returned;
        Ok(block)
    }

    fn emit_op_assign(&mut self, node: ast::OpAssignNode) -> Result<types::Stmt, Report> {
        let meta = ast_node_to_meta(&node);

        if self.is_constant(&node.lval) {
            return Err(self.error("cannot assign new value to a constant", &node.lval));
        }

        let lval = self.emit_expr(node.lval)?;
        let rval = self.emit_expr(node.rval)?;

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

        if !self.ctx.types.is_number(lval.type_id()) {
            let op_str = match node.op.kind {
                TokenKind::PlusEq => "+=",
                TokenKind::MinusEq => "-=",
                TokenKind::StarEq => "*=",
                TokenKind::SlashEq => "/=",
                _ => unreachable!(),
            };
            return Err(self.error(
                &format!(
                    "operator '{}' cannot be used on type '{}'",
                    op_str,
                    self.ctx.types.type_to_string(lval.type_id()),
                ),
                &rval,
            ));
        }

        let op = match node.op.kind {
            TokenKind::PlusEq => types::AssignOp::Plus,
            TokenKind::MinusEq => types::AssignOp::Minus,
            TokenKind::StarEq => types::AssignOp::Mult,
            TokenKind::SlashEq => types::AssignOp::Div,
            _ => unreachable!(),
        };

        Ok(types::Stmt::OpAssign(types::OpAssignNode {
            meta,
            ty: rval.type_id(),
            lval: Box::new(lval),
            rval: Box::new(rval),
            op,
        }))
    }

    fn emit_if(&mut self, node: ast::IfNode) -> Result<types::IfNode, Report> {
        let meta = ast_node_to_meta(&node);

        let expr = self.emit_expr(node.expr)?;
        self.assert_expr_is_type(PrimitiveType::Bool, &expr)?;

        let block = self.emit_block(node.block)?;
        let this_returned = self.has_returned;

        let (elseif, exhaustive_return) = match *node.elseif {
            ast::ElseBlock::ElseIf(node) => {
                self.has_returned = false;
                let block = self.emit_if(*node)?;
                (
                    Box::new(types::ElseBlock::ElseIf(Box::new(block))),
                    self.has_returned,
                )
            }
            ast::ElseBlock::Else(node) => {
                self.has_returned = false;
                let block = self.emit_block(*node)?;
                (
                    Box::new(types::ElseBlock::Else(Box::new(block))),
                    self.has_returned,
                )
            }
            ast::ElseBlock::None => (Box::new(types::ElseBlock::None), false),
        };

        // If this if-block and all subsequent else-if and else blocks return,
        // then we can mark the function as having returned.
        self.has_returned = this_returned && exhaustive_return;

        Ok(types::IfNode {
            meta,
            expr,
            block,
            elseif,
        })
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
        let name = node.name.to_string();

        if typed_expr.type_id() == self.ctx.types.void() {
            return Err(self.error("cannot assign void type to variable", &typed_expr));
        }

        if let Ok(sym) = self.get_symbol(name.as_str()) {
            match sym.kind {
                SymbolKind::Function { .. } => {} // shadowing a function is ok
                SymbolKind::Type => {
                    return Err(self.error_token("shadowing a type is not allowed", &node.name));
                }
            }
        }

        if self.nsl.get(&name).is_ok() {
            return Err(self.error_token("shadowing a namespace is not allowed", &node.name));
        }

        let ty = self.bind(&node.name, typed_expr.type_id(), node.constant)?;
        Ok(types::Stmt::VarDecl(types::VarDeclNode {
            meta,
            ty,
            name,
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

    fn emit_cast(&mut self, node: ast::CastExpr) -> Result<types::Expr, Report> {
        let meta = ast_node_to_meta(&node);
        let expr = self.emit_expr(*node.expr)?;
        let ty = self.eval_type(&node.ty)?;

        let cast_kind = self.check_if_can_cast(&expr, ty);
        if matches!(cast_kind, CastKind::InvalidCast) {
            return Err(self.error_from_to("invalid cast", &meta.pos, &meta.end));
        }

        self.check_const_bounds(&expr, ty, &meta)?;

        Ok(types::Expr::Cast(types::CastNode {
            meta,
            expr: Box::new(expr),
            ty,
            cast_kind,
        }))
    }

    fn check_if_can_cast(&self, expr: &types::Expr, ty: TypeId) -> CastKind {
        if self.ctx.types.equivalent(expr.type_id(), ty) {
            return CastKind::Identity;
        }

        let from_id = self.ctx.types.inner_kind(expr.type_id());
        let to_id = self.ctx.types.inner_kind(ty);
        let from = &self.ctx.types.get(from_id).unwrap().kind;
        let to = &self.ctx.types.get(to_id).unwrap().kind;

        match (from, to) {
            (TypeKind::Primitive(from), TypeKind::Primitive(to)) => {
                let from_int = from.is_int() || from.is_uint();
                let to_int = to.is_int() || to.is_uint();
                if from_int && to_int {
                    if from.bytes() >= to.bytes() {
                        CastKind::IntegerNarrowing
                    } else {
                        CastKind::IntegerWidening
                    }
                } else if from.is_float() && to.is_float() {
                    if from.bytes() >= to.bytes() {
                        CastKind::FloatNarrowing
                    } else {
                        CastKind::FloatWidening
                    }
                } else if from_int && to.is_float() {
                    CastKind::IntToFloat
                } else if from.is_float() && to_int {
                    CastKind::FloatToInt
                } else {
                    CastKind::InvalidCast
                }
            }
            _ => CastKind::InvalidCast,
        }
    }

    fn emit_literal(&mut self, tok: Token) -> Result<types::Expr, Report> {
        let ty = match &tok.kind {
            TokenKind::IntLit(_) => self.ctx.types.primitive_type(PrimitiveType::I32),
            TokenKind::FloatLit(_) => self.ctx.types.primitive_type(PrimitiveType::F32),
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

    fn emit_unary(&mut self, node: ast::UnaryExpr) -> Result<types::Expr, Report> {
        let meta = ast_node_to_meta(&node);
        let rhs = self.emit_expr(*node.rhs)?;

        let op = match node.op.kind {
            TokenKind::Not => UnaryOp::LogicNot,
            TokenKind::Minus => UnaryOp::Minus,
            _ => unreachable!(),
        };

        match op {
            UnaryOp::LogicNot => {
                let bool_t = self.ctx.types.primitive(PrimitiveType::Bool);
                if !self.ctx.types.equivalent(rhs.type_id(), bool_t) {
                    return Err(self.error(
                        &format!(
                            "'!' operator can only be used on type 'bool', got '{}'",
                            self.type_to_string(&rhs)
                        ),
                        &rhs,
                    ));
                }
                Ok(types::Expr::Unary(types::UnaryNode {
                    ty: bool_t,
                    meta,
                    op,
                    rhs: Box::new(rhs),
                }))
            }
            UnaryOp::Minus => {
                if !self.ctx.types.is_number(rhs.type_id()) {
                    return Err(self.error(
                        &format!(
                            "'-' operator can only be used on number types, got '{}'",
                            self.type_to_string(&rhs)
                        ),
                        &rhs,
                    ));
                }
                Ok(types::Expr::Unary(types::UnaryNode {
                    meta,
                    op,
                    ty: rhs.type_id(),
                    rhs: Box::new(rhs),
                }))
            }
        }
    }

    fn emit_binary(&mut self, node: ast::BinaryExpr) -> Result<types::Expr, Report> {
        let meta = ast_node_to_meta(&node);

        let lhs = self.emit_expr(*node.lhs)?;
        let rhs = self.emit_expr(*node.rhs)?;

        if lhs.type_id() != rhs.type_id() {
            return Err(self.error_from_to(
                &format!(
                    "mismatched types in expression: '{}' and '{}'",
                    self.type_to_string(&lhs),
                    self.type_to_string(&rhs),
                ),
                &meta.pos,
                &meta.end,
            ));
        }

        let op: BinaryOp = node.op.kind.into();

        self.check_binary_op_type(&op, lhs.type_id(), &meta)?;

        let ty = match op {
            BinaryOp::Plus | BinaryOp::Minus | BinaryOp::Mult | BinaryOp::Divide => lhs.type_id(),
            BinaryOp::Modulo => self.ctx.types.primitive(PrimitiveType::U32),
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Greater
            | BinaryOp::GreaterEq
            | BinaryOp::Less
            | BinaryOp::LessEq
            | BinaryOp::LogicAnd
            | BinaryOp::LogicOr => self.ctx.types.primitive(PrimitiveType::Bool),
        };

        Ok(types::Expr::Binary(types::BinaryNode {
            ty,
            meta,
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
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
        if let Some(name) = self.if_identifier_get_name(&node.expr)
            && let Ok(ns) = self.nsl.get(name)
        {
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

    fn check_binary_op_type(
        &self,
        op: &BinaryOp,
        ty: TypeId,
        meta: &NodeMeta,
    ) -> Result<(), Report> {
        let bool_t = self.ctx.types.primitive(PrimitiveType::Bool);
        let is_num = self.ctx.types.is_number(ty);
        let is_bool = self.ctx.types.equivalent(ty, bool_t);

        let (valid, op_str) = match op {
            BinaryOp::Plus => (is_num, "+"),
            BinaryOp::Minus => (is_num, "-"),
            BinaryOp::Mult => (is_num, "*"),
            BinaryOp::Divide => (is_num, "/"),
            BinaryOp::Modulo => {
                let resolved = self.ctx.types.inner_kind(ty);
                let is_int = matches!(
                    self.ctx.types.lookup(resolved).kind,
                    TypeKind::Primitive(ref p) if p.is_int() || p.is_uint()
                );
                (is_int, "%")
            }
            BinaryOp::Equal => (is_num || is_bool, "=="),
            BinaryOp::NotEqual => (is_num || is_bool, "!="),
            BinaryOp::Greater => (is_num, ">"),
            BinaryOp::GreaterEq => (is_num, ">="),
            BinaryOp::Less => (is_num, "<"),
            BinaryOp::LessEq => (is_num, "<="),
            BinaryOp::LogicAnd => (is_bool, "&&"),
            BinaryOp::LogicOr => (is_bool, "||"),
        };

        if !valid {
            return Err(self.error_from_to(
                &format!(
                    "operator '{}' cannot be used on type '{}'",
                    op_str,
                    self.ctx.types.type_to_string(ty),
                ),
                &meta.pos,
                &meta.end,
            ));
        }

        Ok(())
    }

    /// Assert that the given expression is the given primitive type.
    fn assert_expr_is_type(&self, expect: PrimitiveType, expr: &types::Expr) -> Result<(), Report> {
        let expect_t = self.ctx.types.primitive(expect);
        if !self.ctx.types.equivalent(expr.type_id(), expect_t) {
            return Err(self.error(
                &format!(
                    "expression must be of type '{}', got '{}'",
                    self.ctx.types.type_to_string(expect_t),
                    self.type_to_string(expr)
                ),
                expr,
            ));
        }
        Ok(())
    }

    fn type_to_string(&self, node: &dyn TypedNode) -> String {
        self.ctx.types.type_to_string(node.type_id())
    }

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
            return match sym.kind {
                SymbolKind::Function { .. } => Ok(sym.ty),
                SymbolKind::Type => Err(self.error_token("a type cannot be used as a value", name)),
            };
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
                TokenKind::IdentLit(name) => self.vars.get(name).is_some_and(|sym| sym.is_const),
                _ => false,
            },
            ast::Expr::Member(node) => self.is_constant(&node.expr),
            ast::Expr::Group(_)
            | ast::Expr::Call(_)
            | ast::Expr::Binary(_)
            | ast::Expr::Unary(_)
            | ast::Expr::Cast(_) => true,
        }
    }

    /// If the given expression is a Token::Ident kind, it returns the identifier name.
    fn if_identifier_get_name<'b>(&self, expr: &'b ast::Expr) -> Option<&'b str> {
        if let ast::Expr::Literal(token) = expr
            && let TokenKind::IdentLit(name) = &token.kind
        {
            return Some(name);
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

    fn check_const_bounds(
        &self,
        expr: &types::Expr,
        ty: TypeId,
        meta: &NodeMeta,
    ) -> Result<(), Report> {
        let to_id = self.ctx.types.inner_kind(ty);
        let TypeKind::Primitive(to) = &self.ctx.types.get(to_id).unwrap().kind else {
            return Ok(());
        };

        let overflows = match try_const_value(expr) {
            Some(ConstVal::Int(n)) => !lit_int_fits(n, to),
            Some(ConstVal::Uint(n)) => !lit_uint_fits(n, to),
            Some(ConstVal::Float(n)) => !lit_float_fits(n, to),
            None => false,
        };

        if overflows {
            let type_name = self.ctx.types.type_to_string(ty);
            Err(self.error_from_to(
                &format!("constant value overflows target type '{type_name}'"),
                &meta.pos,
                &meta.end,
            ))
        } else {
            Ok(())
        }
    }

    /// Evaluate an AST type node to its semantic type id.
    fn eval_type(&self, node: &ast::TypeNode) -> Result<TypeId, Report> {
        match node {
            ast::TypeNode::Ident(token) => self
                .get_symbol_type_id(token)
                .ok_or(self.error_token("not a type", token)),
            ast::TypeNode::Imported { namespace, ty } => {
                let ns = self.nsl.get(&namespace.to_string()).map_or(
                    Err(self.error_token("not an imported namespace", namespace)),
                    Ok,
                )?;

                let sym_id = ns.get(&ty.to_string()).ok_or(
                    self.error_token(&format!("namespace '{namespace}' has no member '{ty}'"), ty),
                )?;

                Ok(self.ctx.symbols.get(sym_id).ty)
            }
        }
    }

    fn get_symbol_type_id(&self, name: &ast::Token) -> Option<TypeId> {
        let name_str = name.to_string();
        self.get_symbol(&name_str).ok().map(|sym| sym.ty)
    }
}

enum ConstVal {
    Int(i64),
    Uint(u64),
    Float(f64),
}

fn try_const_value(expr: &types::Expr) -> Option<ConstVal> {
    match expr {
        types::Expr::Literal(lit) => match &lit.kind {
            LiteralKind::Int(n) => Some(ConstVal::Int(*n)),
            LiteralKind::Uint(n) => Some(ConstVal::Uint(*n)),
            LiteralKind::Float(n) => Some(ConstVal::Float(*n)),
            _ => None,
        },
        types::Expr::Unary(unary) if matches!(unary.op, UnaryOp::Minus) => {
            match try_const_value(&unary.rhs)? {
                ConstVal::Int(n) => Some(ConstVal::Int(n.wrapping_neg())),
                ConstVal::Float(n) => Some(ConstVal::Float(-n)),
                ConstVal::Uint(n) => Some(ConstVal::Int(-(n as i64))),
            }
        }
        _ => None,
    }
}

fn lit_int_fits(n: i64, to: &PrimitiveType) -> bool {
    match to {
        PrimitiveType::I8 => (i8::MIN as i64..=i8::MAX as i64).contains(&n),
        PrimitiveType::I16 => (i16::MIN as i64..=i16::MAX as i64).contains(&n),
        PrimitiveType::I32 => (i32::MIN as i64..=i32::MAX as i64).contains(&n),
        PrimitiveType::I64 => true,
        PrimitiveType::U8 => (0..=u8::MAX as i64).contains(&n),
        PrimitiveType::U16 => (0..=u16::MAX as i64).contains(&n),
        PrimitiveType::U32 => (0..=u32::MAX as i64).contains(&n),
        PrimitiveType::U64 => n >= 0,
        _ => true,
    }
}

fn lit_uint_fits(n: u64, to: &PrimitiveType) -> bool {
    match to {
        PrimitiveType::I8 => n <= i8::MAX as u64,
        PrimitiveType::I16 => n <= i16::MAX as u64,
        PrimitiveType::I32 => n <= i32::MAX as u64,
        PrimitiveType::I64 => n <= i64::MAX as u64,
        PrimitiveType::U8 => n <= u8::MAX as u64,
        PrimitiveType::U16 => n <= u16::MAX as u64,
        PrimitiveType::U32 => n <= u32::MAX as u64,
        PrimitiveType::U64 => true,
        _ => true,
    }
}

fn lit_float_fits(n: f64, to: &PrimitiveType) -> bool {
    match to {
        PrimitiveType::F32 => n >= f32::MIN as f64 && n <= f32::MAX as f64,
        PrimitiveType::I8 => n >= i8::MIN as f64 && n <= i8::MAX as f64,
        PrimitiveType::I16 => n >= i16::MIN as f64 && n <= i16::MAX as f64,
        PrimitiveType::I32 => n >= i32::MIN as f64 && n <= i32::MAX as f64,
        PrimitiveType::I64 => n >= i64::MIN as f64 && n <= i64::MAX as f64,
        PrimitiveType::U8 => n >= 0.0 && n <= u8::MAX as f64,
        PrimitiveType::U16 => n >= 0.0 && n <= u16::MAX as f64,
        PrimitiveType::U32 => n >= 0.0 && n <= u32::MAX as f64,
        PrimitiveType::U64 => n >= 0.0 && n <= u64::MAX as f64,
        _ => true,
    }
}
