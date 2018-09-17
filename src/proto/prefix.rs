//! A module providing an enum for a message prefix.
use std::borrow::ToOwned;
use std::string;
use std::str::FromStr;
use std::fmt;

/// The Prefix indicates "the true origin of the message", according to the server.
/// It will 
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Prefix {
    /// servername
    ServerName(String),
    /// nickname [ ["!" username] "@" hostname ]
    Nickname(String, Option<String>, Option<String>),
}

impl Prefix {
    /// Creates a prefix by parsing a string.
    /// 
    /// #Example
    /// ```
    /// # extern crate irc;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// Prefix::new_from_str("nickname!username@hostname");
    /// Prefix::new_from_str("example.com");
    /// # }
    /// ```
    pub fn new_from_str(s: &str) -> Prefix {
        let mut name = String::new();
        let mut user = None;
        let mut host = None;

        for c in s.chars() {
            match c {
                '.' if user.is_none() && host.is_none() => return Prefix::ServerName(s.to_owned()),
                '!' if user.is_none() => {
                    user = Some(String::new())
                },
                '@' if user.is_some() && host.is_none() => {
                    host = Some(String::new())
                },
                _ => {
                    if host.is_some() {
                        host.as_mut().unwrap().push(c)
                    } else if user.is_some() {
                        user.as_mut().unwrap().push(c)
                    } else {
                        name.push(c)
                    }
                }
            }
        }

        Prefix::Nickname(name, user, host)
    }
}

impl FromStr for Prefix {
    type Err = string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Prefix::new_from_str(s))
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Prefix::ServerName(name) => write!(f, "{}", name),
            Prefix::Nickname(name, None, None) => write!(f, "{}", name),
            Prefix::Nickname(name, Some(user), None) => write!(f, "{}!{}", name, user),
            Prefix::Nickname(name, Some(user), Some(host)) => write!(f, "{}!{}@{}", name, user, host),
            // This is an issue with the type using two Option values when really host implies user
            // Maybe this should do the same as the (name, None, None) case
            Prefix::Nickname(_, None, Some(_)) => panic!("can't display prefix with host but not username"),
        }
    }
}

impl<'a> From<&'a str> for Prefix {
    fn from(s: &str) -> Self {
        Prefix::new_from_str(s)
    }
}

#[cfg(test)]
mod test {
    use super::Prefix::{self, ServerName, Nickname};

    // Checks that str -> parsed -> Display doesn't lose data
    fn test_parse(s: &str) -> Prefix {
        let prefix = Prefix::new_from_str(s);
        let s2 = format!("{}", prefix);
        assert_eq!(s, &s2);
        prefix
    }

    #[test]
    fn parse_word() {
        assert_eq!(
            test_parse("only_nick"),
            Nickname("only_nick".into(), None, None)
        )
    }

    #[test]
    fn parse_host() {
        assert_eq!(
            test_parse("host.tld"),
            ServerName("host.tld".into())
        )
    }

    #[test]
    fn parse_nick_user() {
        assert_eq!(
            test_parse("test!nick"),
            Nickname("test".into(), Some("nick".into()), None)
        )
    }

    #[test]
    fn parse_nick_user_host() {
        assert_eq!(
            test_parse("test!nick@host"),
            Nickname("test".into(), Some("nick".into()), Some("host".into()))
        )
    }

    #[test]
    fn parse_dot_and_symbols() {
        assert_eq!(
            test_parse("test.net@something"),
            ServerName("test.net@something".into())
        )
    }

    #[test]
    fn parse_danger_cases() {
        assert_eq!(
            test_parse("name@name!user"),
            Nickname("name@name".into(), Some("user".into()), None)
        );
        assert_eq!(
            test_parse("name!@"),
            Nickname("name".into(), Some("".into()), Some("".into()))
        );
        assert_eq!(
            test_parse("name!.user"),
            Nickname("name".into(), Some(".user".into()), None)
        );
        assert_eq!(
            test_parse("name!user.user"),
            Nickname("name".into(), Some("user.user".into()), None)
        );
        assert_eq!(
            test_parse("name!user@host.host"),
            Nickname("name".into(), Some("user".into()), Some("host.host".into()))
        );
        assert_eq!(
            test_parse("!user"),
            Nickname("".into(), Some("user".into()), None)
        );
        assert_eq!(
            test_parse("!@host.host"),
            Nickname("".into(), Some("".into()), Some("host.host".into()))
        );
    }
}
