use std::fmt;

pub struct Token {
    pub kind: TokenKind,

    /// The token as a string literal
    pub lexeme: String,
    /// Position of first character in token.
    pub pos: Pos,
    /// Position of character immediately after token
    pub end_pos: Pos,
    /// Is the token an EOF token?
    pub eof: bool,
    /// Is the token invalid?
    pub invalid: bool,
}

pub struct Pos {
    /// Row in file, starting at 0
    pub row: i32,
    /// Column on line, starting at 0
    pub col: i32,
    /// Byte offset in file
    pub offset: i32,
    /// Offset of first character on same line as this Pos
    pub line_begin: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Invalid,
    Newline,
    Eof,

    // Literals, contain the literal value
    IdentLit(String),
    IntLit(i64),
    FloatLit(f64),
    BoolLit(bool),
    StringLit(String),
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

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
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

            // Keywords
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Return => "return",
            TokenKind::Func => "func",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::For => "for",
            TokenKind::Import => "import",
            TokenKind::Package => "package",
            TokenKind::Null => "null",
            TokenKind::Pub => "pub",
            TokenKind::Error => "error",

            // Math
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",

            // Logic
            TokenKind::Eq => "=",
            TokenKind::EqEq => "==",
            TokenKind::NotEq => "!=",
            TokenKind::PlusEq => "+=",
            TokenKind::MinusEq => "-=",
            TokenKind::StarEq => "*=",
            TokenKind::SlashEq => "/=",
            TokenKind::Greater => ">",
            TokenKind::Less => "<",
            TokenKind::GreaterEq => ">=",
            TokenKind::LessEq => "<=",
            TokenKind::Or => "|",
            TokenKind::OrOr => "||",
            TokenKind::And => "&",
            TokenKind::AndAnd => "&&",
            TokenKind::Bang => "!",
            TokenKind::BangEq => "!=",

            // Parenthesis & Brackets
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBrack => "[",
            TokenKind::RBrack => "]",

            // Other symbols
            TokenKind::Dot => ".",
            TokenKind::Comma => ",",
            TokenKind::Semi => ";",
            TokenKind::Colon => ":",
            TokenKind::ColonEq => ":=",
            TokenKind::Question => "?",

            // Primitive types
            TokenKind::Void => "void",
            TokenKind::Int => "int",
            TokenKind::Float => "float",
            TokenKind::String => "string",
            TokenKind::Byte => "byte",
            TokenKind::Bool => "bool",
        };

        write!(f, "{}", s)
    }
}
