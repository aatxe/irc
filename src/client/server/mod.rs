//! Interface for working with IRC Servers.
use std::borrow::ToOwned;
use std::cell::Cell;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{spawn, sleep};
use std::time::Duration as StdDuration;
use client::conn::{Connection, NetConnection};
use client::data::{Command, Config, Message, Response, User};
use client::data::Command::{JOIN, NICK, NICKSERV, PART, PING, PONG, PRIVMSG, MODE};
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

    /// Gets a list of currently joined channels. This will be none if tracking is not supported altogether.
    fn list_channels(&self) -> Option<Vec<String>>;

    /// Gets a list of Users in the specified channel. This will be none if the channel is not
    /// being tracked, or if tracking is not supported altogether. For best results, be sure to
    /// request `multi-prefix` support from the server.
    fn list_users(&self, channel: &str) -> Option<Vec<User>>;
}

/// A thread-safe implementation of an IRC Server connection.
pub struct IrcServer {
    /// The internal, thread-safe server state.
    state: Arc<ServerState>,
    /// A thread-local count of reconnection attempts used for synchronization.
    reconnect_count: Cell<u32>,
}

/// Thread-safe internal state for an IRC server connection.
struct ServerState {
    /// The thread-safe IRC connection.
    conn: Box<Connection + Send + Sync>,
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
    /// A thread-safe check of pong reply.
    waiting_pong_reply: Mutex<bool>,
}

impl ServerState {
    fn new<C>(conn: C, config: Config) -> ServerState where C: Connection + Send + Sync + 'static {
        ServerState {
            conn: Box::new(conn),
            config: config,
            chanlists: Mutex::new(HashMap::new()),
            alt_nick_index: RwLock::new(0),
            reconnect_count: Mutex::new(0),
            last_action_time: Mutex::new(now()),
            last_ping_data: Mutex::new(None),
            waiting_pong_reply: Mutex::new(false),
        }
    }

    fn reconnect(&self) -> Result<()> {
        let mut ping_data = self.last_ping_data.lock().unwrap();
        *ping_data = None;
        let mut waiting_pong_reply = self.waiting_pong_reply.lock().unwrap();
        *waiting_pong_reply = false;
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

    fn last_action_time(&self) -> Tm {
        *self.last_action_time.lock().unwrap()
    }

    fn update_ping_data(&self, data: Timespec) {
        let mut ping_data = self.last_ping_data.lock().unwrap();
        *ping_data = Some(data);
        let mut waiting_pong_reply = self.waiting_pong_reply.lock().unwrap();
        *waiting_pong_reply = true;
    }

    fn last_ping_data(&self) -> Option<Timespec> {
        *self.last_ping_data.lock().unwrap()
    }

    fn waiting_pong_reply(&self) -> bool {
        *self.waiting_pong_reply.lock().unwrap()
    }

    fn check_pong(&self, data: &str) {
        if let Some(ref time) = self.last_ping_data() {
            let fmt = format!("{}", time.sec);
            if fmt == data {
                // found matching pong reply
                let mut waiting_reply = self.waiting_pong_reply.lock().unwrap();
                *waiting_reply = false;
            }
        }
    }

    fn send_impl(&self, msg: Message) {
        while let Err(_) = self.write(msg.clone()) {
            let _ = self.reconnect().and_then(|_| self.identify());
        }
        self.action_taken();
    }

    /// Sanitizes the input string by cutting up to (and including) the first occurence of a line
    /// terminiating phrase (`\r\n`, `\r`, or `\n`).
    fn sanitize(data: &str) -> &str {
        // n.b. ordering matters here to prefer "\r\n" over "\r"
        if let Some((pos, len)) = ["\r\n", "\r", "\n"].iter().flat_map(|needle| {
            data.find(needle).map(|pos| (pos, needle.len()))
        }).min_by_key(|&(pos, _)| pos) {
            data.split_at(pos + len).0
        } else {
            data
        }
    }

    #[cfg(feature = "encode")]
    fn write<M: Into<Message>>(&self, msg: M) -> Result<()> {
        self.conn.send(ServerState::sanitize(&msg.into().to_string()), self.config.encoding())
    }

    #[cfg(not(feature = "encode"))]
    fn write<M: Into<Message>>(&self, msg: M) -> Result<()> {
        self.conn.send(ServerState::sanitize(&msg.into().to_string()))
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
            state: self.state.clone(),
            reconnect_count: self.reconnect_count.clone()
        }
    }
}

impl<'a> Server for ServerState {
    fn config(&self) -> &Config {
        &self.config
    }

    fn send<M: Into<Message>>(&self, msg: M) -> Result<()> where Self: Sized {
        self.send_impl(msg.into());
        Ok(())
    }

    fn iter(&self) -> Box<Iterator<Item = Result<Message>>> {
        panic!("unimplemented")
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_channels(&self) -> Option<Vec<String>> {
        Some(self.chanlists.lock().unwrap().keys().map(|k| k.to_owned()).collect())
    }

    #[cfg(feature = "nochanlists")]
    fn list_channels(&self) -> Option<Vec<String>> {
        None
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
        let msg = msg.into();
        try!(self.handle_sent_message(&msg));
        self.state.send(msg)
    }

    fn iter<'a>(&'a self) -> Box<Iterator<Item = Result<Message>> + 'a> {
        Box::new(ServerIterator::new(self))
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_channels(&self) -> Option<Vec<String>> {
        Some(self.state.chanlists.lock().unwrap().keys().map(|k| k.to_owned()).collect())
    }

    #[cfg(feature = "nochanlists")]
    fn list_channels(&self) -> Option<Vec<String>> {
        None
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
        let state = Arc::new(ServerState::new(conn, config));
        let ping_time = (state.config.ping_time() as i64) * 1000;
        let ping_timeout = (state.config.ping_timeout() as i64) * 1000;
        let weak = Arc::downgrade(&state);

        spawn(move || while let Some(strong) = weak.upgrade() {
            let ping_idle_timespec = {
                let last_action = strong.last_action_time().to_timespec();
                if let Some(last_ping) = strong.last_ping_data() {
                    if last_action < last_ping {
                        last_ping
                    } else {
                        last_action
                    }
                } else {
                    last_action
                }
            };
            let now_timespec = now().to_timespec();
            let sleep_dur_ping_time = ping_time - (now_timespec - ping_idle_timespec).num_milliseconds();
            let sleep_dur_ping_timeout = if let Some(time) = strong.last_ping_data() {
                ping_timeout - (now_timespec - time).num_milliseconds()
            } else {
                ping_timeout
            };

            if strong.waiting_pong_reply() && sleep_dur_ping_timeout < sleep_dur_ping_time {
                // timeout check is earlier
                let sleep_dur = sleep_dur_ping_timeout;
                if sleep_dur > 0 {
                    sleep(StdDuration::from_millis(sleep_dur as u64))
                }
                if strong.waiting_pong_reply() {
                    let _ = strong.reconnect();
                    while let Err(_) = strong.identify() {
                        let _ = strong.reconnect();
                    }
                }
            } else {
                let sleep_dur = sleep_dur_ping_time;
                if sleep_dur > 0 {
                    sleep(StdDuration::from_millis(sleep_dur as u64))
                }
                let now_timespec = now().to_timespec();
                if strong.should_ping() {
                    let data = now_timespec;
                    strong.update_ping_data(data);
                    let fmt = format!("{}", data.sec);
                    while let Err(_) = strong.write(PING(fmt.clone(), None)) {
                        let _ = strong.reconnect();
                    }
                }
            }
        });
        IrcServer { state: state, reconnect_count: Cell::new(0) }
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

    /// Gets the current nickname in use.
    pub fn current_nickname(&self) -> &str {
        let alt_nicks = self.config().alternate_nicknames();
        let index = self.state.alt_nick_index.read().unwrap();
        match *index {
            0 => self.config().nickname(),
            i => alt_nicks[i - 1]
        }
    }

    /// Returns a reference to the server state's channel lists.
    fn chanlists(&self) -> &Mutex<HashMap<String, Vec<User>>> {
        &self.state.chanlists
    }

    /// Handles sent messages internally for basic client functionality.
    fn handle_sent_message(&self, msg: &Message) -> Result<()> {
        match msg.command {
            PART(ref chan, _) => {
                let _ = self.state.chanlists.lock().unwrap().remove(chan);
            },
            _ => ()
        }
        Ok(())
    }

    /// Handles received messages internally for basic client functionality.
    fn handle_message(&self, msg: &Message) -> Result<()> {
        match msg.command {
            PING(ref data, _) => try!(self.send_pong(&data)),
            PONG(_, Some(ref pingdata)) => self.state.check_pong(&pingdata),
            PONG(ref pingdata, None) => self.state.check_pong(&pingdata),
            JOIN(ref chan, _, _) => self.handle_join(msg.source_nickname().unwrap_or(""), chan),
            PART(ref chan, _) => self.handle_part(msg.source_nickname().unwrap_or(""), chan),
            NICK(ref new_nick) => self.handle_nick_change(msg.source_nickname().unwrap_or(""), new_nick),
            MODE(ref chan, ref mode, Some(ref user)) => self.handle_mode(chan, mode, user),
            PRIVMSG(ref target, ref body) => if body.starts_with('\u{001}') {
                let tokens: Vec<_> = {
                    let end = if body.ends_with('\u{001}') {
                        body.len() - 1
                    } else {
                        body.len()
                    };
                    body[1..end].split(' ').collect()
                };
                if target.starts_with('#') {
                    try!(self.handle_ctcp(&target, tokens))
                } else if let Some(user) = msg.source_nickname() {
                    try!(self.handle_ctcp(user, tokens))
                }
            },
            Command::Response(Response::RPL_NAMREPLY, ref args, ref suffix) => {
                self.handle_namreply(args, suffix)
            },
            Command::Response(Response::RPL_ENDOFMOTD, _, _) |
            Command::Response(Response::ERR_NOMOTD, _, _) => {
                try!(self.send_nick_password());
                try!(self.send_umodes());

                let config_chans = self.config().channels();
                for chan in config_chans.iter() {
                    match self.config().channel_key(chan) {
                        Some(key) => try!(self.send_join_with_keys(chan, key)),
                        None => try!(self.send_join(chan))
                    }
                }
                let joined_chans = self.state.chanlists.lock().unwrap();
                for chan in joined_chans.keys().filter(|x| !config_chans.contains(&x.as_str())) {
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

    fn send_nick_password(&self) -> Result<()> {
        if self.config().nick_password().is_empty() {
            Ok(())
        } else {
            let mut index = self.state.alt_nick_index.write().unwrap();
            if self.config().should_ghost() && *index != 0 {
                for seq in &self.config().ghost_sequence() {
                    try!(self.send(NICKSERV(format!(
                        "{} {} {}", seq, self.config().nickname(),
                        self.config().nick_password()
                    ))));
                }
                *index = 0;
                try!(self.send(NICK(self.config().nickname().to_owned())))
            }
            self.send(NICKSERV(format!("IDENTIFY {}", self.config().nick_password())))
        }
    }

    fn send_umodes(&self) -> Result<()> {
        if self.config().umodes().is_empty() {
            Ok(())
        } else {
            self.send_mode(self.current_nickname(), self.config().umodes(), "")
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_join(&self, _: &str, _: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_join(&self, src: &str, chan: &str) {
        if let Some(vec) = self.chanlists().lock().unwrap().get_mut(&chan.to_owned()) {
            if !src.is_empty() {
                vec.push(User::new(src))
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_part(&self, src: &str, chan: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_part(&self, src: &str, chan: &str) {
        if let Some(vec) = self.chanlists().lock().unwrap().get_mut(&chan.to_owned()) {
            if !src.is_empty() {
                if let Some(n) = vec.iter().position(|x| x.get_nickname() == src) {
                    vec.swap_remove(n);
                }
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_nick_change(&self, _: &str, _: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_nick_change(&self, old_nick: &str, new_nick: &str) {
        if old_nick.is_empty() || new_nick.is_empty() { return; }
        let mut chanlists = self.chanlists().lock().unwrap();
        for channel in chanlists.clone().keys() {
            if let Some(vec) = chanlists.get_mut(&channel.to_owned()) {
                for p in vec.iter().position(|x| x.get_nickname() == old_nick) {
                    let new_entry = User::new(new_nick);
                    vec[p] = new_entry;
                }
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_mode(&self, chan: &str, mode: &str, user: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_mode(&self, chan: &str, mode: &str, user: &str) {
        if let Some(vec) = self.chanlists().lock().unwrap().get_mut(chan) {
            if let Some(n) = vec.iter().position(|x| x.get_nickname() == user) {
                vec[n].update_access_level(&mode)
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_namreply(&self, _: &[String], _: &Option<String>) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_namreply(&self, args: &[String], suffix: &Option<String>) {
        if let Some(ref users) = *suffix {
            if args.len() == 3 {
                let chan = &args[2];
                for user in users.split(' ') {
                    let mut chanlists = self.state.chanlists.lock().unwrap();
                    chanlists.entry(chan.clone()).or_insert_with(Vec::new)
                             .push(User::new(user))
                }
            }
        }
    }

    /// Handles CTCP requests if the CTCP feature is enabled.
    #[cfg(feature = "ctcp")]
    fn handle_ctcp(&self, resp: &str, tokens: Vec<&str>) -> Result<()> {
        if tokens.len() == 0 { return Ok(()) }
        match tokens[0] {
            "FINGER" => self.send_ctcp_internal(resp, &format!(
                "FINGER :{} ({})", self.config().real_name(), self.config().username()
            )),
            "VERSION" => {
                self.send_ctcp_internal(resp, &format!("VERSION {}", self.config().version()))
            },
            "SOURCE" => {
                try!(self.send_ctcp_internal(resp, &format!("SOURCE {}", self.config().source())));
                self.send_ctcp_internal(resp, "SOURCE")
            },
            "PING" if tokens.len() > 1 => {
                self.send_ctcp_internal(resp, &format!("PING {}", tokens[1]))
            },
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
    use std::collections::HashMap;
    use std::default::Default;
    use client::conn::MockConnection;
    use client::data::Config;
    #[cfg(not(feature = "nochanlists"))] use client::data::User;
    use client::data::command::Command::{PART, PRIVMSG};

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

    #[cfg(feature = "encode")]
    pub fn get_server_value(server: IrcServer) -> String {
        server.conn().written(server.config().encoding()).unwrap()
    }

    #[cfg(not(feature = "encode"))]
    pub fn get_server_value(server: IrcServer) -> String {
        server.conn().written().unwrap()
    }

    #[test]
    fn iterator() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n\
                   :test!test@test JOIN #test\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(exp));
        let mut messages = String::new();
        for message in server.iter() {
            messages.push_str(&message.unwrap().to_string());
        }
        assert_eq!(&messages[..], exp);
    }

    #[test]
    fn handle_message() {
        let value = "PING :irc.test.net\r\n:irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        }, MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NICKSERV IDENTIFY password\r\nJOIN #test\r\n\
                   JOIN #test2\r\n");
    }

    #[test]
    fn handle_end_motd_with_chan_keys() {
        let value = ":irc.test.net 376 test :End of /MOTD command\r\n";
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            channel_keys: {
                let mut map = HashMap::new();
                map.insert(format!("#test2"), format!("password"));
                Some(map)
            },
            .. Default::default()
        }, MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "JOIN #test\r\nJOIN #test2 password\r\n");
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
        }, MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NICK :test2\r\nNICKSERV GHOST test password\r\n\
                   NICK :test\r\nNICKSERV IDENTIFY password\r\nJOIN #test\r\nJOIN #test2\r\n");
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
        }, MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NICK :test2\r\nNICKSERV RECOVER test password\
                   \r\nNICKSERV RELEASE test password\r\nNICK :test\r\nNICKSERV IDENTIFY password\
                   \r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn handle_end_motd_with_umodes() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_connection(Config {
            nickname: Some(format!("test")),
            umodes: Some(format!("+B")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            .. Default::default()
        }, MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..],
        "MODE test +B\r\nJOIN #test\r\nJOIN #test2\r\n");
    }

    #[test]
    fn nickname_in_use() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NICK :test2\r\n");
    }

    #[test]
    #[should_panic(expected = "All specified nicknames were in use or disallowed.")]
    fn ran_out_of_nicknames() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.pdgn.co 433 * test2 :Nickname is already in use.\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
    }

    #[test]
    fn send() {
        let server = IrcServer::from_connection(test_config(), MockConnection::empty());
        assert!(server.send(PRIVMSG(format!("#test"), format!("Hi there!"))).is_ok());
        assert_eq!(&get_server_value(server)[..], "PRIVMSG #test :Hi there!\r\n");
    }

    #[test]
    fn send_no_newline_injection() {
        let server = IrcServer::from_connection(test_config(), MockConnection::empty());
        assert!(server.send(PRIVMSG(format!("#test"), format!("Hi there!\nJOIN #bad"))).is_ok());
        assert_eq!(&get_server_value(server)[..], "PRIVMSG #test :Hi there!\n");
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn channel_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(server.list_channels().unwrap(), vec!["#test".to_owned()])
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn channel_tracking_names_part() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert!(server.send(PART(format!("#test"), None)).is_ok());
        assert!(server.list_channels().unwrap().is_empty())
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn user_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert!(server.list_users("#test").is_none())
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn finger_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}FINGER\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}PING test\u{001}\r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn time_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}TIME\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
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
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "NOTICE test :\u{001}USERINFO :Testing.\u{001}\
                   \r\n");
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn ctcp_ping_no_timestamp() {
        let value = ":test!test@test PRIVMSG test :\u{001}PING\u{001}\r\n";
        let server = IrcServer::from_connection(test_config(), MockConnection::new(value));
        for message in server.iter() {
            println!("{:?}", message);
        }
        assert_eq!(&get_server_value(server)[..], "");
    }
}
