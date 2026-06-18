/// Byte span in an AFF source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset.
    pub end: usize,
}

impl Span {
    /// Creates a byte span.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Joins two spans into one span covering both.
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// One-based source location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineColumn {
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
}

/// Maps byte spans to line and column locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    line_starts: Vec<usize>,
}

impl SourceMap {
    /// Builds a source map for the provided source.
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { line_starts }
    }

    /// Returns a one-based line and column for the byte offset.
    #[must_use]
    pub fn line_column(&self, offset: usize) -> LineColumn {
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        LineColumn {
            line: line_index + 1,
            column: offset.saturating_sub(self.line_starts[line_index]) + 1,
        }
    }
}
