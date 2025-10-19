use std::collections::HashSet;

use tracing::info;

use crate::{
    ast::{
        BlockNode, CallExpr, Decl, Expr, Field, File, FuncDeclNode, FuncNode, GroupExpr, Node,
        ReturnNode, Stmt, TypeNode, VarNode,
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
    file: File,
    config: &'a Config,

    // Panic mode occurs when the parser encounters an unknown token sequence
    // and needs to synchronize to a 'clean' state. When panic mode starts,
    // the base position is set to the current position. When in panic mode
    // all err() calls are ignored.
    //
    // Functions which parse statements should have a check at the top for
    // panicMode, and return early with an invalid statement if set.
    panic_mode: bool,

    pkg_declared: bool, // Package name declared yet?
}

impl<'a> Parser<'a> {
    fn new(src: Source, tokens: Vec<Token>, config: &'a Config) -> Self {
        Self {
            errs: ErrorSet::new(),
            tokens,
            file: File::new(src),
            pos: 0,
            panic_mode: false,
            pkg_declared: false,
            config,
        }
    }

    fn parse(mut self) -> Res<File> {
        info!("file '{}'", self.file.src.name);

        if self.config.anon_packages {
            info!("ignoring package");
        }

        while self.skip_whitespace_and_not_eof() {
            match self.parse_decl() {
                Ok(decl) => match decl {
                    Decl::Package(name) => self.file.set_package(name),
                    _ => self.file.add_node(decl),
                },
                Err(err) => {
                    self.errs.add(err);

                    // Consume until next 'safe' token to recover.
                    while !self.matches_any(&[TokenKind::Func]) && !self.eof() {
                        self.consume();
                    }

                    self.panic_mode = false;
                }
            }
        }

        if self.errs.len() > 0 {
            info!("fail, finished with {} errors", self.errs.len());
            return Err(self.errs);
        }

        info!("success, part of package '{}'", self.file.pkgname);
        Ok(self.file)
    }

    /// Consume newlines until first non-newline token or eof.
    /// Returns true if not eof after consumption.
    fn skip_whitespace_and_not_eof(&mut self) -> bool {
        while !self.eof_or_panic() && self.matches(TokenKind::Newline) {
            self.consume();
        }
        !self.eof()
    }

    fn parse_decl(&mut self) -> Result<Decl, Error> {
        // Not having checked for eof is a bug in the caller.
        assert!(!self.eof(), "parse_decl called without checking eof");
        let token = self.cur().unwrap();

        // Check that package declaration comes first
        if !self.pkg_declared && !self.config.anon_packages {
            self.pkg_declared = true;
            self.expect_msg(TokenKind::Package, "package declaration")?;
            return self
                .expect_identifier("package name")
                .map(|tok| Decl::Package(tok));
        }

        match token.kind {
            TokenKind::Func | TokenKind::Pub => Ok(Decl::Func(self.parse_function(token)?)),
            TokenKind::Package => {
                if !self.config.anon_packages {
                    Err(self.error_token("only declare package once, and as the first statement"))
                } else {
                    self.consume(); // kw
                    self.expect_identifier("package name")
                        .map(|t| Decl::Package(t))
                }
            }
            // TODO: public extern (re-export)
            TokenKind::Extern => {
                self.consume(); // extern
                Ok(Decl::Extern(self.parse_function_def()?))
            }
            _ => Err(self.error_token("expected declaration")),
        }
    }

    fn parse_function_def(&mut self) -> Result<FuncDeclNode, Error> {
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
            name,
            lparen,
            params,
            rparen,
            ret_type,
        })
    }

    fn parse_function(&mut self, kw: Token) -> Result<FuncNode, Error> {
        let mut public = kw.kind == TokenKind::Pub;
        if public {
            self.consume(); // Consume the 'pub' token
        }

        let decl = self.parse_function_def()?;

        // Automatically set main as public for convenience
        if decl.name.to_string() == "main" {
            public = true;
        }

        let body = self.parse_block()?;

        Ok(FuncNode {
            public,
            name: decl.name,
            lparen: decl.lparen,
            params: decl.params,
            rparen: decl.rparen,
            ret_type: decl.ret_type,
            body,
        })
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
        let symbol = self.expect(TokenKind::Eq)?;
        let expr = self.parse_expr()?;

        if let Expr::Literal(name) = lval {
            if matches!(name.kind, TokenKind::IdentLit(_)) {
                return Ok(Stmt::VarAssign(VarNode {
                    name,
                    symbol,
                    expr,
                    constant: false,
                }));
            }
        }
        panic!("unhandled l-value in assignment")
    }

    fn parse_var_decl(&mut self, lval: Expr, constant: bool) -> Result<Stmt, Error> {
        let symbol = self.must_consume()?;
        let expr = self.parse_expr()?;

        // To not use lval after move
        let err = self.error_node("invalid left hand value in declaration", &lval);
        if let Expr::Literal(name) = lval {
            if matches!(name.kind, TokenKind::IdentLit(_)) {
                return Ok(Stmt::VarDecl(VarNode {
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
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.parse_group()?;

        while self.matches(TokenKind::LParen) {
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
        Error::new(message, from, to, &self.file.src)
    }

    fn error_node(&self, message: &str, node: &dyn Node) -> Error {
        Error::range(message, node.pos(), node.end(), &self.file.src)
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
    fn expect_msg(&mut self, kind: TokenKind, msg: &str) -> Result<Token, Error> {
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
