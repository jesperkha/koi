use crate::token::{Pos, Source, SourceId, SourceMap};

pub type Res<T> = Result<T, Diagnostics>;

pub struct Report {
    pub message: String,
    pub pos: Pos,
    pub length: usize,
}

impl Report {
    pub fn new(msg: &str, from: &Pos, to: &Pos) -> Self {
        Self {
            message: msg.to_owned(),
            pos: from.clone(),
            length: to.col - from.col,
        }
    }

    pub fn new_length(msg: &str, from: &Pos, length: usize) -> Self {
        Self {
            message: msg.to_owned(),
            pos: from.clone(),
            length,
        }
    }

    fn source_id(&self) -> SourceId {
        self.pos.source_id
    }

    fn render(&self, source: &Source) -> String {
        let line = self.pos.row + 1;
        let info = "";

        let line_str = source.line(self.pos.row).to_owned();
        let from = self.pos.col;

        let pad = line_str.len() - line_str.trim_start().len();
        let point_start = if from < pad { 1 } else { from - pad };

        format!(
            "{}\nerror: {}\n    |\n{:<3} |    {}\n    |    {}{}\n{}",
            source.filepath,
            self.message,
            line,
            line_str.trim(),
            " ".repeat(point_start),
            "^".repeat(self.length.max(1)),
            if !info.is_empty() {
                let info = &info;
                format!("    |\n    | {}\n", info)
            } else {
                "".to_string()
            }
        )
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
            let source = map.get(report.source_id()).unwrap();
            s += &report.render(source);
        }

        s
    }

    pub fn report(&self, map: &SourceMap) -> Result<(), String> {
        Err(self.render(map))
    }
}
