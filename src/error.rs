//! Errors for `irc` crate using `error_chain`.

#![allow(missing_docs)]

error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Tls(::native_tls::Error);
    }

    errors {
        /// A parsing error for empty strings as messages.
        ParseEmpty {
            description("Cannot parse an empty string as a message.")
            display("Cannot parse an empty string as a message.")
        }

        /// A parsing error for invalid or missing commands in messages.
        InvalidCommand {
            description("Message contained a missing or invalid Command.")
            display("Message contained a missing or invalid Command.")
        }

        /// A parsing error for failures in subcommand parsing (e.g. CAP and metadata).
        SubCommandParsingFailed {
            description("Failed to parse an IRC subcommand.")
            display("Failed to parse an IRC subcommand.")
        }
    }
}
