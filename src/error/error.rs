use core::fmt;

use crate::token::{File, Pos, Token};

#[derive(Debug, Clone)]
pub struct Error {
    /// Raw error message without formatting
    /// Eg. 'not declared'
    pub message: String,
    filename: String,

    line: usize,
    line_str: String,
    from: usize,
    length: usize,
}

#[derive(Debug)]
pub struct ErrorSet {
    errs: Vec<Error>,
}

pub type Res<T> = Result<T, ErrorSet>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err = format!(
            "{}\nerror: {}\n    |\n{:<3} | {}\n    | {}{}\n",
            self.filename,
            self.message,
            self.line,
            self.line_str,
            " ".repeat(self.from),
            "^".repeat(self.length.max(1)),
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
            filename: file.name.clone(),
        }
    }

    pub fn range(msg: &str, from: &Pos, to: &Pos, file: &File) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            from: from.col,
            length: to.col - from.col,
            filename: file.name.clone(),
        }
    }

    pub fn new_syntax(msg: &str, from: &Pos, length: usize, file: &File) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            from: from.col,
            length: length,
            filename: file.name.clone(),
        }
    }
}

impl fmt::Display for ErrorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.errs.iter().enumerate() {
            write!(f, "{}{}", err, if i == self.size() - 1 { "" } else { "\n" })?;
        }
        Ok(())
    }
}

impl ErrorSet {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }

    pub fn add(&mut self, err: Error) {
        self.errs.push(err);
    }

    pub fn size(&self) -> usize {
        self.errs.len()
    }

    pub fn get(&self, i: usize) -> &Error {
        &self.errs[i]
    }

    pub fn join(&mut self, other: ErrorSet) {
        self.errs.extend_from_slice(&other.errs);
    }
}
