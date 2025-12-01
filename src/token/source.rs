use std::fs::read_to_string;

#[derive(Debug)]
pub struct Source {
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
    pub fn new(filepath: String, src: Vec<u8>) -> Source {
        Source {
            filepath,
            lines: Source::get_line_beginnings(src.as_slice()),
            size: src.len(),
            src: src,
        }
    }

    /// Create new source file, reading the content from named file as source.
    pub fn new_from_file(filename: &str) -> Result<Source, String> {
        read_to_string(filename).map_or(Err(format!("failed to read file '{}'", filename)), |f| {
            Ok(Source::new(filename.to_string(), f.into_bytes()))
        })
    }

    pub fn new_from_string(src: &str) -> Source {
        Source::new("".to_string(), src.to_string().into_bytes())
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

            // Move to the start of the next line
            i = end + 1;
            if i < src.len() && src[i] == b'\n' {
                i += 1;
            }
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

    /// Returns the position of the character before the next newline,
    /// or the last character in `src` if none is found.
    fn find_end_of_line(src: &[u8], offset: usize) -> usize {
        if offset >= src.len() {
            return src.len().saturating_sub(1);
        }

        // If the current character is a newline, return previous character
        if src[offset] == b'\n' {
            return offset.saturating_sub(1);
        }

        // Search for the next newline after `offset`
        match src[offset..].iter().position(|&c| c == b'\n') {
            Some(pos) => offset + pos - 1,       // character before newline
            None => src.len().saturating_sub(1), // no newline found
        }
    }
}
