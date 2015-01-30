//! Utilities and shortcuts for working with IRC servers.
#![stable]

use std::old_io::IoResult;
use client::data::{Command, Config, User};
use client::data::Command::{CAP, INVITE, JOIN, KICK, KILL, MODE, NICK, NOTICE};
use client::data::Command::{OPER, PASS, PONG, PRIVMSG, QUIT, SAMODE, SANICK, TOPIC, USER};
use client::data::command::CapSubCommand::{END, REQ};
use client::data::kinds::{IrcReader, IrcWriter};
#[cfg(feature = "ctcp")] use time::get_time;
use client::server::{Server, ServerIterator};

/// Functionality-providing wrapper for Server.
/// Wrappers are currently not thread-safe, and should be created per-thread, as needed.
#[stable]
pub struct Wrapper<'a, T: IrcReader, U: IrcWriter> {
    server: &'a (Server<'a, T, U> + 'a)
}

impl<'a, T: IrcReader, U: IrcWriter> Server<'a, T, U> for Wrapper<'a, T, U> {
    fn config(&self) -> &Config {
        self.server.config()
    }

    fn send(&self, command: Command) -> IoResult<()> {
        self.server.send(command)
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        self.server.iter()
    }

    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.server.list_users(chan)
    }
}

#[unstable = "More functionality will be added."]
impl<'a, T: IrcReader, U: IrcWriter> Wrapper<'a, T, U> {
    /// Creates a new Wrapper from the given Server.
    #[stable]
    pub fn new(server: &'a Server<'a, T, U>) -> Wrapper<'a, T, U> {
        Wrapper { server: server }
    }

    /// Sends a NICK and USER to identify.
    #[unstable = "Capabilities requests may be moved outside of identify."]
    pub fn identify(&self) -> IoResult<()> {
        // We'll issue a CAP REQ for multi-prefix support to improve access level tracking.
        try!(self.server.send(CAP(REQ, Some("multi-prefix"))));
        try!(self.server.send(CAP(END, None))); // Then, send a CAP END to end the negotiation.
        if self.server.config().password() != "" {
            try!(self.server.send(PASS(self.server.config().password())));
        }
        try!(self.server.send(NICK(self.server.config().nickname())));
        try!(self.server.send(USER(self.server.config().username(), "0",
                              self.server.config().real_name())));
        Ok(())
    }

    /// Sends a PONG with the specified message.
    #[stable]
    pub fn send_pong(&self, msg: &str) -> IoResult<()> {
        self.server.send(PONG(msg, None))
    }

    /// Joins the specified channel or chanlist.
    #[stable]
    pub fn send_join(&self, chanlist: &str) -> IoResult<()> {
        self.server.send(JOIN(chanlist, None))
    }

    /// Attempts to oper up using the specified username and password.
    #[stable]
    pub fn send_oper(&self, username: &str, password: &str) -> IoResult<()> {
        self.server.send(OPER(username, password))
    }

    /// Sends a message to the specified target.
    #[stable]
    pub fn send_privmsg(&self, target: &str, message: &str) -> IoResult<()> {
        for line in message.split_str("\r\n") {
            try!(self.server.send(PRIVMSG(target, line)))
        }
        Ok(())
    }

    /// Sends a notice to the specified target.
    #[stable]
    pub fn send_notice(&self, target: &str, message: &str) -> IoResult<()> {
        for line in message.split_str("\r\n") {
            try!(self.server.send(NOTICE(target, line)))
        }
        Ok(())
    }

    /// Sets the topic of a channel or requests the current one.
    /// If `topic` is an empty string, it won't be included in the message.
    #[unstable = "Design may change."]
    pub fn send_topic(&self, channel: &str, topic: &str) -> IoResult<()> {
        self.server.send(TOPIC(channel, if topic.len() == 0 {
            None
        } else {
            Some(topic)
        }))
    }

    /// Kills the target with the provided message.
    #[stable]
    pub fn send_kill(&self, target: &str, message: &str) -> IoResult<()> {
        self.server.send(KILL(target, message))
    }

    /// Kicks the listed nicknames from the listed channels with a comment.
    /// If `message` is an empty string, it won't be included in the message.
    #[unstable = "Design may change."]
    pub fn send_kick(&self, chanlist: &str, nicklist: &str, message: &str) -> IoResult<()> {
        self.server.send(KICK(chanlist, nicklist, if message.len() == 0 {
            None
        } else {
            Some(message)
        }))
    }

    /// Changes the mode of the target.
    /// If `modeparmas` is an empty string, it won't be included in the message.
    #[unstable = "Design may change."]
    pub fn send_mode(&self, target: &str, mode: &str, modeparams: &str) -> IoResult<()> {
        self.server.send(MODE(target, mode, if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams)
        }))
    }

    /// Changes the mode of the target by force.
    /// If `modeparams` is an empty string, it won't be included in the message.
    #[unstable = "Design may change."]
    pub fn send_samode(&self, target: &str, mode: &str, modeparams: &str) -> IoResult<()> {
        self.server.send(SAMODE(target, mode, if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams)
        }))
    }

    /// Forces a user to change from the old nickname to the new nickname.
    #[stable]
    pub fn send_sanick(&self, old_nick: &str, new_nick: &str) -> IoResult<()> {
        self.server.send(SANICK(old_nick, new_nick))
    }

    /// Invites a user to the specified channel.
    #[stable]
    pub fn send_invite(&self, nick: &str, chan: &str) -> IoResult<()> {
        self.server.send(INVITE(nick, chan))
    }

    /// Quits the server entirely with a message. 
    /// This defaults to `Powered by Rust.` if none is specified.
    #[unstable = "Design may change."]
    pub fn send_quit(&self, msg: &str) -> IoResult<()> {
        self.server.send(QUIT(Some(if msg.len() == 0 {
            "Powered by Rust."
        } else {
            msg
        })))
    }

    /// Sends a CTCP-escaped message to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_ctcp(&self, target: &str, msg: &str) -> IoResult<()> {
        self.send_privmsg(target, &format!("\u{001}{}\u{001}", msg)[])
    }

    /// Sends an action command to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_action(&self, target: &str, msg: &str) -> IoResult<()> {
        self.send_ctcp(target, &format!("ACTION {}", msg)[])
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_finger(&self, target: &str) -> IoResult<()> {
        self.send_ctcp(target, "FINGER")
    }

    /// Sends a version request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_version(&self, target: &str) -> IoResult<()> {
        self.send_ctcp(target, "VERSION")
    }

    /// Sends a source request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_source(&self, target: &str) -> IoResult<()> {
        self.send_ctcp(target, "SOURCE")
    }

    /// Sends a user info request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_user_info(&self, target: &str) -> IoResult<()> {
        self.send_ctcp(target, "USERINFO")
    }

    /// Sends a finger request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_ctcp_ping(&self, target: &str) -> IoResult<()> {
        let time = get_time();
        self.send_ctcp(target, &format!("PING {}.{}", time.sec, time.nsec)[])
    }

    /// Sends a time request to the specified target.
    /// This requires the CTCP feature to be enabled.
    #[stable]
    #[cfg(feature = "ctcp")]
    pub fn send_time(&self, target: &str) -> IoResult<()> {
        self.send_ctcp(target, "TIME")
    }
}

#[cfg(test)]
mod test {
    use super::Wrapper;
    use std::default::Default;
    use std::old_io::MemWriter;
    use std::old_io::util::NullReader;
    use client::conn::Connection;
    use client::data::Config;
    use client::server::IrcServer;
    use client::server::test::{get_server_value, test_config};

    #[test]
    fn identify() {
        let server = IrcServer::from_connection(test_config(), 
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.identify().unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "CAP REQ :multi-prefix\r\nCAP END\r\nNICK :test\r\nUSER test 0 * :test\r\n");
    }

    #[test]
    fn identify_with_password() {
        let server = IrcServer::from_connection(Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            server: Some(format!("irc.test.net")),
            password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }, Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.identify().unwrap();
        }
        assert_eq!(&get_server_value(server)[], "CAP REQ :multi-prefix\r\nCAP END\r\n\
        PASS :password\r\nNICK :test\r\nUSER test 0 * :test\r\n");
    }

    #[test]
    fn send_pong() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_pong("irc.test.net").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PONG :irc.test.net\r\n");
    }

    #[test]
    fn send_join() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_join("#test,#test2,#test3").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "JOIN #test,#test2,#test3\r\n");
    }

    #[test]
    fn send_oper() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_oper("test", "test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "OPER test :test\r\n");
    }

    #[test]
    fn send_privmsg() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_privmsg("#test", "Hi, everybody!").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG #test :Hi, everybody!\r\n");
    }

    #[test]
    fn send_notice() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_notice("#test", "Hi, everybody!").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "NOTICE #test :Hi, everybody!\r\n");
    }

    #[test]
    fn send_topic_no_topic() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_topic("#test", "").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "TOPIC #test\r\n");
    }

    #[test]
    fn send_topic() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_topic("#test", "Testing stuff.").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "TOPIC #test :Testing stuff.\r\n");
    }

    #[test]
    fn send_kill() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_kill("test", "Testing kills.").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "KILL test :Testing kills.\r\n");
    }

    #[test]
    fn send_kick_no_message() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_kick("#test", "test", "").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "KICK #test test\r\n");
    }

    #[test]
    fn send_kick() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_kick("#test", "test", "Testing kicks.").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "KICK #test test :Testing kicks.\r\n");
    }

    #[test]
    fn send_mode_no_modeparams() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_mode("#test", "+i", "").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "MODE #test +i\r\n");
    }

    #[test]
    fn send_mode() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_mode("#test", "+o", "test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "MODE #test +o test\r\n");
    }

    #[test]
    fn send_samode_no_modeparams() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_samode("#test", "+i", "").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "SAMODE #test +i\r\n");
    }

    #[test]
    fn send_samode() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_samode("#test", "+o", "test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "SAMODE #test +o test\r\n");
    }

    #[test]
    fn send_sanick() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_sanick("test", "test2").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "SANICK test test2\r\n");
    }

    #[test]
    fn send_invite() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_invite("test", "#test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "INVITE test #test\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_ctcp("test", "MESSAGE").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}MESSAGE\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_action() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_action("test", "tests.").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}ACTION tests.\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_finger() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_finger("test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}FINGER\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_version() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_version("test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}VERSION\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_source() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_source("test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}SOURCE\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_user_info() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_user_info("test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}USERINFO\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_ctcp_ping() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_ctcp_ping("test").unwrap();
        }
        let val = get_server_value(server);
        println!("{}", val);
        assert!(val.starts_with("PRIVMSG test :\u{001}PING "));
        assert!(val.ends_with("\u{001}\r\n"));
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn send_time() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_time("test").unwrap();
        }
        assert_eq!(&get_server_value(server)[],
        "PRIVMSG test :\u{001}TIME\u{001}\r\n");
    }
}
