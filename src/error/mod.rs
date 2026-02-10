use crate::token::{Pos, SourceMap};

pub type Res<T> = Result<T, Diagnostics>;

pub struct Report {
    pub message: String,
    info: Option<String>, // Additional info if any
    kind: ReportKind,
}

enum ReportKind {
    Error,
    CodeError { pos: Pos, length: usize },
}

impl Report {
    /// Create new plain string error.
    pub fn error(msg: &str) -> Self {
        Self {
            message: msg.to_owned(),
            kind: ReportKind::Error,
            info: None,
        }
    }

    /// Create new code error marking a section of code in the range from-to.
    pub fn code_error(msg: &str, from: &Pos, to: &Pos) -> Self {
        Self {
            message: msg.to_owned(),
            kind: ReportKind::CodeError {
                pos: from.clone(),
                length: to.col - from.col,
            },
            info: None,
        }
    }

    /// Create new code error with a given length instead of end pos.
    pub fn code_error_len(msg: &str, from: &Pos, length: usize) -> Self {
        Self {
            message: msg.to_owned(),
            kind: ReportKind::CodeError {
                pos: from.clone(),
                length,
            },
            info: None,
        }
    }

    /// Append info message to this Report. Returns self for chaining.
    pub fn with_info(mut self, info: &str) -> Self {
        self.info = Some(info.to_string());
        self
    }

    fn render(&self, map: &SourceMap) -> String {
        match &self.kind {
            ReportKind::Error => format!("error: {}", self.message),
            ReportKind::CodeError { pos, length } => {
                let source = map.get(pos.source_id).unwrap();

                let line = pos.row + 1;
                let info = self.info.as_ref().map_or("", |s| s.as_str());
                let length = *length;

                let line_str = source.line(pos.row).to_owned();
                let from = pos.col;

                let pad = line_str.len() - line_str.trim_start().len();
                let point_start = if from < pad { 1 } else { from - pad };

                format!(
                    "{}\nerror: {}\n    |\n{:<3} |    {}\n    |    {}{}\n{}",
                    source.filepath,
                    self.message,
                    line,
                    line_str.trim(),
                    " ".repeat(point_start),
                    "^".repeat(length.max(1)),
                    if !info.is_empty() {
                        let info = &info;
                        format!("    |\n    | {}\n", info)
                    } else {
                        "".to_string()
                    }
                )
            }
        }
    }
}

pub struct Diagnostics {
    reports: Vec<Report>,
}

impl Diagnostics {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }

    pub fn get(&self, index: usize) -> &Report {
        &self.reports[index]
    }

    pub fn num_errors(&self) -> usize {
        self.reports.len()
    }

    pub fn add(&mut self, report: Report) {
        self.reports.push(report);
    }

    pub fn render(&self, map: &SourceMap) -> String {
        let mut s = String::new();
        for report in &self.reports {
            s += &report.render(map);
        }

        s
    }

    pub fn report(&self, map: &SourceMap) -> Result<(), String> {
        Err(self.render(map))
    }
}
