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
    /// [monitor](http://ircv3.net/specs/core/monitor-3.2.html)
    Monitor,
    /// [account-tag](http://ircv3.net/specs/extensions/account-tag-3.2.html)
    AccountTag,
    /// [cap-notify](http://ircv3.net/specs/extensions/cap-notify-3.2.html)
    CapNotify,
    /// [chghost](http://ircv3.net/specs/extensions/chghost-3.2.html)
    ChgHost,
    /// [echo-message](http://ircv3.net/specs/extensions/echo-message-3.2.html)
    EchoMessage,
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
            Capability::Monitor => "monitor",
            Capability::AccountTag => "account-tag",
            Capability::CapNotify => "cap-notify",
            Capability::ChgHost => "chghost",
            Capability::EchoMessage => "echo-message",
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
        assert_eq!(Monitor.as_ref(), "monitor");
        assert_eq!(AccountTag.as_ref(), "account-tag");
        assert_eq!(CapNotify.as_ref(), "cap-notify");
        assert_eq!(ChgHost.as_ref(), "chghost");
        assert_eq!(EchoMessage.as_ref(), "echo-message");
    }
}
