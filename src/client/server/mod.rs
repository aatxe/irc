//! Interface for working with IRC Servers.
//!
//! There are currently two recommended ways to work
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};
use std::thread::{JoinHandle, spawn};
use client::conn::{Connection, NetConnection};
use client::data::{Command, Config, Message, Response, User};
use client::data::Command::{JOIN, NICK, NICKSERV, PART, PING, PRIVMSG, MODE};
use client::server::utils::ServerExt;
use time::{Duration, Timespec, Tm, now};

pub mod utils;

/// An interface for interacting with an IRC server.
pub trait Server {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;

    /// Sends a Command to this Server.
    fn send<M: Into<Message>>(&self, message: M) -> Result<()> where Self: Sized;

    /// Gets an iterator over received messages.
    fn iter<'a>(&'a self) -> Box<Iterator<Item = Result<Message>> + 'a>;

    /// Gets a list of Users in the specified channel. This will be none if the channel is not
    /// being tracked, or if tracking is not supported altogether. For best results, be sure to
    /// request `multi-prefix` support from the server.
    fn list_users(&self, channel: &str) -> Option<Vec<User>>;
}

/// A thread-safe implementation of an IRC Server connection.
pub struct IrcServer {
    /// The channel for sending messages to write.
    tx: Sender<Message>,
    /// The internal, thread-safe server state.
    state: Arc<ServerState>,
    /// A thread-local count of reconnection attempts used for synchronization.
    reconnect_count: Cell<u32>,
}

/// Thread-safe internal state for an IRC server connection.
struct ServerState {
    /// A global copy of the channel for sending messages to write.
    tx: Mutex<Option<Sender<Message>>>,
    /// The thread-safe IRC connection.
    conn: Box<Connection + Send + Sync>,
    /// The handle for the message sending thread.
    write_handle: Mutex<Option<JoinHandle<()>>>,
    /// The configuration used with this connection.
    config: Config,
    /// A thread-safe map of channels to the list of users in them.
    chanlists: Mutex<HashMap<String, Vec<User>>>,
    /// A thread-safe index to track the current alternative nickname being used.
    alt_nick_index: RwLock<usize>,
    /// A thread-safe count of reconnection attempts used for synchronization.
    reconnect_count: Mutex<u32>,
    /// A thread-safe store for the time of the last action.
    last_action_time: Mutex<Tm>,
    /// A thread-safe store for the last ping data.
    last_ping_data: Mutex<Option<Timespec>>,
}

impl ServerState {
    fn new<C>(conn: C, config: Config) -> ServerState where C: Connection + Send + Sync + 'static {
        ServerState {
            tx: Mutex::new(None),
            conn: Box::new(conn),
            write_handle: Mutex::new(None),
            config: config,
            chanlists: Mutex::new(HashMap::new()),
            alt_nick_index: RwLock::new(0),
            reconnect_count: Mutex::new(0),
            last_action_time: Mutex::new(now()),
            last_ping_data: Mutex::new(None),
        }
    }

    fn reconnect(&self) -> Result<()> {
        self.conn.reconnect()
    }

    fn action_taken(&self) {
        let mut time = self.last_action_time.lock().unwrap();
        *time = now();
    }

    fn should_ping(&self) -> bool {
        let time = self.last_action_time.lock().unwrap();
        (now() - *time) > Duration::seconds(self.config.ping_time() as i64)
    }

    fn update_ping_data(&self, data: Timespec) {
        let mut ping_data = self.last_ping_data.lock().unwrap();
        *ping_data = Some(data);
    }

    fn ping_timeout_duration(&self) -> Duration {
        Duration::seconds(self.config.ping_timeout() as i64)
    }

    fn last_ping_data(&self) -> Option<Timespec> {
        self.last_ping_data.lock().unwrap().clone()
    }
}

impl IrcServer {
    /// Creates a new IRC Server connection from the configuration at the specified path,
    /// connecting immediately.
    pub fn new<P: AsRef<Path>>(config: P) -> Result<IrcServer> {
        IrcServer::from_config(try!(Config::load(config)))
    }

    /// Creates a new IRC server connection from the specified configuration, connecting
    /// immediately.
    pub fn from_config(config: Config) -> Result<IrcServer> {
        let conn = try!(if config.use_ssl() {
            NetConnection::connect_ssl(config.server(), config.port())
        } else {
            NetConnection::connect(config.server(), config.port())
        });
        Ok(IrcServer::from_connection(config, conn))
    }
}

impl Clone for IrcServer {
    fn clone(&self) -> IrcServer {
        IrcServer {
            tx: self.tx.clone(),
            state: self.state.clone(),
            reconnect_count: self.reconnect_count.clone()
        }
    }
}

impl Drop for ServerState {
    fn drop(&mut self) {
        let _ = self.tx.lock().unwrap().take();
        let mut guard = self.write_handle.lock().unwrap();
        if let Some(handle) = guard.take() {
            handle.join().unwrap()
        }
    }
}

impl<'a> Server for ServerState {
    fn config(&self) -> &Config {
        &self.config
    }

    fn send<M: Into<Message>>(&self, msg: M) -> Result<()> where Self: Sized {
        let opt_tx = self.tx.lock().unwrap();
        let ref rf_tx = *opt_tx;
        match rf_tx {
            &Some(ref tx) => tx.send(msg.into()).map_err(|e| Error::new(ErrorKind::Other, e)),
            &None => Err(Error::new(ErrorKind::NotFound, "Channel was not found."))
        }
    }

    fn iter(&self) -> Box<Iterator<Item = Result<Message>>> {
        panic!("unimplemented")
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists.lock().unwrap().get(&chan.to_owned()).cloned()
    }


    #[cfg(feature = "nochanlists")]
    fn list_users(&self, _: &str) -> Option<Vec<User>> {
        None
    }
}

impl Server for IrcServer {
    fn config(&self) -> &Config {
        &self.state.config
    }

    fn send<M: Into<Message>>(&self, msg: M) -> Result<()> where Self: Sized {
        self.tx.send(msg.into()).map_err(|e| Error::new(ErrorKind::Other, e))
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item = Result<Message>> + 'a> {
        Box::new(ServerIterator::new(self))
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.state.chanlists.lock().unwrap().get(&chan.to_owned()).cloned()
    }


    #[cfg(feature = "nochanlists")]
    fn list_users(&self, _: &str) -> Option<Vec<User>> {
        None
    }
}

impl IrcServer {
    /// Creates an IRC server from the specified configuration, and any arbitrary sync connection.
    pub fn from_connection<C>(config: Config, conn: C) -> IrcServer
    where C: Connection + Send + Sync + 'static {
        let (tx, rx): (Sender<Message>, Receiver<Message>) = channel();
        let state = Arc::new(ServerState::new(conn, config));
        let weak = Arc::downgrade(&state);
        let write_handle = spawn(move || loop {
            if let Some(strong) = weak.upgrade() {
                if let Some(time) = strong.last_ping_data() {
                    if now().to_timespec() - time > strong.ping_timeout_duration() {
                        let _ = strong.reconnect();
                        while let Err(_) = strong.identify() {
                            let _ = strong.reconnect();
                        }
                    }
                }
            }
            match rx.try_recv() {
                Ok(msg) => if let Some(strong) = weak.upgrade() {
                    while let Err(_) = IrcServer::write(&strong, msg.clone()) {
                        let _ = strong.reconnect().and_then(|_| strong.identify());
                    }
                    strong.action_taken();
                },
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => if let Some(strong) = weak.upgrade() {
                    if strong.should_ping() {
                        let data = now().to_timespec();
                        strong.update_ping_data(data);
                        let fmt = format!("{}", data.sec);
                        while let Err(_) = IrcServer::write(&strong, PING(fmt.clone(), None)) {
                            let _ = strong.reconnect();
                        }
                    }
                },
            }
        });
        let state2 = state.clone();
        let mut handle = state2.write_handle.lock().unwrap();
        *handle = Some(write_handle);
        let mut state_tx = state2.tx.lock().unwrap();
        *state_tx = Some(tx.clone());
        IrcServer { tx: tx, state: state, reconnect_count: Cell::new(0) }
    }

    /// Gets a reference to the IRC server's connection.
    pub fn conn(&self) -> &Box<Connection + Send + Sync> {
        &self.state.conn
    }

    /// Reconnects to the IRC server, disconnecting if necessary.
    pub fn reconnect(&self) -> Result<()> {
        let mut reconnect_count = self.state.reconnect_count.lock().unwrap();
        let res = if self.reconnect_count.get() == *reconnect_count {
            *reconnect_count += 1;
            self.state.reconnect()
        } else {
            Ok(())
        };
        self.reconnect_count.set(*reconnect_count);
        res
    }

    #[cfg(feature = "encode")]
    fn write<M: Into<Message>>(state: &Arc<ServerState>, msg: M) -> Result<()> {
        state.conn.send(&msg.into().into_string(), state.config.encoding())
    }

    #[cfg(not(feature = "encode"))]
    fn write<M: Into<Message>>(state: &Arc<ServerState>, msg: M) -> Result<()> {
        state.conn.send(&msg.into().into_string())
    }

    /// Returns a reference to the server state's channel lists.
    fn chanlists(&self) -> &Mutex<HashMap<String, Vec<User>>> {
        &self.state.chanlists
    }

    /// Handles messages internally for basic client functionality.
    fn handle_message(&self, msg: &Message) -> Result<()> {
        match msg.command {
            PING(ref data, _) => try!(self.send_pong(&data)),
            JOIN(ref chan, _, _) => if cfg!(not(feature = "nochanlists")) {
                if let Some(vec) = self.chanlists().lock().unwrap().get_mut(&chan.to_owned()) {
                    if let Some(src) = msg.source_nickname() {
                        vec.push(User::new(src))
                    }
                }
            },
            PART(ref chan, _) => if cfg!(not(feature = "nochanlists")) {
                if let Some(vec) = self.chanlists().lock().unwrap().get_mut(&chan.to_owned()) {
                    if let Some(src) = msg.source_nickname() {
                        if let Some(n) = vec.iter().position(|x| x.get_nickname() == src) {
                            vec.swap_remove(n);
                        }
                    }
                }
            },
            MODE(ref chan, ref mode, Some(ref user)) => if cfg!(not(feature = "nochanlists")) {
                if let Some(vec) = self.chanlists().lock().unwrap().get_mut(chan) {
                    if let Some(n) = vec.iter().position(|x| x.get_nickname() == user) {
                        vec[n].update_access_level(&mode)
                    }
                }
            },
            PRIVMSG(ref target, ref body) => if body.starts_with("\u{001}") {
                let tokens: Vec<_> = {
                    let end = if body.ends_with("\u{001}") {
                        body.len() - 1
                    } else {
                        body.len()
                    };
                    body[1..end].split(" ").collect()
                };
                if target.starts_with("#") {
                    try!(self.handle_ctcp(&target, tokens))
                } else if let Some(user) = msg.source_nickname() {
                    try!(self.handle_ctcp(user, tokens))
                }
            },
            Command::Response(Response::RPL_NAMREPLY, ref args, ref suffix) => {
                if cfg!(not(feature = "nochanlists")) {
                    if let Some(users) = suffix.clone() {
                        if args.len() == 3 {
                            let ref chan = args[2];
                            for user in users.split(" ") {
                                let mut chanlists = self.state.chanlists.lock().unwrap();
                                chanlists.entry(chan.clone()).or_insert(Vec::new())
                                         .push(User::new(user))
                            }
                        }
                    }
                }
            },
            Command::Response(Response::RPL_ENDOFMOTD, _, _) |
            Command::Response(Response::ERR_NOMOTD, _, _) => {
                if self.config().nick_password() != "" {
                    let mut index = self.state.alt_nick_index.write().unwrap();
                    if self.config().should_ghost() && *index != 0 {
                        for seq in self.config().ghost_sequence().iter() {
                            try!(self.send(NICKSERV(
                                format!("{} {} {}", seq, self.config().nickname(), self.config().nick_password())
                            )));
                        }
                        *index = 0;
                        try!(self.send(NICK(self.config().nickname().to_owned())))
                    }
                    try!(self.send(NICKSERV(
                        format!("IDENTIFY {}", self.config().nick_password())
                    )))
                }
                if self.config().umodes() != "" {
                    try!(self.send_mode(self.config().nickname(), self.config().umodes(), ""))
                }
                for chan in self.config().channels().into_iter() {
                    try!(self.send_join(chan))
                }
            },
            Command::Response(Response::ERR_NICKNAMEINUSE, _, _) |
            Command::Response(Response::ERR_ERRONEOUSNICKNAME, _, _) => {
                let alt_nicks = self.config().alternate_nicknames();
                let mut index = self.state.alt_nick_index.write().unwrap();
                if *index >= alt_nicks.len() {
                    panic!("All specified nicknames were in use or disallowed.")
                } else {
                    try!(self.send(NICK(alt_nicks[*index].to_owned())));
                    *index += 1;
                }
            },
            _ => ()
        }
        Ok(())
    }

    /// Handles CTCP requests if the CTCP feature is enabled.
    #[cfg(feature = "ctcp")]
    fn handle_ctcp(&self, resp: &str, tokens: Vec<&str>) -> Result<()> {
        match tokens[0] {
            "FINGER" => self.send_ctcp_internal(resp, &format!(
                "FINGER :{} ({})", self.config().real_name(), self.config().username()
            )),
            "VERSION" => self.send_ctcp_internal(resp, "VERSION irc:git:Rust"),
            "SOURCE" => {
                try!(self.send_ctcp_internal(resp, "SOURCE https://github.com/aatxe/irc"));
                self.send_ctcp_internal(resp, "SOURCE")
            },
            "PING" => self.send_ctcp_internal(resp, &format!("PING {}", tokens[1])),
            "TIME" => self.send_ctcp_internal(resp, &format!(
                "TIME :{}", now().rfc822z()
            )),
            "USERINFO" => self.send_ctcp_internal(resp, &format!(
                "USERINFO :{}", self.config().user_info()
            )),
            _ => Ok(())
        }
    }

    /// Sends a CTCP-escaped message.
    #[cfg(feature = "ctcp")]
    fn send_ctcp_internal(&self, target: &str, msg: &str) -> Result<()> {
        self.send_notice(target, &format!("\u{001}{}\u{001}", msg))
    }

    /// Handles CTCP requests if the CTCP feature is enabled.
    #[cfg(not(feature = "ctcp"))]
    fn handle_ctcp(&self, _: &str, _: Vec<&str>) -> Result<()> {
        Ok(())
    }
}

/// An Iterator over an IrcServer's incoming Messages.
pub struct ServerIterator<'a> {
    server: &'a IrcServer
}

impl<'a> ServerIterator<'a> {
    /// Creates a new ServerIterator for the desired IrcServer.
    pub fn new(server: &'a IrcServer) -> ServerIterator {
        ServerIterator { server: server }
    }

    /// Gets the next line from the connection.
    #[cfg(feature = "encode")]
    fn get_next_line(&self) -> Result<String> {
        self.server.conn().recv(self.server.config().encoding())
    }

    /// Gets the next line from the connection.
    #[cfg(not(feature = "encode"))]
    fn get_next_line(&self) -> Result<String> {
        self.server.conn().recv()
    }
}

impl<'a> Iterator for ServerIterator<'a> {
    type Item = Result<Message>;
    fn next(&mut self) -> Option<Result<Message>> {
        loop {
            match self.get_next_line() {
                Ok(msg) => match msg.parse() {
                    Ok(res) => {
                        match self.server.handle_message(&res) {
                            Ok(()) => (),
                            Err(err) => return Some(Err(err))
                        }
                        self.server.state.action_taken();
                        return Some(Ok(res))
                    },
                    Err(_) => return Some(Err(Error::new(ErrorKind::InvalidInput,
                        &format!("Failed to parse message. (Message: {})", msg)[..]
                    )))
                },
                Err(ref err) if err.description() == "EOF" => return None,
                Err(_) => {
                    let _ = self.server.reconnect().and_then(|_| self.server.identify());
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{IrcServer, Server};
    use std::default::Default;
    use std::io::{Cursor, sink};
    use client::conn::{Connection, Reconnect};
    use client::data::Config;
    #[cfg(not(feature = "nochanlists"))] use client::data::User;
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

    pub fn get_server_value<T: IrcRead>(server: IrcServer<T, Vec<u8>>) -> String
        where Connection<T, Vec<u8>>: Reconnect {
        String::from_utf8(server.extract_writer()).unwrap()
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
    fn handle_end_motd_with_ghost() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.test.net 376 test2 :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            should_ghost: Some(true),
            .. Default::default()
        }, Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "NICK :test2\r\nNICKSERV GHOST test password\r\nNICK :test\r\nNICKSERV IDENTIFY password\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn handle_end_motd_with_ghost_seq() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.test.net 376 test2 :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            should_ghost: Some(true),
            ghost_sequence: Some(vec![format!("RECOVER"), format!("RELEASE")]),
            .. Default::default()
        }, Connection::new(
           Cursor::new(value.as_bytes().to_vec()), Vec::new()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "NICK :test2\r\nNICKSERV RECOVER test password\r\nNICKSERV RELEASE test password\r\nNICK :test\r\nNICKSERV IDENTIFY password\r\nJOIN #test\r\nJOIN #test2\r\n");
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

    #[test]
    #[cfg(not(feature = "nochanlists"))]
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

    #[test]
    #[cfg(not(feature = "nochanlists"))]
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

    #[test]
    #[cfg(not(feature = "nochanlists"))]
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

    #[test]
    #[cfg(not(feature = "nochanlists"))]
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
    #[cfg(feature = "nochanlists")]
    fn no_user_tracking() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin";
        let server = IrcServer::from_connection(test_config(), Connection::new(
            Cursor::new(value.as_bytes().to_vec()), sink()
        ));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert!(server.list_users("#test").is_none())
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
