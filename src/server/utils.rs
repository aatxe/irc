//! Utilities and shortcuts for working with IRC servers
#![experimental]

use std::io::IoResult;
use data::command::{JOIN, KILL, NICK, OPER, PONG, PRIVMSG, SAMODE, SANICK, USER};
use data::kinds::{IrcReader, IrcWriter};
use server::Server;

/// Sends a NICK and USER to identify
pub fn identify<'a, T, U>(server: &Server<'a, T, U>) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    try!(server.send(NICK(server.config().nickname[])));
    server.send(USER(server.config().username[], "0", server.config().realname[]))
}

pub fn send_pong<'a, T, U>(server: &Server<'a, T, U>, msg: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(PONG(msg, None))
}

pub fn send_join<'a, T, U>(server: &Server<'a, T, U>, chanlist: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(JOIN(chanlist, None))
}

pub fn send_oper<'a, T, U>(server: &Server<'a, T, U>, username: &str, password: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(OPER(username, password))
}

pub fn send_privmsg<'a, T, U>(server: &Server<'a, T, U>, target: &str, message: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(PRIVMSG(target, message))
}

pub fn send_kill<'a, T, U>(server: &Server<'a, T, U>, target: &str, message: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(KILL(target, message))
}

pub fn send_samode<'a, T, U>(server: &Server<'a, T, U>, target: &'a str, mode: &'a str, modeparams: Option<&'a str>) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(SAMODE(target, mode, modeparams))
}

pub fn send_sanick<'a, T, U>(server: &Server<'a, T, U>, old_nick: &str, new_nick: &str) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    server.send(SANICK(old_nick, new_nick))
}
