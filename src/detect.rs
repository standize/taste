//! The detection engine: path, file, buffer, and token detection.
//!
//! Path-only detection ([`detect_path`]) never touches the filesystem.
//! [`detect_file`] may read the start of the file (for shebangs); [`detect_buffer`]
//! works on in-memory content. All three return a rich [`Detection`] so callers
//! can see *how* a language was identified, not just *which*.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::info::LANGUAGES;
use crate::language::Language;
use crate::rules::{PATH_RULES, PathRule, glob_match};

/// How a language was identified, in rough precedence order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionSource {
    ExactFilename,
    SpecialPath,
    Shebang,
    Extension,
    Token,
    Content,
    Fallback,
}

/// Confidence in a detection result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Low,
    Medium,
    High,
    Exact,
}

/// A detection result: the language plus provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detection {
    pub language: Language,
    pub source: DetectionSource,
    pub confidence: Confidence,
    /// Other languages that also claim the same extension, in priority order
    /// (most likely alternative first). Empty unless `source` is
    /// [`DetectionSource::Extension`] and the extension is ambiguous (e.g.
    /// `.h`, `.m`, `.pl`, `.r`, `.fs`).
    pub alternatives: &'static [Language],
}

impl Detection {
    fn new(language: Language, source: DetectionSource, confidence: Confidence) -> Self {
        Self {
            language,
            source,
            confidence,
            alternatives: &[],
        }
    }

    fn with_alternatives(
        language: Language,
        source: DetectionSource,
        confidence: Confidence,
        alternatives: &'static [Language],
    ) -> Self {
        Self {
            language,
            source,
            confidence,
            alternatives,
        }
    }
}

// ---------------------------------------------------------------------------
// Path normalization
// ---------------------------------------------------------------------------

/// Components of a path useful for detection.
#[derive(Debug, Clone)]
pub struct PathParts {
    /// `/`-separated path with duplicate separators collapsed. `None` if the
    /// path is not valid UTF-8.
    pub normalized: Option<String>,
    /// Final path component, if any.
    pub filename: Option<String>,
    /// Lowercased final extension (without the dot), if any.
    pub extension: Option<String>,
}

/// Normalize a path for matching: `\` → `/`, collapse duplicate slashes, and
/// split out filename + extension. Never panics on non-UTF-8 input.
pub fn path_parts(path: &Path) -> PathParts {
    let Some(raw) = path.to_str() else {
        return PathParts {
            normalized: None,
            filename: None,
            extension: None,
        };
    };

    let mut normalized = String::with_capacity(raw.len());
    let mut prev_slash = false;
    for ch in raw.chars() {
        let is_slash = ch == '/' || ch == '\\';
        if is_slash {
            if !prev_slash {
                normalized.push('/');
            }
            prev_slash = true;
        } else {
            normalized.push(ch);
            prev_slash = false;
        }
    }

    let filename = normalized.rsplit('/').next().filter(|s| !s.is_empty());
    let extension = filename.and_then(|f| {
        // A leading dot is part of a dotfile name, not an extension.
        f.rsplit_once('.')
            .filter(|(stem, _)| !stem.is_empty())
            .map(|(_, ext)| ext.to_ascii_lowercase())
    });
    let filename = filename.map(str::to_string);

    PathParts {
        normalized: Some(normalized),
        filename,
        extension,
    }
}

// ---------------------------------------------------------------------------
// Registry lookups (linear scans over the static table; small N)
// ---------------------------------------------------------------------------

fn by_filename(filename: &str) -> Option<Language> {
    LANGUAGES
        .iter()
        .find(|info| info.filenames.contains(&filename))
        .map(|info| info.language)
}

/// Extensions claimed by more than one registry entry, with every candidate
/// language in priority order (most likely match first). Checked before the
/// general linear scan so a contested extension resolves to a deliberate
/// choice plus alternatives, instead of whichever entry happens to come first
/// in [`LANGUAGES`].
static AMBIGUOUS_EXTENSIONS: &[(&str, &[Language])] = &[
    ("h", &[Language::C, Language::CPP, Language::OBJECTIVE_C]),
    ("m", &[Language::OBJECTIVE_C, Language::MATLAB]),
    ("pl", &[Language::PERL, Language::PROLOG]),
    ("r", &[Language::R, Language::REBOL]),
    ("fs", &[Language::FSHARP, Language::FORTH]),
];

/// Resolve an extension to its primary language plus any alternatives, in
/// priority order.
fn by_extension(ext: &str) -> Option<(Language, &'static [Language])> {
    for (amb_ext, candidates) in AMBIGUOUS_EXTENSIONS {
        if amb_ext.eq_ignore_ascii_case(ext) {
            let (primary, alternatives) = candidates.split_first()?;
            return Some((*primary, alternatives));
        }
    }
    LANGUAGES
        .iter()
        .find(|info| info.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
        .map(|info| (info.language, &[][..]))
}

fn by_shebang(exec: &str) -> Option<Language> {
    LANGUAGES
        .iter()
        .find(|info| info.shebangs.contains(&exec))
        .map(|info| info.language)
}

fn by_token(token: &str) -> Option<Language> {
    let lower = token.to_ascii_lowercase();
    LANGUAGES
        .iter()
        .find(|info| info.canonical_name == lower || info.aliases.iter().any(|a| *a == lower))
        .map(|info| info.language)
}

fn special_path(parts: &PathParts) -> Option<(Language, Confidence)> {
    let normalized = parts.normalized.as_deref()?;
    // High-precedence exact rules first.
    for rule in PATH_RULES {
        match rule {
            PathRule::ExactPath { path, language } if *path == normalized => {
                return Some((*language, Confidence::Exact));
            }
            PathRule::ExactFilename { filename, language }
                if parts.filename.as_deref() == Some(*filename) =>
            {
                return Some((*language, Confidence::Exact));
            }
            _ => {}
        }
    }
    // Then fuzzy rules.
    for rule in PATH_RULES {
        let hit = match rule {
            PathRule::Suffix { suffix, .. } => normalized.ends_with(suffix),
            PathRule::Prefix { prefix, .. } => normalized.starts_with(prefix),
            PathRule::Contains { needle, .. } => normalized.contains(needle),
            PathRule::Glob { pattern, .. } => glob_match(pattern, normalized),
            _ => false,
        };
        if hit {
            return Some((rule.language(), Confidence::High));
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Detect from a path string alone, without touching the filesystem.
pub fn detect_path<P: AsRef<Path>>(path: P) -> Option<Detection> {
    let parts = path_parts(path.as_ref());
    detect_from_parts(&parts)
}

fn detect_from_parts(parts: &PathParts) -> Option<Detection> {
    // 1. exact filename
    if let Some(fname) = parts.filename.as_deref()
        && let Some(lang) = by_filename(fname)
    {
        return Some(Detection::new(
            lang,
            DetectionSource::ExactFilename,
            Confidence::Exact,
        ));
    }
    // 2. special path rules
    if let Some((lang, conf)) = special_path(parts) {
        return Some(Detection::new(lang, DetectionSource::SpecialPath, conf));
    }
    // 3. extension
    if let Some(ext) = parts.extension.as_deref()
        && let Some((lang, alternatives)) = by_extension(ext)
    {
        return Some(Detection::with_alternatives(
            lang,
            DetectionSource::Extension,
            Confidence::Medium,
            alternatives,
        ));
    }
    None
}

/// Detect from a file on disk. Reads the first bytes for shebang detection when
/// path-based rules are inconclusive.
pub fn detect_file<P: AsRef<Path>>(path: P) -> Option<Detection> {
    let path = path.as_ref();
    let parts = path_parts(path);

    // Exact filename and special paths outrank a shebang.
    if let Some(fname) = parts.filename.as_deref()
        && let Some(lang) = by_filename(fname)
    {
        return Some(Detection::new(
            lang,
            DetectionSource::ExactFilename,
            Confidence::Exact,
        ));
    }
    if let Some((lang, conf)) = special_path(&parts) {
        return Some(Detection::new(lang, DetectionSource::SpecialPath, conf));
    }

    // Shebang outranks extension for scripts.
    if let Some(det) = read_first_line(path).and_then(|line| shebang_detection(&line)) {
        return Some(det);
    }

    if let Some(ext) = parts.extension.as_deref()
        && let Some((lang, alternatives)) = by_extension(ext)
    {
        return Some(Detection::with_alternatives(
            lang,
            DetectionSource::Extension,
            Confidence::Medium,
            alternatives,
        ));
    }
    None
}

/// Detect from in-memory content, with an optional path for filename/extension
/// hints. Useful for editors, LSPs, and unsaved buffers.
pub fn detect_buffer<P: AsRef<Path>>(path: Option<P>, content: &[u8]) -> Option<Detection> {
    let parts = path.as_ref().map(|p| path_parts(p.as_ref()));

    if let Some(parts) = &parts {
        if let Some(fname) = parts.filename.as_deref()
            && let Some(lang) = by_filename(fname)
        {
            return Some(Detection::new(
                lang,
                DetectionSource::ExactFilename,
                Confidence::Exact,
            ));
        }
        if let Some((lang, conf)) = special_path(parts) {
            return Some(Detection::new(lang, DetectionSource::SpecialPath, conf));
        }
    }

    if let Some(det) = first_line_bytes(content)
        .as_deref()
        .and_then(shebang_detection)
    {
        return Some(det);
    }

    if let Some(parts) = &parts
        && let Some(ext) = parts.extension.as_deref()
        && let Some((lang, alternatives)) = by_extension(ext)
    {
        return Some(Detection::with_alternatives(
            lang,
            DetectionSource::Extension,
            Confidence::Medium,
            alternatives,
        ));
    }
    None
}

/// Detect from a bare token: a language name or alias (e.g. `"py"`, `"node"`).
pub fn detect_token(token: &str) -> Option<Detection> {
    by_token(token).map(|lang| Detection::new(lang, DetectionSource::Token, Confidence::High))
}

/// Simple wrapper returning only the language.
pub fn detect_language<P: AsRef<Path>>(path: P) -> Option<Language> {
    detect_path(path).map(|d| d.language)
}

// ---------------------------------------------------------------------------
// Shebang
// ---------------------------------------------------------------------------

/// Extract the executable name from a shebang line.
///
/// Handles both a direct path and an `env` lookup.
///
/// # Examples
///
/// ```
/// use taste::get_shebang_executable;
///
/// assert_eq!(get_shebang_executable("#!/bin/bash"), Some("bash"));
/// assert_eq!(get_shebang_executable("#!/usr/bin/env python"), Some("python"));
/// ```
pub fn get_shebang_executable(line: &str) -> Option<&str> {
    let shebang = line.strip_prefix("#!")?;
    let mut args = shebang.split_ascii_whitespace();
    let path = args.next()?;
    let exec = path.split('/').next_back()?;

    if exec == "env" {
        // Skip any `-S`/`VAR=val` style env arguments to reach the interpreter.
        args.find(|a| !a.starts_with('-') && !a.contains('='))
    } else {
        Some(exec)
    }
}

fn shebang_detection(line: &str) -> Option<Detection> {
    let exec = get_shebang_executable(line)?;
    // Strip a trailing version suffix is handled by registry entries directly.
    by_shebang(exec).map(|lang| Detection::new(lang, DetectionSource::Shebang, Confidence::High))
}

const READ_LIMIT: usize = 128;

fn read_first_line(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buf = [0u8; READ_LIMIT];
    let len = file.read(&mut buf).ok()?;
    first_line_bytes(&buf[..len])
}

fn first_line_bytes(buf: &[u8]) -> Option<String> {
    let line = buf.split(|b| *b == b'\n').next()?;
    std::str::from_utf8(line).ok().map(str::to_string)
}
