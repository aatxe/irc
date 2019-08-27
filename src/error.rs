//! Errors for `irc` crate using `failure`.

use std::io::Error as IoError;
use std::sync::mpsc::RecvError;

use failure;
use futures_channel::{
    mpsc::{SendError, TrySendError},
    oneshot::Canceled,
};
use native_tls::Error as TlsError;
#[cfg(feature = "json")]
use serde_json::Error as JsonError;
#[cfg(feature = "yaml")]
use serde_yaml::Error as YamlError;
#[cfg(feature = "toml")]
use toml::de::Error as TomlReadError;
#[cfg(feature = "toml")]
use toml::ser::Error as TomlWriteError;

use crate::proto::error::{MessageParseError, ProtocolError};

/// A specialized `Result` type for the `irc` crate.
pub type Result<T> = ::std::result::Result<T, Error>;

/// The main crate-wide error type.
#[derive(Debug, Fail)]
pub enum Error {
    /// An internal I/O error.
    #[fail(display = "an io error occurred")]
    Io(#[cause] IoError),

    /// An internal TLS error.
    #[fail(display = "a TLS error occurred")]
    Tls(#[cause] TlsError),

    /// An internal synchronous channel closed.
    #[fail(display = "a sync channel closed")]
    SyncChannelClosed(#[cause] RecvError),

    /// An internal asynchronous channel closed.
    #[fail(display = "an async channel closed")]
    AsyncChannelClosed(#[cause] SendError),

    /// An internal oneshot channel closed.
    #[fail(display = "a oneshot channel closed")]
    OneShotCanceled(#[cause] Canceled),

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

    /// Stream has already been configured.
    #[fail(display = "stream has already been configured")]
    StreamAlreadyConfigured,

    /// This allows you to produce any `failure::Error` within closures used by
    /// the irc crate. No errors of this kind will ever be produced by the crate
    /// itself.
    #[fail(display = "{}", inner)]
    Custom {
        /// The actual error that occurred.
        inner: failure::Error,
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

impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Error {
        match e {
            ProtocolError::Io(e) => Error::Io(e),
            ProtocolError::InvalidMessage { string, cause } => {
                Error::InvalidMessage { string, cause }
            }
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}

impl From<TlsError> for Error {
    fn from(e: TlsError) -> Error {
        Error::Tls(e)
    }
}

impl From<RecvError> for Error {
    fn from(e: RecvError) -> Error {
        Error::SyncChannelClosed(e)
    }
}

impl From<SendError> for Error {
    fn from(e: SendError) -> Error {
        Error::AsyncChannelClosed(e)
    }
}

impl<T> From<TrySendError<T>> for Error {
    fn from(e: TrySendError<T>) -> Error {
        Error::AsyncChannelClosed(e.into_send_error())
    }
}

impl From<Canceled> for Error {
    fn from(e: Canceled) -> Error {
        Error::OneShotCanceled(e)
    }
}
