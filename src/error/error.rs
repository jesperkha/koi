use core::fmt;

use crate::token::{File, Pos, Token};

#[derive(Debug, Clone)]
pub struct Error {
    /// Raw error message without formatting
    /// Eg. 'not declared'
    pub message: String,

    line: usize,
    line_str: String,
    from: usize,
    length: usize,
}

impl fmt::Display for Error {
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

impl Error {
    pub fn new(msg: &str, from: &Token, to: &Token, file: &File) -> Error {
        Error {
            message: msg.to_string(),
            line: from.pos.row + 1,
            line_str: file.line(from.pos.row).to_owned(),
            from: from.pos.col,
            length: to.end_pos.col - from.pos.col,
        }
    }

    pub fn range(msg: &str, from: &Pos, to: &Pos, file: &File) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            from: from.col,
            length: to.col - from.col,
        }
    }

    pub fn new_syntax(msg: &str, from: &Pos, length: usize, file: &File) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            from: from.col,
            length: length,
        }
    }
}
