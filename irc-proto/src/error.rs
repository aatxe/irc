//! IRC protocol errors using `failure`.

use std::io::Error as IoError;

/// A `Result` type for IRC `ProtocolErrors`.
pub type Result<T> = ::std::result::Result<T, ProtocolError>;

/// An IRC protocol error.
#[derive(Debug, Fail)]
pub enum ProtocolError {
    /// An internal I/O error.
    #[fail(display = "an io error occurred")]
    Io(#[cause] IoError),

    /// Error for invalid messages.
    #[fail(display = "invalid message: {}", string)]
    InvalidMessage {
        /// The string that failed to parse.
        string: String,
        /// The detailed message parsing error.
        #[cause]
        cause: MessageParseError,
    },
}

impl From<IoError> for ProtocolError {
    fn from(e: IoError) -> ProtocolError {
        ProtocolError::Io(e)
    }
}

/// Errors that occur when parsing messages.
#[derive(Debug, Fail)]
pub enum MessageParseError {
    /// The message was empty.
    #[fail(display = "empty message")]
    EmptyMessage,

    /// The command was invalid (i.e. missing).
    #[fail(display = "invalid command")]
    InvalidCommand,

    /// The mode string was malformed.
    #[fail(display = "invalid mode string: {}", string)]
    InvalidModeString {
        /// The invalid mode string.
        string: String,
        /// The detailed mode parsing error.
        #[cause]
        cause: ModeParseError,
    },

    /// The subcommand used was invalid.
    #[fail(display = "invalid {} subcommand: {}", cmd, sub)]
    InvalidSubcommand {
        /// The command whose invalid subcommand was referenced.
        cmd: &'static str,
        /// The invalid subcommand.
        sub: String,
    },
}

/// Errors that occur while parsing mode strings.
#[derive(Debug, Fail)]
pub enum ModeParseError {
    /// Invalid modifier used in a mode string (only + and - are valid).
    #[fail(display = "invalid mode modifier: {}", modifier)]
    InvalidModeModifier {
        /// The invalid mode modifier.
        modifier: char,
    },

    /// Missing modifier used in a mode string.
    #[fail(display = "missing mode modifier")]
    MissingModeModifier,
}
