use crate::token::{File, Pos, SyntaxError, Token, TokenKind};

pub struct Scanner<'a> {
    file: &'a File,
    pos: usize,
    row: usize,
    col: usize,
    line_begin: usize,
}

pub type ScannerResult = Result<Vec<Token>, SyntaxError>;

impl<'a> Scanner<'a> {
    pub fn new(file: &'_ File) -> Scanner<'_> {
        Scanner {
            file,
            pos: 0,
            col: 0,
            row: 0,
            line_begin: 0,
        }
    }

    pub fn scan(&mut self) -> ScannerResult {
        let mut tokens = Vec::new();

        while !self.eof() {
            let (token, consumed) = match self.cur() {
                // Consume all whitespace (not newline)
                // Whitespace tokens are ignored and not added to token list
                v if Scanner::is_whitespace(v) => (
                    Token::new(TokenKind::Whitespace, 0, self.pos()),
                    self.peek_while(Scanner::is_whitespace),
                ),

                // Newline character resets the row and col.
                b'\n' => {
                    let pos = self.pos();
                    self.row += 1;
                    self.col = 0;
                    self.line_begin = self.pos + 1;
                    (Token::new(TokenKind::Newline, 1, pos), 1)
                }

                // Consume word (identifier or keyword)
                v if Scanner::is_alpha(v) => {
                    let length = self.peek_while(Scanner::is_alphanum);
                    let lexeme = self.file.str_range(self.pos, self.pos + length);

                    (
                        Token::new(TokenKind::IdentLit(lexeme.to_owned()), length, self.pos()),
                        length,
                    )
                }

                // Number literal
                v if Scanner::is_number(v) => {
                    let length = self.peek_while(Scanner::is_numeric);
                    let lexeme = self.file.str_range(self.pos, self.pos + length);

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

                _ => return Err(self.error("illegal token", 1)),
            };

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
            row: self.row,
            col: self.col,
            offset: self.pos,
            line_begin: self.line_begin,
        }
    }

    fn cur(&self) -> u8 {
        self.file.src[self.pos]
    }

    fn eof(&self) -> bool {
        self.pos >= self.len()
    }

    fn len(&self) -> usize {
        self.file.src.len()
    }

    fn error(&self, msg: &str, length: usize) -> SyntaxError {
        let mut end_pos = self.pos();
        end_pos.col += length;
        end_pos.offset += length;
        SyntaxError::new(msg, self.pos(), end_pos, &self.file)
    }

    /// Peeks tokens while predicate returns true. Returns number of tokens peeked.
    fn peek_while<P>(&mut self, predicate: P) -> usize
    where
        P: Fn(u8) -> bool,
    {
        let mut consumed = 0;
        while self.pos + consumed < self.len() && predicate(self.file.src[self.pos + consumed]) {
            consumed += 1;
        }

        consumed
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
