//! Interface for working with IRC Servers
#![experimental]
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use conn::Connection;
use data::command::Command;
use data::config::Config;
use data::kinds::{IrcReader, IrcWriter};
use data::message::Message;

/// Trait describing core Server functionality
#[experimental]
pub trait Server<'a, T, U> {
    /// Gets the configuration being used with this Server
    fn config(&self) -> &Config;
    /// Sends a Command to this Server
    fn send(&self, _: Command) -> IoResult<()>;
    /// Gets an Iterator over Messages received by this Server
    fn iter(&'a self) -> ServerIterator<'a, T, U>;
}

/// A thread-safe implementation of an IRC Server connection
#[experimental]
pub struct IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// The thread-safe IRC connection
    conn: Connection<T, U>,
    /// The configuration used with this connection
    config: Config
}

impl<'a> IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    /// Creates a new IRC Server connection from the specified configuration, connecting immediately.
    #[experimental]
    pub fn new(config: &str) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load_utf8(config));
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer {
            conn: conn,
            config: config
        })
    }
}

impl<'a, T, U> Server<'a, T, U> for IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn config(&self) -> &Config {
        &self.config
    }

    fn send(&self, command: Command) -> IoResult<()> {
        self.conn.send(command.to_message())
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        ServerIterator::new(self)
    }
}

impl<'a, T, U> IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// Creates an IRC server from the specified configuration, and any arbitrary Connection
    #[experimental]
    pub fn from_connection(config: &str, conn: Connection<T, U>) -> IoResult<IrcServer<'a, T, U>> {
        Ok(IrcServer {
            conn: conn,
            config: try!(Config::load_utf8(config))
        })
    }
}

/// An Iterator over an IrcServer's incoming Messages
#[experimental]
pub struct ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub server: &'a IrcServer<'a, T, U>
}

impl<'a, T, U> ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// Creates a new ServerIterator for the desired IrcServer
    #[experimental]
    pub fn new(server: &'a IrcServer<'a, T, U>) -> ServerIterator<'a, T, U> {
        ServerIterator {
            server: server
        }
    }
}

impl<'a, T, U> Iterator<Message> for ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    fn next(&mut self) -> Option<Message> {
        let line = self.server.conn.recv();
        if let Err(_) = line { return None }
        from_str(line.unwrap()[])
    }
}
