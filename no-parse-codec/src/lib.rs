use std::{fmt::Display, io, str::FromStr};

use irc_interface::{
    line::LineMessage, InternalIrcMessageIncoming, InternalIrcMessageOutgoing, LineCodec,
};

/// A minimal message codec that makes the irc client functional without actually parsing the messages.
pub type NoParseCodec = LineCodec<UnparsedMessage>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsedMessage(String);

impl Display for UnparsedMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for UnparsedMessage {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, io::Error> {
        Result::<Self, <Self as FromStr>::Err>::Ok(UnparsedMessage(s.into()))
    }
}

impl<T> From<T> for UnparsedMessage
where
    T: Into<String>,
{
    fn from(item: T) -> Self {
        let item: String = item.into();
        if item.ends_with("\r\n") {
            Self(item)
        } else {
            Self(format!("{item}\r\n"))
        }
    }
}

impl LineMessage for UnparsedMessage {
    type Error = io::Error;
}

impl InternalIrcMessageOutgoing for UnparsedMessage {
    fn new_raw(command: String, arguments: Vec<String>) -> Self {
        UnparsedMessage(format!("{} {}\r\n", command, arguments.join(" ")))
    }
}

mod regex {
    //! initialize lazily evaluated regexes
    use once_cell::sync::Lazy;
    use regex::Regex;

    pub(super) static END_OF_MOTD: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(@[\S]* )?(:[\S]* )?376").unwrap());

    pub(super) static ERR_NO_MOTD: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(@[\S]* )?(:[\S]* )?422").unwrap());

    pub(super) static PONG: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(@[\S]* )?(:[\S]* )?PONG").unwrap());

    pub(super) static PING: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(@[\S]* )?(:[\S]* )?PING (?P<token>\S+)").unwrap());

    pub(super) static QUIT: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"^(@[\S]* )?(:[\S]* )?QUIT").unwrap());

    pub(super) static CAP_LS: Lazy<Regex> = Lazy::new(|| {
        Regex::new(r"^(@[\S]* )?(:[\S]* )?CAP \* LS :(?P<capabilities>.*)(\r\n)?$").unwrap()
    });
}

// TODO: I could make this more efficient by adding a slice to the structure which only refers to the part of the string that contains the message (without prefix)

impl InternalIrcMessageIncoming for UnparsedMessage {
    fn is_end_of_motd(&self) -> bool {
        regex::END_OF_MOTD.is_match(&self.0)
    }

    fn is_err_nomotd(&self) -> bool {
        regex::ERR_NO_MOTD.is_match(&self.0)
    }

    // TODO: Write test for ping/pong
    fn is_pong(&self) -> bool {
        regex::PONG.is_match(&self.0)
    }

    fn as_ping<'a>(&'a self) -> Option<&'a str> {
        regex::PING.captures(&self.0).map(|captures| {
            captures
                .name("token")
                .unwrap_or_else(|| unreachable!())
                .as_str()
        })
    }

    fn is_quit(&self) -> bool {
        regex::QUIT.is_match(&self.0)
    }
}

impl UnparsedMessage {
    /// Parse the capability list in a `CAP * LS` response.
    pub fn as_cap_ls<'a>(&'a self) -> Option<&'a str> {
        regex::CAP_LS.captures(&self.0).map(|captures| {
            captures
                .name("capabilities")
                .unwrap_or_else(|| unreachable!())
                .as_str()
        })
    }
}

#[cfg(test)]
mod tests {

    // WARNING: VS Code shows errors here, because it falsely assumes that the irc crate's `essential` feature is activated

    use crate::{NoParseCodec, UnparsedMessage};
    use anyhow::Result;
    use irc::client::codec_tests::TestSuite;

    #[tokio::test]
    async fn handle_end_motd() -> Result<()> {
        TestSuite::<NoParseCodec>::handle_end_motd().await
    }

    #[tokio::test]
    async fn handle_end_motd_with_nick_password() -> Result<()> {
        TestSuite::<NoParseCodec>::handle_end_motd_with_nick_password().await
    }
    #[tokio::test]
    async fn identify() -> Result<()> {
        TestSuite::<NoParseCodec>::identify().await
    }

    #[tokio::test]
    async fn identify_with_password() -> Result<()> {
        TestSuite::<NoParseCodec>::identify_with_password().await
    }

    #[tokio::test]
    async fn send_pong() -> Result<()> {
        TestSuite::<NoParseCodec>::send_pong().await
    }

    #[tokio::test]
    async fn send_join() -> Result<()> {
        TestSuite::<NoParseCodec>::send_join().await
    }

    #[tokio::test]
    async fn send_part() -> Result<()> {
        TestSuite::<NoParseCodec>::send_part().await
    }

    #[test]
    fn as_cap_ls() {
        let test_message_no_match =
            ":testing.snowpoke.ink NOTICE * :*** Looking up your hostname...";
        let test_message_match = ":testing.snowpoke.ink CAP * LS :echo-message inspircd.org/poison inspircd.org/standard-replies message-tags server-time";

        let test_message_no_match: UnparsedMessage = test_message_no_match.into();
        let test_message_match: UnparsedMessage = test_message_match.into();

        assert_eq!(test_message_no_match.as_cap_ls(), None);
        assert_eq!(test_message_match.as_cap_ls(), Some("echo-message inspircd.org/poison inspircd.org/standard-replies message-tags server-time"));
    }
}
