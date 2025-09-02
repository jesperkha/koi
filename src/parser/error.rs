use core::fmt;

use crate::token::{File, Token};

#[derive(Debug, Clone)]
pub struct ParserError {
    message: String,
    line: usize,
    line_str: String,
    from: usize,
    length: usize,
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = format!(
            "error: {}\n{:<3} | {}\n    | {}{}\n",
            self.message,
            self.line,
            self.line_str,
            " ".repeat(self.from),
            "^".repeat(self.length.max(1))
        );
        write!(f, "{}", err)
    }
}

impl ParserError {
    pub fn new(msg: &str, from: Token, to: Token, file: &File) -> ParserError {
        ParserError {
            message: msg.to_string(),
            line: from.pos.row + 1,
            line_str: file.line(from.pos.row).to_owned(),
            from: from.pos.col,
            length: to.end_pos.col - from.pos.col,
        }
    }
}
