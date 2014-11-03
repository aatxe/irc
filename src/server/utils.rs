//! Utilities and shortcuts for working with IRC servers
#![experimental]

use std::io::IoResult;
use data::command::{Command, JOIN, KILL, NICK, OPER, PONG, PRIVMSG, SAMODE, SANICK, USER};
use data::config::Config;
use data::kinds::{IrcReader, IrcWriter};
use server::{Server, ServerIterator};

/// Functionality-providing wrapper for Server
#[experimental]
pub struct Wrapper<'a, T, U> where T: IrcWriter, U: IrcReader {
    server: &'a Server<'a, T, U> + 'a
}

impl<'a, T, U> Server<'a, T, U> for Wrapper<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn config(&self) -> &Config {
        self.server.config()
    }

    fn send(&self, command: Command) -> IoResult<()> {
        self.server.send(command)
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        self.server.iter()
    }
}

impl<'a, T, U> Wrapper<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// Creates a new Wrapper from the given Server
    #[experimental]
    pub fn new(server: &'a Server<'a, T, U>) -> Wrapper<'a, T, U> {
        Wrapper { server: server }
    }

    /// Sends a NICK and USER to identify
    #[experimental]
    pub fn identify(&self) -> IoResult<()> {
        try!(self.server.send(NICK(self.server.config().nickname[])));
        self.server.send(USER(self.server.config().username[], "0", self.server.config().realname[]))
    }

    /// Sends a PONG with the specified message
    #[experimental]
    pub fn send_pong(&self, msg: &str) -> IoResult<()> {
        self.server.send(PONG(msg, None))
    }

    /// Joins the specified channel or chanlist
    #[experimental]
    pub fn send_join(&self, chanlist: &str) -> IoResult<()> {
        self.server.send(JOIN(chanlist, None))
    }

    /// Attempts to oper up using the specified username and password
    #[experimental]
    pub fn send_oper(&self, username: &str, password: &str) -> IoResult<()> {
        self.server.send(OPER(username, password))
    }

    /// Sends a message to the specified target
    #[experimental]
    pub fn send_privmsg(&self, target: &str, message: &str) -> IoResult<()> {
        for line in message.split_str("\r\n") {
            try!(self.server.send(PRIVMSG(target, line)))
        }
        Ok(())
    }

    /// Kills the target with the provided message
    #[experimental]
    pub fn send_kill(&self, target: &str, message: &str) -> IoResult<()> {
        self.server.send(KILL(target, message))
    }

    /// Changes the mode of the target
    #[experimental]
    pub fn send_samode(&self, target: &str, mode: &str, modeparams: &str) -> IoResult<()> {
        self.server.send(SAMODE(target, mode, if modeparams.len() == 0 {
            None
        } else {
            Some(modeparams)
        }))
    }

    /// Forces a user to change from the old nickname to the new nickname
    #[experimental]
    pub fn send_sanick(&self, old_nick: &str, new_nick: &str) -> IoResult<()> {
        self.server.send(SANICK(old_nick, new_nick))
    }
}
