//! Utilities and shortcuts for working with IRC servers
#![experimental]

use std::io::IoResult;
use data::command::{NICK, USER};
use data::kinds::{IrcReader, IrcWriter};
use server::Server;

/// Sends a NICK and USER to identify
pub fn identify<'a, T, U>(server: &Server<'a, T, U>) -> IoResult<()> where T: IrcWriter, U: IrcReader {
    try!(server.send(NICK(server.config().nickname[])));
    server.send(USER(server.config().username[], "0", server.config().realname[]))
}
