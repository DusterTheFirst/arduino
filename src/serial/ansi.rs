use core::fmt::{self, Display, Formatter};

const ANSI_ESCAPE: &str = "\u{1B}[";
const ANSI_ESCAPE_END: &str = "m";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    LightBlack,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    LightWhite,
    TrueColor { r: u8, g: u8, b: u8 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(missing_docs)]
pub enum Style {
    Clear,
    Bold,
    Dimmed,
    Underline,
    Reversed,
    Italic,
    Blink,
    Hidden,
    Strikethrough,
}

pub const fn fg(color: Color) -> EscapeSequence<'static> {
    EscapeSequence::new().set_fg(color)
}

pub const fn bg(color: Color) -> EscapeSequence<'static> {
    EscapeSequence::new().set_bg(color)
}

pub const fn fg_bg(fg: Color, bg: Color) -> EscapeSequence<'static> {
    EscapeSequence::new().set_fg(fg).set_bg(bg)
}

pub const fn styles<'a>(styles: &'a [Style]) -> EscapeSequence<'a> {
    EscapeSequence::new().set_styles(styles)
}

pub const fn clear() -> EscapeSequence<'static> {
    EscapeSequence::new().set_styles(&[Style::Clear])
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EscapeSequence<'a> {
    fg: Option<Color>,
    bg: Option<Color>,
    styles: &'a [Style],
}

impl<'a> EscapeSequence<'a> {
    pub const fn new() -> Self {
        Self {
            bg: None,
            fg: None,
            styles: &[],
        }
    }

    pub const fn set_fg(self, color: Color) -> Self {
        EscapeSequence {
            fg: Some(color),
            ..self
        }
    }

    pub const fn set_bg(self, color: Color) -> Self {
        EscapeSequence {
            bg: Some(color),
            ..self
        }
    }

    pub const fn set_styles(self, styles: &'a [Style]) -> Self {
        EscapeSequence { styles, ..self }
    }
}

#[cfg(not(feature = "no_color"))]
impl<'a> Display for EscapeSequence<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(ANSI_ESCAPE)?;

        // Foreground format
        if let Some(color) = self.fg {
            if let Color::TrueColor { r, g, b } = color {
                write!(f, "38;2;{};{};{}", r, g, b)?;
            } else {
                f.write_str(match color {
                    Color::Black => "30",
                    Color::Red => "31",
                    Color::Green => "32",
                    Color::Yellow => "33",
                    Color::Blue => "34",
                    Color::Magenta => "35",
                    Color::Cyan => "36",
                    Color::White => "37",
                    Color::LightBlack => "90",
                    Color::LightRed => "91",
                    Color::LightGreen => "92",
                    Color::LightYellow => "93",
                    Color::LightBlue => "94",
                    Color::LightMagenta => "95",
                    Color::LightCyan => "96",
                    Color::LightWhite => "97",
                    Color::TrueColor { .. } => unreachable!(),
                })?;
            }
        }

        // Background format
        if let Some(color) = self.bg {
            if let Color::TrueColor { r, g, b } = color {
                write!(f, "48;2;{};{};{}", r, g, b)?;
            } else {
                f.write_str(match color {
                    Color::Black => "40",
                    Color::Red => "41",
                    Color::Green => "42",
                    Color::Yellow => "43",
                    Color::Blue => "44",
                    Color::Magenta => "45",
                    Color::Cyan => "46",
                    Color::White => "47",
                    Color::LightBlack => "100",
                    Color::LightRed => "101",
                    Color::LightGreen => "102",
                    Color::LightYellow => "103",
                    Color::LightBlue => "104",
                    Color::LightMagenta => "105",
                    Color::LightCyan => "106",
                    Color::LightWhite => "107",
                    Color::TrueColor { .. } => unreachable!(),
                })?;
            }
        }

        for style in self.styles {
            f.write_str(match style {
                Style::Clear => "0",
                Style::Bold => "1",
                Style::Dimmed => "2",
                Style::Italic => "3",
                Style::Underline => "4",
                Style::Blink => "5",
                Style::Reversed => "7",
                Style::Hidden => "8",
                Style::Strikethrough => "9",
            })?;
        }

        f.write_str(ANSI_ESCAPE_END)
    }
}

#[cfg(feature = "no_color")]
impl Display for EscapeSequence<'_> {
    fn fmt(&self, _: &mut Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}
