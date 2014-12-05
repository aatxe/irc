//! Interface for working with IRC Servers
#![experimental]
use std::collections::HashMap;
use std::io::{BufferedReader, BufferedWriter, IoResult};
use std::sync::Mutex;
use conn::{Connection, NetStream};
use data::{Command, Config, Message, Response, User};
use data::Command::{JOIN, PONG};
use data::kinds::{IrcReader, IrcWriter};

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
    /// Gets a list of Users in the specified channel.
    fn list_users(&self, _: &str) -> Option<Vec<User>>;
}

/// A thread-safe implementation of an IRC Server connection.
#[experimental]
pub struct IrcServer<T: IrcReader, U: IrcWriter> {
    /// The thread-safe IRC connection.
    conn: Connection<T, U>,
    /// The configuration used with this connection.
    config: Config,
    /// A thread-safe map of channels to the list of users in them.
    chanlists: Mutex<HashMap<String, Vec<User>>>,
}

/// An IrcServer over a buffered NetStream.
pub type NetIrcServer = IrcServer<BufferedReader<NetStream>, BufferedWriter<NetStream>>;

impl IrcServer<BufferedReader<NetStream>, BufferedWriter<NetStream>> {
    /// Creates a new IRC Server connection from the configuration at the specified path, 
    /// connecting immediately.
    #[experimental]
    pub fn new(config: &str) -> IoResult<NetIrcServer> {
        IrcServer::from_config(try!(Config::load_utf8(config)))
    }

    /// Creates a new IRC server connection from the configuration at the specified path with the
    /// specified timeout in milliseconds, connecting immediately.
    #[experimental]
    pub fn with_timeout(config: &str, timeout_ms: u64) -> IoResult<NetIrcServer> {
        IrcServer::from_config_with_timeout(try!(Config::load_utf8(config)), timeout_ms)    
    }

    /// Creates a new IRC server connection from the specified configuration, connecting
    /// immediately.
    #[experimental]
    pub fn from_config(config: Config) -> IoResult<NetIrcServer> {
        let conn = try!(if config.use_ssl() {
            Connection::connect_ssl(config.server(), config.port())
        } else {
            Connection::connect(config.server(), config.port())
        });
        Ok(IrcServer { config: config, conn: conn, chanlists: Mutex::new(HashMap::new()) })
    }

    /// Creates a new IRC server connection from the specified configuration with the specified 
    /// timeout in milliseconds, connecting
    /// immediately.
    #[experimental]
    pub fn from_config_with_timeout(config: Config, timeout_ms: u64) -> IoResult<NetIrcServer> {
        let conn = try!(if config.use_ssl() {
            Connection::connect_ssl_with_timeout(config.server(), config.port(), timeout_ms)
        } else {
            Connection::connect_with_timeout(config.server(), config.port(), timeout_ms)
        });
        Ok(IrcServer { config: config, conn: conn, chanlists: Mutex::new(HashMap::new()) })
    }

}

impl<'a, T: IrcReader, U: IrcWriter> Server<'a, T, U> for IrcServer<T, U> {
    fn config(&self) -> &Config {
        &self.config
    }

    #[cfg(feature = "encode")]
    fn send(&self, command: Command) -> IoResult<()> {
        self.conn.send(command.to_message(), self.config.encoding())
    }

    #[cfg(not(feature = "encode"))]
    fn send(&self, command: Command) -> IoResult<()> {
        self.conn.send(command.to_message())
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        ServerIterator::new(self)
    }

    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists.lock().get(&chan.into_string()).cloned()
    }
}

impl<T: IrcReader, U: IrcWriter> IrcServer<T, U> {
    /// Creates an IRC server from the specified configuration, and any arbitrary Connection.
    #[experimental]
    pub fn from_connection(config: Config, conn: Connection<T, U>) -> IrcServer<T, U> {
        IrcServer { conn: conn, config: config, chanlists: Mutex::new(HashMap::new()) }
    }

    /// Gets a reference to the IRC server's connection.
    pub fn conn(&self) -> &Connection<T, U> {
        &self.conn
    }

    /// Handles messages internally for basic bot functionality.
    #[experimental]
    fn handle_message(&self, msg: &Message) {
        if let Some(resp) = Response::from_message(msg) {
            if resp == Response::RPL_NAMREPLY {
                if let Some(users) = msg.suffix.clone() {
                    if let [_, _, ref chan] = msg.args[] {
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
            } else if resp == Response::RPL_ENDOFMOTD || resp == Response::ERR_NOMOTD {
                for chan in self.config.channels().into_iter() {
                    self.send(JOIN(chan[], None)).unwrap();
                }
            }
            return
        }
        if msg.command[] == "PING" {
            self.send(PONG(msg.suffix.as_ref().unwrap()[], None)).unwrap();
        } else if msg.command[] == "JOIN" || msg.command[] == "PART" {
            let chan = match msg.suffix {
                Some(ref suffix) => suffix[],
                None => msg.args[0][],
            };
            if let Some(vec) = self.chanlists.lock().get_mut(&String::from_str(chan)) {
                if let Some(ref source) = msg.prefix {
                    if let Some(i) = source.find('!') {
                        if msg.command[] == "JOIN" {
                            vec.push(User::new(source[..i]));
                        } else {
                            if let Some(n) = vec.as_slice().position_elem(&User::new(source[..i])) {
                                vec.swap_remove(n);
                            }
                        }
                    }
                }
            }
        } else if let ("MODE", [ref chan, ref mode, ref user]) = (msg.command[], msg.args[]) {
            if let Some(vec) = self.chanlists.lock().get_mut(chan) {
                if let Some(n) = vec.as_slice().position_elem(&User::new(user[])) {
                    vec[n].update_access_level(mode[]);
                }
            }
        }
    }
}

/// An Iterator over an IrcServer's incoming Messages.
#[experimental]
pub struct ServerIterator<'a, T: IrcReader, U: IrcWriter> {
    server: &'a IrcServer<T, U>
}

impl<'a, T: IrcReader, U: IrcWriter> ServerIterator<'a, T, U> {
    /// Creates a new ServerIterator for the desired IrcServer.
    #[experimental]
    pub fn new(server: &IrcServer<T, U>) -> ServerIterator<T, U> {
        ServerIterator {
            server: server
        }
    }

    /// Gets the next line from the connection.
    #[cfg(feature = "encode")]
    fn get_next_line(&self) -> IoResult<String> {
        self.server.conn.recv(self.server.config.encoding())
    }

    /// Gets the next line from the connection.
    #[cfg(not(feature = "encode"))]
    fn get_next_line(&self) -> IoResult<String> {
        self.server.conn.recv()
    }
}

impl<'a, T: IrcReader, U: IrcWriter> Iterator<Message> for ServerIterator<'a, T, U> {
    fn next(&mut self) -> Option<Message> {
        match self.get_next_line() {
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
    use std::default::Default;
    use std::io::{MemReader, MemWriter};
    use std::io::util::{NullReader, NullWriter};
    use conn::Connection;
    use data::{Config, User};
    use data::command::Command::PRIVMSG;
    use data::kinds::IrcReader;

    pub fn test_config() -> Config {
        Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            server: Some(format!("irc.test.net")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }
    }


    pub fn get_server_value<T: IrcReader>(server: IrcServer<T, MemWriter>) -> String {
        String::from_utf8((*server.conn().writer().deref()).get_ref().to_vec()).unwrap()
    }

    #[test]
    fn iterator() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n\
                   :test!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            MemReader::new(exp.as_bytes().to_vec()), NullWriter
        ));
        let mut messages = String::new();
        for message in server.iter() {
            messages.push_str(message.into_string()[]);
        }
        assert_eq!(messages[], exp);
    }

    #[test]
    fn handle_message() {
        let value = "PING :irc.test.net\r\n:irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           MemReader::new(value.as_bytes().to_vec()), MemWriter::new()
        ));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(get_server_value(server)[],
        "PONG :irc.test.net\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn send() {
        let server = IrcServer::from_connection(test_config(), Connection::new(
           NullReader, MemWriter::new()
        ));
        assert!(server.send(PRIVMSG("#test", "Hi there!")).is_ok());
        assert_eq!(get_server_value(server)[],
        "PRIVMSG #test :Hi there!\r\n");
    }

    #[test]
    fn user_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           MemReader::new(value.as_bytes().to_vec()), NullWriter
        ));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("~owner"), User::new("&admin")])
    }

    #[test]
    fn user_tracking_names_join() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :test2!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            MemReader::new(value.as_bytes().to_vec()), NullWriter
        ));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("~owner"), User::new("&admin"), User::new("test2")])
    }

    #[test]
    fn user_tracking_names_part() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :owner!test@test PART #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            MemReader::new(value.as_bytes().to_vec()), NullWriter
        ));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("&admin")])
    }

    #[test]
    fn user_tracking_names_mode() {
        let value = ":irc.test.net 353 test = #test :+test ~owner &admin\r\n\
                     :test!test@test MODE #test +o test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            MemReader::new(value.as_bytes().to_vec()), NullWriter
        ));
        for message in server.iter() {
            println!("{}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("@test"), User::new("~owner"), User::new("&admin")]);
        let mut exp = User::new("@test");
        exp.update_access_level("+v");
        assert_eq!(server.list_users("#test").unwrap()[0].highest_access_level(),
                   exp.highest_access_level());
        // The following tests if the maintained user contains the same entries as what is expected
        // but ignores the ordering of these entries.
        let mut levels = server.list_users("#test").unwrap()[0].access_levels();
        levels.retain(|l| exp.access_levels().contains(l));
        assert_eq!(levels.len(), exp.access_levels().len()); 
    }
}
