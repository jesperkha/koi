pub type SourceId = usize;

pub struct SourceMap {
    sources: Vec<Source>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    pub fn add(&mut self, filepath: String, src: Vec<u8>) {
        let source = Source::new(self.sources.len(), filepath, src);
        self.sources.push(source);
    }

    pub fn add_source(&mut self, source: Source) {
        self.sources.push(source);
    }

    pub fn get(&self, id: SourceId) -> Option<&Source> {
        self.sources.get(id)
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }
}

#[derive(Debug)]
pub struct Source {
    pub id: SourceId,
    pub filepath: String,
    /// File contents
    pub src: Vec<u8>,
    /// File size in bytes
    pub size: usize,
    /// List of byte offsets for first character in each line.
    pub lines: Vec<usize>,
}

impl Source {
    /// Create new file object using given source.
    pub fn new(id: SourceId, filepath: String, src: Vec<u8>) -> Source {
        Source {
            id,
            filepath,
            lines: Source::get_line_beginnings(src.as_slice()),
            size: src.len(),
            src: src,
        }
    }

    pub fn new_from_string(id: SourceId, src: &str) -> Source {
        Self::new(id, "".into(), src.to_string().into_bytes())
    }

    /// Gets a list of offsets for the first character of each line.
    /// First item will always be 0.
    fn get_line_beginnings(src: &[u8]) -> Vec<usize> {
        let mut lines = Vec::new();
        let mut i: usize = 0;

        while i < src.len() {
            lines.push(i);

            // Find the end of the current line
            let end = Self::find_end_of_line(src, i);

            if end == i {
                // Empty line, move to next character
                i += 1;
                continue;
            }

            // Move to the start of the next line
            i = end + 1;
        }

        // Guarantee at least one index
        if lines.is_empty() {
            lines.push(0);
        }

        lines
    }

    /// Get the source text at a given row (linenr -1).
    pub fn line(&self, row: usize) -> &str {
        // Tokens get their positions from the actual file
        // A failed assert here is a bug
        assert!(
            row < self.lines.len(),
            "row out of bounds: {} of {}",
            row,
            self.lines.len()
        );

        let start = self.lines[row];
        let end = Source::find_end_of_line(&self.src, start);
        self.str_range(start, end + 1) // Range is non-inclusive
    }

    /// Get string in range of (from, to) where both are byte offsets.
    /// Panics if to <= from.
    pub fn str_range(&self, from: usize, to: usize) -> &str {
        assert!(from <= to, "range (from, to) where to <= from");
        str::from_utf8(&self.src[from..to]).expect("invalid utf-8")
    }

    /// Returns the position of the next newline character
    /// or the last character in `src` if none is found.
    fn find_end_of_line(src: &[u8], offset: usize) -> usize {
        // Search for the next newline after `offset`
        match src[offset..].iter().position(|&c| c == b'\n') {
            Some(pos) => offset + pos, // character before newline
            None => src.len() - 1,     // no newline found
        }
    }
}
