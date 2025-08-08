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

    /// Create new file using text source for testing.
    pub fn new_test(src: &str) -> File {
        File {
            name: "test_file".to_string(),
            size: src.len() as usize,
            src: src.to_string().into_bytes(),
            lines: File::get_line_beginnings(src.as_bytes()),
        }
    }

    /// Returns the position of the character before the newline,
    /// or last character in source if none is found.
    fn find_end_of_line(src: &[u8], offset: usize) -> usize {
        src[offset..]
            .iter()
            .position(|&c| c == b'\n')
            .and_then(|n| Some(offset + n - 1))
            .unwrap_or(src.len() - 1)
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
        assert!(row < self.lines.len(), "row out of bounds");

        let start = self.lines[row];
        let end = File::find_end_of_line(&self.src, start);
        str::from_utf8(&self.src[start..end]).expect("Expected valid UTF-8")
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
