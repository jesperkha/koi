use std::collections::HashSet;

use tracing::info;

use crate::{
    ast::{
        Ast, BlockNode, CallExpr, Decl, Expr, Field, File, FuncDeclNode, FuncNode, GroupExpr,
        ImportNode, MemberNode, Node, PackageNode, ReturnNode, Stmt, TypeNode, VarAssignNode,
        VarDeclNode,
    },
    config::Config,
    error::{Error, ErrorSet, Res},
    token::{Source, Token, TokenKind},
};

pub fn parse(src: Source, tokens: Vec<Token>, config: &Config) -> Res<File> {
    let parser = Parser::new(src, tokens, config);
    parser.parse()
}

struct Parser<'a> {
    errs: ErrorSet,
    tokens: Vec<Token>,
    pos: usize,
    config: &'a Config,
    src: Source,

    // Panic mode occurs when the parser encounters an unknown token sequence
    // and needs to synchronize to a 'clean' state. When panic mode starts,
    // the base position is set to the current position. When in panic mode
    // all err() calls are ignored.
    //
    // Functions which parse statements should have a check at the top for
    // panicMode, and return early with an invalid statement if set.
    panic_mode: bool,
}

impl<'a> Parser<'a> {
    fn new(src: Source, tokens: Vec<Token>, config: &'a Config) -> Self {
        Self {
            errs: ErrorSet::new(),
            tokens,
            pos: 0,
            panic_mode: false,
            config,
            src,
        }
    }

    fn parse(mut self) -> Res<File> {
        info!("file '{}'", self.src.filepath);

        // Parse package declaration as it must come first
        let package = if !self.config.anon_packages {
            self.skip_whitespace_and_not_eof();
            self.parse_package_decl()
                .map_err(|err| ErrorSet::new_from(err))?
        } else {
            // Bogus token, never used if package is anon
            PackageNode {
                kw: self.cur_or_last(),
                name: self.cur_or_last(),
            }
        };

        let package_name = if !self.config.anon_packages {
            package.name.to_string()
        } else {
            // Anon packages are only used in tests and scripting, so the
            // package should have all the benefits of the main package.
            String::from("main")
        };

        // Then parse all imports as they must come before the main code
        self.skip_whitespace_and_not_eof();
        let imports = self
            .parse_imports()
            .map_err(|err| ErrorSet::new_from(err))?;

        if self.config.anon_packages {
            info!("ignoring package");
        }

        let mut decls = Vec::new();

        while self.skip_whitespace_and_not_eof() {
            match self.parse_decl() {
                Ok(decl) => decls.push(decl),
                Err(err) => {
                    self.errs.add(err);
                    self.recover_from_error();
                }
            }
        }

        if self.errs.len() > 0 {
            info!("fail, finished with {} errors", self.errs.len());
            return Err(self.errs);
        }

        info!("success, part of package '{}'", package_name);

        let ast = Ast {
            package,
            imports,
            decls,
        };

        Ok(File::new(package_name, self.src, ast))
    }

    /// Consume newlines until first non-newline token or eof.
    /// Returns true if not eof after consumption.
    fn skip_whitespace_and_not_eof(&mut self) -> bool {
        while !self.eof_or_panic() && self.matches(TokenKind::Newline) {
            self.consume();
        }
        !self.eof()
    }

    // Consume until next 'safe' token to recover. Sets panic_mode to false.
    fn recover_from_error(&mut self) {
        self.consume(); // consume at least first token in case it is the one causing the panic
        while !self.eof() && !self.matches_any(&[TokenKind::Func, TokenKind::Extern]) {
            self.consume();
        }

        self.panic_mode = false;
    }

    fn parse_package_decl(&mut self) -> Result<PackageNode, Error> {
        let kw = self.expect(TokenKind::Package)?;
        let name = self.expect_identifier("package name")?;
        Ok(PackageNode { kw, name })
    }

    fn parse_imports(&mut self) -> Result<Vec<ImportNode>, Error> {
        let mut imports = Vec::new();

        while self.matches(TokenKind::Import) {
            imports.push(self.parse_import()?);
            self.skip_whitespace_and_not_eof();
        }

        Ok(imports)
    }

    fn parse_import(&mut self) -> Result<ImportNode, Error> {
        // Import statements have three variations:
        //
        // 1. import foo.bar
        // 2. import foo as bar
        // 3. import foo { bar, faz }
        //
        // Variants 2 and 3 cannot be used together.

        let kw = self.expect(TokenKind::Import)?;

        // Collect the imported package names (foo.bar etc)
        let mut names = vec![self.expect_identifier("package name")?];
        while self.matches(TokenKind::Dot) {
            self.consume(); // dot
            names.push(self.expect_identifier("package name")?);
        }

        let mut alias = None;
        let mut imports = Vec::new();

        let end_tok = match self.cur().map_or(TokenKind::Newline, |t| t.kind) {
            // If the next token is 'as', this is variation 2
            TokenKind::As => {
                self.consume(); // as
                let name = self.expect_identifier("import alias name")?;
                alias = Some(name.clone());
                name
            }
            // If it is '{', this is variation 3
            TokenKind::LBrace => {
                self.consume(); // lbrace
                imports = self.collect_item_names("imported item name")?;
                let rbrace = self.expect(TokenKind::RBrace)?;

                // Breaking the rule of not combining variants 2 and 3
                if self.matches(TokenKind::As) {
                    return Err(self.error_token("alias is not allowed after named imports"));
                }

                rbrace
            }
            // Otherwise its variation 1
            _ => names
                .last()
                .expect("empty name vector should be handled")
                .clone(),
        };

        Ok(ImportNode {
            kw,
            names,
            imports,
            alias,
            end_tok,
        })
    }

    /// Parse a list of items separated by comma and an arbitrary amount of newlines.
    /// On parse error, an expect-error is returned with the given item_name.
    fn collect_item_names(&mut self, item_name: &str) -> Result<Vec<Token>, Error> {
        let mut items = Vec::new();

        self.skip_whitespace_and_not_eof();
        items.push(self.expect_identifier("import item")?);

        while self.matches(TokenKind::Comma) {
            self.consume(); // comma
            self.skip_whitespace_and_not_eof();

            if self.matches(TokenKind::RBrace) {
                break;
            }

            items.push(self.expect_identifier(item_name)?);
        }

        self.skip_whitespace_and_not_eof();
        Ok(items)
    }

    fn parse_decl(&mut self) -> Result<Decl, Error> {
        // Not having checked for eof is a bug in the caller.
        assert!(!self.eof(), "parse_decl called without checking eof");
        let token = self.cur().unwrap();

        match token.kind {
            TokenKind::Pub => self.parse_public_decl(),
            TokenKind::Func => self.parse_function(false),
            TokenKind::Extern => self.parse_extern(false),
            _ => Err(self.error_token("expected declaration")),
        }
    }

    fn parse_public_decl(&mut self) -> Result<Decl, Error> {
        self.consume(); // pub
        let Some(token) = self.cur() else {
            return Err(self.error_token("unexpected eof"));
        };

        match token.kind {
            TokenKind::Func => self.parse_function(true),
            TokenKind::Extern => self.parse_extern(true),
            _ => Err(self.error_token("illegal public declaration")),
        }
    }

    fn parse_extern(&mut self, public: bool) -> Result<Decl, Error> {
        self.consume(); // extern
        self.parse_function_def(public).map(|def| Decl::Extern(def))
    }

    fn parse_function_def(&mut self, public: bool) -> Result<FuncDeclNode, Error> {
        self.expect(TokenKind::Func)?;

        let name = self.expect_identifier("function name")?;
        let lparen = self.expect(TokenKind::LParen)?;

        // If the next token is not a right paren we parse parameters.
        let mut params = Vec::new();
        let mut param_names = HashSet::new();
        if !self.matches(TokenKind::RParen) {
            while !self.eof_or_panic() {
                let field = self.parse_field("parameter name")?;

                // If name already exists
                if !param_names.insert(field.name.to_string()) {
                    return Err(self.error_from_to(
                        "duplicate parameter name",
                        &field.name,
                        &field.name,
                    ));
                }

                params.push(field);

                // Done?
                if self.matches(TokenKind::RParen) {
                    break;
                }

                // Must be a comma
                self.expect(TokenKind::Comma)?;
            }

            // Bug if empty
            assert!(!params.is_empty(), "function parameters cannot be empty");
        };

        // Closing parenthesis
        let rparen = self.expect(TokenKind::RParen)?;

        // Check for return type
        let ret_type = if self.matches(TokenKind::LBrace) || self.matches(TokenKind::Newline) {
            None
        } else {
            Some(self.parse_type()?)
        };

        Ok(FuncDeclNode {
            public,
            name,
            lparen,
            params,
            rparen,
            ret_type,
        })
    }

    fn parse_function(&mut self, public: bool) -> Result<Decl, Error> {
        let mut public = public;
        let decl = self.parse_function_def(public)?;

        // Automatically set main as public for convenience
        if decl.name.to_string() == "main" {
            public = true;
        }

        let body = self.parse_block()?;

        Ok(Decl::Func(FuncNode {
            public,
            name: decl.name,
            lparen: decl.lparen,
            params: decl.params,
            rparen: decl.rparen,
            ret_type: decl.ret_type,
            body,
        }))
    }

    fn parse_field(&mut self, field_name: &str) -> Result<Field, Error> {
        let name = self.expect_identifier(field_name)?;
        let typ = self.parse_type()?;
        Ok(Field { name, typ })
    }

    fn parse_block(&mut self) -> Result<BlockNode, Error> {
        let mut stmts = Vec::new();
        let lbrace = self.expect(TokenKind::LBrace)?;

        while !self.eof_or_panic() {
            while self.matches(TokenKind::Newline) {
                self.consume();
            }

            if self.matches(TokenKind::RBrace) {
                break;
            }

            if self.eof() {
                return Err(self.error_token("unexpected end of file while parsing block"));
            }

            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        let rbrace = self.expect(TokenKind::RBrace)?;
        Ok(BlockNode {
            lbrace: lbrace,
            stmts,
            rbrace,
        })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Error> {
        assert!(!self.eof(), "parse_stmt called without checking eof");
        let token = self.cur().unwrap();

        match token.kind {
            TokenKind::Return => Ok(Stmt::Return(self.parse_return()?)),
            _ => {
                let expr = self.parse_expr()?;

                match self.cur_or_last().kind {
                    // Check for variable declaration or assignment
                    TokenKind::ColonColon => self.parse_var_decl(expr, true),
                    TokenKind::ColonEq => self.parse_var_decl(expr, false),
                    TokenKind::Eq => self.parse_var_assign(expr),

                    // Otherwise just expression
                    _ => Ok(Stmt::ExprStmt(expr)),
                }
            }
        }
    }

    fn parse_var_assign(&mut self, lval: Expr) -> Result<Stmt, Error> {
        let equal = self.expect(TokenKind::Eq)?;
        let expr = self.parse_expr()?;

        if let Expr::Literal(name) = &lval {
            if matches!(&name.kind, TokenKind::IdentLit(_)) {
                return Ok(Stmt::VarAssign(VarAssignNode { lval, equal, expr }));
            }
        }

        Err(self.error_node("invalid left hand value in assignment", &lval))
    }

    fn parse_var_decl(&mut self, lval: Expr, constant: bool) -> Result<Stmt, Error> {
        let symbol = self.must_consume()?;
        let expr = self.parse_expr()?;

        // To not use lval after move
        let err = self.error_node("invalid left hand value in declaration", &lval);
        if let Expr::Literal(name) = lval {
            if matches!(name.kind, TokenKind::IdentLit(_)) {
                return Ok(Stmt::VarDecl(VarDeclNode {
                    name,
                    symbol,
                    expr,
                    constant,
                }));
            }
        }

        Err(err)
    }

    fn parse_return(&mut self) -> Result<ReturnNode, Error> {
        assert!(self.matches(TokenKind::Return));
        let kw = self.consume().unwrap();

        let expr = if self.matches(TokenKind::Newline) {
            None
        } else {
            Some(self.parse_expr()?)
        };

        Ok(ReturnNode { kw, expr })
    }

    fn parse_expr(&mut self) -> Result<Expr, Error> {
        self.parse_call_and_member()
    }

    fn parse_call_and_member(&mut self) -> Result<Expr, Error> {
        let mut expr = self.parse_group()?;

        loop {
            // Call expression
            if self.matches(TokenKind::LParen) {
                let lparen = self.must_consume()?;
                let mut args = Vec::new();

                while !self.matches(TokenKind::RParen) {
                    args.push(self.parse_expr()?);
                    if self.matches(TokenKind::RParen) {
                        break;
                    }

                    self.expect(TokenKind::Comma)?;
                }

                let rparen = self.expect(TokenKind::RParen)?;
                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                    lparen,
                    rparen,
                })

            // Member expression
            } else if self.matches(TokenKind::Dot) {
                let dot = self.must_consume()?;
                let field = self.expect_identifier("field name")?;

                expr = Expr::Member(MemberNode {
                    expr: Box::new(expr),
                    dot,
                    field,
                });

            // Done
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_group(&mut self) -> Result<Expr, Error> {
        if self.matches(TokenKind::LParen) {
            let lparen = self.must_consume()?;
            let inner = self.parse_expr()?;
            let rparen = self.expect(TokenKind::RParen)?;

            return Ok(Expr::Group(GroupExpr {
                lparen,
                inner: Box::new(inner),
                rparen,
            }));
        }

        self.parse_literal()
    }

    fn parse_literal(&mut self) -> Result<Expr, Error> {
        let Some(token) = self.cur() else {
            return Err(self.error_token("expected expression"));
        };

        match token.kind {
            TokenKind::IntLit(_)
            | TokenKind::IdentLit(_)
            | TokenKind::FloatLit(_)
            | TokenKind::StringLit(_)
            | TokenKind::True
            | TokenKind::False
            | TokenKind::CharLit(_) => {
                self.consume();
                Ok(Expr::Literal(token))
            }
            _ => Err(self.error_token("expected expression")),
        }
    }

    fn parse_type(&mut self) -> Result<TypeNode, Error> {
        let Some(token) = self.cur() else {
            return Err(self.error_token("expected type"));
        };

        match token.kind {
            TokenKind::IdentLit(_) => {
                self.consume();
                Ok(TypeNode::Ident(token))
            }
            TokenKind::IntType
            | TokenKind::FloatType
            | TokenKind::BoolType
            | TokenKind::Void
            | TokenKind::StringType => {
                self.consume();
                Ok(TypeNode::Primitive(token))
            }
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBrack => {
                Err(self.error_token("expected type"))
            }
            _ => Err(self.error_token("invalid type")),
        }
    }

    /// Create error marking the current token.
    fn error_token(&self, message: &str) -> Error {
        self.error_from_to(message, &self.cur_or_last(), &self.cur_or_last())
    }

    /// Create error marking the given token range.
    fn error_from_to(&self, message: &str, from: &Token, to: &Token) -> Error {
        Error::new(message, from, to, &self.src)
    }

    fn error_node(&self, message: &str, node: &dyn Node) -> Error {
        Error::range(message, node.pos(), node.end(), &self.src)
    }

    fn cur(&self) -> Option<Token> {
        self.tokens.get(self.pos).cloned()
    }

    fn cur_or_last(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens.get(self.pos).unwrap().clone()
        } else {
            self.tokens.last().unwrap().clone()
        }
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let pos = self.pos;
            self.pos += 1;
            Some(self.tokens[pos].clone())
        } else {
            None
        }
    }

    /// Consumes current token and returns it. Errors if EOF.
    fn must_consume(&mut self) -> Result<Token, Error> {
        self.consume()
            .map_or(Err(self.error_token("unexpected end of file")), |t| Ok(t))
    }

    /// Expects the current token to be of a specific kind.
    /// Returns token if it matches, else error.
    fn expect(&mut self, kind: TokenKind) -> Result<Token, Error> {
        self.expect_pred(&format!("{}", kind), |t| t.kind == kind)
    }

    /// Same as expect but with a message
    fn _expect_msg(&mut self, kind: TokenKind, msg: &str) -> Result<Token, Error> {
        self.expect_pred(msg, |t| t.kind == kind)
    }

    /// Expects the current token to match a predicate.
    /// Returns token if it matches, else error.
    /// Message is prefixed with "expected ".
    fn expect_pred<P>(&mut self, message: &str, predicate: P) -> Result<Token, Error>
    where
        P: Fn(Token) -> bool,
    {
        if let Some(tok) = self.cur() {
            if predicate(tok) {
                let pos = self.pos;
                self.pos += 1;
                return Ok(self.tokens[pos].clone());
            }
        }
        Err(self.error_token(&format!("expected {}", message)))
    }

    /// Expects the current token to be an identifier with any content.
    fn expect_identifier(&mut self, message: &str) -> Result<Token, Error> {
        self.expect_pred(message, |t| matches!(t.kind, TokenKind::IdentLit(_)))
    }

    fn matches(&self, kind: TokenKind) -> bool {
        if let Some(tok) = self.cur() {
            tok.kind == kind
        } else {
            false
        }
    }

    fn matches_any(&self, kinds: &[TokenKind]) -> bool {
        for k in kinds {
            if self.matches(k.to_owned()) {
                return true;
            }
        }
        return false;
    }

    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn eof_or_panic(&self) -> bool {
        self.eof() || self.panic_mode
    }
}
