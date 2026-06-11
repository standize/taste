//! Language identity.
//!
//! Languages are identified by a small opaque [`LanguageId`] rather than a huge
//! public enum, so the registry can grow (eventually generated) without churning
//! the public API. Common languages are exposed as `const` values on
//! [`Language`].

use crate::category::LanguageCategory;
use crate::comment::CommentStyle;
use crate::info::{LANGUAGES, LanguageInfo};
use crate::meta::{Color, Icon};

/// Opaque, stable identifier for a language.
///
/// The numeric value is the index into the internal registry table; treat it as
/// opaque outside the crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LanguageId(pub(crate) u16);

impl LanguageId {
    /// The raw index. Exposed for serialization/debugging; not a stable wire id.
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}

/// A detected language. Cheap to copy; carries only its [`LanguageId`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Language {
    id: LanguageId,
}

impl Language {
    pub(crate) const fn from_index(index: u16) -> Self {
        Self {
            id: LanguageId(index),
        }
    }

    pub const fn id(self) -> LanguageId {
        self.id
    }

    /// Full metadata record for this language.
    pub fn info(self) -> &'static LanguageInfo {
        &LANGUAGES[self.id.0 as usize]
    }

    /// Canonical (machine) name, e.g. `"rust"`.
    pub fn name(self) -> &'static str {
        self.info().canonical_name
    }

    /// Human-facing name, e.g. `"Rust"`.
    pub fn display_name(self) -> &'static str {
        self.info().display_name
    }

    pub fn category(self) -> LanguageCategory {
        self.info().category
    }

    pub fn comments(self) -> CommentStyle {
        self.info().comments
    }

    pub fn color(self) -> Option<Color> {
        self.info().color
    }

    pub fn icon(self) -> Option<Icon> {
        self.info().icon
    }
}

// Common-language constants. The index MUST match the order of `LANGUAGES` in
// `info.rs`; the `language_constants_match_table` test enforces this.
impl Language {
    pub const RUST: Language = Language::from_index(0);
    pub const PYTHON: Language = Language::from_index(1);
    pub const MAKEFILE: Language = Language::from_index(2);
    pub const SHELL: Language = Language::from_index(3);
    pub const JAVASCRIPT: Language = Language::from_index(4);
    pub const INI: Language = Language::from_index(5);
    pub const GIT_CONFIG: Language = Language::from_index(6);
    pub const GIT_REBASE_TODO: Language = Language::from_index(7);
    pub const NGINX: Language = Language::from_index(8);
    pub const CRONTAB: Language = Language::from_index(9);
    pub const MPD_CONFIG: Language = Language::from_index(10);
    pub const TOML: Language = Language::from_index(11);
    pub const JSON: Language = Language::from_index(12);
    pub const MARKDOWN: Language = Language::from_index(13);
    pub const C: Language = Language::from_index(14);
    pub const CPP: Language = Language::from_index(15);
    pub const GO: Language = Language::from_index(16);
    pub const TYPESCRIPT: Language = Language::from_index(17);
    pub const HTML: Language = Language::from_index(18);
    pub const CSS: Language = Language::from_index(19);
    pub const YAML: Language = Language::from_index(20);
    pub const XML: Language = Language::from_index(21);
    pub const JAVA: Language = Language::from_index(22);
    pub const CSHARP: Language = Language::from_index(23);
    pub const RUBY: Language = Language::from_index(24);
    pub const PHP: Language = Language::from_index(25);
    pub const LUA: Language = Language::from_index(26);
    pub const SQL: Language = Language::from_index(27);
    pub const DOCKERFILE: Language = Language::from_index(28);
    pub const ZIG: Language = Language::from_index(29);
    pub const OXYGEN: Language = Language::from_index(30);
    pub const VERILOG: Language = Language::from_index(31);
    pub const SYSTEMVERILOG: Language = Language::from_index(32);
    pub const VHDL: Language = Language::from_index(33);
    pub const BSV: Language = Language::from_index(34);
    pub const BLUESPEC_HASKELL: Language = Language::from_index(35);
    pub const SDC: Language = Language::from_index(36);
}
