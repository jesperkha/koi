use std::{
    collections::{HashMap, hash_map::Values},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::util::FilePath;

pub type SourceId = usize;

static SOURCE_ID: AtomicUsize = AtomicUsize::new(0);

fn next_source_id() -> usize {
    SOURCE_ID.fetch_add(1, Ordering::Relaxed)
}

pub struct SourceMap {
    map: HashMap<SourceId, Source>,
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn one(source: Source) -> Self {
        let mut map = Self::new();
        map.add(source);
        map
    }

    pub fn add(&mut self, source: Source) {
        self.map.insert(source.id, source);
    }

    pub fn get(&self, id: SourceId) -> Option<&Source> {
        self.map.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn sources(&self) -> Values<'_, usize, Source> {
        self.map.values()
    }

    pub fn join(&mut self, other: SourceMap) {
        self.map.extend(other.map);
    }
}

#[derive(Debug)]
pub struct Source {
    pub id: SourceId,
    pub filepath: FilePath,
    pub src: Vec<u8>,
    pub size: usize,
    pub lines: Vec<usize>,
}

impl Source {
    pub fn new(filepath: FilePath, src: Vec<u8>) -> Source {
        Source {
            id: next_source_id(),
            filepath,
            lines: Source::get_line_beginnings(src.as_slice()),
            size: src.len(),
            src,
        }
    }

    pub fn new_str(filepath: String, src: String) -> Source {
        Self::new(filepath.into(), src.into_bytes())
    }

    fn get_line_beginnings(src: &[u8]) -> Vec<usize> {
        let mut lines = Vec::new();
        let mut i: usize = 0;

        while i < src.len() {
            lines.push(i);
            let end = Self::find_end_of_line(src, i);
            if end == i {
                i += 1;
                continue;
            }
            i = end + 1;
        }

        if lines.is_empty() {
            lines.push(0);
        }

        lines
    }

    pub fn line(&self, row: usize) -> &str {
        assert!(
            row < self.lines.len(),
            "row out of bounds: {} of {}",
            row,
            self.lines.len()
        );

        let start = self.lines[row];
        let end = Source::find_end_of_line(&self.src, start);
        self.str_range(start, end + 1)
    }

    pub fn str_range(&self, from: usize, to: usize) -> &str {
        assert!(from <= to, "range (from, to) where to <= from");
        str::from_utf8(&self.src[from..to]).expect("invalid utf-8")
    }

    fn find_end_of_line(src: &[u8], offset: usize) -> usize {
        match src[offset..].iter().position(|&c| c == b'\n') {
            Some(pos) => offset + pos,
            None => src.len() - 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Pos {
    pub row: usize,
    pub col: usize,
    pub offset: usize,
    pub line_begin: usize,
    pub source_id: SourceId,
}
