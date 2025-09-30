use std::fs::read_to_string;

pub struct File {
    /// Name of file, including extension
    pub name: String,
    /// File contents
    pub src: Vec<u8>,
    /// File size in bytes
    pub size: usize,
    /// List of byte offsets for first character in each line.
    pub lines: Vec<usize>,
}

pub struct FileSet {
    pub files: Vec<File>,
}

impl File {
    /// Create new file object using given source.
    pub fn new(filename: String, src: Vec<u8>) -> File {
        File {
            lines: File::get_line_beginnings(src.as_slice()),
            name: filename,
            size: src.len() as usize,
            src: src,
        }
    }

    /// Create new file, reading the content from named file as source.
    pub fn new_from_file(filename: &str) -> File {
        let bytes = read_to_string(filename)
            .expect("failed to read file")
            .into_bytes();
        File::new(filename.to_string(), bytes)
    }

    /// Create new file for testing
    pub fn new_test_file(src: &str) -> File {
        File::new("test".to_string(), src.to_string().into_bytes())
    }

    /// Gets a list of offsets for first character of each line.
    /// First item will always be 0.
    fn get_line_beginnings(src: &[u8]) -> Vec<usize> {
        let mut lines = Vec::new();
        let mut i: usize = 0;

        while i < src.len() {
            lines.push(i);
            i = File::find_end_of_line(src, i);
            i += 2;
        }

        // Guarantee at least one index
        if lines.len() == 0 {
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
        let end = File::find_end_of_line(&self.src, start);
        self.str_range(start, end + 1) // Range is non-inclusive
    }

    /// Get string in range of (from, to) where both are byte offsets.
    /// Panics if to <= from.
    pub fn str_range(&self, from: usize, to: usize) -> &str {
        assert!(from <= to, "range (from, to) where to <= from");
        str::from_utf8(&self.src[from..to]).expect("invalid utf-8")
    }

    /// Returns the position of the character before the newline,
    /// or last character in source if none is found.
    fn find_end_of_line(src: &[u8], offset: usize) -> usize {
        if src[offset] == b'\n' {
            return if offset == 0 { 0 } else { offset - 1 };
        }

        src[offset..]
            .iter()
            .position(|&c| c == b'\n')
            .and_then(|n| {
                if n == 0 {
                    Some(offset)
                } else {
                    Some(offset + n - 1)
                }
            })
            .unwrap_or(src.len() - 1)
    }
}

impl FileSet {
    pub fn new() -> FileSet {
        FileSet { files: Vec::new() }
    }

    pub fn add(&mut self, file: File) {
        self.files.push(file);
    }
}
