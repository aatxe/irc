//! Interface for working with IRC Servers
#![experimental]
use std::io::{BufferedReader, BufferedWriter, IoResult, TcpStream};
use conn::Connection;
use data::command::{Command, JOIN, PONG};
use data::config::Config;
use data::kinds::{IrcReader, IrcWriter};
use data::message::Message;

pub mod utils;

/// Trait describing core Server functionality.
#[experimental]
pub trait Server<'a, T, U> {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;
    /// Sends a Command to this Server.
    fn send(&self, _: Command) -> IoResult<()>;
    /// Gets an Iterator over Messages received by this Server.
    fn iter(&'a self) -> ServerIterator<'a, T, U>;
}

/// A thread-safe implementation of an IRC Server connection.
#[experimental]
pub struct IrcServer<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// The thread-safe IRC connection.
    conn: Connection<T, U>,
    /// The configuration used with this connection.
    config: Config
}

impl<'a> IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>> {
    /// Creates a new IRC Server connection from the configuration at the specified path, connecting immediately.
    #[experimental]
    pub fn new(config: &str) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let config = try!(Config::load_utf8(config));
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer::from_connection(config, conn))
    }

    /// Creates a new IRC server connection from the specified configuration, connecting immediately.
    #[experimental]
    pub fn from_config(config: Config) -> IoResult<IrcServer<'a, BufferedWriter<TcpStream>, BufferedReader<TcpStream>>> {
        let conn = try!(Connection::connect(config.server[], config.port));
        Ok(IrcServer::from_connection(config, conn))
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
    /// Creates an IRC server from the specified configuration, and any arbitrary Connection.
    #[experimental]
    pub fn from_connection(config: Config, conn: Connection<T, U>) -> IrcServer<'a, T, U> {
        IrcServer {
            conn: conn,
            config: config
        }
    }

    /// Gets a reference to the IRC server's connection.
    pub fn conn(&self) -> &Connection<T, U> {
        &self.conn
    }

    /// Handles messages internally for basic bot functionality.
    #[experimental]
    fn handle_message(&self, message: &Message) {
        if message.command[] == "PING" {
            self.send(PONG(message.suffix.as_ref().unwrap()[], None)).unwrap();
        } else if message.command[] == "376" || message.command[] == "422" {
            for chan in self.config.channels.iter() {
                self.send(JOIN(chan[], None)).unwrap();
            }
        }
        /* TODO: implement more message handling */
    }
}

/// An Iterator over an IrcServer's incoming Messages.
#[experimental]
pub struct ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    pub server: &'a IrcServer<'a, T, U>
}

impl<'a, T, U> ServerIterator<'a, T, U> where T: IrcWriter, U: IrcReader {
    /// Creates a new ServerIterator for the desired IrcServer.
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

#[cfg(test)]
mod test {
    use super::{IrcServer, Server};
    use std::collections::HashMap;
    use std::io::{MemReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use conn::Connection;
    use data::Config;
    use data::command::PRIVMSG;
    use data::kinds::IrcReader;

    pub fn test_config() -> Config {
        Config {
            owners: vec![format!("test")],
            nickname: format!("test"),
            username: format!("test"),
            realname: format!("test"),
            password: String::new(),
            server: format!("irc.test.net"),
            port: 6667,
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        }
    }

    pub fn get_server_value<U>(server: IrcServer<MemWriter, U>) -> String where U: IrcReader {
        String::from_utf8(server.conn().writer().get_ref().to_vec()).unwrap()
    }

    #[test]
    fn iterator() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n:test!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(NullWriter, MemReader::new(exp.as_bytes().to_vec())));
        let mut messages = String::new();
        for message in server.iter() {
            messages.push_str(message.into_string()[]);
        }
        assert_eq!(messages[], exp);
    }

    #[test]
    fn handle_message() {
        let value = "PING :irc.test.net\r\n:irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(MemWriter::new(), MemReader::new(value.as_bytes().to_vec())));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(get_server_value(server)[],
        "PONG :irc.test.net\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn send() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(MemWriter::new(), NullReader));
        assert!(server.send(PRIVMSG("#test", "Hi there!")).is_ok());
        assert_eq!(get_server_value(server)[],
        "PRIVMSG #test :Hi there!\r\n");
    }
}
