//! Visual metadata: colors and icons.

/// 24-bit RGB color, used for language colors and icon tints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// A glyph (typically a Nerd Font codepoint) plus an optional tint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Icon {
    pub glyph: char,
    pub color: Option<Color>,
}

impl Icon {
    pub const fn new(glyph: char) -> Self {
        Self { glyph, color: None }
    }

    pub const fn tinted(glyph: char, color: Color) -> Self {
        Self {
            glyph,
            color: Some(color),
        }
    }
}
