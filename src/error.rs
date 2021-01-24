//! Errors for `irc` crate using `failure`.

use std::io::Error as IoError;
use std::sync::mpsc::RecvError;

use thiserror::Error;
use tokio::sync::mpsc::error::{SendError, TrySendError};

#[cfg(feature = "tls-rust")]
use tokio_rustls::webpki::InvalidDNSNameError;

use crate::proto::error::{MessageParseError, ProtocolError};

/// A specialized `Result` type for the `irc` crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The main crate-wide error type.
#[derive(Debug, Error)]
pub enum Error {
    /// An internal I/O error.
    #[error("an io error occurred")]
    Io(#[source] IoError),

    /// An internal proxy error.
    #[cfg(feature = "proxy")]
    #[error("a proxy error occurred")]
    Proxy(tokio_socks::Error),

    /// An internal TLS error.
    #[cfg(feature = "tls-native")]
    #[error("a TLS error occurred")]
    Tls(#[source] native_tls::Error),

    /// An internal DNS error.
    #[cfg(feature = "tls-rust")]
    #[error("a DNS error occurred")]
    Dns(#[source] InvalidDNSNameError),

    /// An internal synchronous channel closed.
    #[error("a sync channel closed")]
    SyncChannelClosed(#[source] RecvError),

    /// An internal asynchronous channel closed.
    #[error("an async channel closed")]
    AsyncChannelClosed,

    /// An internal oneshot channel closed.
    #[error("a oneshot channel closed")]
    OneShotCanceled,

    /// Error for invalid configurations.
    #[error("invalid config: {}", path)]
    InvalidConfig {
        /// The path to the configuration, or "<none>" if none specified.
        path: String,
        /// The detailed configuration error.
        #[source]
        cause: ConfigError,
    },

    /// Error for invalid messages.
    #[error("invalid message: {}", string)]
    InvalidMessage {
        /// The string that failed to parse.
        string: String,
        /// The detailed message parsing error.
        #[source]
        cause: MessageParseError,
    },

    /// Mutex for a logged transport was poisoned making the log inaccessible.
    #[error("mutex for a logged transport was poisoned")]
    PoisonedLog,

    /// Ping timed out due to no response.
    #[error("connection reset: no ping response")]
    PingTimeout,

    /// Failed to lookup an unknown codec.
    #[error("unknown codec: {}", codec)]
    UnknownCodec {
        /// The attempted codec.
        codec: String,
    },

    /// Failed to encode or decode something with the given codec.
    #[error("codec {} failed: {}", codec, data)]
    CodecFailed {
        /// The canonical codec name.
        codec: &'static str,
        /// The data that failed to encode or decode.
        data: String,
    },

    /// All specified nicknames were in use or unusable.
    #[error("none of the specified nicknames were usable")]
    NoUsableNick,

    /// Stream has already been configured.
    #[error("stream has already been configured")]
    StreamAlreadyConfigured,
}

/// Errors that occur with configurations.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to parse as TOML.
    #[cfg(feature = "toml_config")]
    #[error("invalid toml")]
    InvalidToml(#[source] TomlError),

    /// Failed to parse as JSON.
    #[cfg(feature = "json_config")]
    #[error("invalid json")]
    InvalidJson(#[source] serde_json::Error),

    /// Failed to parse as YAML.
    #[cfg(feature = "yaml_config")]
    #[error("invalid yaml")]
    InvalidYaml(#[source] serde_yaml::Error),

    /// Failed to parse the given format because it was disabled at compile-time.
    #[error("config format disabled: {}", format)]
    ConfigFormatDisabled {
        /// The disabled file format.
        format: &'static str,
    },

    /// Could not identify the given file format.
    #[error("config format unknown: {}", format)]
    UnknownConfigFormat {
        /// The unknown file extension.
        format: String,
    },

    /// File was missing an extension to identify file format.
    #[error("missing format extension")]
    MissingExtension,

    /// Configuration does not specify a nickname.
    #[error("nickname not specified")]
    NicknameNotSpecified,

    /// Configuration does not specify a server.
    #[error("server not specified")]
    ServerNotSpecified,

    /// The specified file could not be read.
    #[error("could not read file {}", file)]
    FileMissing {
        /// The supposed location of the file.
        file: String,
    },
}

/// A wrapper that combines toml's serialization and deserialization errors.
#[cfg(feature = "toml_config")]
#[derive(Debug, Error)]
pub enum TomlError {
    /// A TOML deserialization error.
    #[error("deserialization failed")]
    Read(#[source] toml::de::Error),
    /// A TOML serialization error.
    #[error("serialization failed")]
    Write(#[source] toml::ser::Error),
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

#[cfg(feature = "proxy")]
impl From<tokio_socks::Error> for Error {
    fn from(e: tokio_socks::Error) -> Error {
        Error::Proxy(e)
    }
}

#[cfg(feature = "tls-native")]
impl From<native_tls::Error> for Error {
    fn from(e: native_tls::Error) -> Error {
        Error::Tls(e)
    }
}

#[cfg(feature = "tls-rust")]
impl From<InvalidDNSNameError> for Error {
    fn from(e: InvalidDNSNameError) -> Error {
        Error::Dns(e)
    }
}

impl From<RecvError> for Error {
    fn from(e: RecvError) -> Error {
        Error::SyncChannelClosed(e)
    }
}

impl<T> From<SendError<T>> for Error {
    fn from(_: SendError<T>) -> Error {
        Error::AsyncChannelClosed
    }
}

impl<T> From<TrySendError<T>> for Error {
    fn from(_: TrySendError<T>) -> Error {
        Error::AsyncChannelClosed
    }
}
