use std::{ffi::OsStr, fs::File, io::Read, path::Path};

/// Extracts the executable name from a line with shebang.
///
/// It can handle both direct path to the executable and the use of `env` to look up $PATH.
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
        args.next()
    } else {
        Some(exec)
    }
}

#[non_exhaustive]
pub enum Language {
    Makefile,
    Python,
    Rust,
}

// language detection
#[allow(unused_variables)]
impl Language {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        Self::from_filename(path.file_name()?)
            .or_else(|| Self::from_extension(path.extension()?))
            .or_else(|| Self::from_first_line(path))

        // FEAT: LATER: detect by full path; e.g. `~/.ssh/config`
        // | - path.is_absolute()
        // | - path.canonicalize()

        // FIX: MAYBE: separate `from_path()` and `detect()`
        // | - `from_path()`: depends purely on the given path string
        // | - `detect()`: may attempt `File::read()` and `Path::canonicalize()`
    }

    pub fn from_filename<T: AsRef<OsStr>>(fname: T) -> Option<Self> {
        Self::from_filename_impl(fname.as_ref())
    }

    fn from_filename_impl(fname: &OsStr) -> Option<Self> {
        use Language::*;

        // STYLE: `Some(match fname {})`
        // TEST: ASAP: is it always valid to match inner bytes of `&str` and `&OsStr`?
        match fname.as_encoded_bytes() {
            b"Makefile" => Some(Makefile),
            _ => None,
        }
    }

    pub fn from_extension<T: AsRef<OsStr>>(ext: T) -> Option<Self> {
        Self::from_extension_impl(ext.as_ref())
    }

    fn from_extension_impl(ext: &OsStr) -> Option<Self> {
        use Language::*;

        match ext.as_encoded_bytes() {
            b"py" => Some(Python),
            b"rs" => Some(Rust),
            _ => None,
        }
    }

    pub fn from_executable<T: AsRef<OsStr>>(exec: T) -> Option<Self> {
        Self::from_executable_impl(exec.as_ref())
    }

    pub fn from_executable_impl(exec: &OsStr) -> Option<Self> {
        use Language::*;

        match exec.as_encoded_bytes() {
            b"python" | b"python3" => Some(Python),
            _ => None,
        }
    }

    pub fn from_first_line(path: &Path) -> Option<Self> {
        const READ_LIMIT: usize = 128;

        let mut file = File::open(path).ok()?;
        let mut buf = [0; READ_LIMIT];

        let len = file.read(&mut buf).ok()?;
        let buf = &buf[..len];

        let first_line = buf.split(|b| *b == b'\n').next()?;
        let first_line = std::str::from_utf8(first_line).ok()?;

        let exec = get_shebang_executable(first_line)?;
        Self::from_executable(exec)
    }
}

// language information
impl Language {
    pub fn name(&self) -> &'static str {
        todo!()
    }

    pub fn line_comments(&self) -> &'static [&'static str] {
        todo!()
    }

    pub fn block_comments(&self) -> &'static [(&'static str, &'static str)] {
        todo!()
    }

    // FEAT: make sub-crate with Nerd-Fonts glyphs as `const char`s
    pub fn icon(&self) -> Option<(char, (u8, u8, u8))> {
        todo!()
    }

    pub fn color(&self) -> Option<(u8, u8, u8)> {
        todo!()
    }

    // pub fn kind(&self) -> LanguageKind {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
