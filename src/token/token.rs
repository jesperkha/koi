use std::fmt;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,

    /// Position of first character in token.
    pub pos: Pos,
    /// Position of character immediately after token
    pub end_pos: Pos,
    /// Is the token an EOF token?
    pub eof: bool,
    /// Is the token invalid?
    pub invalid: bool,
    /// Byte length of token
    pub length: usize,
}

impl Token {
    /// Create new Token. Sets token flags based on kind.
    pub fn new(kind: TokenKind, length: usize, pos: Pos) -> Token {
        let end_pos = Pos {
            row: pos.row,
            col: pos.col + length,
            offset: pos.offset + length,
            line_begin: pos.line_begin,
        };

        Token {
            length: length,
            eof: kind.eq(&TokenKind::Eof),
            invalid: kind.eq(&TokenKind::Invalid),
            pos: pos,
            end_pos: end_pos,
            kind: kind,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} ({}, {})", self.kind, self.pos.row, self.pos.col)
    }
}

pub fn display_tokens(tokens: &[Token]) -> String {
    format!(
        "[\n{}\n]",
        tokens
            .iter()
            .map(|t| format!("    {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

#[derive(Debug, PartialEq, Eq)]
pub struct Pos {
    /// Row in file, starting at 0
    pub row: usize,
    /// Column on line, starting at 0
    pub col: usize,
    /// Byte offset in file
    pub offset: usize,
    /// Offset of first character on same line as this Pos
    pub line_begin: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Invalid,
    Whitespace, // Ignored by scanner
    Newline,
    Eof,

    // Literals, contain the literal value
    IdentLit(String),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String), // String does not include quotes
    CharLit(u8),

    // Keywords
    True,
    False,
    Return,
    Func,
    If,
    Else,
    For,
    Import,
    Package,
    Null,
    Pub,
    Error,

    // Math
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    // Logic
    Eq,
    EqEq,
    NotEq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    Greater,
    Less,
    GreaterEq,
    LessEq,
    Or,
    OrOr,
    And,
    AndAnd,
    Bang,
    BangEq,

    // Parenthesis
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,

    // Other symbols
    Dot,
    Comma,
    Semi,
    Colon,
    ColonEq,
    Question,

    // Primitive types
    Void,
    Int,
    Float,
    String,
    Byte,
    Bool,
}

/// Reserved token lexemes
static RESERVED: &[(&str, TokenKind)] = &[
    // Keywords
    ("true", TokenKind::True),
    ("false", TokenKind::False),
    ("return", TokenKind::Return),
    ("func", TokenKind::Func),
    ("if", TokenKind::If),
    ("else", TokenKind::Else),
    ("for", TokenKind::For),
    ("import", TokenKind::Import),
    ("package", TokenKind::Package),
    ("null", TokenKind::Null),
    ("pub", TokenKind::Pub),
    ("error", TokenKind::Error),
    // Math
    ("+", TokenKind::Plus),
    ("-", TokenKind::Minus),
    ("*", TokenKind::Star),
    ("/", TokenKind::Slash),
    ("%", TokenKind::Percent),
    // Logic
    ("=", TokenKind::Eq),
    ("==", TokenKind::EqEq),
    ("!=", TokenKind::NotEq),
    ("+=", TokenKind::PlusEq),
    ("-=", TokenKind::MinusEq),
    ("*=", TokenKind::StarEq),
    ("/=", TokenKind::SlashEq),
    (">", TokenKind::Greater),
    ("<", TokenKind::Less),
    (">=", TokenKind::GreaterEq),
    ("<=", TokenKind::LessEq),
    ("|", TokenKind::Or),
    ("||", TokenKind::OrOr),
    ("&", TokenKind::And),
    ("&&", TokenKind::AndAnd),
    ("!", TokenKind::Bang),
    ("!=", TokenKind::BangEq),
    // Parenthesis & Brackets
    ("(", TokenKind::LParen),
    (")", TokenKind::RParen),
    ("{", TokenKind::LBrace),
    ("}", TokenKind::RBrace),
    ("[", TokenKind::LBrack),
    ("]", TokenKind::RBrack),
    // Other symbols
    (".", TokenKind::Dot),
    (",", TokenKind::Comma),
    (";", TokenKind::Semi),
    (":", TokenKind::Colon),
    (":=", TokenKind::ColonEq),
    ("?", TokenKind::Question),
    // Primitive types
    ("void", TokenKind::Void),
    ("int", TokenKind::Int),
    ("float", TokenKind::Float),
    ("string", TokenKind::String),
    ("byte", TokenKind::Byte),
    ("bool", TokenKind::Bool),
];

pub fn str_to_token(s: &str) -> Option<&TokenKind> {
    RESERVED.iter().find(|(kw, _)| *kw == s).map(|(_, t)| t)
}

pub fn token_to_str(t: TokenKind) -> Option<&'static str> {
    RESERVED
        .iter()
        .find(|(_, tok)| *tok == t)
        .map(|(kw, _)| *kw)
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenKind::Whitespace => panic!("whitespace tokens should be discarded"),

            TokenKind::Invalid => "INVALID",
            TokenKind::Eof => "EOF",
            TokenKind::Newline => "NEWLINE",

            // Literals
            TokenKind::IdentLit(ident) => &ident,
            TokenKind::IntLit(n) => &n.to_string(),
            TokenKind::FloatLit(f) => &f.to_string(),
            TokenKind::BoolLit(b) => &b.to_string(),
            TokenKind::StringLit(s) => s,
            TokenKind::CharLit(c) => &c.to_string(),

            k => token_to_str(k.clone()).expect("kind was not found in RESERVED map"),
        };

        write!(f, "{}", s)
    }
}
