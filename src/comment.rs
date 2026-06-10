//! Comment syntax metadata.

/// A block-comment delimiter pair, e.g. `/*` … `*/`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockComment {
    pub start: &'static str,
    pub end: &'static str,
}

impl BlockComment {
    pub const fn new(start: &'static str, end: &'static str) -> Self {
        Self { start, end }
    }
}

/// The comment styles a language supports.
///
/// `line` lists line-comment prefixes (most languages have exactly one).
/// `block` lists block-comment delimiter pairs (often empty).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentStyle {
    pub line: &'static [&'static str],
    pub block: &'static [BlockComment],
}

impl CommentStyle {
    /// A language with no known comment syntax.
    pub const NONE: CommentStyle = CommentStyle {
        line: &[],
        block: &[],
    };

    pub const fn new(line: &'static [&'static str], block: &'static [BlockComment]) -> Self {
        Self { line, block }
    }

    /// The primary (first) line-comment prefix, if any.
    pub fn primary_line(&self) -> Option<&'static str> {
        self.line.first().copied()
    }
}
