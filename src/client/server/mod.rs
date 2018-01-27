//! The primary API for communicating with an IRC server.
//!
//! This API provides the ability to connect to an IRC server via the
//! [IrcServer](struct.IrcServer.html) type. The [Server](trait.Server.html) trait that
//! [IrcServer](struct.IrcServer.html) implements provides methods for communicating with this
//! server. An extension trait, [ServerExt](./utils/trait.ServerExt.html), provides short-hand for
//! sending a variety of important messages without referring to their entries in
//! [proto::command](../../proto/command/enum.Command.html).
//!
//! # Examples
//!
//! Using these APIs, we can connect to a server and send a one-off message (in this case,
//! identifying with the server).
//!
//! ```no_run
//! # extern crate irc;
//! use irc::client::prelude::{IrcServer, ServerExt};
//!
//! # fn main() {
//! let server = IrcServer::new("config.toml").unwrap();
//! // identify comes from `ServerExt`
//! server.identify().unwrap();
//! # }
//! ```
//!
//! We can then use functions from [Server](trait.Server.html) to receive messages from the
//! server in a blocking fashion and perform any desired actions in response. The following code
//! performs a simple call-and-response when the bot's name is mentioned in a channel.
//!
//! ```no_run
//! # extern crate irc;
//! # use irc::client::prelude::{IrcServer, ServerExt};
//! use irc::client::prelude::{Server, Command};
//!
//! # fn main() {
//! # let server = IrcServer::new("config.toml").unwrap();
//! # server.identify().unwrap();
//! server.for_each_incoming(|irc_msg| {
//!   match irc_msg.command {
//!     Command::PRIVMSG(channel, message) => if message.contains(server.current_nickname()) {
//!       server.send_privmsg(&channel, "beep boop").unwrap();
//!     }
//!     _ => ()
//!   }
//! }).unwrap();
//! # }
//! ```
#[cfg(feature = "ctcp")]
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

#[cfg(feature = "ctcp")]
use chrono::prelude::*;
use futures::{Async, Poll, Future, Sink, Stream};
use futures::stream::SplitStream;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_core::reactor::{Core, Handle};

use error;
use client::conn::{Connection, ConnectionFuture};
use client::data::{Config, User};
use client::server::utils::ServerExt;
use client::transport::LogView;
use proto::{ChannelMode, Command, Message, Mode, Response};
use proto::Command::{JOIN, NICK, NICKSERV, PART, PRIVMSG, ChannelMODE, QUIT};

pub mod utils;

/// Trait extending all IRC streams with `for_each_incoming` convenience function.
///
/// This is typically used in conjunction with [Server::stream](trait.Server.html#tymethod.stream)
/// in order to use an API akin to
/// [Server::for_each_incoming](trait.Server.html#method.for_each_incoming).
///
/// # Example
///
/// ```no_run
/// # extern crate irc;
/// # use irc::client::prelude::{IrcServer, Server, Command, ServerExt};
/// use irc::client::prelude::EachIncomingExt;
///
/// # fn main() {
/// # let server = IrcServer::new("config.toml").unwrap();
/// # server.identify().unwrap();
/// server.stream().for_each_incoming(|irc_msg| {
///   match irc_msg.command {
///     Command::PRIVMSG(channel, message) => if message.contains(server.current_nickname()) {
///       server.send_privmsg(&channel, "beep boop").unwrap();
///     }
///     _ => ()
///   }
/// }).unwrap();
/// # }
/// ```
pub trait EachIncomingExt: Stream<Item=Message, Error=error::Error> {
    /// Blocks on the stream, running the given function on each incoming message as they arrive.
    fn for_each_incoming<F>(self, mut f: F) -> error::Result<()>
        where
        F: FnMut(Message) -> (),
        Self: Sized,
    {
        self.for_each(|msg| {
            f(msg);
            Ok(())
        }).wait()
    }
}

impl<T> EachIncomingExt for T where T: Stream<Item=Message, Error=error::Error> {}

/// An interface for communicating with an IRC server.
pub trait Server {
    /// Gets the configuration being used with this Server.
    fn config(&self) -> &Config;

    /// Sends a [Command](../../proto/command/enum.Command.html) to this Server. This is the core
    /// primitive for sending messages to the server. In practice, it's often more pleasant (and
    /// more idiomatic) to use the functions defined on [ServerExt](./utils/trait.ServerExt.html).
    /// They capture a lot of the more repetitive aspects of sending messages.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// # let server = IrcServer::new("config.toml").unwrap();
    /// server.send(Command::NICK("example".to_owned())).unwrap();
    /// server.send(Command::USER("user".to_owned(), "0".to_owned(), "name".to_owned())).unwrap();
    /// # }
    /// ```
    fn send<M: Into<Message>>(&self, message: M) -> error::Result<()>
    where
        Self: Sized;

    /// Gets a stream of incoming messages from the Server. This is only necessary when trying to
    /// set up more complex clients, and requires use of the `futures` crate. Most IRC bots should
    /// be able to get by using only `for_each_incoming` to handle received messages. You can find
    /// some examples of more complex setups using `stream` in the
    /// [GitHub repository](https://github.com/aatxe/irc/tree/master/examples).
    ///
    /// **Note**: The stream can only be returned once. Subsequent attempts will cause a panic.
    fn stream(&self) -> ServerStream;

    /// Blocks on the stream, running the given function on each incoming message as they arrive.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use irc::client::prelude::{IrcServer, ServerExt, Server, Command};
    /// # fn main() {
    /// # let server = IrcServer::new("config.toml").unwrap();
    /// # server.identify().unwrap();
    /// server.for_each_incoming(|irc_msg| {
    ///   match irc_msg.command {
    ///     Command::PRIVMSG(channel, message) => if message.contains(server.current_nickname()) {
    ///       server.send_privmsg(&channel, "beep boop").unwrap();
    ///     }
    ///     _ => ()
    ///   }
    /// }).unwrap();
    /// # }
    /// ```
    fn for_each_incoming<F>(&self, f: F) -> error::Result<()>
    where
        F: FnMut(Message) -> (),
    {
        self.stream().for_each_incoming(f)
    }

    /// Gets a list of currently joined channels. This will be `None` if tracking is disabled
    /// altogether via the `nochanlists` feature.
    fn list_channels(&self) -> Option<Vec<String>>;

    /// Gets a list of [Users](../data/user/struct.User.html) in the specified channel. If the
    /// specified channel hasn't been joined or the `nochanlists` feature is enabled, this function
    /// will return `None`.
    ///
    /// For best results, be sure to request `multi-prefix` support from the server. This will allow
    /// for more accurate tracking of user rank (e.g. oper, half-op, etc.).
    /// # Requesting multi-prefix support
    /// ```no_run
    /// # extern crate irc;
    /// # use irc::client::prelude::{IrcServer, ServerExt, Server, Command};
    /// use irc::proto::caps::Capability;
    ///
    /// # fn main() {
    /// # let server = IrcServer::new("config.toml").unwrap();
    /// server.send_cap_req(&[Capability::MultiPrefix]).unwrap();
    /// server.identify().unwrap();
    /// # }
    /// ```
    fn list_users(&self, channel: &str) -> Option<Vec<User>>;
}

/// A stream of `Messages` from the `IrcServer`.
///
/// Interaction with this stream relies on the `futures` API, but is only expected for less
/// traditional use cases. To learn more, you can view the documentation for the
/// [futures](https://docs.rs/futures/) crate, or the tutorials for
/// [tokio](https://tokio.rs/docs/getting-started/futures/).
#[derive(Debug)]
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
#[derive(Debug)]
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
        self.handle_sent_message(msg)?;
        Ok((&self.outgoing).unbounded_send(
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
    fn current_nickname(&self) -> &str {
        let alt_nicks = self.config().alternate_nicknames();
        let index = self.alt_nick_index.read().unwrap();
        match *index {
            0 => self.config().nickname().expect(
                "current_nickname should not be callable if nickname is not defined."
            ),
            i => alt_nicks[i - 1],
        }
    }

    /// Handles sent messages internally for basic client functionality.
    fn handle_sent_message(&self, msg: &Message) -> error::Result<()> {
        trace!("[SENT] {}", msg.to_string());
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
        trace!("[RECV] {}", msg.to_string());
        match msg.command {
            JOIN(ref chan, _, _) => self.handle_join(msg.source_nickname().unwrap_or(""), chan),
            /// This will panic if not specified.
            PART(ref chan, _) => self.handle_part(msg.source_nickname().unwrap_or(""), chan),
            QUIT(_) => self.handle_quit(msg.source_nickname().unwrap_or("")),
            NICK(ref new_nick) => {
                self.handle_nick_change(msg.source_nickname().unwrap_or(""), new_nick)
            }
            ChannelMODE(ref chan, ref modes) => self.handle_mode(chan, modes),
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
                        self.handle_ctcp(target, &tokens)?
                    } else if let Some(user) = msg.source_nickname() {
                        self.handle_ctcp(user, &tokens)?
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
                    self.send(NICK(alt_nicks[*index].to_owned()))?;
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
                    self.send(NICKSERV(format!(
                        "{} {} {}",
                        seq,
                        self.config().nickname()?,
                        self.config().nick_password()
                    )))?;
                }
                *index = 0;
                self.send(NICK(self.config().nickname()?.to_owned()))?
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
            self.send_mode(self.current_nickname(), &Mode::as_user_modes(self.config().umodes())?)
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
    fn handle_mode(&self, _: &str, _: &[Mode<ChannelMODE>]) {}

    #[cfg(not(feature = "nochanlists"))]
    fn handle_mode(&self, chan: &str, modes: &[Mode<ChannelMode>]) {
        for mode in modes {
            match *mode {
                Mode::Plus(_, Some(ref user)) | Mode::Minus(_, Some(ref user)) => {
                    if let Some(vec) = self.chanlists.lock().unwrap().get_mut(chan) {
                        if let Some(n) = vec.iter().position(|x| x.get_nickname() == user) {
                            vec[n].update_access_level(mode)
                        }
                    }
                }
                _ => (),
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
    fn handle_ctcp(&self, resp: &str, tokens: &[&str]) -> error::Result<()> {
        if tokens.is_empty() {
            return Ok(());
        }
        if tokens[0].eq_ignore_ascii_case("FINGER") {
            self.send_ctcp_internal(
                resp,
                &format!(
                    "FINGER :{} ({})",
                    self.config().real_name(),
                    self.config().username()
                ),
            )
        } else if tokens[0].eq_ignore_ascii_case("VERSION") {
            self.send_ctcp_internal(resp, &format!("VERSION {}", self.config().version()))
        } else if tokens[0].eq_ignore_ascii_case("SOURCE") {
            self.send_ctcp_internal(
                resp,
                &format!("SOURCE {}", self.config().source()),
            )?;
            self.send_ctcp_internal(resp, "SOURCE")
        } else if tokens[0].eq_ignore_ascii_case("PING") && tokens.len() > 1 {
            self.send_ctcp_internal(resp, &format!("PING {}", tokens[1]))
        } else if tokens[0].eq_ignore_ascii_case("TIME") {
            self.send_ctcp_internal(resp, &format!("TIME :{}", Local::now().to_rfc2822()))
        } else if tokens[0].eq_ignore_ascii_case("USERINFO") {
            self.send_ctcp_internal(resp, &format!("USERINFO :{}", self.config().user_info()))
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "ctcp")]
    fn send_ctcp_internal(&self, target: &str, msg: &str) -> error::Result<()> {
        self.send_notice(target, &format!("\u{001}{}\u{001}", msg))
    }

    #[cfg(not(feature = "ctcp"))]
    fn handle_ctcp(&self, _: &str, _: &[&str]) -> error::Result<()> {
        Ok(())
    }
}

/// The canonical implementation of a connection to an IRC server.
///
/// The type itself provides a number of methods to create new connections, but most of the API
/// surface is in the form of the [Server](trait.Server.html) and
/// [ServerExt](./utils/trait.ServerExt.html) traits that provide methods of communicating with the
/// server after connection. Cloning an `IrcServer` is relatively cheap, as it's equivalent to
/// cloning a single `Arc`. This may be useful for setting up multiple threads with access to one
/// connection.
///
/// For a full example usage, see [irc::client::server](./index.html).
#[derive(Clone, Debug)]
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
            state: Arc::clone(&self.state),
            stream: self.state.incoming.lock().unwrap().take().expect(
                "Stream was already obtained once, and cannot be reobtained."
            ),
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
    /// Creates a new IRC Server connection from the configuration at the specified path, connecting
    /// immediately. This function is short-hand for loading the configuration and then calling
    /// `IrcServer::from_config` and consequently inherits its behaviors.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// let server = IrcServer::new("config.toml").unwrap();
    /// # }
    /// ```
    pub fn new<P: AsRef<Path>>(config: P) -> error::Result<IrcServer> {
        IrcServer::from_config(Config::load(config)?)
    }

    /// Creates a new IRC server connection from the specified configuration, connecting
    /// immediately. Due to current design limitations, error handling here is somewhat limited. In
    /// particular, failed connections will cause the program to panic because the connection
    /// attempt is made on a freshly created thread. If you need to avoid this behavior and handle
    /// errors more gracefully, it is recommended that you use an
    /// [IrcReactor](../reactor/struct.IrcReactor.html) instead.
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # fn main() {
    /// let config = Config {
    ///   nickname: Some("example".to_owned()),
    ///   server: Some("irc.example.com".to_owned()),
    ///   .. Default::default()
    /// };
    /// let server = IrcServer::from_config(config).unwrap();
    /// # }
    /// ```
    pub fn from_config(config: Config) -> error::Result<IrcServer> {
        // Setting up a remote reactor running for the length of the connection.
        let (tx_outgoing, rx_outgoing) = mpsc::unbounded();
        let (tx_incoming, rx_incoming) = oneshot::channel();
        let (tx_view, rx_view) = oneshot::channel();

        let mut reactor = Core::new()?;
        let handle = reactor.handle();
        // Attempting to connect here (as opposed to on the thread) allows more errors to happen
        // immediately, rather than to occur as panics on the thread. In particular, non-resolving
        // server names, and failed SSL setups will appear here.
        let conn = reactor.run(Connection::new(&config, &handle)?)?;

        let _ = thread::spawn(move || {
            let mut reactor = Core::new().unwrap();

            tx_view.send(conn.log_view()).unwrap();
            let (sink, stream) = conn.split();

            let outgoing_future = sink.send_all(rx_outgoing.map_err(|_| {
                let res: error::Error = error::ErrorKind::ChannelError.into();
                res
            })).map(|_| ()).map_err(|e| panic!("{}", e));

            // Send the stream half back to the original thread.
            tx_incoming.send(stream).unwrap();

            reactor.run(outgoing_future).unwrap();
        });

        Ok(IrcServer {
            state: Arc::new(ServerState::new(rx_incoming.wait()?, tx_outgoing, config)),
            view: rx_view.wait()?,
        })
    }

    /// Creates a new IRC server connection from the specified configuration and on the event loop
    /// corresponding to the given handle. This can be used to set up a number of IrcServers on a
    /// single, shared event loop. It can also be used to take more control over execution and error
    /// handling. Connection will not occur until the event loop is run.
    ///
    /// Proper usage requires familiarity with `tokio` and `futures`. You can find more information
    /// in the crate documentation for [tokio-core](http://docs.rs/tokio-core) or
    /// [futures](http://docs.rs/futures). Additionally, you can find detailed tutorials on using
    /// both libraries on the [tokio website](https://tokio.rs/docs/getting-started/tokio/). An easy
    /// to use abstraction that does not require this knowledge is available via
    /// [IrcReactors](../reactor/struct.IrcReactor.html).
    ///
    /// # Example
    /// ```no_run
    /// # extern crate irc;
    /// # extern crate tokio_core;
    /// # use std::default::Default;
    /// # use irc::client::prelude::*;
    /// # use irc::client::server::PackedIrcServer;
    /// # use irc::error;
    /// # use tokio_core::reactor::Core;
    /// # fn main() {
    /// # let config = Config {
    /// #  nickname: Some("example".to_owned()),
    /// #  server: Some("irc.example.com".to_owned()),
    /// #  .. Default::default()
    /// # };
    /// let mut reactor = Core::new().unwrap();
    /// let future = IrcServer::new_future(reactor.handle(), &config).unwrap();
    /// // immediate connection errors (like no internet) will turn up here...
    /// let PackedIrcServer(server, future) = reactor.run(future).unwrap();
    /// // runtime errors (like disconnections and so forth) will turn up here...
    /// reactor.run(server.stream().for_each(move |irc_msg| {
    ///   // processing messages works like usual
    ///   process_msg(&server, irc_msg)
    /// }).join(future)).unwrap();
    /// # }
    /// # fn process_msg(server: &IrcServer, message: Message) -> error::Result<()> { Ok(()) }
    /// ```
    pub fn new_future(handle: Handle, config: &Config) -> error::Result<IrcServerFuture> {
        let (tx_outgoing, rx_outgoing) = mpsc::unbounded();

        Ok(IrcServerFuture {
            conn: Connection::new(config, &handle)?,
            _handle: handle,
            config: config,
            tx_outgoing: Some(tx_outgoing),
            rx_outgoing: Some(rx_outgoing),
        })
    }

    /// Gets the current nickname in use. This may be the primary username set in the configuration,
    /// or it could be any of the alternative nicknames listed as well. As a result, this is the
    /// preferred way to refer to the client's nickname.
    pub fn current_nickname(&self) -> &str {
        self.state.current_nickname()
    }

    /// Gets the log view from the internal transport. Only used for unit testing.
    #[cfg(test)]
    fn log_view(&self) -> &LogView {
        self.view.as_ref().unwrap()
    }
}

/// A future representing the eventual creation of an `IrcServer`.
///
/// Interaction with this future relies on the `futures` API, but is only expected for more advanced
/// use cases. To learn more, you can view the documentation for the
/// [futures](https://docs.rs/futures/) crate, or the tutorials for
/// [tokio](https://tokio.rs/docs/getting-started/futures/). An easy to use abstraction that does
/// not require this knowledge is available via [IrcReactors](../reactor/struct.IrcReactor.html).
    #[derive(Debug)]
pub struct IrcServerFuture<'a> {
    conn: ConnectionFuture<'a>,
    _handle: Handle,
    config: &'a Config,
    tx_outgoing: Option<UnboundedSender<Message>>,
    rx_outgoing: Option<UnboundedReceiver<Message>>,
}

impl<'a> Future for IrcServerFuture<'a> {
    type Item = PackedIrcServer;
    type Error = error::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let conn = try_ready!(self.conn.poll());

        let view = conn.log_view();
        let (sink, stream) = conn.split();

        let outgoing_future = sink.send_all(self.rx_outgoing.take().unwrap().map_err(|()| {
            let res: error::Error = error::ErrorKind::ChannelError.into();
            res
        })).map(|_| ());

        let server = IrcServer {
            state: Arc::new(ServerState::new(
                stream, self.tx_outgoing.take().unwrap(), self.config.clone()
            )),
            view: view,
        };
        Ok(Async::Ready(PackedIrcServer(server, Box::new(outgoing_future))))
    }
}

/// An `IrcServer` packaged with a future that drives its message sending. In order for the server
/// to actually work properly, this future _must_ be running.
///
/// This type should only be used by advanced users who are familiar with the implementation of this
/// crate. An easy to use abstraction that does not require this knowledge is available via
/// [IrcReactors](../reactor/struct.IrcReactor.html).
pub struct PackedIrcServer(pub IrcServer, pub Box<Future<Item = (), Error = error::Error>>);

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::default::Default;
    use std::thread;
    use std::time::Duration;

    use super::{IrcServer, Server};
    use client::data::Config;
    #[cfg(not(feature = "nochanlists"))]
    use client::data::User;
    use proto::{ChannelMode, Mode};
    use proto::command::Command::{PART, PRIVMSG};

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
        server.for_each_incoming(|message| {
            messages.push_str(&message.to_string());
        }).unwrap();
        assert_eq!(&messages[..], exp);
    }

    #[test]
    fn handle_message() {
        let value = ":irc.test.net 376 test :End of /MOTD command.\r\n";
        let server = IrcServer::from_config(Config {
            mock_initial_value: Some(value.to_owned()),
            ..test_config()
        }).unwrap();
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
        assert_eq!(
            server.list_users("#test").unwrap(),
            vec![User::new("@test"), User::new("~owner"), User::new("&admin")]
        );
        let mut exp = User::new("@test");
        exp.update_access_level(&Mode::Plus(ChannelMode::Voice, None));
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            "NOTICE test :\u{001}FINGER :test (test)\u{001}\r\n"
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
        assert_eq!(
            &get_server_value(server)[..],
            &format!(
                "NOTICE test :\u{001}VERSION {}\u{001}\r\n",
                ::VERSION_STR,
            )
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
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
        server.for_each_incoming(|message| {
            println!("{:?}", message);
        }).unwrap();
        assert_eq!(&get_server_value(server)[..], "");
    }
}
