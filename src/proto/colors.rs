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
pub trait FormattedStringExt {

    /// Returns true if the string contains color, bold, underline or italics
    fn is_formatted(&self) -> bool;

    /// Returns the string with all color, bold, underline and italics stripped
    fn strip_formatting(&self) -> Cow<str>;

}


impl FormattedStringExt for str {
    fn is_formatted(&self) -> bool {
        self.contains('\x02') || // bold
            self.contains('\x1F') || // underline
            self.contains('\x16') || // reverse
            self.contains('\x0F') || // normal
            self.contains('\x03') // color
    }

    fn strip_formatting(&self) -> Cow<str> {
        let mut parser = Parser {
            state: ParserState::Text,
        };

        let result: Cow<str> = self
            .chars()
            .filter(move |cur| {
                match parser.state {
                    ParserState::Text if *cur == '\x03' => {
                        parser.state = ParserState::ColorCode;
                        false
                    },
                    ParserState::Text => !['\x02', '\x1F', '\x16', '\x0F'].contains(cur),
                    ParserState::ColorCode if  (*cur).is_digit(10) => {
                        parser.state = ParserState::Foreground1;
                        false
                    },
                    ParserState::Foreground1 if (*cur).is_digit(6) => {
                        parser.state = ParserState::Foreground2;
                        false
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
                        parser.state = ParserState::Text;
                        false
                    }
                    _ => true
                }
            })
            .collect();

        result
    }


}

impl FormattedStringExt for String {
    fn is_formatted(&self) -> bool {
        (&self[..]).is_formatted()
    }
    fn strip_formatting(&self) -> Cow<str> {
        (&self[..]).strip_formatting()
    }
}
