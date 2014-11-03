//! Utilities and shortcuts for working with IRC servers
#![experimental]

use std::io::IoResult;
use data::command::{JOIN, NICK, PONG, USER};
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
