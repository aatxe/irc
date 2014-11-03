//! Interface for working with IRC Servers
#![experimental]
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use conn::Connection;
use data::command::{Command, PONG};
use data::config::Config;
use data::kinds::{IrcReader, IrcWriter};
use data::message::Message;

pub mod utils;

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
    /// Creates a new IRC Server connection from the configuration at the specified path, connecting immediately.
    #[experimental]
    pub fn new(config: &str) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load_utf8(config));
        let conn = try!(Connection::connect(config.server[], config.port));
        IrcServer::from_connection(config, conn)
    }

    /// Creates a new IRC server connection from the specified configuration, connecting immediately.
    #[experimental]
    pub fn from_config(config: Config) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let conn = try!(Connection::connect(config.server[], config.port));
        IrcServer::from_connection(config, conn)
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
    pub fn from_connection(config: Config, conn: Connection<T, U>) -> IoResult<IrcServer<'a, T, U>> {
        Ok(IrcServer {
            conn: conn,
            config: config
        })
    }

    fn handle_message(&self, message: &Message) {
        if message.command[] == "PING" {
            self.send(PONG(message.suffix.as_ref().unwrap()[], None)).unwrap();
        }
        /* TODO: implement more message handling */
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
        match line {
            Err(_) => None,
            Ok(msg) => {
                let message = from_str(msg[]);
                self.server.handle_message(message.as_ref().unwrap());
                message
            }
        }
    }
}
