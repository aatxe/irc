//! Enumeration of all supported IRCv3 capability extensions.

/// List of all supported IRCv3 capability extensions from the
/// [IRCv3 specifications](http://ircv3.net/irc/). 
#[derive(Debug, PartialEq)]
pub enum Capability {
    /// [multi-prefix](http://ircv3.net/specs/extensions/multi-prefix-3.1.html)
    MultiPrefix,
    /// [account-notify](http://ircv3.net/specs/extensions/account-notify-3.1.html)
    AccountNotify,
}

impl AsRef<str> for Capability {
    fn as_ref(&self) -> &str {
        match *self {
            Capability::MultiPrefix => "multi-prefix",
            Capability::AccountNotify => "account-notify",
        }
    }
}

#[cfg(test)]
mod test {
    use super::Capability::*;

    #[test]
    fn to_str() {
        assert_eq!(MultiPrefix.as_ref(), "multi-prefix");
        assert_eq!(AccountNotify.as_ref(), "account-notify");
    }
}
