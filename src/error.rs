//! Errors for `irc` crate using `failure`.

use std::io::Error as IoError;
use std::sync::mpsc::RecvError;

use failure;
use futures::sync::mpsc::SendError;
use futures::sync::oneshot::Canceled;
use native_tls::Error as TlsError;
#[cfg(feature = "json")]
use serde_json::Error as JsonError;
#[cfg(feature = "yaml")]
use serde_yaml::Error as YamlError;
use tokio::executor::SpawnError;
use tokio_timer::TimerError;
#[cfg(feature = "toml")]
use toml::de::Error as TomlReadError;
#[cfg(feature = "toml")]
use toml::ser::Error as TomlWriteError;

use proto::Message;
use proto::error::{ProtocolError, MessageParseError};

/// A specialized `Result` type for the `irc` crate.
pub type Result<T> = ::std::result::Result<T, IrcError>;

/// The main crate-wide error type.
#[derive(Debug, Fail)]
pub enum IrcError {
    /// An internal I/O error.
    #[fail(display = "an io error occurred")]
    Io(#[cause] IoError),

    /// An internal TLS error.
    #[fail(display = "a TLS error occurred")]
    Tls(#[cause] TlsError),

    /// An error caused by Tokio being unable to spawn a task.
    #[fail(display = "unable to spawn task")]
    Spawn(#[cause] SpawnError),

    /// An internal synchronous channel closed.
    #[fail(display = "a sync channel closed")]
    SyncChannelClosed(#[cause] RecvError),

    /// An internal asynchronous channel closed.
    #[fail(display = "an async channel closed")]
    AsyncChannelClosed(#[cause] SendError<Message>),

    /// An internal oneshot channel closed.
    #[fail(display = "a oneshot channel closed")]
    OneShotCanceled(#[cause] Canceled),

    /// An internal timer error.
    #[fail(display = "timer failed")]
    Timer(#[cause] TimerError),

    /// Error for invalid configurations.
    #[fail(display = "invalid config: {}", path)]
    InvalidConfig {
        /// The path to the configuration, or "<none>" if none specified.
        path: String,
        /// The detailed configuration error.
        #[cause]
        cause: ConfigError,
    },

    /// Error for invalid messages.
    #[fail(display = "invalid message: {}", string)]
    InvalidMessage {
        /// The string that failed to parse.
        string: String,
        /// The detailed message parsing error.
        #[cause]
        cause: MessageParseError,
    },

    /// Mutex for a logged transport was poisoned making the log inaccessible.
    #[fail(display = "mutex for a logged transport was poisoned")]
    PoisonedLog,

    /// Ping timed out due to no response.
    #[fail(display = "connection reset: no ping response")]
    PingTimeout,

    /// Failed to lookup an unknown codec.
    #[fail(display = "unknown codec: {}", codec)]
    UnknownCodec {
        /// The attempted codec.
        codec: String,
    },

    /// Failed to encode or decode something with the given codec.
    #[fail(display = "codec {} failed: {}", codec, data)]
    CodecFailed {
        /// The canonical codec name.
        codec: &'static str,
        /// The data that failed to encode or decode.
        data: String,
    },

    /// All specified nicknames were in use or unusable.
    #[fail(display = "none of the specified nicknames were usable")]
    NoUsableNick,

    /// This allows you to produce any `failure::Error` within closures used by
    /// the irc crate. No errors of this kind will ever be produced by the crate
    /// itself.
    #[fail(display = "{}", inner)]
    Custom {
        /// The actual error that occurred.
        inner: failure::Error
    },
}

/// Errors that occur with configurations.
#[derive(Debug, Fail)]
pub enum ConfigError {
    /// Failed to parse as TOML.
    #[cfg(feature = "toml")]
    #[fail(display = "invalid toml")]
    InvalidToml(#[cause] TomlError),

    /// Failed to parse as JSON.
    #[cfg(feature = "json")]
    #[fail(display = "invalid json")]
    InvalidJson(#[cause] JsonError),

    /// Failed to parse as YAML.
    #[cfg(feature = "yaml")]
    #[fail(display = "invalid yaml")]
    InvalidYaml(#[cause] YamlError),

    /// Failed to parse the given format because it was disabled at compile-time.
    #[fail(display = "config format disabled: {}", format)]
    ConfigFormatDisabled {
        /// The disabled file format.
        format: &'static str,
    },

    /// Could not identify the given file format.
    #[fail(display = "config format unknown: {}", format)]
    UnknownConfigFormat {
        /// The unknown file extension.
        format: String,
    },

    /// File was missing an extension to identify file format.
    #[fail(display = "missing format extension")]
    MissingExtension,

    /// Configuration does not specify a nickname.
    #[fail(display = "nickname not specified")]
    NicknameNotSpecified,

    /// Configuration does not specify a server.
    #[fail(display = "server not specified")]
    ServerNotSpecified,
}

/// A wrapper that combines toml's serialization and deserialization errors.
#[cfg(feature = "toml")]
#[derive(Debug, Fail)]
pub enum TomlError {
    /// A TOML deserialization error.
    #[fail(display = "deserialization failed")]
    Read(#[cause] TomlReadError),
    /// A TOML serialization error.
    #[fail(display = "serialization failed")]
    Write(#[cause] TomlWriteError),
}

impl From<ProtocolError> for IrcError {
    fn from(e: ProtocolError) -> IrcError {
        match e {
            ProtocolError::Io(e) => IrcError::Io(e),
            ProtocolError::InvalidMessage { string, cause } => IrcError::InvalidMessage {
                string, cause
            },
        }
    }
}

impl From<IoError> for IrcError {
    fn from(e: IoError) -> IrcError {
        IrcError::Io(e)
    }
}

impl From<TlsError> for IrcError {
    fn from(e: TlsError) -> IrcError {
        IrcError::Tls(e)
    }
}

impl From<SpawnError> for IrcError {
    fn from(e: SpawnError) -> IrcError {
        IrcError::Spawn(e)
    }
}

impl From<RecvError> for IrcError {
    fn from(e: RecvError) -> IrcError {
        IrcError::SyncChannelClosed(e)
    }
}

impl From<SendError<Message>> for IrcError {
    fn from(e: SendError<Message>) -> IrcError {
        IrcError::AsyncChannelClosed(e)
    }
}

impl From<Canceled> for IrcError {
    fn from(e: Canceled) -> IrcError {
        IrcError::OneShotCanceled(e)
    }
}

impl From<TimerError> for IrcError {
    fn from(e: TimerError) -> IrcError {
        IrcError::Timer(e)
    }
}
