//! Curated special-path override rules (Issue #2).
//!
//! These exist for paths that extension/filename lookup alone cannot resolve,
//! e.g. globbed config trees or filenames that only mean something at a
//! particular path. Keeping them as data here avoids scattering special cases
//! through the detection functions.

use crate::language::Language;

/// A single override rule, matched against a normalized (`/`-separated) path.
#[derive(Debug, Clone, Copy)]
pub enum PathRule {
    /// The whole normalized path equals `path`.
    ExactPath {
        path: &'static str,
        language: Language,
    },
    /// The final path component equals `filename`.
    ExactFilename {
        filename: &'static str,
        language: Language,
    },
    /// The normalized path ends with `suffix`.
    Suffix {
        suffix: &'static str,
        language: Language,
    },
    /// The normalized path starts with `prefix`.
    Prefix {
        prefix: &'static str,
        language: Language,
    },
    /// The normalized path contains `needle`.
    Contains {
        needle: &'static str,
        language: Language,
    },
    /// The normalized path matches a glob (`*` within a segment, `**` across
    /// segments).
    Glob {
        pattern: &'static str,
        language: Language,
    },
}

impl PathRule {
    pub fn language(&self) -> Language {
        match self {
            PathRule::ExactPath { language, .. }
            | PathRule::ExactFilename { language, .. }
            | PathRule::Suffix { language, .. }
            | PathRule::Prefix { language, .. }
            | PathRule::Contains { language, .. }
            | PathRule::Glob { language, .. } => *language,
        }
    }
}

/// Override rules, evaluated by precedence groups in [`crate::detect`].
pub static PATH_RULES: &[PathRule] = &[
    PathRule::ExactPath {
        path: "etc/crontab",
        language: Language::CRONTAB,
    },
    PathRule::Suffix {
        suffix: ".git/config",
        language: Language::GIT_CONFIG,
    },
    PathRule::Suffix {
        suffix: "mpd.conf",
        language: Language::MPD_CONFIG,
    },
    PathRule::Glob {
        pattern: "nginx/**/*.conf",
        language: Language::NGINX,
    },
];

/// Match `pattern` against `path`, where `*` matches any run of characters
/// except `/`, and `**` matches any run including `/`. Both strings are assumed
/// already normalized (forward slashes, no duplicate separators).
pub fn glob_match(pattern: &str, path: &str) -> bool {
    glob_inner(pattern.as_bytes(), path.as_bytes())
}

fn glob_inner(pat: &[u8], text: &[u8]) -> bool {
    if pat.is_empty() {
        return text.is_empty();
    }
    if pat[0] == b'*' {
        if pat.len() >= 2 && pat[1] == b'*' {
            // `**` matches any run, including `/`.
            let rest = &pat[2..];
            for i in 0..=text.len() {
                if glob_inner(rest, &text[i..]) {
                    return true;
                }
            }
            // `**/foo` should also match `foo` with zero intervening dirs.
            if rest.first() == Some(&b'/') && glob_inner(&rest[1..], text) {
                return true;
            }
            return false;
        }
        // Single `*` matches any run of non-`/` characters (including empty).
        let rest = &pat[1..];
        let mut i = 0;
        loop {
            if glob_inner(rest, &text[i..]) {
                return true;
            }
            if i >= text.len() || text[i] == b'/' {
                return false;
            }
            i += 1;
        }
    }
    if !text.is_empty() && pat[0] == text[0] {
        return glob_inner(&pat[1..], &text[1..]);
    }
    false
}
