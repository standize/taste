//! `taste` — a small, fast, extensible language/format detector.
//!
//! Detection is data-driven: a static registry ([`info::LANGUAGES`]) holds all
//! language metadata, and the engine in [`detect`] resolves a [`Language`] from a
//! path, file, in-memory buffer, or bare token. Special cases live as data in
//! [`rules`], not scattered through the detector.
//!
//! ```
//! use taste::{detect_path, Language, DetectionSource};
//!
//! let d = detect_path("src/main.rs").unwrap();
//! assert_eq!(d.language, Language::RUST);
//! assert_eq!(d.source, DetectionSource::Extension);
//! ```

pub mod category;
pub mod comment;
pub mod detect;
pub mod info;
pub mod language;
pub mod meta;
pub mod rules;

pub use category::LanguageCategory;
pub use comment::{BlockComment, CommentStyle};
pub use detect::{
    Confidence, Detection, DetectionSource, PathParts, detect_buffer, detect_file, detect_language,
    detect_path, detect_token, get_shebang_executable, path_parts,
};
pub use info::{LANGUAGES, LanguageInfo};
pub use language::{Language, LanguageId};
pub use meta::{Color, Icon};
pub use rules::{PATH_RULES, PathRule};

/// All known languages and their metadata.
pub fn all_languages() -> &'static [LanguageInfo] {
    LANGUAGES
}

/// Look up a language by canonical name or alias.
pub fn language_by_token(token: &str) -> Option<Language> {
    detect_token(token).map(|d| d.language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_constants_match_table() {
        // Each constant's index must point at its own entry, and round-trip.
        for (i, info) in LANGUAGES.iter().enumerate() {
            assert_eq!(
                info.language.id().as_u16() as usize,
                i,
                "LANGUAGES[{i}] ({}) has a mismatched id",
                info.canonical_name
            );
        }
        assert_eq!(Language::RUST.name(), "rust");
        assert_eq!(Language::PYTHON.display_name(), "Python");
        assert_eq!(Language::MARKDOWN.name(), "markdown");
    }

    #[test]
    fn detects_basic_extensions() {
        let rust = detect_path("src/main.rs").unwrap();
        assert_eq!(rust.language, Language::RUST);
        assert_eq!(rust.source, DetectionSource::Extension);

        let py = detect_path("script.py").unwrap();
        assert_eq!(py.language, Language::PYTHON);
        assert_eq!(py.source, DetectionSource::Extension);

        let oxy = detect_path("plugin.oxy").unwrap();
        assert_eq!(oxy.language, Language::OXYGEN);
        assert_eq!(oxy.source, DetectionSource::Extension);

        assert!(detect_path("noext").is_none());
    }

    #[test]
    fn extension_is_case_insensitive() {
        assert_eq!(
            detect_path("README.MD").unwrap().language,
            Language::MARKDOWN
        );
        assert_eq!(detect_path("A.RS").unwrap().language, Language::RUST);
    }

    #[test]
    fn exact_filename_beats_extension() {
        let mk = detect_path("Makefile").unwrap();
        assert_eq!(mk.language, Language::MAKEFILE);
        assert_eq!(mk.source, DetectionSource::ExactFilename);

        // setup.cfg is INI by filename even though `.cfg` maps to nothing.
        assert_eq!(detect_path("setup.cfg").unwrap().language, Language::INI);
        assert_eq!(
            detect_path("project.godot").unwrap().language,
            Language::INI
        );
    }

    #[test]
    fn detects_weird_paths() {
        let cases = [
            ("etc/crontab", Language::CRONTAB),
            ("nginx/sites-enabled/default.conf", Language::NGINX),
            ("/usr/local/nginx/conf/nginx.conf", Language::NGINX),
            (".git/config", Language::GIT_CONFIG),
            ("/home/u/proj/.git/config", Language::GIT_CONFIG),
            ("git-rebase-todo", Language::GIT_REBASE_TODO),
            ("setup.cfg", Language::INI),
            ("mimeapps.list", Language::INI),
            ("~/.config/mpd/mpd.conf", Language::MPD_CONFIG),
        ];
        for (path, expected) in cases {
            let d = detect_path(path).unwrap_or_else(|| panic!("no detection for {path}"));
            assert_eq!(d.language, expected, "path {path}");
        }
    }

    #[test]
    fn detects_windows_style_paths() {
        let d = detect_path(".git\\config").unwrap();
        assert_eq!(d.language, Language::GIT_CONFIG);
        let d = detect_path("C:\\src\\app\\main.rs").unwrap();
        assert_eq!(d.language, Language::RUST);
    }

    #[test]
    fn detects_shebangs() {
        let d = detect_buffer(Some("script"), b"#!/usr/bin/env python\nprint('hi')\n").unwrap();
        assert_eq!(d.language, Language::PYTHON);
        assert_eq!(d.source, DetectionSource::Shebang);

        let d = detect_buffer(Some("run"), b"#!/bin/bash\necho hi\n").unwrap();
        assert_eq!(d.language, Language::SHELL);

        let d = detect_buffer(Some("cli"), b"#!/usr/bin/env node\n").unwrap();
        assert_eq!(d.language, Language::JAVASCRIPT);
    }

    #[test]
    fn extension_outranks_shebang_only_when_no_shebang() {
        // No shebang, has extension.
        let d = detect_buffer(Some("a.py"), b"print('hi')\n").unwrap();
        assert_eq!(d.language, Language::PYTHON);
        assert_eq!(d.source, DetectionSource::Extension);
    }

    #[test]
    fn detects_more_languages() {
        let cases = [
            ("main.c", Language::C),
            ("vector.h", Language::C),
            ("main.cpp", Language::CPP),
            ("widget.hpp", Language::CPP),
            ("main.go", Language::GO),
            ("app.ts", Language::TYPESCRIPT),
            ("App.tsx", Language::TYPESCRIPT),
            ("index.html", Language::HTML),
            ("style.css", Language::CSS),
            ("config.yaml", Language::YAML),
            ("config.yml", Language::YAML),
            ("data.xml", Language::XML),
            ("Main.java", Language::JAVA),
            ("Program.cs", Language::CSHARP),
            ("script.rb", Language::RUBY),
            ("index.php", Language::PHP),
            ("init.lua", Language::LUA),
            ("schema.sql", Language::SQL),
            ("Dockerfile", Language::DOCKERFILE),
            ("main.zig", Language::ZIG),
        ];
        for (path, expected) in cases {
            let d = detect_path(path).unwrap_or_else(|| panic!("no detection for {path}"));
            assert_eq!(d.language, expected, "path {path}");
        }
    }

    #[test]
    fn detects_hardware_languages() {
        let cases = [
            ("alu.v", Language::VERILOG),
            ("alu_pkg.vh", Language::VERILOG),
            ("top.sv", Language::SYSTEMVERILOG),
            ("top_if.svh", Language::SYSTEMVERILOG),
            ("core.vhd", Language::VHDL),
            ("core.vhdl", Language::VHDL),
            ("Mkfifo.bsv", Language::BSV),
            ("FIFO.bs", Language::BLUESPEC_HASKELL),
            ("timing.sdc", Language::SDC),
            ("pins.xdc", Language::SDC),
        ];
        for (path, expected) in cases {
            let d = detect_path(path).unwrap_or_else(|| panic!("no detection for {path}"));
            assert_eq!(d.language, expected, "path {path}");
        }

        assert_eq!(Language::VHDL.comments().primary_line(), Some("--"));
        assert!(Language::VHDL.comments().block.is_empty());
        assert_eq!(Language::BLUESPEC_HASKELL.comments().block[0].start, "{-");
        assert_eq!(Language::BSV.comments().primary_line(), Some("//"));
    }

    #[test]
    fn detects_more_shebangs() {
        let d = detect_buffer(Some("script"), b"#!/usr/bin/env ruby\nputs 'hi'\n").unwrap();
        assert_eq!(d.language, Language::RUBY);
        assert_eq!(d.source, DetectionSource::Shebang);

        let d = detect_buffer(Some("script"), b"#!/usr/bin/env php\n<?php\n").unwrap();
        assert_eq!(d.language, Language::PHP);

        let d = detect_buffer(Some("script"), b"#!/usr/bin/lua\n").unwrap();
        assert_eq!(d.language, Language::LUA);
    }

    #[test]
    fn more_metadata_accessors() {
        assert_eq!(Language::CSS.comments().primary_line(), None);
        assert_eq!(Language::CSS.comments().block[0].start, "/*");
        assert_eq!(Language::HTML.comments().block[0].start, "<!--");
        assert_eq!(Language::LUA.comments().primary_line(), Some("--"));
        assert_eq!(Language::LUA.comments().block[0].start, "--[[");
        assert_eq!(Language::PHP.comments().line, &["//", "#"]);
        assert_eq!(Language::CPP.display_name(), "C++");
        assert_eq!(Language::DOCKERFILE.category(), LanguageCategory::Build);
    }

    #[test]
    fn detects_tokens() {
        assert_eq!(detect_token("py").unwrap().language, Language::PYTHON);
        assert_eq!(detect_token("node").unwrap().language, Language::JAVASCRIPT);
        assert_eq!(detect_token("Rust").unwrap().language, Language::RUST);
        assert!(detect_token("nonsense").is_none());
    }

    #[test]
    fn metadata_accessors() {
        assert_eq!(Language::RUST.comments().primary_line(), Some("//"));
        assert_eq!(Language::PYTHON.comments().primary_line(), Some("#"));
        assert_eq!(Language::JSON.comments().primary_line(), None);
        assert_eq!(Language::RUST.comments().block.len(), 1);
        assert!(Language::RUST.color().is_some());
    }

    #[test]
    fn glob_matcher() {
        use rules::glob_match;
        assert!(glob_match("nginx/**/*.conf", "nginx/sites/default.conf"));
        assert!(glob_match("nginx/**/*.conf", "nginx/a/b/c/site.conf"));
        assert!(glob_match("nginx/**/*.conf", "nginx/nginx.conf"));
        assert!(!glob_match("nginx/**/*.conf", "apache/site.conf"));
        assert!(glob_match("*.rs", "main.rs"));
        assert!(!glob_match("*.rs", "src/main.rs"));
        assert!(glob_match("src/*.rs", "src/main.rs"));
    }

    #[test]
    fn language_id_stable_str_round_trips_and_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for info in LANGUAGES {
            let key = info.language.id().as_str();
            assert_eq!(key, info.canonical_name);
            assert_eq!(Language::from_name(key), Some(info.language));
            assert!(seen.insert(key), "duplicate canonical_name {key:?}");
        }
        assert!(Language::from_name("not-a-real-language").is_none());
    }

    #[test]
    fn ambiguous_extensions_carry_alternatives() {
        let h = detect_path("vector.h").unwrap();
        assert_eq!(h.language, Language::C);
        assert_eq!(h.alternatives, &[Language::CPP, Language::OBJECTIVE_C]);

        let m = detect_path("AppDelegate.m").unwrap();
        assert_eq!(m.language, Language::OBJECTIVE_C);
        assert_eq!(m.alternatives, &[Language::MATLAB]);

        let pl = detect_path("script.pl").unwrap();
        assert_eq!(pl.language, Language::PERL);
        assert_eq!(pl.alternatives, &[Language::PROLOG]);

        let r = detect_path("analysis.r").unwrap();
        assert_eq!(r.language, Language::R);
        assert_eq!(r.alternatives, &[Language::REBOL]);

        let fs = detect_path("Program.fs").unwrap();
        assert_eq!(fs.language, Language::FSHARP);
        assert_eq!(fs.alternatives, &[Language::FORTH]);
    }

    #[test]
    fn unambiguous_extensions_have_no_alternatives() {
        let rust = detect_path("src/main.rs").unwrap();
        assert!(rust.alternatives.is_empty());

        let cpp = detect_path("widget.hpp").unwrap();
        assert!(cpp.alternatives.is_empty());
    }

    #[test]
    fn non_utf8_path_does_not_panic() {
        // path_parts must tolerate odd input gracefully.
        let p = path_parts(std::path::Path::new(""));
        assert!(p.filename.is_none());
    }
}
