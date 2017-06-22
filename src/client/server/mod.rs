//! Interface for working with IRC Servers.
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use error;
use client::conn::Connection;
use client::data::{Config, User};
use client::server::utils::ServerExt;
use client::transport::LogView;
use proto::{Command, Message, Response};
use proto::Command::{JOIN, NICK, NICKSERV, PART, PRIVMSG, MODE, QUIT};
use futures::{Async, Poll, Future, Sink, Stream};
use futures::stream::SplitStream;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::sync::mpsc::UnboundedSender;
#[cfg(feature = "ctcp")]
use time;
use tokio_core::reactor::Core;

pub mod utils;

/// An interface for interacting with an IRC server.
pub trait Server {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;

    /// Sends a Command to this Server.
    fn send<M: Into<Message>>(&self, message: M) -> error::Result<()>
    where
        Self: Sized;

    /// Gets a stream of incoming messages from the Server.
    fn stream(&self) -> ServerStream;

    /// Blocks on the stream, running the given function on each incoming message as they arrive.
    fn for_each_incoming<F>(&self, mut f: F) -> ()
    where
        F: FnMut(Message) -> (),
    {
        self.stream().for_each(|msg| {
            f(msg);
            Ok(())
        }).wait().unwrap()
    }

    /// Gets a list of currently joined channels. This will be none if tracking is not supported
    /// altogether (such as when the `nochanlists` feature is enabled).
    fn list_channels(&self) -> Option<Vec<String>>;

    /// Gets a list of Users in the specified channel. This will be none if the channel is not
    /// being tracked, or if tracking is not supported altogether. For best results, be sure to
    /// request `multi-prefix` support from the server.
    fn list_users(&self, channel: &str) -> Option<Vec<User>>;
}

/// A stream of `Messages` from the `IrcServer`. Interaction with this stream relies on the
/// `futures` API.
pub struct ServerStream {
    state: Arc<ServerState>,
    stream: SplitStream<Connection>,
}

impl Stream for ServerStream {
    type Item = Message;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match try_ready!(self.stream.poll()) {
            Some(msg) => {
                self.state.handle_message(&msg)?;
                Ok(Async::Ready(Some(msg)))
            }
            None => Ok(Async::Ready(None)),
        }
    }
}

/// Thread-safe internal state for an IRC server connection.
struct ServerState {
    /// The configuration used with this connection.
    config: Config,
    /// A thread-safe map of channels to the list of users in them.
    chanlists: Mutex<HashMap<String, Vec<User>>>,
    /// A thread-safe index to track the current alternative nickname being used.
    alt_nick_index: RwLock<usize>,
    /// A thread-safe internal IRC stream used for the reading API.
    incoming: Mutex<Option<SplitStream<Connection>>>,
    /// A thread-safe copy of the outgoing channel.
    outgoing: UnboundedSender<Message>,
}

impl<'a> Server for ServerState {
    fn config(&self) -> &Config {
        &self.config
    }

    fn send<M: Into<Message>>(&self, msg: M) -> error::Result<()>
    where
        Self: Sized,
    {
        let msg = &msg.into();
        try!(self.handle_sent_message(&msg));
        Ok((&self.outgoing).send(
            ServerState::sanitize(&msg.to_string())
                .into(),
        )?)
    }

    fn stream(&self) -> ServerStream {
        unimplemented!()
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_channels(&self) -> Option<Vec<String>> {
        Some(
            self.chanlists
                .lock()
                .unwrap()
                .keys()
                .map(|k| k.to_owned())
                .collect(),
        )
    }

    #[cfg(feature = "nochanlists")]
    fn list_channels(&self) -> Option<Vec<String>> {
        None
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.chanlists
            .lock()
            .unwrap()
            .get(&chan.to_owned())
            .cloned()
    }

    #[cfg(feature = "nochanlists")]
    fn list_users(&self, _: &str) -> Option<Vec<User>> {
        None
    }
}

impl ServerState {
    fn new(
        incoming: SplitStream<Connection>,
        outgoing: UnboundedSender<Message>,
        config: Config,
    ) -> ServerState {
        ServerState {
            config: config,
            chanlists: Mutex::new(HashMap::new()),
            alt_nick_index: RwLock::new(0),
            incoming: Mutex::new(Some(incoming)),
            outgoing: outgoing,
        }
    }

    /// Sanitizes the input string by cutting up to (and including) the first occurence of a line
    /// terminiating phrase (`\r\n`, `\r`, or `\n`). This is used in sending messages back to
    /// prevent the injection of additional commands.
    fn sanitize(data: &str) -> &str {
        // n.b. ordering matters here to prefer "\r\n" over "\r"
        if let Some((pos, len)) = ["\r\n", "\r", "\n"]
            .iter()
            .flat_map(|needle| data.find(needle).map(|pos| (pos, needle.len())))
            .min_by_key(|&(pos, _)| pos)
        {
            data.split_at(pos + len).0
        } else {
            data
        }
    }

    /// Gets the current nickname in use.
    pub fn current_nickname(&self) -> &str {
        let alt_nicks = self.config().alternate_nicknames();
        let index = self.alt_nick_index.read().unwrap();
        match *index {
            0 => self.config().nickname(),
            i => alt_nicks[i - 1],
        }
    }

    /// Handles sent messages internally for basic client functionality.
    fn handle_sent_message(&self, msg: &Message) -> error::Result<()> {
        match msg.command {
            PART(ref chan, _) => {
                let _ = self.chanlists.lock().unwrap().remove(chan);
            }
            _ => (),
        }
        Ok(())
    }

    /// Handles received messages internally for basic client functionality.
    fn handle_message(&self, msg: &Message) -> error::Result<()> {
        match msg.command {
            JOIN(ref chan, _, _) => self.handle_join(msg.source_nickname().unwrap_or(""), chan),
            PART(ref chan, _) => self.handle_part(msg.source_nickname().unwrap_or(""), chan),
            QUIT(_) => self.handle_quit(msg.source_nickname().unwrap_or("")),
            NICK(ref new_nick) => {
                self.handle_nick_change(msg.source_nickname().unwrap_or(""), new_nick)
            }
            MODE(ref chan, ref mode, Some(ref user)) => self.handle_mode(chan, mode, user),
            PRIVMSG(ref target, ref body) => {
                if body.starts_with('\u{001}') {
                    let tokens: Vec<_> = {
                        let end = if body.ends_with('\u{001}') {
                            body.len() - 1
                        } else {
                            body.len()
                        };
                        body[1..end].split(' ').collect()
                    };
                    if target.starts_with('#') {
                        try!(self.handle_ctcp(target, tokens))
                    } else if let Some(user) = msg.source_nickname() {
                        try!(self.handle_ctcp(user, tokens))
                    }
                }
            }
            Command::Response(Response::RPL_NAMREPLY, ref args, ref suffix) => {
                self.handle_namreply(args, suffix)
            }
            Command::Response(Response::RPL_ENDOFMOTD, _, _) |
            Command::Response(Response::ERR_NOMOTD, _, _) => {
                self.send_nick_password()?;
                self.send_umodes()?;

                let config_chans = self.config().channels();
                for chan in &config_chans {
                    match self.config().channel_key(chan) {
                        Some(key) => self.send_join_with_keys(chan, key)?,
                        None => self.send_join(chan)?,
                    }
                }
                let joined_chans = self.chanlists.lock().unwrap();
                for chan in joined_chans.keys().filter(
                    |x| !config_chans.contains(&x.as_str()),
                )
                {
                    self.send_join(chan)?
                }
            }
            Command::Response(Response::ERR_NICKNAMEINUSE, _, _) |
            Command::Response(Response::ERR_ERRONEOUSNICKNAME, _, _) => {
                let alt_nicks = self.config().alternate_nicknames();
                let mut index = self.alt_nick_index.write().unwrap();
                if *index >= alt_nicks.len() {
                    panic!("All specified nicknames were in use or disallowed.")
                } else {
                    try!(self.send(NICK(alt_nicks[*index].to_owned())));
                    *index += 1;
                }
            }
            _ => (),
        }
        Ok(())
    }

    fn send_nick_password(&self) -> error::Result<()> {
        if self.config().nick_password().is_empty() {
            Ok(())
        } else {
            let mut index = self.alt_nick_index.write().unwrap();
            if self.config().should_ghost() && *index != 0 {
                for seq in &self.config().ghost_sequence() {
                    try!(self.send(NICKSERV(format!(
                        "{} {} {}",
                        seq,
                        self.config().nickname(),
                        self.config().nick_password()
                    ))));
                }
                *index = 0;
                try!(self.send(NICK(self.config().nickname().to_owned())))
            }
            self.send(NICKSERV(
                format!("IDENTIFY {}", self.config().nick_password()),
            ))
        }
    }

    fn send_umodes(&self) -> error::Result<()> {
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
        if let Some(vec) = self.chanlists.lock().unwrap().get_mut(&chan.to_owned()) {
            if !src.is_empty() {
                vec.push(User::new(src))
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_part(&self, src: &str, chan: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_part(&self, src: &str, chan: &str) {
        if let Some(vec) = self.chanlists.lock().unwrap().get_mut(&chan.to_owned()) {
            if !src.is_empty() {
                if let Some(n) = vec.iter().position(|x| x.get_nickname() == src) {
                    vec.swap_remove(n);
                }
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_quit(&self, _: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_quit(&self, src: &str) {
        if src.is_empty() {
            return;
        }
        let mut chanlists = self.chanlists.lock().unwrap();
        for channel in chanlists.clone().keys() {
            if let Some(vec) = chanlists.get_mut(&channel.to_owned()) {
                if let Some(p) = vec.iter().position(|x| x.get_nickname() == src) {
                    vec.swap_remove(p);
                }
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_nick_change(&self, _: &str, _: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_nick_change(&self, old_nick: &str, new_nick: &str) {
        if old_nick.is_empty() || new_nick.is_empty() {
            return;
        }
        let mut chanlists = self.chanlists.lock().unwrap();
        for channel in chanlists.clone().keys() {
            if let Some(vec) = chanlists.get_mut(&channel.to_owned()) {
                if let Some(n) = vec.iter().position(|x| x.get_nickname() == old_nick) {
                    let new_entry = User::new(new_nick);
                    vec[n] = new_entry;
                }
            }
        }
    }

    #[cfg(feature = "nochanlists")]
    fn handle_mode(&self, chan: &str, mode: &str, user: &str) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_mode(&self, chan: &str, mode: &str, user: &str) {
        if let Some(vec) = self.chanlists.lock().unwrap().get_mut(chan) {
            if let Some(n) = vec.iter().position(|x| x.get_nickname() == user) {
                vec[n].update_access_level(mode)
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
                    let mut chanlists = self.chanlists.lock().unwrap();
                    chanlists
                        .entry(chan.clone())
                        .or_insert_with(Vec::new)
                        .push(User::new(user))
                }
            }
        }
    }

    #[cfg(feature = "ctcp")]
    fn handle_ctcp(&self, resp: &str, tokens: Vec<&str>) -> error::Result<()> {
        if tokens.is_empty() {
            return Ok(());
        }
        match tokens[0] {
            "FINGER" => {
                self.send_ctcp_internal(
                    resp,
                    &format!(
                        "FINGER :{} ({})",
                        self.config().real_name(),
                        self.config().username()
                    ),
                )
            }
            "VERSION" => {
                self.send_ctcp_internal(resp, &format!("VERSION {}", self.config().version()))
            }
            "SOURCE" => {
                try!(self.send_ctcp_internal(
                    resp,
                    &format!("SOURCE {}", self.config().source()),
                ));
                self.send_ctcp_internal(resp, "SOURCE")
            }
            "PING" if tokens.len() > 1 => {
                self.send_ctcp_internal(resp, &format!("PING {}", tokens[1]))
            }
            "TIME" => self.send_ctcp_internal(resp, &format!("TIME :{}", time::now().rfc822z())),
            "USERINFO" => {
                self.send_ctcp_internal(resp, &format!("USERINFO :{}", self.config().user_info()))
            }
            _ => Ok(()),
        }
    }

    #[cfg(feature = "ctcp")]
    fn send_ctcp_internal(&self, target: &str, msg: &str) -> error::Result<()> {
        self.send_notice(target, &format!("\u{001}{}\u{001}", msg))
    }

    #[cfg(not(feature = "ctcp"))]
    fn handle_ctcp(&self, _: &str, _: Vec<&str>) -> error::Result<()> {
        Ok(())
    }
}

/// A thread-safe implementation of an IRC Server connection.
#[derive(Clone)]
pub struct IrcServer {
    /// The internal, thread-safe server state.
    state: Arc<ServerState>,
    /// A view of the logs for a mock connection.
    view: Option<LogView>,
}

impl Server for IrcServer {
    fn config(&self) -> &Config {
        &self.state.config
    }

    fn send<M: Into<Message>>(&self, msg: M) -> error::Result<()>
    where
        Self: Sized,
    {
        self.state.send(msg)
    }

    fn stream(&self) -> ServerStream {
        ServerStream {
            state: self.state.clone(),
            stream: self.state.incoming.lock().unwrap().take().unwrap(),
        }
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_channels(&self) -> Option<Vec<String>> {
        Some(
            self.state
                .chanlists
                .lock()
                .unwrap()
                .keys()
                .map(|k| k.to_owned())
                .collect(),
        )
    }

    #[cfg(feature = "nochanlists")]
    fn list_channels(&self) -> Option<Vec<String>> {
        None
    }

    #[cfg(not(feature = "nochanlists"))]
    fn list_users(&self, chan: &str) -> Option<Vec<User>> {
        self.state
            .chanlists
            .lock()
            .unwrap()
            .get(&chan.to_owned())
            .cloned()
    }

    #[cfg(feature = "nochanlists")]
    fn list_users(&self, _: &str) -> Option<Vec<User>> {
        None
    }
}

impl IrcServer {
    /// Creates a new IRC Server connection from the configuration at the specified path,
    /// connecting immediately.
    pub fn new<P: AsRef<Path>>(config: P) -> error::Result<IrcServer> {
        IrcServer::from_config(Config::load(config)?)
    }

    /// Creates a new IRC server connection from the specified configuration, connecting
    /// immediately.
    pub fn from_config(config: Config) -> error::Result<IrcServer> {
        // Setting up a remote reactor running for the length of the connection.
        let (tx_outgoing, rx_outgoing) = mpsc::unbounded();
        let (tx_incoming, rx_incoming) = oneshot::channel();
        let (tx_view, rx_view) = oneshot::channel();

        let cfg = config.clone();
        let _ = thread::spawn(move || {
            let mut reactor = Core::new().unwrap();

            // Setting up internal processing stuffs.
            let handle = reactor.handle();
            let conn = reactor
                .run(Connection::new(&cfg, &handle).unwrap())
                .unwrap();

            tx_view.send(conn.log_view()).unwrap();
            let (sink, stream) = conn.split();

            let outgoing_future = sink.send_all(rx_outgoing.map_err(|_| {
                let res: error::Error = error::ErrorKind::ChannelError.into();
                res
            })).map(|_| ()).map_err(|_| ());

            // Send the stream half back to the original thread.
            tx_incoming.send(stream).unwrap();

            reactor.run(outgoing_future).unwrap();
        });

        Ok(IrcServer {
            state: Arc::new(ServerState::new(rx_incoming.wait()?, tx_outgoing, config)),
            view: rx_view.wait()?,
        })
    }

    /// Gets the log view from the internal transport. Only used for unit testing.
    #[cfg(test)]
    fn log_view(&self) -> &LogView {
        self.view.as_ref().unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::{IrcServer, Server};
    use std::thread;
    use std::time::Duration;
    use std::collections::HashMap;
    use std::default::Default;
    use client::data::Config;
    #[cfg(not(feature = "nochanlists"))]
    use client::data::User;
    use proto::command::Command::{PART, PRIVMSG};
    use futures::{Future, Stream};

    pub fn test_config() -> Config {
        Config {
            owners: Some(vec![format!("test")]),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            server: Some(format!("irc.test.net")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            user_info: Some(format!("Testing.")),
            use_mock_connection: Some(true),
            ..Default::default()
        }
    }

    pub fn get_server_value(server: IrcServer) -> String {
        // We sleep here because of synchronization issues.
        // We can't guarantee that everything will have been sent by the time of this call.
        thread::sleep(Duration::from_millis(100));
        server.log_view().sent().unwrap().iter().fold(String::new(), |mut acc, msg| {
            acc.push_str(&msg.to_string());
            acc
        })
    }

    #[test]
    fn stream() {
        let exp = "PRIVMSG test :Hi!\r\nPRIVMSG test :This is a test!\r\n\
                   :test!test@test JOIN #test\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(exp.to_owned()),
            ..test_config()
        }).unwrap();
        let mut messages = String::new();
        server.stream().for_each(|message| {
            messages.push_str(&message.to_string());
            Ok(())
        }).wait().unwrap();
        assert_eq!(&messages[..], exp);
    }

    #[test]
    fn handle_message() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "JOIN #test\r\nJOIN #test2\r\n"
        );
    }

    #[test]
    fn handle_end_motd_with_nick_password() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NICKSERV IDENTIFY password\r\nJOIN #test\r\n\
                   JOIN #test2\r\n"
        );
    }

    #[test]
    fn handle_end_motd_with_chan_keys() {
        let value = ":irc.test.net 376 test :End of /MOTD command\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            nickname: Some(format!("test")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            channel_keys: {
                let mut map = HashMap::new();
                map.insert(format!("#test2"), format!("password"));
                Some(map)
            },
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "JOIN #test\r\nJOIN #test2 password\r\n"
        );
    }

    #[test]
    fn handle_end_motd_with_ghost() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.test.net 376 test2 :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            should_ghost: Some(true),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NICK :test2\r\nNICKSERV GHOST test password\r\n\
                   NICK :test\r\nNICKSERV IDENTIFY password\r\nJOIN #test\r\nJOIN #test2\r\n"
        );
    }

    #[test]
    fn handle_end_motd_with_ghost_seq() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.test.net 376 test2 :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            nickname: Some(format!("test")),
            alt_nicks: Some(vec![format!("test2")]),
            nick_password: Some(format!("password")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            should_ghost: Some(true),
            ghost_sequence: Some(vec![format!("RECOVER"), format!("RELEASE")]),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NICK :test2\r\nNICKSERV RECOVER test password\
                   \r\nNICKSERV RELEASE test password\r\nNICK :test\r\nNICKSERV IDENTIFY password\
                   \r\nJOIN #test\r\nJOIN #test2\r\n"
        );
    }

    #[test]
    fn handle_end_motd_with_umodes() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            nickname: Some(format!("test")),
            umodes: Some(format!("+B")),
            channels: Some(vec![format!("#test"), format!("#test2")]),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "MODE test +B\r\nJOIN #test\r\nJOIN #test2\r\n"
        );
    }

    #[test]
    fn nickname_in_use() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(&get_server_value(server)[..], "NICK :test2\r\n");
    }

    #[test]
    #[should_panic(expected = "All specified nicknames were in use or disallowed.")]
    fn ran_out_of_nicknames() {
        let value = ":irc.pdgn.co 433 * test :Nickname is already in use.\r\n\
                     :irc.pdgn.co 433 * test2 :Nickname is already in use.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
    }

    #[test]
    fn send() {
        let server = IrcServer::from_config(test_config()).unwrap();
        assert!(
            server
                .send(PRIVMSG(format!("#test"), format!("Hi there!")))
                .is_ok()
        );
        assert_eq!(
            &get_server_value(server)[..],
            "PRIVMSG #test :Hi there!\r\n"
        );
    }

    #[test]
    fn send_no_newline_injection() {
        let server = IrcServer::from_config(test_config()).unwrap();
        assert!(
            server
                .send(PRIVMSG(format!("#test"), format!("Hi there!\r\nJOIN #bad")))
                .is_ok()
        );
        assert_eq!(&get_server_value(server)[..], "PRIVMSG #test :Hi there!\r\n");
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn channel_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(server.list_channels().unwrap(), vec!["#test".to_owned()])
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn channel_tracking_names_part() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert!(server.send(PART(format!("#test"), None)).is_ok());
        assert!(server.list_channels().unwrap().is_empty())
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn user_tracking_names() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            server.list_users("#test").unwrap(),
            vec![User::new("test"), User::new("~owner"), User::new("&admin")]
        )
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn user_tracking_names_join() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :test2!test@test JOIN #test\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            server.list_users("#test").unwrap(),
            vec![
                User::new("test"),
                User::new("~owner"),
                User::new("&admin"),
                User::new("test2"),
            ]
        )
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn user_tracking_names_part() {
        let value = ":irc.test.net 353 test = #test :test ~owner &admin\r\n\
                     :owner!test@test PART #test\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            server.list_users("#test").unwrap(),
            vec![User::new("test"), User::new("&admin")]
        )
    }

    #[test]
    #[cfg(not(feature = "nochanlists"))]
    fn user_tracking_names_mode() {
        let value = ":irc.test.net 353 test = #test :+test ~owner &admin\r\n\
                     :test!test@test MODE #test +o test\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            server.list_users("#test").unwrap(),
            vec![User::new("@test"), User::new("~owner"), User::new("&admin")]
        );
        let mut exp = User::new("@test");
        exp.update_access_level("+v");
        assert_eq!(
            server.list_users("#test").unwrap()[0].highest_access_level(),
            exp.highest_access_level()
        );
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
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert!(server.list_users("#test").is_none())
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn finger_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}FINGER\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}FINGER :test (test)\u{001}\
                   \r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn version_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}VERSION\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}VERSION irc:git:Rust\u{001}\
                   \r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn source_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}SOURCE\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}SOURCE https://github.com/aatxe/irc\u{001}\r\n\
         NOTICE test :\u{001}SOURCE\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn ctcp_ping_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}PING test\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}PING test\u{001}\r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn time_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}TIME\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        let val = get_server_value(server);
        assert!(val.starts_with("NOTICE test :\u{001}TIME :"));
        assert!(val.ends_with("\u{001}\r\n"));
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn user_info_response() {
        let value = ":test!test@test PRIVMSG test :\u{001}USERINFO\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}USERINFO :Testing.\u{001}\
                   \r\n"
        );
    }

    #[test]
    #[cfg(feature = "ctcp")]
    fn ctcp_ping_no_timestamp() {
        let value = ":test!test@test PRIVMSG test :\u{001}PING\u{001}\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.stream().for_each(|message| {
            println!("{:?}", message);
            Ok(())
        }).wait().unwrap();
        assert_eq!(&get_server_value(server)[..], "");
    }
}
