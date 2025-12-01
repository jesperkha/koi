use core::fmt;

use crate::token::{Pos, Source, Token};

// TODO: compact errors based on config

#[derive(Debug, Clone)]
pub struct Error {
    /// Raw error message without formatting
    /// Eg. 'not declared'
    pub message: String,
    filename: String,

    line: usize,
    line_str: String,
    length: usize,
    from: usize,

    info: String,
}

impl Error {
    pub fn new(msg: &str, from: &Token, to: &Token, file: &Source) -> Error {
        Error {
            message: msg.to_string(),
            line: from.pos.row + 1,
            line_str: file.line(from.pos.row).to_owned(),
            length: to.end_pos.col - from.pos.col,
            filename: file.filepath.clone(),
            info: String::new(),
            from: from.pos.col,
        }
    }

    pub fn range(msg: &str, from: &Pos, to: &Pos, file: &Source) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            length: to.col - from.col,
            filename: file.filepath.clone(),
            info: String::new(),
            from: from.col,
        }
    }

    pub fn new_syntax(msg: &str, from: &Pos, length: usize, file: &Source) -> Error {
        Error {
            message: msg.to_string(),
            line: from.row + 1,
            line_str: file.line(from.row).to_owned(),
            length: length,
            filename: file.filepath.clone(),
            info: String::new(),
            from: from.col,
        }
    }

    pub fn with_info(mut self, info: &str) -> Self {
        self.info = info.to_string();
        self
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pad = self.line_str.len() - self.line_str.trim_start().len();
        let point_start = if self.from < pad { 1 } else { self.from - pad };

        let err = format!(
            "{}\nerror: {}\n    |\n{:<3} |    {}\n    |    {}{}\n{}",
            self.filename,
            self.message,
            self.line,
            self.line_str.trim(),
            " ".repeat(point_start),
            "^".repeat(self.length.max(1)),
            if !self.info.is_empty() {
                let info = &self.info;
                format!("    |\n    | {}\n", info)
            } else {
                "".to_string()
            }
        );
        write!(f, "{}", err)
    }
}

#[derive(Debug)]
pub struct ErrorSet {
    errs: Vec<Error>,
}

impl ErrorSet {
    pub fn new() -> Self {
        Self { errs: Vec::new() }
    }

    pub fn new_from(err: Error) -> Self {
        let mut s = Self::new();
        s.add(err);
        s
    }

    /// Return this as Err value if more than one error is contained, else Ok(v).
    pub fn err_or<T>(self, v: T) -> Result<T, Self> {
        if self.len() > 0 { Err(self) } else { Ok(v) }
    }

    pub fn add(&mut self, err: Error) {
        self.errs.push(err);
    }

    pub fn len(&self) -> usize {
        self.errs.len()
    }

    pub fn get(&self, i: usize) -> &Error {
        &self.errs[i]
    }

    pub fn join(&mut self, other: ErrorSet) {
        self.errs.extend_from_slice(&other.errs);
    }
}

pub type Res<T> = Result<T, ErrorSet>;

impl fmt::Display for ErrorSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, err) in self.errs.iter().enumerate() {
            write!(f, "{}{}", err, if i == self.len() - 1 { "" } else { "\n" })?;
        }
        Ok(())
    }
}
