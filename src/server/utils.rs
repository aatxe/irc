//! Utilities and shortcuts for working with IRC servers.
#![experimental]

use std::io::IoResult;
use data::{Command, Config, User};
use data::Command::{CAP, INVITE, JOIN, KICK, KILL, MODE, NICK, NICKSERV, NOTICE};
use data::Command::{OPER, PASS, PONG, PRIVMSG, SAMODE, SANICK, TOPIC, USER};
use data::command::CapSubCommand::{END, REQ};
use data::kinds::{IrcReader, IrcWriter};
use server::{Server, ServerIterator};

/// Functionality-providing wrapper for Server.
/// Wrappers are currently not thread-safe, and should be created per-thread, as needed.
#[experimental]
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

impl<'a, T: IrcReader, U: IrcWriter> Wrapper<'a, T, U> {
    /// Creates a new Wrapper from the given Server.
    #[experimental]
    pub fn new(server: &'a Server<'a, T, U>) -> Wrapper<'a, T, U> {
        Wrapper { server: server }
    }

    /// Sends a NICK and USER to identify.
    #[experimental]
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
        if self.server.config().nick_password() != "" {
            try!(self.server.send(NICKSERV(
                format!("IDENTIFY {}", self.server.config().nick_password())[]
            )));
        }
        Ok(())
    }

    /// Sends a PONG with the specified message.
    #[experimental]
    pub fn send_pong(&self, msg: &str) -> IoResult<()> {
        self.server.send(PONG(msg, None))
    }

    /// Joins the specified channel or chanlist.
    #[experimental]
    pub fn send_join(&self, chanlist: &str) -> IoResult<()> {
        self.server.send(JOIN(chanlist, None))
    }

    /// Attempts to oper up using the specified username and password.
    #[experimental]
    pub fn send_oper(&self, username: &str, password: &str) -> IoResult<()> {
        self.server.send(OPER(username, password))
    }

    /// Sends a message to the specified target.
    #[experimental]
    pub fn send_privmsg(&self, target: &str, message: &str) -> IoResult<()> {
        for line in message.split_str("\r\n") {
            try!(self.server.send(PRIVMSG(target, line)))
        }
        Ok(())
    }

    /// Sends a notice to the specified target.
    #[experimental]
    pub fn send_notice(&self, target: &str, message: &str) -> IoResult<()> {
        for line in message.split_str("\r\n") {
            try!(self.server.send(NOTICE(target, line)))
        }
        Ok(())
    }

    /// Sets the topic of a channel or requests the current one.
    /// If `topic` is an empty string, it won't be included in the message.
    #[experimental]
    pub fn send_topic(&self, channel: &str, topic: &str) -> IoResult<()> {
        self.server.send(TOPIC(channel, if topic.len() == 0 {
            None
        } else {
            Some(topic)
        }))
    }

    /// Kills the target with the provided message.
    #[experimental]
    pub fn send_kill(&self, target: &str, message: &str) -> IoResult<()> {
        self.server.send(KILL(target, message))
    }

    /// Kicks the listed nicknames from the listed channels with a comment.
    /// If `message` is an empty string, it won't be included in the message.
    #[experimental]
    pub fn send_kick(&self, chanlist: &str, nicklist: &str, message: &str) -> IoResult<()> {
        self.server.send(KICK(chanlist, nicklist, if message.len() == 0 {
            None
        } else {
            Some(message)
        }))
    }

    /// Changes the mode of the target.
    /// If `modeparmas` is an empty string, it won't be included in the message.
    #[experimental]
    pub fn send_mode(&self, target: &str, mode: &str, modeparams: &str) -> IoResult<()> {
        self.server.send(MODE(target, mode, if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams)
        }))
    }

    /// Changes the mode of the target by force.
    /// If `modeparams` is an empty string, it won't be included in the message.
    #[experimental]
    pub fn send_samode(&self, target: &str, mode: &str, modeparams: &str) -> IoResult<()> {
        self.server.send(SAMODE(target, mode, if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams)
        }))
    }

    /// Forces a user to change from the old nickname to the new nickname.
    #[experimental]
    pub fn send_sanick(&self, old_nick: &str, new_nick: &str) -> IoResult<()> {
        self.server.send(SANICK(old_nick, new_nick))
    }

    /// Invites a user to the specified channel.
    #[experimental]
    pub fn send_invite(&self, nick: &str, chan: &str) -> IoResult<()> {
        self.server.send(INVITE(nick, chan))
    }
}

#[cfg(test)]
mod test {
    use super::Wrapper;
    use std::default::Default;
    use std::io::MemWriter;
    use std::io::util::NullReader;
    use conn::Connection;
    use data::Config;
    use server::IrcServer;
    use server::test::{get_server_value, test_config};

    #[test]
    fn identify() {
        let server = IrcServer::from_connection(test_config(), 
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.identify().unwrap();
        }
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[], "CAP REQ :multi-prefix\r\nCAP END\r\n\
        PASS :password\r\nNICK :test\r\nUSER test 0 * :test\r\n");
    }

    #[test]
    fn identify_with_nick_password() {
        let server = IrcServer::from_config(Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            server: Some(format!("irc.test.net")),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }, Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.identify().unwrap();
        }
        assert_eq!(get_server_value(server)[], "CAP REQ :multi-prefix\r\nCAP END\r\nNICK :test\r\n\
                   USER test 0 * :test\r\nNICKSERV :IDENTIFY password\r\n");
    }

    #[test]
    fn send_pong() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullReader, MemWriter::new()));
        {
            let wrapper = Wrapper::new(&server);
            wrapper.send_pong("irc.test.net").unwrap();
        }
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
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
        assert_eq!(get_server_value(server)[],
        "INVITE test #test\r\n");
    }
}
