//! Broad grouping of a language by purpose.

/// High-level category a [`crate::Language`] belongs to.
///
/// Used by editors and tooling to group or filter languages without caring
/// about the exact language identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageCategory {
    /// General-purpose programming language source.
    Code,
    /// Structured data (JSON, CSV, …).
    Data,
    /// Prose / documentation (Markdown, plain text, …).
    Document,
    /// Markup languages (HTML, XML, …).
    Markup,
    /// Configuration formats (INI, TOML, gitconfig, …).
    Config,
    /// Domain-specific languages.
    Dsl,
    /// Build systems and scripts (Makefile, …).
    Build,
    /// Dependency lock files.
    Lockfile,
    /// Anything that does not fit the categories above.
    Other,
}
