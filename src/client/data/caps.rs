//! Enumeration of all supported IRCv3 capability extensions.

/// List of all supported IRCv3 capability extensions from the
/// [IRCv3 specifications](http://ircv3.net/irc/). 
#[derive(Debug, PartialEq)]
pub enum Capability {
    /// [multi-prefix](http://ircv3.net/specs/extensions/multi-prefix-3.1.html)
    MultiPrefix,
    /// [account-notify](http://ircv3.net/specs/extensions/account-notify-3.1.html)
    AccountNotify,
    /// [away-notify](http://ircv3.net/specs/extensions/away-notify-3.1.html)
    AwayNotify,
    /// [extended-join](http://ircv3.net/specs/extensions/extended-join-3.1.html)
    ExtendedJoin,
}

/// List of IRCv3 capability negotiation versions.
pub enum NegotiationVersion {
    /// [IRCv3.1](http://ircv3.net/specs/core/capability-negotiation-3.1.html)
    V301,
    /// [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    V302,
}

impl AsRef<str> for Capability {
    fn as_ref(&self) -> &str {
        match *self {
            Capability::MultiPrefix => "multi-prefix",
            Capability::AccountNotify => "account-notify",
            Capability::AwayNotify => "away-notify",
            Capability::ExtendedJoin => "extended-join",
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
        assert_eq!(AwayNotify.as_ref(), "away-notify");
        assert_eq!(ExtendedJoin.as_ref(), "extended-join");
    }
}
