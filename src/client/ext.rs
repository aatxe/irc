//! Utilities and shortcuts for working with IRC servers.
//!
//! This module provides the [`ClientExt`](trait.ClientExt.html) trait which is the idiomatic way of
//! sending messages to an IRC server. This trait is automatically implemented for everything that
//! implements [`Client`](../trait.Client.html) and is designed to provide important functionality
//! without clutter.
//!
//! # Examples
//!
//! Using these APIs, we can connect to a server and send a one-off message (in this case,
//! identifying with the server).
//!
//! ```no_run
//! # extern crate irc;
//! use irc::client::prelude::{IrcClient, ClientExt};
//!
//! # fn main() {
//! let server = IrcClient::new("config.toml").unwrap();
//! // identify and send_privmsg both come from `ClientExt`
//! server.identify().unwrap();
//! server.send_privmsg("#example", "Hello, world!").unwrap();
//! # }
//! ```
//!
//! `ClientExt::identify` also plays an important role in performing IRCv3 capability negotiations.
//! In particular, calling `identify` will close the negotiations (and otherwise indicate IRCv3
//! compatibility). This means that all IRCv3 capability requests should be performed before calling
//! `identify`. For example:
//!
//! ```no_run
//! # extern crate irc;
//! # use irc::client::prelude::*;
//! # fn main() {
//! # let server = IrcClient::new("config.toml").unwrap();
//! server.send_cap_req(&[Capability::MultiPrefix, Capability::UserhostInNames]).unwrap();
//! server.identify().unwrap();
//! # }
//! ```
use std::string::ToString;

#[cfg(feature = "ctcp")]
use chrono::prelude::*;

use error::Result;
use proto::{Capability, Command, Mode, NegotiationVersion};
use proto::command::CapSubCommand::{END, LS, REQ};
use proto::command::Command::*;
use proto::mode::ModeType;
use client::Client;

/// Idiomatic extensions for sending messages to an IRC server as a [`Client`](../trait.Client.html).
pub trait ClientExt: Client {
    /// Sends a request for a list of server capabilities for a specific IRCv3 version.
    fn send_cap_ls(&self, version: NegotiationVersion) -> Result<()>
    where
        Self: Sized,
    {
        self.send(Command::CAP(
            None,
            LS,
            match version {
                NegotiationVersion::V301 => None,
                NegotiationVersion::V302 => Some("302".to_owned()),
            },
            None,
        ))
    }

    /// Sends an IRCv3 capabilities request for the specified extensions.
    fn send_cap_req(&self, extensions: &[Capability]) -> Result<()>
    where
        Self: Sized,
    {
        let append = |mut s: String, c| {
            s.push_str(c);
            s.push(' ');
            s
        };
        let mut exts = extensions.iter().map(|c| c.as_ref()).fold(
            String::new(),
            append,
        );
        let len = exts.len() - 1;
        exts.truncate(len);
        self.send(CAP(None, REQ, None, Some(exts)))
    }

    /// Sends a CAP END, NICK and USER to identify.
    fn identify(&self) -> Result<()>
    where
        Self: Sized,
    {
        // Send a CAP END to signify that we're IRCv3-compliant (and to end negotiations!).
        self.send(CAP(None, END, None, None))?;
        if self.config().password() != "" {
            self.send(PASS(self.config().password().to_owned()))?;
        }
        self.send(NICK(self.config().nickname()?.to_owned()))?;
        self.send(USER(
            self.config().username().to_owned(),
            "0".to_owned(),
            self.config().real_name().to_owned(),
        ))?;
        Ok(())
    }

    /// Sends a SASL AUTHENTICATE message with the specified data.
    fn send_sasl<S: ToString>(&self, data: S) -> Result<()>
    where
        Self: Sized,
    {
        self.send(AUTHENTICATE(data.to_string()))
    }

    /// Sends a SASL AUTHENTICATE request to use the PLAIN mechanism.
    fn send_sasl_plain(&self) -> Result<()>
    where
        Self: Sized,
    {
        self.send_sasl("PLAIN")
    }


    /// Sends a SASL AUTHENTICATE request to use the EXTERNAL mechanism.
    fn send_sasl_external(&self) -> Result<()>
    where
        Self: Sized,
    {
        self.send_sasl("EXTERNAL")
    }

    /// Sends a SASL AUTHENTICATE request to abort authentication.
    fn send_sasl_abort(&self) -> Result<()>
    where
        Self: Sized,
    {
        self.send_sasl("*")
    }

    /// Sends a PONG with the specified message.
    fn send_pong<S>(&self, msg: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send(PONG(msg.to_string(), None))
    }

    /// Joins the specified channel or chanlist.
    fn send_join<S>(&self, chanlist: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send(JOIN(chanlist.to_string(), None, None))
    }

    /// Joins the specified channel or chanlist using the specified key or keylist.
    fn send_join_with_keys<S1, S2>(&self, chanlist: &str, keylist: &str) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send(JOIN(chanlist.to_string(), Some(keylist.to_string()), None))
    }

    /// Parts the specified channel or chanlist.
    fn send_part<S>(&self, chanlist: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send(PART(chanlist.to_string(), None))
    }

    /// Attempts to oper up using the specified username and password.
    fn send_oper<S1, S2>(&self, username: S1, password: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send(OPER(username.to_string(), password.to_string()))
    }

    /// Sends a message to the specified target.
    fn send_privmsg<S1, S2>(&self, target: S1, message: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        let message = message.to_string();
        for line in message.split("\r\n") {
            self.send(PRIVMSG(target.to_string(), line.to_string()))?
        }
        Ok(())
    }

    /// Sends a notice to the specified target.
    fn send_notice<S1, S2>(&self, target: S1, message: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        let message = message.to_string();
        for line in message.split("\r\n") {
            self.send(NOTICE(target.to_string(), line.to_string()))?
        }
        Ok(())
    }

    /// Sets the topic of a channel or requests the current one.
    /// If `topic` is an empty string, it won't be included in the message.
    fn send_topic<S1, S2>(&self, channel: S1, topic: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        let topic = topic.to_string();
        self.send(TOPIC(
            channel.to_string(),
            if topic.is_empty() {
                None
            } else {
                Some(topic)
            },
        ))
    }

    /// Kills the target with the provided message.
    fn send_kill<S1, S2>(&self, target: S1, message: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send(KILL(target.to_string(), message.to_string()))
    }

    /// Kicks the listed nicknames from the listed channels with a comment.
    /// If `message` is an empty string, it won't be included in the message.
    fn send_kick<S1, S2, S3>(&self, chanlist: S1, nicklist: S2, message: S3) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
        S3: ToString,
    {
        let message = message.to_string();
        self.send(KICK(
            chanlist.to_string(),
            nicklist.to_string(),
            if message.is_empty() {
                None
            } else {
                Some(message)
            },
        ))
    }

    /// Changes the modes for the specified target.
    fn send_mode<S, T>(&self, target: S, modes: &[Mode<T>]) -> Result<()>
    where
        Self: Sized,
        S: ToString,
        T: ModeType,
    {
        self.send(T::mode(&target.to_string(), modes))
    }

    /// Changes the mode of the target by force.
    /// If `modeparams` is an empty string, it won't be included in the message.
    fn send_samode<S1, S2, S3>(&self, target: S1, mode: S2, modeparams: S3) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
        S3: ToString,
    {
        let modeparams = modeparams.to_string();
        self.send(SAMODE(
            target.to_string(),
            mode.to_string(),
            if modeparams.is_empty() {
                None
            } else {
                Some(modeparams)
            },
        ))
    }

    /// Forces a user to change from the old nickname to the new nickname.
    fn send_sanick<S1, S2>(&self, old_nick: S1, new_nick: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send(SANICK(old_nick.to_string(), new_nick.to_string()))
    }

    /// Invites a user to the specified channel.
    fn send_invite<S1, S2>(&self, nick: S1, chan: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send(INVITE(nick.to_string(), chan.to_string()))
    }

    /// Quits the server entirely with a message.
    /// This defaults to `Powered by Rust.` if none is specified.
    fn send_quit<S>(&self, msg: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        let msg = msg.to_string();
        self.send(QUIT(Some(if msg.is_empty() {
            "Powered by Rust.".to_string()
        } else {
            msg
        })))
    }

    /// Sends a CTCP-escaped message to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_ctcp<S1, S2>(&self, target: S1, msg: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send_privmsg(target, &format!("\u{001}{}\u{001}", msg.to_string())[..])
    }

    /// Sends an action command to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_action<S1, S2>(&self, target: S1, msg: S2) -> Result<()>
    where
        Self: Sized,
        S1: ToString,
        S2: ToString,
    {
        self.send_ctcp(target, &format!("ACTION {}", msg.to_string())[..])
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_finger<S: ToString>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send_ctcp(target, "FINGER")
    }

    /// Sends a version request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_version<S>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send_ctcp(target, "VERSION")
    }

    /// Sends a source request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_source<S>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send_ctcp(target, "SOURCE")
    }

    /// Sends a user info request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_user_info<S>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send_ctcp(target, "USERINFO")
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_ctcp_ping<S>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        let time = Local::now();
        self.send_ctcp(target, &format!("PING {}", time.timestamp())[..])
    }

    /// Sends a time request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_time<S>(&self, target: S) -> Result<()>
    where
        Self: Sized,
        S: ToString,
    {
        self.send_ctcp(target, "TIME")
    }
}

impl<C> ClientExt for C where C: Client {}

#[cfg(test)]
mod test {
    use super::ClientExt;
    use client::data::Config;
    use client::IrcClient;
    use client::test::{get_client_value, test_config};
    use proto::{ChannelMode, Mode};

    #[test]
    fn identify() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.identify().unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "CAP END\r\nNICK :test\r\n\
             USER test 0 * :test\r\n"
        );
    }

    #[test]
    fn identify_with_password() {
        let client = IrcClient::from_config(Config {
            nickname: Some(format!("test")),
            password: Some(format!("password")),
            ..test_config()
        }).unwrap();
        client.identify().unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "CAP END\r\nPASS :password\r\nNICK :test\r\n\
             USER test 0 * :test\r\n"
        );
    }

    #[test]
    fn send_pong() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_pong("irc.test.net").unwrap();
        assert_eq!(&get_client_value(client)[..], "PONG :irc.test.net\r\n");
    }

    #[test]
    fn send_join() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_join("#test,#test2,#test3").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "JOIN #test,#test2,#test3\r\n"
        );
    }

    #[test]
    fn send_part() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_part("#test").unwrap();
        assert_eq!(&get_client_value(client)[..], "PART #test\r\n");
    }

    #[test]
    fn send_oper() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_oper("test", "test").unwrap();
        assert_eq!(&get_client_value(client)[..], "OPER test :test\r\n");
    }

    #[test]
    fn send_privmsg() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_privmsg("#test", "Hi, everybody!").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG #test :Hi, everybody!\r\n"
        );
    }

    #[test]
    fn send_notice() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_notice("#test", "Hi, everybody!").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "NOTICE #test :Hi, everybody!\r\n"
        );
    }

    #[test]
    fn send_topic_no_topic() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_topic("#test", "").unwrap();
        assert_eq!(&get_client_value(client)[..], "TOPIC #test\r\n");
    }

    #[test]
    fn send_topic() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_topic("#test", "Testing stuff.").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "TOPIC #test :Testing stuff.\r\n"
        );
    }

    #[test]
    fn send_kill() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_kill("test", "Testing kills.").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "KILL test :Testing kills.\r\n"
        );
    }

    #[test]
    fn send_kick_no_message() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_kick("#test", "test", "").unwrap();
        assert_eq!(&get_client_value(client)[..], "KICK #test test\r\n");
    }

    #[test]
    fn send_kick() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_kick("#test", "test", "Testing kicks.").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "KICK #test test :Testing kicks.\r\n"
        );
    }

    #[test]
    fn send_mode_no_modeparams() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_mode("#test", &[Mode::Plus(ChannelMode::InviteOnly, None)]).unwrap();
        assert_eq!(&get_client_value(client)[..], "MODE #test +i\r\n");
    }

    #[test]
    fn send_mode() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_mode("#test", &[Mode::Plus(ChannelMode::Oper, Some("test".to_owned()))])
              .unwrap();
        assert_eq!(&get_client_value(client)[..], "MODE #test +o test\r\n");
    }

    #[test]
    fn send_samode_no_modeparams() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_samode("#test", "+i", "").unwrap();
        assert_eq!(&get_client_value(client)[..], "SAMODE #test +i\r\n");
    }

    #[test]
    fn send_samode() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_samode("#test", "+o", "test").unwrap();
        assert_eq!(&get_client_value(client)[..], "SAMODE #test +o test\r\n");
    }

    #[test]
    fn send_sanick() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_sanick("test", "test2").unwrap();
        assert_eq!(&get_client_value(client)[..], "SANICK test test2\r\n");
    }

    #[test]
    fn send_invite() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_invite("test", "#test").unwrap();
        assert_eq!(&get_client_value(client)[..], "INVITE test #test\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_ctcp("test", "MESSAGE").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}MESSAGE\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_action() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_action("test", "tests.").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}ACTION tests.\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_finger() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_finger("test").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}FINGER\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_version() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_version("test").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}VERSION\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_source() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_source("test").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}SOURCE\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_user_info() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_user_info("test").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}USERINFO\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp_ping() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_ctcp_ping("test").unwrap();
        let val = get_client_value(client);
        println!("{}", val);
        assert!(val.starts_with("PRIVMSG test :\u{001}PING "));
        assert!(val.ends_with("\u{001}\r\n"));
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_time() {
        let client = IrcClient::from_config(test_config()).unwrap();
        client.send_time("test").unwrap();
        assert_eq!(
            &get_client_value(client)[..],
            "PRIVMSG test :\u{001}TIME\u{001}\r\n"
        );
    }
}
