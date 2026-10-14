use std::str::FromStr;

/// Support for https://ircv3.net/specs/extensions/standard-replies
/// Implements the list of reply codes in the IRCv3 registry: https://ircv3.net/registry

trait FromCode {
    fn from_code(code: &str) -> Option<Self> where Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultilineCodes {
    MaxBytes,
    MaxLines,
    InvalidTarget,
    Invalid
}

impl FromCode for MultilineCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "MULTILINE_MAX_BYTES" => Self::MaxBytes,
            "MULTILINE_MAX_LINES" => Self::MaxLines,
            "MULTILINE_INVALID_TARGET" => Self::InvalidTarget,
            "MULTILINE_INVALID" => Self::Invalid,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatHistoryCodes {
    InvalidParams,
    InvalidTarget,
    MessageError,
    NeedMoreParams,
    UnknownCommand
}

impl FromCode for ChatHistoryCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "INVALID_PARAMS" => Self::InvalidParams,
            "INVALID_TARGET" => Self::InvalidTarget,
            "MESSAGE_ERROR" => Self::MessageError,
            "NEED_MORE_PARAMS" => Self::NeedMoreParams,
            "UNKNOWN_COMMAND" => Self::UnknownCommand,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinCodes {
    ChannelRenamed
}

impl FromCode for JoinCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "CHANNEL_RENAMED" => Self::ChannelRenamed,
            _ => return None
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NickCodes {
    Reserved
}

impl FromCode for NickCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "NICKNAME_RESERVED" => Self::Reserved,
            _ => return None
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactCodes {
    InvalidTarget,
    Forbidden,
    WindowExpired,
    UnknownMsgid,
}

impl FromCode for RedactCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "INVALID_TARGET" => Self::InvalidTarget,
            "REACT_FORBIDDEN" => Self::Forbidden,
            "REACT_WINDOW_EXPIRED" => Self::WindowExpired,
            "UNKNOWN_MSGID" => Self::UnknownMsgid,
            _ => return None
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterCodes {
    AccountExists,
    AccountNameMustBeNick,
    AlreadyAuthenticated,
    BadAccountName,
    CompleteConnectionRequired,
    InvalidEmail,
    NeedNick,
    TemporarilyUnavailable,
    UnacceptableEmail,
    UnacceptablePassword,
    WeakPassword
}

impl FromCode for RegisterCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "ACCOUNT_EXISTS" => Self::AccountExists,
            "ACCOUNT_NAME_MUST_BE_NICK" => Self::AccountNameMustBeNick,
            "ALREADY_AUTHENTICATED" => Self::AlreadyAuthenticated,
            "BAD_ACCOUNT_NAME" => Self::BadAccountName,
            "COMPLETE_CONNECTION_REQUIRED" => Self::CompleteConnectionRequired,
            "INVALID_EMAIL" => Self::InvalidEmail,
            "NEED_NICK" => Self::NeedNick,
            "TEMPORARILY_UNAVAILABLE" => Self::TemporarilyUnavailable,
            "UNACCEPTABLE_EMAIL" => Self::UnacceptableEmail,
            "UNACCEPTABLE_PASSWORD" => Self::UnacceptablePassword,
            "WEAK_PASSWORD" => Self::WeakPassword,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenameCodes {
    ChannelNameInUse,
    CannotRename
}

impl FromCode for RenameCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "CHANNEL_NAME_IN_USE" => Self::ChannelNameInUse,
            "CANNOT_RENAME" => Self::CannotRename,
            _ => return None,
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetNameCodes {
    CannotChangeRealname,
    InvalidRealname
}

impl FromCode for SetNameCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "CANNOT_CHANGE_REALNAME" => Self::CannotChangeRealname,
            "INVALID_REALNAME" => Self::InvalidRealname,
            _ => return None,
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyCodes {
    AlreadyAuthenticated,
    InvalidCode,
    CompleteConnectionRequired,
    TemporarilyUnavailable
}

impl FromCode for VerifyCodes {
    fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "ALREADY_AUTHENTICATED" => Self::AlreadyAuthenticated,
            "INVALID_CODE" => Self::InvalidCode,
            "COMPLETE_CONNECTION_REQUIRED" => Self::CompleteConnectionRequired,
            "TEMPORARILY_UNAVAILABLE" => Self::TemporarilyUnavailable,
            _ => return None,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StandardCodes {
    AccountRequired,
    InvalidUtf8,

    Multiline(MultilineCodes),
    ChatHistory(ChatHistoryCodes),
    Join(JoinCodes),
    Nick(NickCodes),
    Redact(RedactCodes),
    Register(RegisterCodes),
    Rename(RenameCodes),
    SetName(SetNameCodes),
    Verify(VerifyCodes),

    Custom(String)
}

impl StandardCodes {
    fn known_from_message(command: &str, code: &str) -> Option<Self> {
        Some(match command {
            "BATCH" => Self::Multiline(MultilineCodes::from_code(code)?),
            "CHATHISTORY" => Self::ChatHistory(ChatHistoryCodes::from_code(code)?),
            "JOIN" => Self::Join(JoinCodes::from_code(code)?),
            "NICK" => Self::Nick(NickCodes::from_code(code)?),
            "REDACT" => Self::Redact(RedactCodes::from_code(code)?),
            "REGISTER" => Self::Register(RegisterCodes::from_code(code)?),
            "RENAME" => Self::Rename(RenameCodes::from_code(code)?),
            "SETNAME" => Self::SetName(SetNameCodes::from_code(code)?),
            "VERIFY" => Self::Verify(VerifyCodes::from_code(code)?),
            _ => {
                match code {
                    "ACCOUNT_REQUIRED" => Self::AccountRequired,
                    "INVALID_UTF8" => Self::InvalidUtf8,
                    _ => Self::Custom(code.to_string())
                }
            }
        })
    }

    pub fn from_message(command: &str, code: &str) -> Self {
        Self::known_from_message(command, code).unwrap_or_else(|| Self::Custom(code.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandardTypes {
    Fail,
    Warn,
    Note
}

impl StandardTypes {
    pub fn is_standard_type(s: &str) -> bool {
        s.eq_ignore_ascii_case("FAIL") || s.eq_ignore_ascii_case("WARN") || s.eq_ignore_ascii_case("NOTE")
    }
}

impl FromStr for StandardTypes {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_uppercase().as_str() {
            "FAIL" => Ok(Self::Fail),
            "WARN" => Ok(Self::Warn),
            "NOTE" => Ok(Self::Note),
            _ => Err("Unexpected standard response type, neither fail, warn or note.")
        }
    }
}

#[cfg(test)]
mod test {
    use super::StandardCodes;

    #[test]
    fn parse_spec_example1() {
        let (command, code) = ("ACC", "REG_INVALID_CALLBACK");
        assert_eq!(StandardCodes::Custom("REG_INVALID_CALLBACK".to_string()), StandardCodes::from_message(command, code));
    }

    #[test]
    fn parse_spec_example2() {
        let (command, code) = ("BOX", "BOXES_INVALID");
        assert_eq!(StandardCodes::Custom("BOXES_INVALID".to_string()), StandardCodes::from_message(command, code));
    }

    #[test]
    fn parse_spec_example3() {
        let (command, code) = ("*", "ACCOUNT_REQUIRED_TO_CONNECT");
        assert_eq!(StandardCodes::Custom("ACCOUNT_REQUIRED_TO_CONNECT".to_string()), StandardCodes::from_message(command, code));
    }

    #[test]
    fn parse_batch_example() {
        let (command, code) = ("BATCH", "MULTILINE_MAX_BYTES");
        assert_eq!(StandardCodes::Multiline(crate::standard_reply::MultilineCodes::MaxBytes), StandardCodes::from_message(command, code));
    }
}
