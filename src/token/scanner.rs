use tracing::{debug, info, trace};

use crate::{
    config::Config,
    error::{Diagnostics, Report, Res},
    token::{Pos, Source, Token, TokenKind, str_to_token},
};

pub fn scan(src: &Source, config: &Config) -> Res<Vec<Token>> {
    let scanner = Scanner::new(src, config);
    scanner.scan()
}

struct Scanner<'a> {
    source: &'a Source,
    pos: usize,
    row: usize,
    col: usize,
    line_begin: usize,
    _config: &'a Config,
    diag: Diagnostics,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a Source, config: &'a Config) -> Self {
        Scanner {
            _config: config,
            source,
            pos: 0,
            col: 0,
            row: 0,
            line_begin: 0,
            diag: Diagnostics::new(),
        }
    }

    fn scan(mut self) -> Res<Vec<Token>> {
        info!("Scanning file: {}", self.source.filepath);

        // No input
        if self.eof() {
            info!("No input");
            return Ok(Vec::new());
        }

        while !self.eof() {
            match self.scan_all() {
                // If ok and we did not encounter errors before, return result
                // Otherwise ignore result as one or more errors have been raised
                Ok(toks) => {
                    if self.diag.is_empty() {
                        debug!("success: {} tokens", toks.len());
                        return Ok(toks);
                    }
                }
                // If error add to set and skip to next 'safe' spot
                Err(err) => {
                    self.diag.add(err);
                    self.pos += self.peek_while(|p| !Scanner::is_whitespace(p))
                }
            }
        }

        info!("Fail: finished with {} errors", self.diag.num_errors());
        Err(self.diag)
    }

    fn scan_all(&mut self) -> Result<Vec<Token>, Report> {
        let mut tokens = Vec::new();

        while !self.eof() {
            let (token, consumed) = match self.cur() {
                // Whitespace tokens are ignored and not added to token list
                v if Scanner::is_whitespace(v) => (
                    Token::new(TokenKind::Whitespace, 0, self.pos()),
                    self.peek_while(Scanner::is_whitespace),
                ),

                // Line comment
                b'/' if matches!(self.peek(), Some(b'/')) => {
                    let len = self.peek_while(|b| b != b'\n');
                    let lexeme = self.source.str_range(self.pos, self.pos + len);
                    let tok = Token::new(TokenKind::Comment(lexeme.to_owned()), len, self.pos());
                    (tok, len)
                }

                // Block comment
                b'/' if matches!(self.peek(), Some(b'*')) => {
                    let mut depth = 1;
                    let mut i = self.pos + 2; // Skip opening /*

                    while i + 1 < self.len() && depth > 0 {
                        let c1 = self.at(i);
                        let c2 = self.at(i + 1);

                        if c1 == b'/' && c2 == b'*' {
                            depth += 1;
                            i += 2;
                            continue;
                        } else if c1 == b'*' && c2 == b'/' {
                            depth -= 1;
                            i += 2;
                            continue;
                        }

                        i += 1;
                    }

                    if depth != 0 {
                        return Err(Report::new_length(
                            "block comment was not terminated",
                            &self.pos(),
                            2,
                        ));
                    }

                    (
                        Token::new(TokenKind::Whitespace, 0, self.pos()),
                        i - self.pos,
                    )
                }

                // Newline character resets the row and col.
                b'\n' => {
                    let pos = self.pos();
                    self.row += 1;
                    self.col = 0;
                    self.line_begin = self.pos + 1;
                    (Token::new(TokenKind::Newline, 1, pos), 1)
                }

                // Identifier or keyword
                v if Scanner::is_alpha(v) => {
                    let length = self.peek_while(Scanner::is_alphanum);
                    let lexeme = self.source.str_range(self.pos, self.pos + length);

                    if let Some(k) = str_to_token(lexeme) {
                        (Token::new(k.clone(), length, self.pos()), length)
                    } else {
                        (
                            Token::new(TokenKind::IdentLit(lexeme.to_owned()), length, self.pos()),
                            length,
                        )
                    }
                }

                // Number
                v if Scanner::is_number(v) => {
                    let mut length = self.peek_while(Scanner::is_numeric);
                    let mut lexeme = self.source.str_range(self.pos, self.pos + length);

                    // Ignore ending period for now, checked by Checker
                    if lexeme.ends_with(".") {
                        length -= 1;
                        lexeme = lexeme.trim_end_matches(".");
                    }

                    let kind = if lexeme.contains('.') {
                        match lexeme.parse() {
                            Ok(f) => TokenKind::FloatLit(f),
                            _ => return Err(self.error("invalid number literal", length)),
                        }
                    } else {
                        match lexeme.parse() {
                            Ok(f) => TokenKind::IntLit(f),
                            _ => return Err(self.error("invalid number literal", length)),
                        }
                    };

                    (Token::new(kind, length, self.pos()), length)
                }

                // String
                b'"' => self.scan_string(b'"')?,

                // Byte string
                b'\'' => {
                    let (tokens, length) = self.scan_string(b'\'')?;
                    if length != 3 {
                        return Err(self.error("byte string must be exactly one character", length));
                    }

                    (tokens, length)
                }

                // Match either one or two tokens (single/double symbol)
                _ => {
                    let try_match = |len| {
                        let lexeme = self.source.str_range(self.pos, self.pos + len);
                        str_to_token(&lexeme)
                            .map(|kind| Token::new(kind.to_owned(), len, self.pos()))
                    };

                    if let Some(token) = self
                        .peek()
                        .filter(|&c| !Scanner::is_alphanum(c))
                        .and_then(|_| try_match(2))
                    {
                        (token, 2)
                    } else if let Some(token) = try_match(1) {
                        (token, 1)
                    } else {
                        return Err(self.error("illegal token", 1));
                    }
                }
            };

            trace!("consumed token: '{}'", token);
            self.pos += consumed;

            // Col must not advance after a newline. It is reset to 0 above and must remain 0
            // before next iteration. Incrementing now would cause the first token on the new
            // line to have col=1
            if !token.kind.eq(&TokenKind::Newline) {
                self.col += consumed;
            }

            if !token.kind.eq(&TokenKind::Whitespace) {
                tokens.push(token);
            }
        }

        Ok(tokens)
    }

    fn pos(&self) -> Pos {
        Pos {
            source_id: self.source.id,
            row: self.row,
            col: self.col,
            offset: self.pos,
            line_begin: self.line_begin,
        }
    }

    fn at(&self, pos: usize) -> u8 {
        assert!(
            pos < self.len(),
            "tried to access pos {} when src is {}",
            pos,
            self.len()
        );
        self.source.src[pos]
    }

    fn peek(&self) -> Option<u8> {
        if self.pos + 1 >= self.len() {
            None
        } else {
            Some(self.at(self.pos + 1))
        }
    }

    fn cur(&self) -> u8 {
        self.source.src[self.pos]
    }

    fn eof(&self) -> bool {
        self.pos >= self.len()
    }

    fn len(&self) -> usize {
        self.source.src.len()
    }

    fn error(&self, msg: &str, length: usize) -> Report {
        Report::new_length(msg, &self.pos(), length)
    }

    /// Peeks tokens while predicate returns true. Returns number of tokens peeked.
    fn peek_while<P>(&mut self, predicate: P) -> usize
    where
        P: Fn(u8) -> bool,
    {
        let mut consumed = 0;
        while self.pos + consumed < self.len() && predicate(self.source.src[self.pos + consumed]) {
            consumed += 1;
        }

        consumed
    }

    /// Scans a string literal, starting at the current position.
    fn scan_string(&mut self, quote: u8) -> Result<(Token, usize), Report> {
        self.pos += 1;
        let mut length = self.peek_while(|b| b != quote && b != b'\n');
        self.pos -= 1;

        // Was string actually closed?
        let check_pos = self.pos + length + 1; // Of end quote
        if check_pos >= self.len() || self.at(check_pos) != quote {
            let mut pos = self.pos();
            pos.col += check_pos;
            pos.offset += check_pos;
            return Err(Report::new_length("expected end quote", &pos, 1));
        }

        length += 2; // Include start and end quote
        let lexeme = self.source.str_range(self.pos + 1, self.pos + length - 1);

        Ok((
            Token::new(TokenKind::StringLit(lexeme.to_string()), length, self.pos()),
            length,
        ))
    }

    fn is_number(n: u8) -> bool {
        n >= b'0' && n <= b'9'
    }

    fn is_numeric(n: u8) -> bool {
        Scanner::is_number(n) || n == b'.'
    }

    fn is_whitespace(b: u8) -> bool {
        b == b' ' || b == b'\t' || b == b'\r'
    }

    fn is_alpha(b: u8) -> bool {
        (b >= b'a' && b <= b'z') || (b >= b'A' && b <= b'Z') || b == b'_'
    }

    fn is_alphanum(b: u8) -> bool {
        Scanner::is_alpha(b) || Scanner::is_number(b)
    }
}
