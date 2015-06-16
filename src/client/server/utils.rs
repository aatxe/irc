//! Utilities and shortcuts for working with IRC servers.
use std::io::Result;
use std::borrow::ToOwned;
use client::data::{Capability, NegotiationVersion};
use client::data::Command::{CAP, INVITE, JOIN, KICK, KILL, MODE, NICK, NOTICE};
use client::data::Command::{OPER, PASS, PONG, PRIVMSG, QUIT, SAMODE, SANICK, TOPIC, USER};
use client::data::command::CapSubCommand::{END, LS, REQ};
use client::data::kinds::{IrcRead, IrcWrite};
#[cfg(feature = "ctcp")] use time::get_time;
use client::server::Server;

/// Extensions for Server capabilities that make it easier to work directly with the protocol.
pub trait ServerExt<'a, T, U>: Server<'a, T, U> {
    /// Sends a request for a list of server capabilities for a specific IRCv3 version.
    fn send_cap_ls(&self, version: NegotiationVersion) -> Result<()> where Self: Sized {
        self.send(CAP(None, LS, match version {
            NegotiationVersion::V301 => None,
            NegotiationVersion::V302 => Some("302".to_owned()),
        }, None))
    }

    /// Sends an IRCv3 capabilities request for the specified extensions.
    fn send_cap_req(&self, extensions: &[Capability]) -> Result<()> where Self: Sized {
        let append = |mut s: String, c| { s.push_str(c); s.push(' '); s };
        let mut exts = extensions.iter().map(|c| c.as_ref()).fold(String::new(), append);
        let len = exts.len() - 1;
        exts.truncate(len);
        self.send(CAP(None, REQ, None, Some(exts)))
    }

    /// Sends a CAP END, NICK and USER to identify.
    fn identify(&self) -> Result<()> where Self: Sized {
        // Send a CAP END to signify that we're IRCv3-compliant (and to end negotiations!).
        try!(self.send(CAP(None, END, None, None)));
        if self.config().password() != "" {
            try!(self.send(PASS(self.config().password().to_owned())));
        }
        try!(self.send(NICK(self.config().nickname().to_owned())));
        try!(self.send(USER(self.config().username().to_owned(), "0".to_owned(),
                            self.config().real_name().to_owned())));
        Ok(())
    }

    /// Sends a PONG with the specified message.
    fn send_pong(&self, msg: &str) -> Result<()> where Self: Sized {
        self.send(PONG(msg.to_owned(), None))
    }

    /// Joins the specified channel or chanlist.
    fn send_join(&self, chanlist: &str) -> Result<()> where Self: Sized {
        self.send(JOIN(chanlist.to_owned(), None, None))
    }

    /// Attempts to oper up using the specified username and password.
    fn send_oper(&self, username: &str, password: &str) -> Result<()> where Self: Sized {
        self.send(OPER(username.to_owned(), password.to_owned()))
    }

    /// Sends a message to the specified target.
    fn send_privmsg(&self, target: &str, message: &str) -> Result<()> where Self: Sized {
        for line in message.split("\r\n") {
            try!(self.send(PRIVMSG(target.to_owned(), line.to_owned())))
        }
        Ok(())
    }

    /// Sends a notice to the specified target.
    fn send_notice(&self, target: &str, message: &str) -> Result<()> where Self: Sized {
        for line in message.split("\r\n") {
            try!(self.send(NOTICE(target.to_owned(), line.to_owned())))
        }
        Ok(())
    }

    /// Sets the topic of a channel or requests the current one.
    /// If `topic` is an empty string, it won't be included in the message.
    fn send_topic(&self, channel: &str, topic: &str) -> Result<()> where Self: Sized {
        self.send(TOPIC(channel.to_owned(), if topic.len() == 0 {
            None
        } else {
            Some(topic.to_owned())
        }))
    }

    /// Kills the target with the provided message.
    fn send_kill(&self, target: &str, message: &str) -> Result<()> where Self: Sized {
        self.send(KILL(target.to_owned(), message.to_owned()))
    }

    /// Kicks the listed nicknames from the listed channels with a comment.
    /// If `message` is an empty string, it won't be included in the message.
    fn send_kick(&self, chanlist: &str, nicklist: &str, message: &str) -> Result<()>
    where Self: Sized {
        self.send(KICK(chanlist.to_owned(), nicklist.to_owned(), if message.len() == 0 {
            None
        } else {
            Some(message.to_owned())
        }))
    }

    /// Changes the mode of the target.
    /// If `modeparmas` is an empty string, it won't be included in the message.
    fn send_mode(&self, target: &str, mode: &str, modeparams: &str) -> Result<()>
    where Self: Sized {
        self.send(MODE(target.to_owned(), mode.to_owned(), if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams.to_owned())
        }))
    }

    /// Changes the mode of the target by force.
    /// If `modeparams` is an empty string, it won't be included in the message.
    fn send_samode(&self, target: &str, mode: &str, modeparams: &str) -> Result<()>
    where Self: Sized {
        self.send(SAMODE(target.to_owned(), mode.to_owned(), if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams.to_owned())
        }))
    }

    /// Forces a user to change from the old nickname to the new nickname.
    fn send_sanick(&self, old_nick: &str, new_nick: &str) -> Result<()> where Self: Sized {
        self.send(SANICK(old_nick.to_owned(), new_nick.to_owned()))
    }

    /// Invites a user to the specified channel.
    fn send_invite(&self, nick: &str, chan: &str) -> Result<()> where Self: Sized {
        self.send(INVITE(nick.to_owned(), chan.to_owned()))
    }

    /// Quits the server entirely with a message. 
    /// This defaults to `Powered by Rust.` if none is specified.
    fn send_quit(&self, msg: &str) -> Result<()> where Self: Sized {
        self.send(QUIT(Some(if msg.len() == 0 {
            "Powered by Rust.".to_owned()
        } else {
            msg.to_owned()
        })))
    }

    /// Sends a CTCP-escaped message to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_ctcp(&self, target: &str, msg: &str) -> Result<()> where Self: Sized {
        self.send_privmsg(target, &format!("\u{001}{}\u{001}", msg)[..])
    }

    /// Sends an action command to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_action(&self, target: &str, msg: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, &format!("ACTION {}", msg)[..])
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_finger(&self, target: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, "FINGER")
    }

    /// Sends a version request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_version(&self, target: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, "VERSION")
    }

    /// Sends a source request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_source(&self, target: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, "SOURCE")
    }

    /// Sends a user info request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_user_info(&self, target: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, "USERINFO")
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_ctcp_ping(&self, target: &str) -> Result<()> where Self: Sized {
        let time = get_time();
        self.send_ctcp(target, &format!("PING {}.{}", time.sec, time.nsec)[..])
    }

    /// Sends a time request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[cfg(feature = "ctcp")]
    fn send_time(&self, target: &str) -> Result<()> where Self: Sized {
        self.send_ctcp(target, "TIME")
    }
}

impl<'a, T: IrcRead, U: IrcWrite, K: Server<'a, T, U>> ServerExt<'a, T, U> for K {}

#[cfg(test)]
mod test {
    use super::ServerExt;
    use std::default::Default;
    use client::conn::Connection;
    use client::data::Config;
    use client::server::IrcServer;
    use client::server::test::{get_server_value, test_config};
    use client::test::buf_empty;

    #[test]
    fn identify() {
        let server = IrcServer::from_connection(test_config(), 
                     Connection::new(buf_empty(), Vec::new()));
        server.identify().unwrap();
        assert_eq!(&get_server_value(server)[..], "CAP END\r\nNICK :test\r\n\
                                                   USER test 0 * :test\r\n");
    }

    #[test]
    fn identify_with_password() {
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            password: Some(format!("password")),
            .. Default::default()
        }, Connection::new(buf_empty(), Vec::new()));
        server.identify().unwrap();
        assert_eq!(&get_server_value(server)[..], "CAP END\r\nPASS :password\r\nNICK :test\r\n\
                                                   USER test 0 * :test\r\n");
    }

    #[test]
    fn send_pong() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_pong("irc.test.net").unwrap();
        assert_eq!(&get_server_value(server)[..], "PONG :irc.test.net\r\n");
    }

    #[test]
    fn send_join() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_join("#test,#test2,#test3").unwrap();
        assert_eq!(&get_server_value(server)[..], "JOIN #test,#test2,#test3\r\n");
    }

    #[test]
    fn send_oper() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_oper("test", "test").unwrap();
        assert_eq!(&get_server_value(server)[..], "OPER test :test\r\n");
    }

    #[test]
    fn send_privmsg() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_privmsg("#test", "Hi, everybody!").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG #test :Hi, everybody!\r\n");
    }

    #[test]
    fn send_notice() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_notice("#test", "Hi, everybody!").unwrap();
        assert_eq!(&get_server_value(server)[..], "NOTICE #test :Hi, everybody!\r\n");
    }

    #[test]
    fn send_topic_no_topic() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_topic("#test", "").unwrap();
        assert_eq!(&get_server_value(server)[..], "TOPIC #test\r\n");
    }

    #[test]
    fn send_topic() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_topic("#test", "Testing stuff.").unwrap();
        assert_eq!(&get_server_value(server)[..], "TOPIC #test :Testing stuff.\r\n");
    }

    #[test]
    fn send_kill() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_kill("test", "Testing kills.").unwrap();
        assert_eq!(&get_server_value(server)[..], "KILL test :Testing kills.\r\n");
    }

    #[test]
    fn send_kick_no_message() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_kick("#test", "test", "").unwrap();
        assert_eq!(&get_server_value(server)[..], "KICK #test test\r\n");
    }

    #[test]
    fn send_kick() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_kick("#test", "test", "Testing kicks.").unwrap();
        assert_eq!(&get_server_value(server)[..], "KICK #test test :Testing kicks.\r\n");
    }

    #[test]
    fn send_mode_no_modeparams() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_mode("#test", "+i", "").unwrap();
        assert_eq!(&get_server_value(server)[..], "MODE #test +i\r\n");
    }

    #[test]
    fn send_mode() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_mode("#test", "+o", "test").unwrap();
        assert_eq!(&get_server_value(server)[..], "MODE #test +o test\r\n");
    }

    #[test]
    fn send_samode_no_modeparams() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_samode("#test", "+i", "").unwrap();
        assert_eq!(&get_server_value(server)[..], "SAMODE #test +i\r\n");
    }

    #[test]
    fn send_samode() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_samode("#test", "+o", "test").unwrap();
        assert_eq!(&get_server_value(server)[..], "SAMODE #test +o test\r\n");
    }

    #[test]
    fn send_sanick() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_sanick("test", "test2").unwrap();
        assert_eq!(&get_server_value(server)[..], "SANICK test test2\r\n");
    }

    #[test]
    fn send_invite() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_invite("test", "#test").unwrap();
        assert_eq!(&get_server_value(server)[..], "INVITE test #test\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_ctcp("test", "MESSAGE").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}MESSAGE\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_action() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_action("test", "tests.").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}ACTION tests.\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_finger() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_finger("test").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}FINGER\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_version() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_version("test").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}VERSION\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_source() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_source("test").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}SOURCE\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_user_info() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_user_info("test").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}USERINFO\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp_ping() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_ctcp_ping("test").unwrap();
        let val = get_server_value(server);
        println!("{}", val);
        assert!(val.starts_with("PRIVMSG test :\u{001}PING "));
        assert!(val.ends_with("\u{001}\r\n"));
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_time() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(buf_empty(), Vec::new()));
        server.send_time("test").unwrap();
        assert_eq!(&get_server_value(server)[..], "PRIVMSG test :\u{001}TIME\u{001}\r\n");
    }
}
