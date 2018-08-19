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

const FORMAT_CHARACTERS: &[char] = &[
    '\x02', // bold
    '\x1F', // underline
    '\x16', // reverse
    '\x0F', // normal
    '\x03', // color
];

impl<'a> FormattedStringExt<'a> for &'a str {
    fn is_formatted(&self) -> bool {
        self.contains(FORMAT_CHARACTERS)
    }

    fn strip_formatting(self) -> Cow<'a, str> {
        if !self.is_formatted() {
            return Cow::Borrowed(self);
        }
        let mut s = String::from(self);
        strip_formatting(&mut s);
        Cow::Owned(s)
    }
}

fn strip_formatting(buf: &mut String) {
    let mut parser = Parser {
        state: ParserState::Text,
    };
    let mut prev: char = '\x00';
    buf.retain(|cur| {
        let result = match parser.state {
            ParserState::Text | ParserState::Foreground1 | ParserState::Foreground2 if cur == '\x03' => {
                parser.state = ParserState::ColorCode;
                false
            },
            ParserState::Text => !FORMAT_CHARACTERS.contains(&cur),
            ParserState::ColorCode if cur.is_digit(10) => {
                parser.state = ParserState::Foreground1;
                false
            },
            ParserState::Foreground1 if cur.is_digit(6) => {
                // can only consume another digit if previous char was 1.
                if prev == '1' {
                    parser.state = ParserState::Foreground2;
                    false
                } else {
                    parser.state = ParserState::Text;
                    true
                }
            },
            ParserState::Foreground1 if cur == ','  => {
                parser.state = ParserState::Comma;
                false
            },
            ParserState::Foreground2 if cur == ',' => {
                parser.state = ParserState::Comma;
                false
            },
            ParserState::Comma if (cur.is_digit(10)) => {
                parser.state = ParserState::Background1;
                false
            },
            ParserState::Background1 if cur.is_digit(6) => {
                // can only consume another digit if previous char was 1.
                parser.state = ParserState::Text;
                if prev == '1' {
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
        prev = cur;
        return result
    });
}

impl FormattedStringExt<'static> for String {
    fn is_formatted(&self) -> bool {
        self.as_str().is_formatted()
    }
    fn strip_formatting(mut self) -> Cow<'static, str> {
        if !self.is_formatted() {
            return Cow::Owned(self);
        }
        strip_formatting(&mut self);
        Cow::Owned(self)
    }
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;
    use proto::colors::FormattedStringExt;

    macro_rules! test_formatted_string_ext {
        { $( $name:ident ( $($line:tt)* ), )* } => {
            $(
            mod $name {
                use super::*;
                test_formatted_string_ext!(@ $($line)*);
            }
            )*
        };
        (@ $text:expr, should stripped into $expected:expr) => {
            #[test]
            fn test_formatted() {
                assert!($text.is_formatted());
            }
            #[test]
            fn test_strip() {
                assert_eq!($text.strip_formatting(), $expected);
            }
        };
        (@ $text:expr, is not formatted) => {
            #[test]
            fn test_formatted() {
                assert!(!$text.is_formatted());
            }
            #[test]
            fn test_strip() {
                assert_eq!($text.strip_formatting(), $text);
            }
        }
    }

    test_formatted_string_ext! {
        blank("", is not formatted),
        blank2("    ", is not formatted),
        blank3("\t\r\n", is not formatted),
        bold("l\x02ol", should stripped into "lol"),
        bold_from_string(String::from("l\x02ol"), should stripped into "lol"),
        bold_hangul("우왕\x02굳", should stripped into "우왕굳"),
        fg_color("l\x033ol", should stripped into "lol"),
        fg_color2("l\x0312ol", should stripped into "lol"),
        fg_bg_11("l\x031,2ol", should stripped into "lol"),
        fg_bg_21("l\x0312,3ol", should stripped into "lol"),
        fg_bg_12("l\x031,12ol", should stripped into "lol"),
        fg_bg_22("l\x0312,13ol", should stripped into "lol"),
        string_with_multiple_colors("hoo\x034r\x033a\x0312y", should stripped into "hooray"),
        string_with_digit_after_color("\x0344\x0355\x0366", should stripped into "456"),
        string_with_multiple_2digit_colors("hoo\x0310r\x0311a\x0312y", should stripped into "hooray"),
        string_with_digit_after_2digit_color("\x031212\x031111\x031010", should stripped into "121110"),
        thinking("🤔...", is not formatted),
        unformatted("a plain text", is not formatted),
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
