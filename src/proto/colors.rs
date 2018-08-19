//! An extension trait that provides the ability to strip IRC colors from a string
use std::borrow::Cow;

#[derive(PartialEq)]
enum ParserState {
    Text,
    ColorCode,
    Foreground1,
    Foreground2,
    Comma,
    Background1,
}
struct Parser {
    state: ParserState,
}

/// An extension trait giving strings a function to strip IRC colors
pub trait FormattedStringExt<'a> {

    /// Returns true if the string contains color, bold, underline or italics
    fn is_formatted(&self) -> bool;

    /// Returns the string with all color, bold, underline and italics stripped
    fn strip_formatting(self) -> Cow<'a, str>;

}


impl<'a> FormattedStringExt<'a> for &'a str {
    fn is_formatted(&self) -> bool {
        self.contains('\x02') || // bold
            self.contains('\x1F') || // underline
            self.contains('\x16') || // reverse
            self.contains('\x0F') || // normal
            self.contains('\x03') // color
    }

    fn strip_formatting(self) -> Cow<'a, str> {
        if !self.is_formatted() {
            return Cow::Borrowed(self);
        }
        Cow::Owned(internal::strip_formatting(self))
    }
}

mod internal {  // to reduce commit diff
    use super::*;
    pub(super) fn strip_formatting(input: &str) -> String {
        let mut parser = Parser {
            state: ParserState::Text,
        };
        let mut prev: char = '\x00';
        input
            .chars()
            .filter(move |cur| {
                let result = match parser.state {
                    ParserState::Text | ParserState::Foreground1 | ParserState::Foreground2 if *cur == '\x03' => {
                        parser.state = ParserState::ColorCode;
                        false
                    },
                    ParserState::Text => !['\x02', '\x1F', '\x16', '\x0F'].contains(cur),
                    ParserState::ColorCode if  (*cur).is_digit(10) => {
                        parser.state = ParserState::Foreground1;
                        false
                    },
                    ParserState::Foreground1 if (*cur).is_digit(6) => {
                        // can only consume another digit if previous char was 1.
                        if (prev) == '1' {
                            parser.state = ParserState::Foreground2;
                            false
                        } else {
                            parser.state = ParserState::Text;
                            true
                        }
                    },
                    ParserState::Foreground1 if *cur == ','  => {
                        parser.state = ParserState::Comma;
                        false
                    },
                    ParserState::Foreground2 if *cur == ',' => {
                        parser.state = ParserState::Comma;
                        false
                    },
                    ParserState::Comma if ((*cur).is_digit(10)) => {
                        parser.state = ParserState::Background1;
                        false
                    },
                    ParserState::Background1 if (*cur).is_digit(6) => {
                        // can only consume another digit if previous char was 1.
                        parser.state = ParserState::Text;
                        if (prev) == '1' {
                            false
                        } else {
                            true
                        }
                    }
                    _ => {
                        parser.state = ParserState::Text;
                        true
                    }
                };
                prev = *cur;
                return result
            })
            .collect()
    }
}

impl FormattedStringExt<'static> for String {
    fn is_formatted(&self) -> bool {
        self.as_str().is_formatted()
    }
    fn strip_formatting(self) -> Cow<'static, str> {
        if !self.is_formatted() {
            return Cow::Owned(self);
        }
        Cow::Owned(internal::strip_formatting(&self))
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use proto::colors::FormattedStringExt;

    #[test]
    fn test_strip_bold() {
        assert_eq!("l\x02ol".strip_formatting(), "lol");
    }
    #[test]
    fn test_strip_bold_from_string() {
        assert_eq!(String::from("l\x02ol").strip_formatting(), "lol");
    }

    #[test]
    fn test_strip_fg_color() {
        assert_eq!("l\x033ol".strip_formatting(), "lol");
    }

    #[test]
    fn test_strip_fg_color2() {
        assert_eq!("l\x0312ol".strip_formatting(), "lol");
    }

    #[test]
    fn test_strip_fg_bg_11() {
        assert_eq!("l\x031,2ol".strip_formatting(), "lol");
    }
    #[test]
    fn test_strip_fg_bg_21() {
        assert_eq!("l\x0312,3ol".strip_formatting(), "lol");
    }
    #[test]
    fn test_strip_fg_bg_12() {
        assert_eq!("l\x031,12ol".strip_formatting(), "lol");
    }
    #[test]
    fn test_strip_fg_bg_22() {
        assert_eq!("l\x0312,13ol".strip_formatting(), "lol");
    }
    #[test]
    fn test_strip_string_with_multiple_colors() {
        assert_eq!("hoo\x034r\x033a\x0312y".strip_formatting(), "hooray");
    }
    #[test]
    fn test_strip_string_with_digit_after_color() {
        assert_eq!("\x0344\x0355\x0366".strip_formatting(), "456");
    }
    #[test]
    fn test_strip_string_with_multiple_2digit_colors() {
        assert_eq!("hoo\x0310r\x0311a\x0312y".strip_formatting(), "hooray");
    }
    #[test]
    fn test_strip_string_with_digit_after_2digit_color() {
        assert_eq!("\x031212\x031111\x031010".strip_formatting(), "121110");
    }

    #[test]
    fn test_strip_no_allocation_for_unformatted_text() {
        if let Cow::Borrowed(formatted) = "plain text".strip_formatting() {
            assert_eq!(formatted, "plain text");
        } else {
            panic!("allocation detected");
        }
    }
}