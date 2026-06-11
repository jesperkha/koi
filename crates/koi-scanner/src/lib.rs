use tracing::{debug, info, trace};

use koi_ast::{Pos, Source, Token, TokenKind, str_to_token};
use koi_common::{config::Config, error::{Diagnostics, Report, Res}};

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

        if self.eof() {
            info!("No input");
            return Ok(Vec::new());
        }

        while !self.eof() {
            match self.scan_all() {
                Ok(toks) => {
                    if self.diag.is_empty() {
                        debug!("Success: {} tokens", toks.len());
                        return Ok(toks);
                    }
                }
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
                v if Scanner::is_whitespace(v) => (
                    Token::new(TokenKind::Whitespace, 0, self.pos()),
                    self.peek_while(Scanner::is_whitespace),
                ),

                b'/' if matches!(self.peek(), Some(b'/')) => {
                    let len = self.peek_while(|b| b != b'\n');
                    (Token::new(TokenKind::LineComment, 0, self.pos()), len)
                }

                b'/' if matches!(self.peek(), Some(b'*')) => {
                    let mut depth = 1;
                    let mut i = self.pos + 2;

                    while i + 1 < self.len() && depth > 0 {
                        match (self.at(i), self.at(i + 1)) {
                            (b'/', b'*') => {
                                depth += 1;
                                i += 2;
                            }
                            (b'*', b'/') => {
                                depth -= 1;
                                i += 2;
                            }
                            _ => {
                                i += 1;
                            }
                        }
                    }

                    if depth != 0 {
                        return Err(Report::code_error_len(
                            "block comment was not terminated",
                            &self.pos(),
                            2,
                        ));
                    }

                    for j in self.pos..i {
                        if self.source.src[j] == b'\n' {
                            self.row += 1;
                            self.line_begin = j + 1;
                            self.col = 0;
                        }
                    }
                    self.col = i - self.line_begin;

                    (
                        Token::new(TokenKind::BlockComment, 0, self.pos()),
                        i - self.pos,
                    )
                }

                b'\n' => {
                    let pos = self.pos();
                    self.row += 1;
                    self.col = 0;
                    self.line_begin = self.pos + 1;
                    (Token::new(TokenKind::Newline, 1, pos), 1)
                }

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

                v if Scanner::is_number(v) => {
                    if v == b'0' && matches!(self.peek(), Some(b'x') | Some(b'X')) {
                        let prefix_len = 2;
                        let digits_start = self.pos + prefix_len;
                        let digits_len = {
                            let mut n = 0;
                            while digits_start + n < self.len()
                                && Scanner::is_hex_digit(self.source.src[digits_start + n])
                            {
                                n += 1;
                            }
                            n
                        };

                        if digits_len == 0 {
                            return Err(self.error("hex literal has no digits", prefix_len));
                        }

                        let length = prefix_len + digits_len;
                        let digits = self
                            .source
                            .str_range(digits_start, digits_start + digits_len);
                        let value = i64::from_str_radix(digits, 16)
                            .map_err(|_| self.error("invalid hex literal", length))?;

                        (
                            Token::new(TokenKind::IntLit(value), length, self.pos()),
                            length,
                        )
                    } else {
                        let mut length = self.peek_while(Scanner::is_numeric);
                        let mut lexeme = self.source.str_range(self.pos, self.pos + length);

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
                }

                b'"' => self.scan_string(b'"')?,

                b'\'' => {
                    let (tokens, length) = self.scan_string(b'\'')?;
                    if length != 3 {
                        return Err(self.error("byte string must be exactly one character", length));
                    }

                    (tokens, length)
                }

                _ => {
                    let try_match = |len| {
                        let lexeme = self.source.str_range(self.pos, self.pos + len);
                        str_to_token(lexeme)
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

            if !matches!(token.kind, TokenKind::Newline | TokenKind::BlockComment) {
                self.col += consumed;
            }

            if !matches!(
                token.kind,
                TokenKind::Whitespace | TokenKind::BlockComment | TokenKind::LineComment
            ) {
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
        Report::code_error_len(msg, &self.pos(), length)
    }

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

    fn scan_string(&mut self, quote: u8) -> Result<(Token, usize), Report> {
        self.pos += 1;
        let mut length = self.peek_while(|b| b != quote && b != b'\n');
        self.pos -= 1;

        let check_pos = self.pos + length + 1;
        if check_pos >= self.len() || self.at(check_pos) != quote {
            let mut pos = self.pos();
            pos.col += check_pos;
            pos.offset += check_pos;
            return Err(Report::code_error_len("expected end quote", &pos, 1));
        }

        length += 2;
        let lexeme = self.source.str_range(self.pos + 1, self.pos + length - 1);

        Ok((
            Token::new(TokenKind::StringLit(lexeme.to_string()), length, self.pos()),
            length,
        ))
    }

    fn is_number(n: u8) -> bool {
        n.is_ascii_digit()
    }

    fn is_numeric(n: u8) -> bool {
        Scanner::is_number(n) || n == b'.'
    }

    fn is_hex_digit(n: u8) -> bool {
        n.is_ascii_hexdigit()
    }

    fn is_whitespace(b: u8) -> bool {
        b == b' ' || b == b'\t' || b == b'\r'
    }

    fn is_alpha(b: u8) -> bool {
        b.is_ascii_lowercase() || b.is_ascii_uppercase() || b == b'_'
    }

    fn is_alphanum(b: u8) -> bool {
        Scanner::is_alpha(b) || Scanner::is_number(b)
    }
}
