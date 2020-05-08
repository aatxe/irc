//! A client-side IRC prelude, re-exporting the complete high-level IRC client API.
//!
//! # Structure
//! A connection to an IRC server is created via an `IrcClient` which is configured using a
//! `Config` struct that defines data such as which server to connect to, on what port, and
//! using what nickname. The `Client` trait provides an API for actually interacting with the
//! server once a connection has been established. This API intentionally offers only a single
//! method to send `Commands` because it makes it easy to see the whole set of possible
//! interactions with a server. The `ClientExt` trait addresses this deficiency by defining a
//! number of methods that provide a more clear and succinct interface for sending various
//! common IRC commands to the server. An `IrcReactor` can be used to create and manage multiple
//! `IrcClients` with more fine-grained control over error management.
//!
//! The various `proto` types capture details of the IRC protocol that are used throughout the
//! client API. `Message`, `Command`, and `Response` are used to send and receive messages along
//! the connection, and are naturally at the heart of communication in the IRC protocol.
//! `Capability` and `NegotiationVersion` are used to determine (with the server) what IRCv3
//! functionality to enable for the connection. Certain parts of the API offer suggestions for
//! extensions that will improve the user experience, and give examples of how to enable them
//! using `Capability`. `Mode`, `ChannelMode`, and `UserMode` are used in a high-level API for
//! dealing with IRC channel and user modes. They appear in methods for sending mode commands,
//! as well as in the parsed form of received mode commands.

#[cfg(feature = "proxy")]
pub use crate::client::data::ProxyType;

pub use crate::{
    client::{data::Config, Client, Sender},
    proto::{
        Capability, ChannelExt, ChannelMode, Command, Message, Mode, NegotiationVersion, Prefix,
        Response, UserMode,
    },
};
