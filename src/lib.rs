use std::{fs::File, io::Read, path::Path};

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
    pub fn from_path(path: &Path) -> Option<Self> {
        // FEAT: ASAP: implement `from_path()` without too much `OsStr::to_str()`
        todo!()
    }

    pub fn from_filename(fname: &str) -> Option<Self> {
        use Language::*;

        match fname {
            "Makefile" => Some(Makefile),
            _ => None,
        }
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        use Language::*;

        match ext {
            "py" => Some(Python),
            "rs" => Some(Rust),
            _ => None,
        }
    }

    pub fn from_executable(exec: &str) -> Option<Self> {
        use Language::*;

        match exec {
            "python" | "python3" => Some(Python),
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

    pub fn icon(&self) -> char {
        todo!()
    }

    pub fn color(&self) -> (u8, u8, u8) {
        todo!()
    }

    // pub fn grammar(&self) -> Maybe? {
    //     todo!()
    // }
}

#[cfg(test)]
mod tests {
    // use super::*;
}
