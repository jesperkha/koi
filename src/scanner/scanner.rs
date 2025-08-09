use crate::token::{Pos, Token, TokenKind};

pub struct Scanner {
    src: Vec<u8>,
    pos: usize,
    row: usize,
    col: usize,
    line_begin: usize,
}

#[derive(Debug)]
pub struct SyntaxError {
    pub message: String,
}

pub type ScannerResult = Result<Vec<Token>, SyntaxError>;

impl Scanner {
    pub fn new(src: Vec<u8>) -> Scanner {
        Scanner {
            src,
            pos: 0,
            col: 0,
            row: 0,
            line_begin: 0,
        }
    }

    pub fn scan(&mut self) -> ScannerResult {
        let mut tokens = Vec::new();

        while !self.eof() {
            let (token, consumed) = match self.src[self.pos] {
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
                    let byte_range = &self.src[self.pos..self.pos + length];
                    let lexeme = str::from_utf8(byte_range)
                        .expect("Expected valid UTF-8 string")
                        .to_owned();

                    (
                        Token::new(TokenKind::IdentLit(lexeme), length, self.pos()),
                        length,
                    )
                }

                _ => return Err(Scanner::error(format!("illegal token `{}`", self.cur()))),
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

    fn error(message: String) -> SyntaxError {
        SyntaxError { message }
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
        self.src[self.pos]
    }

    /// Peeks tokens while predicate returns true. Returns number of tokens peeked.
    fn peek_while<P>(&mut self, predicate: P) -> usize
    where
        P: Fn(u8) -> bool,
    {
        let mut consumed = 0;

        while self.pos + consumed < self.src.len() && predicate(self.src[self.pos + consumed]) {
            consumed += 1;
        }

        consumed
    }

    fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    fn is_number(n: u8) -> bool {
        n >= b'0' && n <= b'9'
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
