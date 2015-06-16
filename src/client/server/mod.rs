//! Interface for working with IRC Servers.
//!
//! There are currently two recommended ways to work
use std::borrow::ToOwned;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{BufReader, BufWriter, Error, ErrorKind, Result};
use std::path::Path;
use std::sync::{Mutex, RwLock};
use std::iter::Map;
use client::conn::{Connection, NetStream};
use client::data::{Command, Config, Message, Response, User};
use client::data::Command::{JOIN, NICK, NICKSERV, PONG, MODE};
use client::data::kinds::{IrcRead, IrcWrite};
#[cfg(feature = "ctcp")] use time::now;

pub mod utils;

/// Trait describing core Server functionality.
pub trait Server<'a, T, U> {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;
    /// Sends a Command to this Server.
    fn send<M: Into<Message>>(&self, message: M) -> Result<()> where Self: Sized;
    /// Gets an Iterator over Messages received by this Server.
    fn iter(&'a self) -> ServerIterator<'a, T, U>;
    /// Gets an Iterator over Commands received by this Server.
    fn iter_cmd(&'a self) -> ServerCmdIterator<'a, T, U>;
    /// Gets a list of Users in the specified channel. This will be none if the channel is not
    /// being tracked, or if tracking is not supported altogether. For best results, be sure to
    /// request `multi-prefix` support from the server.
    fn list_users(&self, channel: &str) -> Option<Vec<User>>;
}

/// A thread-safe implementation of an IRC Server connection.
pub struct IrcServer<T: IrcRead, U: IrcWrite> {
    /// The thread-safe IRC connection.
    conn: Connection<T, U>,
    /// The configuration used with this connection.
    config: Config,
    /// A thread-safe map of channels to the list of users in them.
    chanlists: Mutex<HashMap<String, Vec<User>>>,
    /// A thread-safe index to track the current alternative nickname being used.
    alt_nick_index: RwLock<usize>,
}

/// An IrcServer over a buffered NetStream.
pub type NetIrcServer = IrcServer<BufReader<NetStream>, BufWriter<NetStream>>;

impl IrcServer<BufReader<NetStream>, BufWriter<NetStream>> {
    /// Creates a new IRC Server connection from the configuration at the specified path,
    /// connecting immediately.
    pub fn new<P: AsRef<Path>>(config: P) -> Result<NetIrcServer> {
        IrcServer::from_config(try!(Config::load(config)))
    }

    /// Creates a new IRC server connection from the specified configuration, connecting
    /// immediately.
    pub fn from_config(config: Config) -> Result<NetIrcServer> {
        let conn = try!(if config.use_ssl() {
            Connection::connect_ssl(config.server(), config.port())
        } else {
            Connection::connect(config.server(), config.port())
        });
        Ok(IrcServer { config: config, conn: conn, chanlists: Mutex::new(HashMap::new()),
                       alt_nick_index: RwLock::new(0) })
    }

    /// Reconnects to the IRC server.
    pub fn reconnect(&self) -> Result<()> {
        self.conn.reconnect(self.config().server(), self.config.port())
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Server<'a, T, U> for IrcServer<T, U> {
    fn config(&self) -> &Config {
        &self.config
    }

    #[cfg(feature = "encode")]
    fn send<M: Into<Message>>(&self, msg: M) -> Result<()> {
        self.conn.send(msg, self.config.encoding())
    }

    #[cfg(not(feature = "encode"))]
    fn send<M: Into<Message>>(&self, msg: M) -> Result<()> where Self: Sized {
        self.conn.send(msg)
    }

    fn iter(&'a self) -> ServerIterator<'a, T, U> {
        ServerIterator::new(self)
    }

    fn iter_cmd(&'a self) -> ServerCmdIterator<'a, T, U> {
        self.iter().map(Command::from_message_io)
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists.lock().unwrap().get(&chan.to_owned()).cloned()
    }


    #[cfg(feature = "nochanlists")]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        None
    }
}

impl<T: IrcRead, U: IrcWrite> IrcServer<T, U> {
    /// Creates an IRC server from the specified configuration, and any arbitrary Connection.
    pub fn from_connection(config: Config, conn: Connection<T, U>) -> IrcServer<T, U> {
        IrcServer { conn: conn, config: config, chanlists: Mutex::new(HashMap::new()),
                    alt_nick_index: RwLock::new(0) }
    }

    /// Gets a reference to the IRC server's connection.
    pub fn conn(&self) -> &Connection<T, U> {
        &self.conn
    }

    /// Handles messages internally for basic bot functionality.
    fn handle_message(&self, msg: &Message) {
        if let Some(resp) = Response::from_message(msg) {
            if resp == Response::RPL_NAMREPLY {
                if cfg!(not(feature = "nochanlists")) {
                    if let Some(users) = msg.suffix.clone() {
                        if msg.args.len() == 3 { 
                            // TODO: replace with slice pattern matching when/if stable
                            let ref chan = msg.args[2];
                            for user in users.split(" ") {
                                if match self.chanlists.lock().unwrap().get_mut(chan) {
                                    Some(vec) => { vec.push(User::new(user)); false },
                                    None => true,
                                } {
                                    self.chanlists.lock().unwrap().insert(chan.clone(), 
                                                                          vec!(User::new(user)));
                                }
                            }
                        }
                    }
                }
            } else if resp == Response::RPL_ENDOFMOTD || resp == Response::ERR_NOMOTD {
                if self.config.nick_password() != "" {
                    self.send(NICKSERV(
                        format!("IDENTIFY {}", self.config.nick_password())
                    )).unwrap();
                }
                if self.config.umodes() != "" {
                    self.send(MODE(self.config.nickname().to_owned(),
                                   self.config.umodes().to_owned(), None)).unwrap();
                }
                for chan in self.config.channels().into_iter() {
                    self.send(JOIN(chan.to_owned(), None, None)).unwrap();
                }
            } else if resp == Response::ERR_NICKNAMEINUSE ||
                      resp == Response::ERR_ERRONEOUSNICKNAME {
                let alt_nicks = self.config.get_alternate_nicknames();
                let mut index = self.alt_nick_index.write().unwrap();
                if *index >= alt_nicks.len() {
                    panic!("All specified nicknames were in use.")
                } else {
                    self.send(NICK(alt_nicks[*index].to_owned())).unwrap();
                    *index += 1;
                }
            }
            return
        }
        if &msg.command[..] == "PING" {
            self.send(PONG(msg.suffix.as_ref().unwrap().to_owned(), None)).unwrap();
        } else if cfg!(not(feature = "nochanlists")) && 
                  (&msg.command[..] == "JOIN" || &msg.command[..] == "PART") {
            let chan = match msg.suffix {
                Some(ref suffix) => &suffix[..],
                None => &msg.args[0][..],
            };
            if let Some(vec) = self.chanlists.lock().unwrap().get_mut(&chan.to_string()) {
                if let Some(ref src) = msg.prefix {
                    if let Some(i) = src.find('!') {
                        if &msg.command[..] == "JOIN" {
                            vec.push(User::new(&src[..i]));
                        } else {
                            if let Some(n) = vec.iter().position(|x| x.get_nickname() == &src[..i]) {
                                vec.swap_remove(n);
                            }
                        }
                    }
                }
            }
        } else if let ("MODE", 3) = (&msg.command[..], msg.args.len()) {
            let ref chan = msg.args[0]; // TODO: replace with slice pattern matching when/if stable
            let ref mode = msg.args[1];
            let ref user = msg.args[2];
            if cfg!(not(feature = "nochanlists")) {
                if let Some(vec) = self.chanlists.lock().unwrap().get_mut(chan) {
                    if let Some(n) = vec.iter().position(|x| &x.get_nickname() == user) {
                        vec[n].update_access_level(&mode);
                    }
                }
            }
        } else {
            self.handle_ctcp(msg);
        }
    }

    /// Handles CTCP requests if the CTCP feature is enabled.
    #[cfg(feature = "ctcp")]
    fn handle_ctcp(&self, msg: &Message) {
        let source = match msg.prefix {
            Some(ref source) => source.find('!').map_or(&source[..], |i| &source[..i]),
            None => "",
        };
        if let ("PRIVMSG", 1) = (&msg.command[..], msg.args.len()) {
            // TODO: replace with slice pattern matching when/if stable
            let ref target = msg.args[0];
            let resp = if target.starts_with("#") { &target[..] } else { source };
            match msg.suffix {
                Some(ref msg) if msg.starts_with("\u{001}") => {
                    let tokens: Vec<_> = {
                        let end = if msg.ends_with("\u{001}") {
                            msg.len() - 1
                        } else {
                            msg.len()
                        };
                        msg[1..end].split(" ").collect()
                    };
                    match tokens[0] {
                        "FINGER" => self.send_ctcp_internal(resp, &format!("FINGER :{} ({})",
                                                                  self.config.real_name(),
                                                                  self.config.username())),
                        "VERSION" => self.send_ctcp_internal(resp, "VERSION irc:git:Rust"),
                        "SOURCE" => {
                            self.send_ctcp_internal(resp, "SOURCE https://github.com/aatxe/irc");
                            self.send_ctcp_internal(resp, "SOURCE");
                        },
                        "PING" => self.send_ctcp_internal(resp, &format!("PING {}", tokens[1])),
                        "TIME" => self.send_ctcp_internal(resp, &format!("TIME :{}", 
                                                                now().rfc822z())),
                        "USERINFO" => self.send_ctcp_internal(resp, &format!("USERINFO :{}",
                                                                    self.config.user_info())),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }

    /// Sends a CTCP-escaped message.
    #[cfg(feature = "ctcp")]
    fn send_ctcp_internal(&self, target: &str, msg: &str) {
        self.send(Command::NOTICE(target.to_owned(), format!("\u{001}{}\u{001}", msg))).unwrap();
    }

    /// Handles CTCP requests if the CTCP feature is enabled.
    #[cfg(not(feature = "ctcp"))] fn handle_ctcp(&self, _: &Message) {}
}

/// An Iterator over an IrcServer's incoming Messages.
pub struct ServerIterator<'a, T: IrcRead, U: IrcWrite> {
    server: &'a IrcServer<T, U>
}

/// An Iterator over an IrcServer's incoming Commands.
pub type ServerCmdIterator<'a, T, U> =
    Map<ServerIterator<'a, T, U>, fn(Result<Message>) -> Result<Command>>;

impl<'a, T: IrcRead, U: IrcWrite> ServerIterator<'a, T, U> {
    /// Creates a new ServerIterator for the desired IrcServer.
    pub fn new(server: &IrcServer<T, U>) -> ServerIterator<T, U> {
        ServerIterator { server: server }
    }

    /// Gets the next line from the connection.
    #[cfg(feature = "encode")]
    fn get_next_line(&self) -> Result<String> {
        self.server.conn.recv(self.server.config.encoding())
    }

    /// Gets the next line from the connection.
    #[cfg(not(feature = "encode"))]
    fn get_next_line(&self) -> Result<String> {
        self.server.conn.recv()
    }
}

impl<'a, T: IrcRead, U: IrcWrite> Iterator for ServerIterator<'a, T, U> {
    type Item = Result<Message>;
    fn next(&mut self) -> Option<Result<Message>> {
        let res = self.get_next_line().and_then(|msg|
             match msg.parse() {
                Ok(msg) => {
                    self.server.handle_message(&msg);
                    Ok(msg)
                },
                Err(_) => Err(Error::new(ErrorKind::InvalidInput, 
                    &format!("Failed to parse message. (Message: {})", msg)[..]
                ))
            }
        );
        match res {
            Err(ref err) if err.kind() == ErrorKind::ConnectionAborted => None,
            Err(ref err) if err.kind() == ErrorKind::ConnectionReset   => None,
            Err(ref err) if err.description() == "EOF" => None,
            _ => Some(res)
        }
    }
}

#[cfg(test)]
mod test {
    use super::{IrcServer, Server};
    use std::default::Default;
    use std::io::{Cursor, sink};
    use client::conn::Connection;
    use client::data::{Config, Message, User};
    use client::data::command::Command::PRIVMSG;
    use client::data::kinds::IrcRead;
    use client::test::buf_empty;

    pub fn test_config() -> Config {
        Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            server: Some(format!("irc.test.net")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            user_info: Some(format!("Testing.")),
            .. Default::default()
        }
    }

    pub fn get_server_value<T: IrcRead>(server: IrcServer<T, Vec<u8>>) -> String {
        let vec = server.conn().writer().clone();
        String::from_utf8(vec).unwrap()
    }

    #[test]
    fn iterator() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n\
                   :test!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(exp.as_bytes().to_vec()), sink()
        ));
        let mut messages = String::new();
        for message in server.iter() {
            messages.push_str(&message.unwrap().into_string());
        }
        assert_eq!(&messages[..], exp);
    }

    #[test]
    fn iterator_cmd() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n\
                   JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(exp.as_bytes().to_vec()), sink()
        ));
        let mut messages = String::new();
        for command in server.iter_cmd() {
            let msg: Message = command.unwrap().into();
            messages.push_str(&msg.into_string());
        }
        assert_eq!(&messages[..], exp);
    }

    #[test]
    fn handle_message() {
        let value = "PING :irc.test.net\r\n:irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "PONG :irc.test.net\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn handle_end_motd_with_nick_password() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(Config {
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }, Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "NICKSERV IDENTIFY password\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn handle_end_motd_with_umodes() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            umodes: Some(format!("+B")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }, Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "MODE test +B\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn nickname_in_use() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NICK :test2\r\n");
    }

    #[test]
    #[should_panic(message = "All specified nicknames were in use.")]
    fn ran_out_of_nicknames() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.pdgn.co 433 * test2 :Nickname is already in use.\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
    }

    #[test]
    fn send() {
        let server = IrcServer::from_connection(test_config(), Connection::new(
           buf_empty(), Vec::new()
        ));
        assert!(server.send(PRIVMSG(format!("#test"), format!("Hi there!"))).is_ok());
        assert_eq!(&get_server_value(server)[..], "PRIVMSG #test :Hi there!\r\n");
    }

    #[cfg(not(feature = "nochanlists"))]
    #[test]
    fn user_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
           Cursor::new(value.as_bytes().to_vec()), sink()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("~owner"), User::new("&admin")])
    }

    #[cfg(not(feature = "nochanlists"))]
    #[test]
    fn user_tracking_names_join() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :test2!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), sink()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("~owner"), User::new("&admin"), User::new("test2")])
    }

    #[cfg(not(feature = "nochanlists"))]
    #[test]
    fn user_tracking_names_part() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :owner!test@test PART #test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), sink()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(server.list_users("#test").unwrap(),
        vec![User::new("test"), User::new("&admin")])
    }

    #[cfg(not(feature = "nochanlists"))]
    #[test]
    fn user_tracking_names_mode() {
        let value = ":irc.test.net 353 test = #test :+test ~owner &admin\r\n\
                     :test!test@test MODE #test +o test\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), sink()
        ));
        for message in server.iter() {
            println!("{:?}", message);
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

    #[test]
    #[cfg(feature = "ctcp")]
    fn finger_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}FINGER\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}FINGER :test (test)\u{001}\
                   \r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn version_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}VERSION\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}VERSION irc:git:Rust\u{001}\
                   \r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn source_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}SOURCE\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "NOTICE test :\u{001}SOURCE https://github.com/aatxe/irc\u{001}\r\n\
         NOTICE test :\u{001}SOURCE\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn ctcp_ping_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}PING test\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}PING test\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn time_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}TIME\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        let val = get_server_value(server);
        assert!(val.starts_with("NOTICE test :\u{001}TIME :"));
        assert!(val.ends_with("\u{001}\r\n"));
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn user_info_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}USERINFO\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}USERINFO :Testing.\u{001}\
                   \r\n");
    }
}
