use crate::{
    ast::{Ast, BlockNode, Decl, Expr, Field, FuncNode, ReturnNode, Stmt, TypeNode, no_type},
    error::{Error, ErrorSet},
    token::{File, Token, TokenKind},
};

pub struct Parser<'a> {
    errs: ErrorSet,
    tokens: Vec<Token>,
    pos: usize,
    file: &'a File,

    // Panic mode occurs when the parser encounters an unknown token sequence
    // and needs to synchronize to a 'clean' state. When panic mode starts,
    // the base position is set to the current position. When in panic mode
    // all err() calls are ignored.
    //
    // Functions which parse statements should have a check at the top for
    // panicMode, and return early with an invalid statement if set.
    panic_mode: bool,
    // base_pos: usize,
}

pub type ParserResult = Result<Ast, ErrorSet>;

impl<'a> Parser<'a> {
    pub fn parse(file: &'a File, tokens: Vec<Token>) -> ParserResult {
        let mut s = Self {
            errs: ErrorSet::new(),
            tokens,
            file,
            pos: 0,
            panic_mode: false,
        };

        let mut ast = Ast::new();

        while s.skip_whitespace_and_not_eof() {
            match s.parse_decl() {
                Ok(decl) => ast.add_node(decl),
                Err(err) => {
                    // TODO: add panic mode to not return early on error
                    s.errs.add(err);
                    return Err(s.errs);
                }
            }
        }

        if s.errs.size() > 0 {
            return Err(s.errs);
        }

        Ok(ast)
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

        match token.kind {
            TokenKind::Func | TokenKind::Pub => {
                let func = self.parse_function(token)?;
                Ok(Decl::Func(func))
            }

            _ => {
                // Handle other declaration types
                Err(self.error_token("expected declaration"))
            }
        }
    }

    fn parse_function(&mut self, kw: Token) -> Result<FuncNode, Error> {
        let public = kw.kind == TokenKind::Pub;
        if public {
            self.consume(); // Consume the 'pub' token
        }

        self.consume(); // Consume the 'func' token

        let name = self.expect_identifier("function name")?;
        let lparen = self.expect(TokenKind::LParen)?;

        // If the next token is not a right paren we parse parameters.
        let params = if self.matches(TokenKind::RParen) {
            None
        } else {
            let mut fields = Vec::new();

            while !self.eof_or_panic() {
                let field = self.parse_field("parameter name")?;
                fields.push(field);

                // Done?
                if self.matches(TokenKind::RParen) {
                    break;
                }

                // Must be a comma
                self.expect(TokenKind::Comma)?;
            }

            // Bug if empty
            assert!(!fields.is_empty(), "function parameters cannot be empty");
            Some(fields)
        };

        // Closing parenthesis
        let rparen = self.expect(TokenKind::RParen)?;

        // Check for return type
        let ret_type = if self.matches(TokenKind::LBrace) {
            None
        } else {
            Some(self.parse_type()?)
        };

        let body = self.parse_block()?;

        let func = FuncNode {
            public,
            name: name.clone(),
            lparen: lparen.clone(),
            params,
            rparen: rparen.clone(),
            ret_type,
            body,
            sem_ret_type: no_type(),
        };

        Ok(func)
    }

    fn parse_field(&mut self, field_name: &str) -> Result<Field, Error> {
        let name = self.expect_identifier(field_name)?;
        let typ = self.parse_type()?;
        Ok(Field {
            name,
            typ,
            sem_type: no_type(),
        })
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
            TokenKind::Return => {
                let ret = self.parse_return()?;
                Ok(Stmt::Return(ret))
            }
            _ => {
                let expr = self.parse_expr()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    fn parse_return(&mut self) -> Result<ReturnNode, Error> {
        // Assert and consume the 'return' token
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
        self.parse_literal()
    }

    fn parse_literal(&mut self) -> Result<Expr, Error> {
        let Some(token) = self.cur() else {
            return Err(self.error_token("expected literal expression"));
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
            _ => Err(self.error_token("expected literal expression")),
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
            TokenKind::IntType | TokenKind::FloatType | TokenKind::BoolType | TokenKind::Void => {
                self.consume();
                Ok(TypeNode::Primitive(token))
            }
            _ => Err(self.error_token("invalid type")),
        }
    }

    /// Create error marking the current token.
    fn error_token(&self, message: &str) -> Error {
        self.error_from_to(message, self.cur_or_last(), self.cur_or_last())
    }

    /// Create error marking the given token range.
    fn error_from_to(&self, message: &str, from: Token, to: Token) -> Error {
        Error::new(message, &from, &to, self.file)
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

    /// Expects the current token to be of a specific kind.
    /// Returns token if it matches, else error.
    fn expect(&mut self, kind: TokenKind) -> Result<Token, Error> {
        self.expect_pred(&format!("{}", kind), |t| t.kind == kind)
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

    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn eof_or_panic(&self) -> bool {
        self.eof() || self.panic_mode
    }
}
