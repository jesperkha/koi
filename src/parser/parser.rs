use core::panic;

use crate::{
    ast::{Ast, BlockNode, Decl, Expr, FuncNode, Stmt},
    parser::ParserError,
    token::{File, Token, TokenKind},
};

pub struct Parser<'a> {
    errors: Vec<ParserError>,
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
    base_pos: usize,
}

pub type ParserResult = Result<Ast, Vec<ParserError>>;

impl<'a> Parser<'a> {
    pub fn new(file: &'a File, tokens: Vec<Token>) -> Self {
        Parser {
            errors: Vec::new(),
            tokens,
            file,
            pos: 0,
            panic_mode: false,
            base_pos: 0,
        }
    }

    pub fn parse(&mut self) -> ParserResult {
        let mut ast = Ast::new();
        let decl = self.parse_decl();

        match decl {
            Ok(decl) => ast.add_node(decl),
            Err(err) => {
                self.errors.push(err);
            }
        }

        if self.errors.len() > 0 {
            return Err(self.errors.clone());
        }
        Ok(ast)
    }

    fn parse_decl(&mut self) -> Result<Decl, ParserError> {
        let Some(token) = self.cur() else {
            // Not having checked for eof is a bug in the caller.
            panic!("parse_decl called without checking eof");
        };

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

    fn parse_function(&mut self, kw: Token) -> Result<FuncNode, ParserError> {
        let public = kw.kind == TokenKind::Pub;
        if public {
            self.consume(); // Consume the 'pub' token
        }

        self.consume(); // Consume the 'func' token

        let name = self.expect_identifier("function name")?;
        let lparen = self.expect(TokenKind::LParen)?;
        let rparen = self.expect(TokenKind::RParen)?;

        let body = self.parse_block()?;

        let func = FuncNode {
            public,
            name: name.clone(),
            lparen: lparen.clone(),
            params: None,
            rparen: rparen.clone(),
            ret_type: None,
            body,
        };

        Ok(func)
    }

    fn parse_block(&mut self) -> Result<BlockNode, ParserError> {
        let mut stmts = Vec::new();
        let lbrace = self.expect(TokenKind::LBrace)?;

        while !self.eof_or_panic() {
            while self.matches(TokenKind::Newline) {
                self.consume();
            }

            if self.matches(TokenKind::RBrace) {
                break;
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

    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        todo!()
    }

    fn parse_expr() -> Result<Expr, ParserError> {
        todo!()
    }

    /// Create error marking the current token.
    fn error_token(&self, message: &str) -> ParserError {
        self.error_from_to(message, self.cur_or_last(), self.cur_or_last())
    }

    /// Create error marking the given token range.
    fn error_from_to(&self, message: &str, from: Token, to: Token) -> ParserError {
        ParserError::new(message, from, to, self.file)
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

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.pos + 1).cloned()
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
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParserError> {
        self.expect_pred(&format!("{}", kind), |t| t.kind == kind)
    }

    /// Expects the current token to match a predicate.
    /// Returns token if it matches, else error.
    /// Message is prefixed with "expected ".
    fn expect_pred<P>(&mut self, message: &str, predicate: P) -> Result<Token, ParserError>
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
    fn expect_identifier(&mut self, message: &str) -> Result<Token, ParserError> {
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
