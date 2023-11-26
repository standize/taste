use std::path::Path;

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
        todo!()
    }

    pub fn from_filename(fname: &str) -> Option<Self> {
        todo!()
    }

    pub fn from_extension(ext: &str) -> Option<Self> {
        todo!()
    }

    pub fn from_executable(exec: &str) -> Option<Self> {
        todo!()
    }

    pub fn from_first_line(path: &Path) -> Option<Self> {
        todo!()
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
