//! Interface for working with IRC Servers
#![experimental]
use std::collections::HashMap;
use std::io::{BufferedStream, IoResult};
use std::sync::Mutex;
use conn::{Connection, NetStream};
use data::command::{Command, JOIN, PONG};
use data::config::Config;
use data::kinds::IrcStream;
use data::message::Message;
use data::user::User;

pub mod utils;

/// Trait describing core Server functionality.
#[experimental]
pub trait Server<'a, T> {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;
    /// Sends a Command to this Server.
    fn send(&self, _: Command) -> IoResult<()>;
    /// Gets an Iterator over Messages received by this Server.
    fn iter(&'a self) -> ServerIterator<'a, T>;
}

/// A thread-safe implementation of an IRC Server connection.
#[experimental]
pub struct IrcServer<'a, T> where T: IrcStream {
    /// The thread-safe IRC connection.
    conn: Connection<T>,
    /// The configuration used with this connection.
    config: Config,
    /// A thread-safe map of channels to the list of users in them.
    chanlists: Mutex<HashMap<String, Vec<User>>>,
}

impl<'a> IrcServer<'a, BufferedStream<NetStream>> {
    /// Creates a new IRC Server connection from the configuration at the specified path, connecting immediately.
    #[experimental]
    pub fn new(config: &str) -> IoResult<IrcServer<'a, BufferedStream<NetStream>>> {
        let config = try!(Config::load_utf8(config));
        let conn = try!(if config.use_ssl {
            Connection::connect_ssl(config.server[], config.port)
        } else {
            Connection::connect(config.server[], config.port)
        });
        Ok(IrcServer { config: config, conn: conn, chanlists: Mutex::new(HashMap::new()) })
    }

    /// Creates a new IRC server connection from the specified configuration, connecting immediately.
    #[experimental]
    pub fn from_config(config: Config) -> IoResult<IrcServer<'a, BufferedStream<NetStream>>> {
        let conn = try!(if config.use_ssl {
            Connection::connect_ssl(config.server[], config.port)
        } else {
            Connection::connect(config.server[], config.port)
        });
        Ok(IrcServer { config: config, conn: conn, chanlists: Mutex::new(HashMap::new()) })
    }
}

impl<'a, T> Server<'a, T> for IrcServer<'a, T> where T: IrcStream {
    fn config(&self) -> &Config {
        &self.config
    }

    fn send(&self, command: Command) -> IoResult<()> {
        self.conn.send(command.to_message())
    }

    fn iter(&'a self) -> ServerIterator<'a, T> {
        ServerIterator::new(self)
    }
}

impl<'a, T> IrcServer<'a, T> where T: IrcStream {
    /// Creates an IRC server from the specified configuration, and any arbitrary Connection.
    #[experimental]
    pub fn from_connection(config: Config, conn: Connection<T>) -> IrcServer<'a, T> {
        IrcServer { conn: conn, config: config, chanlists: Mutex::new(HashMap::new()) }
    }

    /// Gets a reference to the IRC server's connection.
    pub fn conn(&self) -> &Connection<T> {
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
        } /* FIXME: it's not really clear why this stuff is broken. 
        else if message.command[] == "353" { // /NAMES
            if let Some(users) = message.suffix.clone() {
                if let [_, _, ref chan] = message.args[] {
                    for user in users.split_str(" ") {
                        if match self.chanlists.lock().get_mut(chan) {
                            Some(vec) => { vec.push(User::new(user)); false },
                            None => true,
                        } {
                            self.chanlists.lock().insert(chan.clone(), vec!(User::new(user)));
                        }
                    }
                }
            }
        } else if message.command[] == "JOIN" || message.command[] == "PART" {
            let chan = match message.suffix {
                Some(ref suffix) => suffix[],
                None => message.args[0][],
            };
            if let Some(vec) = self.chanlists.lock().get_mut(&String::from_str(chan)) {
                if let Some(ref source) = message.suffix {
                    if let Some(i) = source.find('!') {
                        if message.command[] == "JOIN" {
                            vec.push(User::new(source[..i]));
                        } else {
                            if let Some(n) = vec.as_slice().position_elem(&User::new(source[..i])) {
                                vec.swap_remove(n);
                            }
                        }
                    }
                }
            }
        } */
        /* TODO: implement more message handling */
    }
}

/// An Iterator over an IrcServer's incoming Messages.
#[experimental]
pub struct ServerIterator<'a, T> where T: IrcStream {
    pub server: &'a IrcServer<'a, T>
}

impl<'a, T> ServerIterator<'a, T> where T: IrcStream {
    /// Creates a new ServerIterator for the desired IrcServer.
    #[experimental]
    pub fn new(server: &'a IrcServer<'a, T>) -> ServerIterator<'a, T> {
        ServerIterator {
            server: server
        }
    }
}

impl<'a, T> Iterator<Message> for ServerIterator<'a, T> where T: IrcStream {
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
    use conn::{Connection, IoStream};
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
            use_ssl: false,
            channels: vec![format!("#test"), format!("#test2")],
            options: HashMap::new(),
        }
    }

    pub fn get_server_value<U>(server: IrcServer<IoStream<MemWriter, U>>) -> String where U: IrcReader {
        String::from_utf8(server.conn().stream().value()).unwrap()
    }

    #[test]
    fn iterator() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n:test!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(IoStream::new(NullWriter, MemReader::new(exp.as_bytes().to_vec()))));
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
                     Connection::new(IoStream::new(MemWriter::new(), MemReader::new(value.as_bytes().to_vec()))));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(get_server_value(server)[],
        "PONG :irc.test.net\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn send() {
        let server = IrcServer::from_connection(test_config(),
                     Connection::new(IoStream::new(MemWriter::new(), NullReader)));
        assert!(server.send(PRIVMSG("#test", "Hi there!")).is_ok());
        assert_eq!(get_server_value(server)[],
        "PRIVMSG #test :Hi there!\r\n");
    }
}
