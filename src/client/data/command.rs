//! Enumeration of all available client commands.
use std::io::{Error, ErrorKind, Result};
use std::result::Result as StdResult;
use std::str::FromStr;
use client::data::Message;

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812). This
/// also includes commands from the
/// [capabilities extension](https://tools.ietf.org/html/draft-mitchell-irc-capabilities-01).
/// Additionally, this includes some common additional commands from popular IRCds.
#[derive(Debug, PartialEq)]
pub enum Command {
    // 3.1 Connection Registration
    /// PASS :password
    PASS(String),
    /// NICK :nickname
    NICK(String),
    /// USER user mode * :realname
    USER(String, String, String),
    /// OPER name :password
    OPER(String, String),
    /// MODE nickname modes
    /// MODE channel modes [modeparams]
    MODE(String, String, Option<String>),
    /// SERVICE nickname reserved distribution type reserved :info
    SERVICE(String, String, String, String, String, String),
    /// QUIT :comment
    QUIT(Option<String>),
    /// SQUIT server :comment
    SQUIT(String, String),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys]
    JOIN(String, Option<String>, Option<String>),
    /// PART chanlist :[comment]
    PART(String, Option<String>),
    // MODE is already defined.
    // MODE(String, String, Option<String>),
    /// TOPIC channel :[topic]
    TOPIC(String, Option<String>),
    /// NAMES [chanlist :[target]]
    NAMES(Option<String>, Option<String>),
    /// LIST [chanlist :[target]]
    LIST(Option<String>, Option<String>),
    /// INVITE nickname channel
    INVITE(String, String),
    /// KICK chanlist userlist :[comment]
    KICK(String, String, Option<String>),

    // 3.3 Sending messages
    /// PRIVMSG msgtarget :message
    PRIVMSG(String, String),
    /// NOTICE msgtarget :message
    NOTICE(String, String),

    // 3.4 Server queries and commands
    /// MOTD :[target]
    MOTD(Option<String>),
    /// LUSERS [mask :[target]]
    LUSERS(Option<String>, Option<String>),
    /// VERSION :[target]
    VERSION(Option<String>),
    /// STATS [query :[target]]
    STATS(Option<String>, Option<String>),
    /// LINKS [[remote server] server :mask]
    LINKS(Option<String>, Option<String>),
    /// TIME :[target]
    TIME(Option<String>),
    /// CONNECT target server port :[remote server]
    CONNECT(String, String, Option<String>),
    /// TRACE :[target]
    TRACE(Option<String>),
    /// ADMIN :[target]
    ADMIN(Option<String>),
    /// INFO :[target]
    INFO(Option<String>),

    // 3.5 Service Query and Commands
    /// SERVLIST [mask :[type]]
    SERVLIST(Option<String>, Option<String>),
    /// SQUERY servicename text
    SQUERY(String, String),

    // 3.6 User based queries
    /// WHO [mask ["o"]]
    WHO(Option<String>, Option<bool>),
    /// WHOIS [target] masklist
    WHOIS(Option<String>, String),
    /// WHOWAS nicklist [count :[target]]
    WHOWAS(String, Option<String>, Option<String>),

    // 3.7 Miscellaneous messages
    /// KILL nickname :comment
    KILL(String, String),
    /// PING server1 :[server2]
    PING(String, Option<String>),
    /// PONG server :[server2]
    PONG(String, Option<String>),
    /// ERROR :message
    ERROR(String),


    // 4 Optional Features
    /// AWAY :[message]
    AWAY(Option<String>),
    /// REHASH
    REHASH,
    /// DIE
    DIE,
    /// RESTART
    RESTART,
    /// SUMMON user [target :[channel]]
    SUMMON(String, Option<String>, Option<String>),
    /// USERS :[target]
    USERS(Option<String>),
    /// WALLOPS :Text to be sent
    WALLOPS(String),
    /// USERHOST space-separated nicklist
    USERHOST(Vec<String>),
    /// ISON space-separated nicklist
    ISON(Vec<String>),

    // Non-RFC commands from InspIRCd
    /// SAJOIN nickname channel
    SAJOIN(String, String),
    /// SAMODE target modes [modeparams]
    SAMODE(String, String, Option<String>),
    /// SANICK old nickname new nickname
    SANICK(String, String),
    /// SAPART nickname :comment
    SAPART(String, String),
    /// SAQUIT nickname :comment
    SAQUIT(String, String),
    /// NICKSERV message
    NICKSERV(String),
    /// CHANSERV message
    CHANSERV(String),
    /// OPERSERV message
    OPERSERV(String),
    /// BOTSERV message
    BOTSERV(String),
    /// HOSTSERV message
    HOSTSERV(String),
    /// MEMOSERV message
    MEMOSERV(String),

    // IRCv3 support
    /// CAP [*] COMMAND [*] :[param]
    CAP(Option<String>, CapSubCommand, Option<String>, Option<String>),

    // IRCv3.1 extensions
    /// ACCOUNT [account name]
    ACCOUNT(String),
    // AWAY is already defined as a send-only message.
    // AWAY(Option<String>),
    // JOIN is already defined.
    // JOIN(String, Option<String>, Option<String>),

    // IRCv3.2 extensions
    /// MONITOR command [nicklist]
    MONITOR(String, Option<String>)
}

impl Into<Message> for Command {
    /// Converts a Command into a Message.
    fn into(self) -> Message {
        match self {
            Command::PASS(p) => Message::from_owned(None, string("PASS"), None, Some(p)),
            Command::NICK(n) => Message::from_owned(None, string("NICK"), None, Some(n)),
            Command::USER(u, m, r) =>
                Message::from_owned(None, string("USER"), Some(vec![u, m, string("*")]), Some(r)),
            Command::OPER(u, p) =>
                Message::from_owned(None, string("OPER"), Some(vec![u]), Some(p)),
            Command::MODE(t, m, Some(p)) =>
                Message::from_owned(None, string("MODE"), Some(vec![t, m, p]), None),
            Command::MODE(t, m, None) =>
                Message::from_owned(None, string("MODE"), Some(vec![t, m]), None),
            Command::SERVICE(n, r, d, t, re, i) =>
                Message::from_owned(None, string("SERVICE"), Some(vec![n, r, d, t, re]), Some(i)),
            Command::QUIT(Some(m)) => Message::from_owned(None, string("QUIT"), None, Some(m)),
            Command::QUIT(None) => Message::from_owned(None, string("QUIT"), None, None),
            Command::SQUIT(s, c) =>
                Message::from_owned(None, string("SQUIT"), Some(vec![s]), Some(c)),
            Command::JOIN(c, Some(k), n) =>
                Message::from_owned(None, string("JOIN"), Some(vec![c, k]), n),
            Command::JOIN(c, None, n) =>
                Message::from_owned(None, string("JOIN"), Some(vec![c]), n),
            Command::PART(c, Some(m)) =>
                Message::from_owned(None, string("PART"), Some(vec![c]), Some(m)),
            Command::PART(c, None) =>
                Message::from_owned(None, string("PART"), Some(vec![c]), None),
            Command::TOPIC(c, Some(t)) =>
                Message::from_owned(None, string("TOPIC"), Some(vec![c]), Some(t)),
            Command::TOPIC(c, None) =>
                Message::from_owned(None, string("TOPIC"), Some(vec![c]), None),
            Command::NAMES(Some(c), Some(t)) =>
                Message::from_owned(None, string("NAMES"), Some(vec![c]), Some(t)),
            Command::NAMES(Some(c), None) =>
                Message::from_owned(None, string("NAMES"), Some(vec![c]), None),
            Command::NAMES(None, _) => Message::from_owned(None, string("NAMES"), None, None),
            Command::LIST(Some(c), Some(t)) =>
                Message::from_owned(None, string("LIST"), Some(vec![c]), Some(t)),
            Command::LIST(Some(c), None) =>
                Message::from_owned(None, string("LIST"), Some(vec![c]), None),
            Command::LIST(None, _) => Message::from_owned(None, string("LIST"), None, None),
            Command::INVITE(n, c) =>
                Message::from_owned(None, string("INVITE"), Some(vec![n, c]), None),
            Command::KICK(c, n, Some(r)) =>
                Message::from_owned(None, string("KICK"), Some(vec![c, n]), Some(r)),
            Command::KICK(c, n, None) =>
                Message::from_owned(None, string("KICK"), Some(vec![c, n]), None),
            Command::PRIVMSG(t, m) =>
                Message::from_owned(None, string("PRIVMSG"), Some(vec![t]), Some(m)),
            Command::NOTICE(t, m) =>
                Message::from_owned(None, string("NOTICE"), Some(vec![t]), Some(m)),
            Command::MOTD(Some(t)) => Message::from_owned(None, string("MOTD"), None, Some(t)),
            Command::MOTD(None) => Message::from_owned(None, string("MOTD"), None, None),
            Command::LUSERS(Some(m), Some(t)) =>
                Message::from_owned(None, string("LUSERS"), Some(vec![m]), Some(t)),
            Command::LUSERS(Some(m), None) =>
                Message::from_owned(None, string("LUSERS"), Some(vec![m]), None),
            Command::LUSERS(None, _) => Message::from_owned(None, string("LUSERS"), None, None),
            Command::VERSION(Some(t)) =>
                Message::from_owned(None, string("VERSION"), None, Some(t)),
            Command::VERSION(None) => Message::from_owned(None, string("VERSION"), None, None),
            Command::STATS(Some(q), Some(t)) =>
                Message::from_owned(None, string("STATS"), Some(vec![q]), Some(t)),
            Command::STATS(Some(q), None) =>
                Message::from_owned(None, string("STATS"), Some(vec![q]), None),
            Command::STATS(None, _) => Message::from_owned(None, string("STATS"), None, None),
            Command::LINKS(Some(r), Some(s)) =>
                Message::from_owned(None, string("LINKS"), Some(vec![r]), Some(s)),
            Command::LINKS(None, Some(s)) =>
                Message::from_owned(None, string("LINKS"), None, Some(s)),
            Command::LINKS(_, None) => Message::from_owned(None, string("LINKS"), None, None),
            Command::TIME(Some(t)) => Message::from_owned(None, string("TIME"), None, Some(t)),
            Command::TIME(None) => Message::from_owned(None, string("TIME"), None, None),
            Command::CONNECT(t, p, Some(r)) =>
                Message::from_owned(None, string("CONNECT"), Some(vec![t, p]), Some(r)),
            Command::CONNECT(t, p, None) =>
                Message::from_owned(None, string("CONNECT"), Some(vec![t, p]), None),
            Command::TRACE(Some(t)) => Message::from_owned(None, string("TRACE"), None, Some(t)),
            Command::TRACE(None) => Message::from_owned(None, string("TRACE"), None, None),
            Command::ADMIN(Some(t)) => Message::from_owned(None, string("ADMIN"), None, Some(t)),
            Command::ADMIN(None) => Message::from_owned(None, string("ADMIN"), None, None),
            Command::INFO(Some(t)) => Message::from_owned(None, string("INFO"), None, Some(t)),
            Command::INFO(None) => Message::from_owned(None, string("INFO"), None, None),
            Command::SERVLIST(Some(m), Some(t)) =>
                Message::from_owned(None, string("SERVLIST"), Some(vec![m]), Some(t)),
            Command::SERVLIST(Some(m), None) =>
                Message::from_owned(None, string("SERVLIST"), Some(vec![m]), None),
            Command::SERVLIST(None, _) =>
                Message::from_owned(None, string("SERVLIST"), None, None),
            Command::SQUERY(s, t) =>
                Message::from_owned(None, string("SQUERY"), Some(vec![s, t]), None),
            Command::WHO(Some(s), Some(true)) =>
                Message::from_owned(None, string("WHO"), Some(vec![s, string("o")]), None),
            Command::WHO(Some(s), _) =>
                Message::from_owned(None, string("WHO"), Some(vec![s]), None),
            Command::WHO(None, _) => Message::from_owned(None, string("WHO"), None, None),
            Command::WHOIS(Some(t), m) =>
                Message::from_owned(None, string("WHOIS"), Some(vec![t, m]), None),
            Command::WHOIS(None, m) =>
                Message::from_owned(None, string("WHOIS"), Some(vec![m]), None),
            Command::WHOWAS(n, Some(c), Some(t)) =>
                Message::from_owned(None, string("WHOWAS"), Some(vec![n, c]), Some(t)),
            Command::WHOWAS(n, Some(c), None) =>
                Message::from_owned(None, string("WHOWAS"), Some(vec![n, c]), None),
            Command::WHOWAS(n, None, _) =>
                Message::from_owned(None, string("WHOWAS"), Some(vec![n]), None),
            Command::KILL(n, c) =>
                Message::from_owned(None, string("KILL"), Some(vec![n]), Some(c)),
            Command::PING(s, Some(t)) =>
                Message::from_owned(None, string("PING"), Some(vec![s]), Some(t)),
            Command::PING(s, None) => Message::from_owned(None, string("PING"), None, Some(s)),
            Command::PONG(s, Some(t)) =>
                Message::from_owned(None, string("PONG"), Some(vec![s]), Some(t)),
            Command::PONG(s, None) => Message::from_owned(None, string("PONG"), None, Some(s)),
            Command::ERROR(m) => Message::from_owned(None, string("ERROR"), None, Some(m)),
            Command::AWAY(m) => Message::from_owned(None, string("AWAY"), None, m),
            Command::REHASH => Message::from_owned(None, string("REHASH"), None, None),
            Command::DIE => Message::from_owned(None, string("DIE"), None, None),
            Command::RESTART => Message::from_owned(None, string("RESTART"), None, None),
            Command::SUMMON(u, Some(t), Some(c)) =>
                Message::from_owned(None, string("SUMMON"), Some(vec![u, t]), Some(c)),
            Command::SUMMON(u, Some(t), None) =>
                Message::from_owned(None, string("SUMMON"), Some(vec![u, t]), None),
            Command::SUMMON(u, None, _) =>
                Message::from_owned(None, string("SUMMON"), Some(vec![u]), None),
            Command::USERS(Some(t)) => Message::from_owned(None, string("USERS"), None, Some(t)),
            Command::USERS(None) => Message::from_owned(None, string("USERS"), None, None),
            Command::WALLOPS(t) => Message::from_owned(None, string("WALLOPS"), None, Some(t)),
            Command::USERHOST(u) => Message::from_owned(None, string("USERHOST"), Some(u), None),
            Command::ISON(u) => Message::from_owned(None, string("ISON"), Some(u), None),

            Command::SAJOIN(n, c) =>
                Message::from_owned(None, string("SAJOIN"), Some(vec![n, c]), None),
            Command::SAMODE(t, m, Some(p)) =>
                Message::from_owned(None, string("SAMODE"), Some(vec![t, m, p]), None),
            Command::SAMODE(t, m, None) =>
                Message::from_owned(None, string("SAMODE"), Some(vec![t, m]), None),
            Command::SANICK(o, n) =>
                Message::from_owned(None, string("SANICK"), Some(vec![o, n]), None),
            Command::SAPART(c, r) =>
                Message::from_owned(None, string("SAPART"), Some(vec![c]), Some(r)),
            Command::SAQUIT(c, r) =>
                Message::from_owned(None, string("SAQUIT"), Some(vec![c]), Some(r)),

            Command::NICKSERV(m) =>
                Message::from_owned(None, string("NICKSERV"), Some(vec![m]), None),
            Command::CHANSERV(m) =>
                Message::from_owned(None, string("CHANSERV"), Some(vec![m]), None),
            Command::OPERSERV(m) =>
                Message::from_owned(None, string("OPERSERV"), Some(vec![m]), None),
            Command::BOTSERV(m) =>
                Message::from_owned(None, string("BOTSERV"), Some(vec![m]), None),
            Command::HOSTSERV(m) =>
                Message::from_owned(None, string("HOSTSERV"), Some(vec![m]), None),
            Command::MEMOSERV(m) =>
                Message::from_owned(None, string("MEMOSERV"), Some(vec![m]), None),

            Command::CAP(None, s, None, p) =>
                Message::from_owned(None, string("CAP"), Some(vec![s.string()]), p),
            Command::CAP(Some(k), s, None,  p) =>
                Message::from_owned(None, string("CAP"), Some(vec![k, s.string()]), p),
            Command::CAP(None, s, Some(c), p) =>
                Message::from_owned(None, string("CAP"), Some(vec![s.string(), c]), p),
            Command::CAP(Some(k), s, Some(c), p) =>
                Message::from_owned(None, string("CAP"), Some(vec![k, s.string(), c]), p),

            Command::ACCOUNT(a) =>
                Message::from_owned(None, string("ACCOUNT"), Some(vec![a]), None),

            Command::MONITOR(c, Some(t)) =>
                Message::from_owned(None, string("MONITOR"), Some(vec![c, t]), None),
            Command::MONITOR(c, None) =>
                Message::from_owned(None, string("MONITOR"), Some(vec![c]), None),
        }
    }
}

/// Converts a static str to an owned String.
fn string(s: &'static str) -> String {
    s.to_owned()
}

impl Command {
    /// Converts a Message into a Command.
    pub fn from_message(m: &Message) -> Result<Command> {
        Ok(if let "PASS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::PASS(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::PASS(m.args[0].clone())
                }
            }
        } else if let "NICK" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::NICK(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NICK(m.args[0].clone())
                }
            }
        } else if let "USER" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::USER(m.args[0].clone(), m.args[1].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 3 { return Err(invalid_input()) }
                    Command::USER(m.args[0].clone(), m.args[1].clone(), m.args[2].clone())
                }
            }
        } else if let "OPER" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::OPER(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::OPER(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "MODE" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::MODE(m.args[0].clone(), m.args[1].clone(), Some(suffix.clone()))
                }
                None => if m.args.len() == 3 {
                    Command::MODE(m.args[0].clone(), m.args[1].clone(), Some(m.args[2].clone()))
                } else if m.args.len() == 2 {
                    Command::MODE(m.args[0].clone(), m.args[1].clone(), None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SERVICE" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 5 { return Err(invalid_input()) }
                    Command::SERVICE(m.args[0].clone(), m.args[1].clone(), m.args[2].clone(),
                                     m.args[3].clone(), m.args[4].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 6 { return Err(invalid_input()) }
                    Command::SERVICE(m.args[0].clone(), m.args[1].clone(), m.args[2].clone(),
                                     m.args[3].clone(), m.args[4].clone(), m.args[5].clone())
                }
            }
        } else if let "QUIT" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::QUIT(Some(suffix.clone())),
                None => Command::QUIT(None)
            }
        } else if let "SQUIT" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUIT(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUIT(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "JOIN" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::JOIN(suffix.clone(), None, None)
                } else if m.args.len() == 1 {
                    Command::JOIN(m.args[0].clone(), Some(suffix.clone()), None)
                } else if m.args.len() == 2 {
                    Command::JOIN(m.args[0].clone(), Some(m.args[1].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::JOIN(m.args[0].clone(), None, None)
                } else if m.args.len() == 2 {
                    Command::JOIN(m.args[0].clone(), Some(m.args[1].clone()), None)
                } else if m.args.len() == 3 {
                    Command::JOIN(m.args[0].clone(), Some(m.args[1].clone()), 
                                  Some(m.args[2].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PART" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PART(suffix.clone(), None)
                } else if m.args.len() == 1 {
                    Command::PART(m.args[0].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PART(m.args[0].clone(), None)
                } else if m.args.len() == 2 {
                    Command::PART(m.args[0].clone(), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TOPIC" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::TOPIC(suffix.clone(), None)
                } else if m.args.len() == 1 {
                    Command::TOPIC(m.args[0].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::TOPIC(m.args[0].clone(), None)
                } else if m.args.len() == 2 {
                    Command::TOPIC(m.args[0].clone(), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "NAMES" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::NAMES(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::NAMES(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::NAMES(None, None)
                } else if m.args.len() == 1 {
                    Command::NAMES(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::NAMES(Some(m.args[0].clone()), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LIST" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LIST(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::LIST(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LIST(None, None)
                } else if m.args.len() == 1 {
                    Command::LIST(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::LIST(Some(m.args[0].clone()), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "INVITE" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::INVITE(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::INVITE(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "KICK" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(m.args[0].clone(), m.args[1].clone(), Some(suffix.clone()))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(m.args[0].clone(), m.args[1].clone(), None)
                },
            }
        } else if let "PRIVMSG" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::PRIVMSG(m.args[0].clone(), suffix.clone())
                },
                None => return Err(invalid_input())
            }
        } else if let "NOTICE" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NOTICE(m.args[0].clone(), suffix.clone())
                },
                None => return Err(invalid_input())
            }
        } else if let "MOTD" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::MOTD(Some(suffix.clone())),
                None => Command::MOTD(None)
            }
        } else if let "LUSERS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LUSERS(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::LUSERS(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LUSERS(None, None)
                } else if m.args.len() == 1 {
                    Command::LUSERS(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::LUSERS(Some(m.args[0].clone()), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "VERSION" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::VERSION(Some(suffix.clone())),
                None => Command::VERSION(None)
            }
        } else if let "STATS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::STATS(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::STATS(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::STATS(None, None)
                } else if m.args.len() == 1 {
                    Command::STATS(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::STATS(Some(m.args[0].clone()), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LINKS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LINKS(None, Some(suffix.clone()))
                } else if m.args.len() == 1 {
                    Command::LINKS(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LINKS(None, None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TIME" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::TIME(Some(suffix.clone())),
                None => Command::TIME(None)
            }
        } else if let "CONNECT" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(m.args[0].clone(), m.args[1].clone(), Some(suffix.clone()))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(m.args[0].clone(), m.args[1].clone(), None)
                }
            }
        } else if let "TRACE" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::TRACE(Some(suffix.clone())),
                None => Command::TRACE(None)
            }
        } else if let "ADMIN" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::ADMIN(Some(suffix.clone())),
                None => Command::ADMIN(None)
            }
        } else if let "INFO" = &m.command[..] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::INFO(Some(suffix.clone())),
                None => Command::INFO(None)
            }
        } else if let "SERVLIST" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::SERVLIST(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::SERVLIST(Some(m.args[0].clone()), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::SERVLIST(None, None)
                } else if m.args.len() == 1 {
                    Command::SERVLIST(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::SERVLIST(Some(m.args[0].clone()), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SQUERY" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUERY(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUERY(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "WHO" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHO(Some(suffix.clone()), None)
                } else if m.args.len() == 1 {
                    Command::WHO(Some(m.args[0].clone()), Some(&suffix[..] == "o"))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::WHO(None, None)
                } else if m.args.len() == 1 {
                    Command::WHO(Some(m.args[0].clone()), None)
                } else if m.args.len() == 2 {
                    Command::WHO(Some(m.args[0].clone()), Some(&m.args[1][..] == "o"))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOIS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHOIS(None, suffix.clone())
                } else if m.args.len() == 1 {
                    Command::WHOIS(Some(m.args[0].clone()), suffix.clone())
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::WHOIS(None, m.args[0].clone())
                } else if m.args.len() == 2 {
                    Command::WHOIS(Some(m.args[0].clone()), m.args[1].clone())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOWAS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHOWAS(suffix.clone(), None, None)
                } else if m.args.len() == 1 {
                    Command::WHOWAS(m.args[0].clone(), None, Some(suffix.clone()))
                } else if m.args.len() == 2 {
                    Command::WHOWAS(m.args[0].clone(), Some(m.args[1].clone()), 
                                    Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::WHOWAS(m.args[0].clone(), None, None)
                } else if m.args.len() == 2 {
                    Command::WHOWAS(m.args[0].clone(), None, Some(m.args[1].clone()))
                } else if m.args.len() == 3 {
                    Command::WHOWAS(m.args[0].clone(), Some(m.args[1].clone()), 
                                    Some(m.args[2].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "KILL" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::KILL(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KILL(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "PING" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PING(suffix.clone(), None)
                } else if m.args.len() == 1 {
                    Command::PING(m.args[0].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PING(m.args[0].clone(), None)
                } else if m.args.len() == 2 {
                    Command::PING(m.args[0].clone(), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PONG" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PONG(suffix.clone(), None)
                } else if m.args.len() == 1 {
                    Command::PONG(m.args[0].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PONG(m.args[0].clone(), None)
                } else if m.args.len() == 2 {
                    Command::PONG(m.args[0].clone(), Some(m.args[1].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "ERROR" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::ERROR(suffix.clone())
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "AWAY" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::AWAY(Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "REHASH" = &m.command[..] {
            if m.args.len() == 0 {
                Command::REHASH
            } else {
                return Err(invalid_input())
            }
        } else if let "DIE" = &m.command[..] {
            if m.args.len() == 0 {
                Command::DIE
            } else {
                return Err(invalid_input())
            }
        } else if let "RESTART" = &m.command[..] {
            if m.args.len() == 0 {
                Command::RESTART
            } else {
                return Err(invalid_input())
            }
        } else if let "SUMMON" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::SUMMON(suffix.clone(), None, None)
                } else if m.args.len() == 1 {
                    Command::SUMMON(m.args[0].clone(), Some(suffix.clone()), None)
                } else if m.args.len() == 2 {
                    Command::SUMMON(m.args[0].clone(), Some(m.args[1].clone()), 
                                    Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::SUMMON(m.args[0].clone(), None, None)
                } else if m.args.len() == 2 {
                    Command::SUMMON(m.args[0].clone(), Some(m.args[1].clone()), None)
                } else if m.args.len() == 3 {
                    Command::SUMMON(m.args[0].clone(), Some(m.args[1].clone()), 
                                    Some(m.args[2].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "USERS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::USERS(Some(suffix.clone()))
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::USERS(Some(m.args[0].clone()))
                }
            }
        } else if let "WALLOPS" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::WALLOPS(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::WALLOPS(m.args[0].clone())
                }
            }
        } else if let "USERHOST" = &m.command[..] {
            if m.suffix.is_none() {
                Command::USERHOST(m.args.clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "ISON" = &m.command[..] {
            if m.suffix.is_none() {
                Command::USERHOST(m.args.clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "SAJOIN" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAJOIN(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAJOIN(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "SAMODE" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 1 {
                    Command::SAMODE(m.args[0].clone(), suffix.clone(), None)
                } else if m.args.len() == 2 {
                    Command::SAMODE(m.args[0].clone(), m.args[1].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 2 {
                    Command::SAMODE(m.args[0].clone(), m.args[1].clone(), None)
                } else if m.args.len() == 3 {
                    Command::SAMODE(m.args[0].clone(), m.args[1].clone(), Some(m.args[2].clone()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SANICK" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SANICK(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SANICK(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "SAPART" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAPART(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAPART(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "SAQUIT" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAQUIT(m.args[0].clone(), suffix.clone())
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAQUIT(m.args[0].clone(), m.args[1].clone())
                }
            }
        } else if let "NICKSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::NICKSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NICKSERV(m.args[0].clone())
                }
            }
        } else if let "CHANSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::CHANSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::CHANSERV(m.args[0].clone())
                }
            }
        } else if let "OPERSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::OPERSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::OPERSERV(m.args[0].clone())
                }
            }
        } else if let "BOTSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::BOTSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::BOTSERV(m.args[0].clone())
                }
            }
        } else if let "HOSTSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::HOSTSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::HOSTSERV(m.args[0].clone())
                }
            }
        } else if let "MEMOSERV" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::MEMOSERV(suffix.clone())
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::MEMOSERV(m.args[0].clone())
                }
            }
        } else if let "CAP" = &m.command[..] {
            if m.args.len() == 1 {
                if let Ok(cmd) = m.args[0].parse() {
                    match m.suffix {
                        Some(ref suffix) => Command::CAP(None, cmd, None, Some(suffix.clone())),
                        None => Command::CAP(None, cmd, None, None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 2 {
                if let Ok(cmd) = m.args[0].parse() {
                    match m.suffix {
                        Some(ref suffix) => Command::CAP(None, cmd, Some(m.args[1].clone()), 
                                                         Some(suffix.clone())),
                        None => Command::CAP(None, cmd, Some(m.args[1].clone()), None),
                    }
                } else if let Ok(cmd) = m.args[1].parse() {
                    match m.suffix {
                        Some(ref suffix) => Command::CAP(Some(m.args[0].clone()), cmd, None,
                                                         Some(suffix.clone())),
                        None => Command::CAP(Some(m.args[0].clone()), cmd, None, None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 3 {
                if let Ok(cmd) = m.args[1].parse() {
                    match m.suffix {
                        Some(ref suffix) => Command::CAP(Some(m.args[0].clone()), cmd, 
                                                         Some(m.args[2].clone()),
                                                         Some(suffix.clone())),
                        None => Command::CAP(Some(m.args[0].clone()), cmd, Some(m.args[2].clone()),
                                             None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else {
                return Err(invalid_input())
            }
        } else if let "ACCOUNT" = &m.command[..] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::ACCOUNT(suffix.clone())
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::ACCOUNT(m.args[0].clone())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "MONITOR" = &m.command[..] {
            if m.args.len() == 1 {
                Command::MONITOR(m.args[0].clone(), m.suffix.clone())
            } else {
                return Err(invalid_input())
            }
        } else {
            return Err(invalid_input())
        })
    }

    /// Converts a potential Message result into a potential Command result.
    pub fn from_message_io(m: Result<Message>) -> Result<Command> {
        m.and_then(|msg| Command::from_message(&msg))
    }
}

/// A list of all of the subcommands for the capabilities extension.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CapSubCommand {
    /// Requests a list of the server's capabilities.
    LS,
    /// Requests a list of the server's capabilities.
    LIST,
    /// Requests specific capabilities blindly.
    REQ,
    /// Acknowledges capabilities.
    ACK,
    /// Does not acknowledge certain capabilities.
    NAK,
    /// Ends the capability negotiation before registration.
    END,
    /// Signals that new capabilities are now being offered.
    NEW,
    /// Signasl that the specified capabilities are cancelled and no longer available.
    DEL,
}

impl CapSubCommand {
    /// Gets the string that corresponds to this subcommand.
    pub fn to_str(&self) -> &str {
        match self {
            &CapSubCommand::LS    => "LS",
            &CapSubCommand::LIST  => "LIST",
            &CapSubCommand::REQ   => "REQ",
            &CapSubCommand::ACK   => "ACK",
            &CapSubCommand::NAK   => "NAK",
            &CapSubCommand::END   => "END",
            &CapSubCommand::NEW   => "NEW",
            &CapSubCommand::DEL   => "DEL",
        }
    }

    // This makes some earlier lines shorter.
    fn string(&self) -> String {
        self.to_str().to_owned()
    }
}

impl FromStr for CapSubCommand {
    type Err = &'static str;
    fn from_str(s: &str) -> StdResult<CapSubCommand, &'static str> {
        match s {
            "LS"    => Ok(CapSubCommand::LS),
            "LIST"  => Ok(CapSubCommand::LIST),
            "REQ"   => Ok(CapSubCommand::REQ),
            "ACK"   => Ok(CapSubCommand::ACK),
            "NAK"   => Ok(CapSubCommand::NAK),
            "END"   => Ok(CapSubCommand::END),
            "NEW"   => Ok(CapSubCommand::NEW),
            "DEL"   => Ok(CapSubCommand::DEL),
            _       => Err("Failed to parse CAP subcommand."),
        }
    }
}

/// Produces an invalid_input IoError.
fn invalid_input() -> Error {
    Error::new(ErrorKind::InvalidInput, "Failed to parse malformed message as command.")
}
