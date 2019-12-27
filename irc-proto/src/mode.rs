//! A module defining an API for IRC user and channel modes.
use std::fmt;

use command::Command;
use error::MessageParseError;
use error::MessageParseError::InvalidModeString;
use error::ModeParseError::*;

/// A marker trait for different kinds of Modes.
pub trait ModeType: fmt::Display + fmt::Debug + Clone + PartialEq {
    /// Creates a command of this kind.
    fn mode(target: &str, modes: &[Mode<Self>]) -> Command;

    /// Returns true if this mode takes an argument, and false otherwise.
    fn takes_arg(&self) -> bool;
}

/// User modes for the MODE command.
#[derive(Clone, Debug, PartialEq)]
pub enum UserMode {
    /// a - user is flagged as away
    Away,
    /// i - marks a users as invisible
    Invisible,
    /// w - user receives wallops
    Wallops,
    /// r - restricted user connection
    Restricted,
    /// o - operator flag
    Oper,
    /// O - local operator flag
    LocalOper,
    /// s - marks a user for receipt of server notices
    ServerNotices,
    /// x - masked hostname
    MaskedHost,

    /// Any other unknown-to-the-crate mode.
    Unknown(char),
}

impl ModeType for UserMode {
    fn mode(target: &str, modes: &[Mode<Self>]) -> Command {
        Command::UserMODE(target.to_owned(), modes.to_owned())
    }

    fn takes_arg(&self) -> bool {
        false
    }
}

impl UserMode {
    fn from_char(c: char) -> UserMode {
        use self::UserMode::*;

        match c {
            'a' => Away,
            'i' => Invisible,
            'w' => Wallops,
            'r' => Restricted,
            'o' => Oper,
            'O' => LocalOper,
            's' => ServerNotices,
            'x' => MaskedHost,
            _ => Unknown(c),
        }
    }
}

impl fmt::Display for UserMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UserMode::*;

        write!(
            f,
            "{}",
            match *self {
                Away => 'a',
                Invisible => 'i',
                Wallops => 'w',
                Restricted => 'r',
                Oper => 'o',
                LocalOper => 'O',
                ServerNotices => 's',
                MaskedHost => 'x',
                Unknown(c) => c,
            }
        )
    }
}

/// Channel modes for the MODE command.
#[derive(Clone, Debug, PartialEq)]
pub enum ChannelMode {
    /// b - ban the user from joining or speaking in the channel
    Ban,
    /// e - exemptions from bans
    Exception,
    /// l - limit the maximum number of users in a channel
    Limit,
    /// i - channel becomes invite-only
    InviteOnly,
    /// I - exception to invite-only rule
    InviteException,
    /// k - specify channel key
    Key,
    /// m - channel is in moderated mode
    Moderated,
    /// r - entry for registered users only
    RegisteredOnly,
    /// s - channel is hidden from listings
    Secret,
    /// t - require permissions to edit topic
    ProtectedTopic,
    /// n - users must join channels to message them
    NoExternalMessages,

    /// q - user gets founder permission
    Founder,
    /// a - user gets admin or protected permission
    Admin,
    /// o - user gets oper permission
    Oper,
    /// h - user gets halfop permission
    Halfop,
    /// v - user gets voice permission
    Voice,

    /// Any other unknown-to-the-crate mode.
    Unknown(char),
}

impl ModeType for ChannelMode {
    fn mode(target: &str, modes: &[Mode<Self>]) -> Command {
        Command::ChannelMODE(target.to_owned(), modes.to_owned())
    }

    fn takes_arg(&self) -> bool {
        use self::ChannelMode::*;

        match *self {
            Ban | Exception | Limit | InviteException | Key | Founder | Admin | Oper | Halfop
            | Voice => true,
            _ => false,
        }
    }
}

impl ChannelMode {
    fn from_char(c: char) -> ChannelMode {
        use self::ChannelMode::*;

        match c {
            'b' => Ban,
            'e' => Exception,
            'l' => Limit,
            'i' => InviteOnly,
            'I' => InviteException,
            'k' => Key,
            'm' => Moderated,
            'r' => RegisteredOnly,
            's' => Secret,
            't' => ProtectedTopic,
            'n' => NoExternalMessages,
            'q' => Founder,
            'a' => Admin,
            'o' => Oper,
            'h' => Halfop,
            'v' => Voice,
            _ => Unknown(c),
        }
    }
}

impl fmt::Display for ChannelMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ChannelMode::*;

        write!(
            f,
            "{}",
            match *self {
                Ban => 'b',
                Exception => 'e',
                Limit => 'l',
                InviteOnly => 'i',
                InviteException => 'I',
                Key => 'k',
                Moderated => 'm',
                RegisteredOnly => 'r',
                Secret => 's',
                ProtectedTopic => 't',
                NoExternalMessages => 'n',
                Founder => 'q',
                Admin => 'a',
                Oper => 'o',
                Halfop => 'h',
                Voice => 'v',
                Unknown(c) => c,
            }
        )
    }
}

/// A mode argument for the MODE command.
#[derive(Clone, Debug, PartialEq)]
pub enum Mode<T>
where
    T: ModeType,
{
    /// Adding the specified mode, optionally with an argument.
    Plus(T, Option<String>),
    /// Removing the specified mode, optionally with an argument.
    Minus(T, Option<String>),
}

impl<T> Mode<T>
where
    T: ModeType,
{
    /// Creates a plus mode with an `&str` argument.
    pub fn plus(inner: T, arg: Option<&str>) -> Mode<T> {
        Mode::Plus(inner, arg.map(|s| s.to_owned()))
    }

    /// Creates a minus mode with an `&str` argument.
    pub fn minus(inner: T, arg: Option<&str>) -> Mode<T> {
        Mode::Minus(inner, arg.map(|s| s.to_owned()))
    }
}

impl<T> fmt::Display for Mode<T>
where
    T: ModeType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Mode::Plus(ref mode, Some(ref arg)) => write!(f, "{}{} {}", "+", mode, arg),
            Mode::Minus(ref mode, Some(ref arg)) => write!(f, "{}{} {}", "-", mode, arg),
            Mode::Plus(ref mode, None) => write!(f, "{}{}", "+", mode),
            Mode::Minus(ref mode, None) => write!(f, "{}{}", "-", mode),
        }
    }
}

enum PlusMinus {
    Plus,
    Minus,
}

// MODE user [modes]
impl Mode<UserMode> {
    // TODO: turning more edge cases into errors.
    /// Parses the specified mode string as user modes.
    pub fn as_user_modes(s: &str) -> Result<Vec<Mode<UserMode>>, MessageParseError> {
        use self::PlusMinus::*;

        let mut res = vec![];
        let mut pieces = s.split(' ');
        for term in pieces.clone() {
            if term.starts_with('+') || term.starts_with('-') {
                let _ = pieces.next();

                let mut chars = term.chars();
                let init = match chars.next() {
                    Some('+') => Plus,
                    Some('-') => Minus,
                    Some(c) => {
                        return Err(InvalidModeString {
                            string: s.to_owned(),
                            cause: InvalidModeModifier { modifier: c },
                        })
                    }
                    None => {
                        return Err(InvalidModeString {
                            string: s.to_owned(),
                            cause: MissingModeModifier,
                        })
                    }
                };

                for c in chars {
                    let mode = UserMode::from_char(c);
                    let arg = if mode.takes_arg() {
                        pieces.next()
                    } else {
                        None
                    };
                    res.push(match init {
                        Plus => Mode::Plus(mode, arg.map(|s| s.to_owned())),
                        Minus => Mode::Minus(mode, arg.map(|s| s.to_owned())),
                    })
                }
            }
        }

        Ok(res)
    }
}

// MODE channel [modes [modeparams]]
impl Mode<ChannelMode> {
    // TODO: turning more edge cases into errors.
    /// Parses the specified mode string as channel modes.
    pub fn as_channel_modes(s: &str) -> Result<Vec<Mode<ChannelMode>>, MessageParseError> {
        use self::PlusMinus::*;

        let mut res = vec![];
        let mut pieces = s.split(' ');
        for term in pieces.clone() {
            if term.starts_with('+') || term.starts_with('-') {
                let _ = pieces.next();

                let mut chars = term.chars();
                let init = match chars.next() {
                    Some('+') => Plus,
                    Some('-') => Minus,
                    Some(c) => {
                        return Err(InvalidModeString {
                            string: s.to_owned(),
                            cause: InvalidModeModifier { modifier: c },
                        })
                    }
                    None => {
                        return Err(InvalidModeString {
                            string: s.to_owned(),
                            cause: MissingModeModifier,
                        })
                    }
                };

                for c in chars {
                    let mode = ChannelMode::from_char(c);
                    let arg = if mode.takes_arg() {
                        pieces.next()
                    } else {
                        None
                    };
                    res.push(match init {
                        Plus => Mode::Plus(mode, arg.map(|s| s.to_owned())),
                        Minus => Mode::Minus(mode, arg.map(|s| s.to_owned())),
                    })
                }
            }
        }

        Ok(res)
    }
}
