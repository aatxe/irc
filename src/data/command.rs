//! Enumeration of all available client commands.
#![stable]
use std::io::{InvalidInput, IoError, IoResult};
use std::str::FromStr;
use data::message::{Message, ToMessage};

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812). This
/// also includes commands from the
/// [capabilities extension](https://tools.ietf.org/html/draft-mitchell-irc-capabilities-01).
/// Additionally, this includes some common additional commands from popular IRCds.
#[stable]
#[deriving(Show, PartialEq)]
pub enum Command<'a> {
    // 3.1 Connection Registration
    /// PASS password
    PASS(&'a str),
    /// NICK nickname
    NICK(&'a str),
    /// USER user mode * realname
    USER(&'a str, &'a str, &'a str),
    /// OPER name password
    OPER(&'a str, &'a str),
    /// MODE nickname modes
    /// MODE channel modes [modeparams]
    MODE(&'a str, &'a str, Option<&'a str>),
    /// SERVICE nickname reserved distribution type reserved info
    SERVICE(&'a str, &'a str, &'a str, &'a str, &'a str, &'a str),
    /// QUIT Quit Message
    QUIT(Option<&'a str>),
    /// SQUIT server comment
    SQUIT(&'a str, &'a str),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys]
    JOIN(&'a str, Option<&'a str>),
    /// PART chanlist [Part Message]
    PART(&'a str, Option<&'a str>),
    // MODE is already defined.
    // MODE(&'a str, &'a str, Option<&'a str>),
    /// TOPIC channel [topic]
    TOPIC(&'a str, Option<&'a str>),
    /// NAMES [chanlist [target]]
    NAMES(Option<&'a str>, Option<&'a str>),
    /// LIST [chanlist [target]]
    LIST(Option<&'a str>, Option<&'a str>),
    /// INVITE nickname channel
    INVITE(&'a str, &'a str),
    /// KICK chanlist userlist [comment]
    KICK(&'a str, &'a str, Option<&'a str>),

    // 3.3 Sending messages
    /// PRIVMSG msgtarget text to be sent
    PRIVMSG(&'a str, &'a str),
    /// NOTICE msgtarget text
    NOTICE(&'a str, &'a str),

    // 3.4 Server queries and commands
    /// MOTD [target]
    MOTD(Option<&'a str>),
    /// LUSERS [mask [target]]
    LUSERS(Option<&'a str>, Option<&'a str>),
    /// VERSION [target]
    VERSION(Option<&'a str>),
    /// STATS [query [target]]
    STATS(Option<&'a str>, Option<&'a str>),
    /// LINKS [[remote server] server mask]
    LINKS(Option<&'a str>, Option<&'a str>),
    /// TIME [target]
    TIME(Option<&'a str>),
    /// CONNECT target server port [remote server]
    CONNECT(&'a str, &'a str, Option<&'a str>),
    /// TRACE [target]
    TRACE(Option<&'a str>),
    /// ADMIN [target]
    ADMIN(Option<&'a str>),
    /// INFO [target]
    INFO(Option<&'a str>),

    // 3.5 Service Query and Commands
    /// SERVLIST [mask [type]]
    SERVLIST(Option<&'a str>, Option<&'a str>),
    /// SQUERY servicename text
    SQUERY(&'a str, &'a str),

    // 3.6 User based queries
    /// WHO [mask ["o"]]
    WHO(Option<&'a str>, Option<bool>),
    /// WHOIS [target] masklist
    WHOIS(Option<&'a str>, &'a str),
    /// WHOWAS nicklist [count [target]]
    WHOWAS(&'a str, Option<&'a str>, Option<&'a str>),

    // 3.7 Miscellaneous messages
    /// KILL nickname comment
    KILL(&'a str, &'a str),
    /// PING server1 [server2]
    PING(&'a str, Option<&'a str>),
    /// PONG server [server2]
    PONG(&'a str, Option<&'a str>),
    /// ERROR error message
    ERROR(&'a str),


    // 4 Optional Features
    /// AWAY [text]
    AWAY(Option<&'a str>),
    /// REHASH
    REHASH,
    /// DIE
    DIE,
    /// RESTART
    RESTART,
    /// SUMMON user [target [channel]]
    SUMMON(&'a str, Option<&'a str>, Option<&'a str>),
    /// USERS [target]
    USERS(Option<&'a str>),
    /// WALLOPS Text to be sent
    WALLOPS(&'a str),
    /// USERHOST space-separated nicklist
    USERHOST(Vec<&'a str>),
    /// ISON space-separated nicklist
    ISON(Vec<&'a str>),

    // Non-RFC commands from InspIRCd
    /// SAJOIN nickname channel
    SAJOIN(&'a str, &'a str),
    /// SAMODE target modes [modeparams]
    SAMODE(&'a str, &'a str, Option<&'a str>),
    /// SANICK old nickname new nickname
    SANICK(&'a str, &'a str),
    /// SAPART nickname reason
    SAPART(&'a str, &'a str),
    /// SAQUIT nickname reason
    SAQUIT(&'a str, &'a str),
    /// NICKSERV message
    NICKSERV(&'a str),
    /// CHANSERV message
    CHANSERV(&'a str),
    /// OPERSERV message
    OPERSERV(&'a str),
    /// BOTSERV message
    BOTSERV(&'a str),
    /// HOSTSERV message
    HOSTSERV(&'a str),
    /// MEMOSERV message
    MEMOSERV(&'a str),

    // Capabilities extension to IRCv3
    /// CAP COMMAND [param]
    CAP(CapSubCommand, Option<&'a str>),
}

impl<'a> ToMessage for Command<'a> {
    /// Converts a Command into a Message.
    #[stable]
    fn to_message(&self) -> Message {
        match *self {
            Command::PASS(p) => Message::new(None, "PASS", None, Some(p)),
            Command::NICK(n) => Message::new(None, "NICK", None, Some(n)),
            Command::USER(u, m, r) => Message::new(None, "USER", Some(vec![u, m, "*"]), Some(r)),
            Command::OPER(u, p) => Message::new(None, "OPER", Some(vec![u]), Some(p)),
            Command::MODE(t, m, Some(p)) => Message::new(None, "MODE", Some(vec![t, m, p]), None),
            Command::MODE(t, m, None) => Message::new(None, "MODE", Some(vec![t, m]), None),
            Command::SERVICE(n, r, d, t, re, i) => Message::new(None, "SERVICE", 
                                                                Some(vec![n, r, d, t, re]), 
                                                                Some(i)),
            Command::QUIT(Some(m)) => Message::new(None, "QUIT", None, Some(m)),
            Command::QUIT(None) => Message::new(None, "QUIT", None, None),
            Command::SQUIT(s, c) => Message::new(None, "SQUIT", Some(vec![s]), Some(c)),
            Command::JOIN(c, Some(k)) => Message::new(None, "JOIN", Some(vec![c, k]), None),
            Command::JOIN(c, None) => Message::new(None, "JOIN", Some(vec![c]), None),
            Command::PART(c, Some(m)) => Message::new(None, "PART", Some(vec![c]), Some(m)),
            Command::PART(c, None) => Message::new(None, "PART", Some(vec![c]), None),
            Command::TOPIC(c, Some(t)) => Message::new(None, "TOPIC", Some(vec![c]), Some(t)),
            Command::TOPIC(c, None) => Message::new(None, "TOPIC", Some(vec![c]), None),
            Command::NAMES(Some(c), Some(t)) => Message::new(None, "NAMES", Some(vec![c]), Some(t)),
            Command::NAMES(Some(c), None) => Message::new(None, "NAMES", Some(vec![c]), None),
            Command::NAMES(None, _) => Message::new(None, "NAMES", None, None),
            Command::LIST(Some(c), Some(t)) => Message::new(None, "LIST", Some(vec![c]), Some(t)),
            Command::LIST(Some(c), None) => Message::new(None, "LIST", Some(vec![c]), None),
            Command::LIST(None, _) => Message::new(None, "LIST", None, None),
            Command::INVITE(n, c) => Message::new(None, "INVITE", Some(vec![n, c]), None),
            Command::KICK(c, n, Some(r)) => Message::new(None, "KICK", Some(vec![c, n]), Some(r)),
            Command::KICK(c, n, None) => Message::new(None, "KICK", Some(vec![c, n]), None),
            Command::PRIVMSG(t, m) => Message::new(None, "PRIVMSG", Some(vec![t]), Some(m)),
            Command::NOTICE(t, m) => Message::new(None, "NOTICE", Some(vec![t]), Some(m)),
            Command::MOTD(Some(t)) => Message::new(None, "MOTD", None, Some(t)),
            Command::MOTD(None) => Message::new(None, "MOTD", None, None),
            Command::LUSERS(Some(m), Some(t)) => Message::new(None, "LUSERS", Some(vec![m]),
                                                              Some(t)),
            Command::LUSERS(Some(m), None) => Message::new(None, "LUSERS", Some(vec![m]), None),
            Command::LUSERS(None, _) => Message::new(None, "LUSERS", None, None),
            Command::VERSION(Some(t)) => Message::new(None, "VERSION", None, Some(t)),
            Command::VERSION(None) => Message::new(None, "VERSION", None, None),
            Command::STATS(Some(q), Some(t)) => Message::new(None, "STATS", Some(vec![q]), Some(t)),
            Command::STATS(Some(q), None) => Message::new(None, "STATS", Some(vec![q]), None),
            Command::STATS(None, _) => Message::new(None, "STATS", None, None),
            Command::LINKS(Some(r), Some(s)) => Message::new(None, "LINKS", Some(vec![r]), Some(s)),
            Command::LINKS(None, Some(s)) => Message::new(None, "LINKS", None, Some(s)),
            Command::LINKS(_, None) => Message::new(None, "LINKS", None, None),
            Command::TIME(Some(t)) => Message::new(None, "TIME", None, Some(t)),
            Command::TIME(None) => Message::new(None, "TIME", None, None),
            Command::CONNECT(t, p, Some(r)) => Message::new(None, "CONNECT", Some(vec![t, p]),
                                                            Some(r)),
            Command::CONNECT(t, p, None) => Message::new(None, "CONNECT", Some(vec![t, p]), None),
            Command::TRACE(Some(t)) => Message::new(None, "TRACE", None, Some(t)),
            Command::TRACE(None) => Message::new(None, "TRACE", None, None),
            Command::ADMIN(Some(t)) => Message::new(None, "ADMIN", None, Some(t)),
            Command::ADMIN(None) => Message::new(None, "ADMIN", None, None),
            Command::INFO(Some(t)) => Message::new(None, "INFO", None, Some(t)),
            Command::INFO(None) => Message::new(None, "INFO", None, None),
            Command::SERVLIST(Some(m), Some(t)) => Message::new(None, "SERVLIST", Some(vec![m]),
                                                                Some(t)),
            Command::SERVLIST(Some(m), None) => Message::new(None, "SERVLIST", Some(vec![m]), None),
            Command::SERVLIST(None, _) => Message::new(None, "SERVLIST", None, None),
            Command::SQUERY(s, t) => Message::new(None, "SQUERY", Some(vec![s, t]), None),
            Command::WHO(Some(s), Some(true)) => Message::new(None, "WHO", Some(vec![s, "o"]),
                                                              None),
            Command::WHO(Some(s), _) => Message::new(None, "WHO", Some(vec![s]), None),
            Command::WHO(None, _) => Message::new(None, "WHO", None, None),
            Command::WHOIS(Some(t), m) => Message::new(None, "WHOIS", Some(vec![t, m]), None),
            Command::WHOIS(None, m) => Message::new(None, "WHOIS", Some(vec![m]), None),
            Command::WHOWAS(n, Some(c), Some(t)) => Message::new(None, "WHOWAS", Some(vec![n, c]),
                                                                 Some(t)),
            Command::WHOWAS(n, Some(c), None) => Message::new(None, "WHOWAS", Some(vec![n, c]),
                                                              None),
            Command::WHOWAS(n, None, _) => Message::new(None, "WHOWAS", Some(vec![n]), None),
            Command::KILL(n, c) => Message::new(None, "KILL", Some(vec![n]), Some(c)),
            Command::PING(s, Some(t)) => Message::new(None, "PING", Some(vec![s]), Some(t)),
            Command::PING(s, None) => Message::new(None, "PING", None, Some(s)),
            Command::PONG(s, Some(t)) => Message::new(None, "PONG", Some(vec![s]), Some(t)),
            Command::PONG(s, None) => Message::new(None, "PONG", None, Some(s)),
            Command::ERROR(m) => Message::new(None, "ERROR", None, Some(m)),
            Command::AWAY(Some(m)) => Message::new(None, "AWAY", None, Some(m)),
            Command::AWAY(None) => Message::new(None, "AWAY", None, None),
            Command::REHASH => Message::new(None, "REHASH", None, None),
            Command::DIE => Message::new(None, "DIE", None, None),
            Command::RESTART => Message::new(None, "RESTART", None, None),
            Command::SUMMON(u, Some(t), Some(c)) => Message::new(None, "SUMMON", Some(vec![u, t]),
                                                                 Some(c)),
            Command::SUMMON(u, Some(t), None) => Message::new(None, "SUMMON", Some(vec![u, t]),
                                                              None),
            Command::SUMMON(u, None, _) => Message::new(None, "SUMMON", Some(vec![u]), None),
            Command::USERS(Some(t)) => Message::new(None, "USERS", None, Some(t)),
            Command::USERS(None) => Message::new(None, "USERS", None, None),
            Command::WALLOPS(t) => Message::new(None, "WALLOPS", None, Some(t)),
            Command::USERHOST(ref u) => Message::new(None, "USERHOST", Some(u.clone()), None),
            Command::ISON(ref u) => Message::new(None, "ISON", Some(u.clone()), None),
            Command::SAJOIN(n, c) => Message::new(None, "SAJOIN", Some(vec![n, c]), None),
            Command::SAMODE(t, m, Some(p)) => Message::new(None, "SAMODE", Some(vec![t, m, p]),
                                                           None),
            Command::SAMODE(t, m, None) => Message::new(None, "SAMODE", Some(vec![t, m]), None),
            Command::SANICK(o, n) => Message::new(None, "SANICK", Some(vec![o, n]), None),
            Command::SAPART(c, r) => Message::new(None, "SAPART", Some(vec![c]), Some(r)),
            Command::SAQUIT(c, r) => Message::new(None, "SAQUIT", Some(vec![c]), Some(r)),
            Command::NICKSERV(m) => Message::new(None, "NICKSERV", Some(vec![m]), None),
            Command::CHANSERV(m) => Message::new(None, "CHANSERV", Some(vec![m]), None),
            Command::OPERSERV(m) => Message::new(None, "OPERSERV", Some(vec![m]), None),
            Command::BOTSERV(m) => Message::new(None, "BOTSERV", Some(vec![m]), None),
            Command::HOSTSERV(m) => Message::new(None, "HOSTSERV", Some(vec![m]), None),
            Command::MEMOSERV(m) => Message::new(None, "MEMOSERV", Some(vec![m]), None),
            Command::CAP(s, p) => Message::new(None, "CAP", Some(vec![s.to_str()]), p),
        }
    }
}

impl<'a> Command<'a> {
    /// Converts a Message into a Command.
    #[stable]
    pub fn from_message(m: &'a Message) -> IoResult<Command<'a>> {
        Ok(if let "PASS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::PASS(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::PASS(m.args[0][])
                }
            }
        } else if let "NICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::NICK(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NICK(m.args[0][])
                }
            }
        } else if let "USER" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::USER(m.args[0][], m.args[1][], suffix[])
                },
                None => {
                    if m.args.len() != 3 { return Err(invalid_input()) }
                    Command::USER(m.args[0][], m.args[1][], m.args[2][])
                }
            }
        } else if let "OPER" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::OPER(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::OPER(m.args[0][], m.args[1][])
                }
            }
        } else if let "MODE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::MODE(m.args[0][], m.args[1][], Some(suffix[]))
                }
                None => if m.args.len() == 3 {
                    Command::MODE(m.args[0][], m.args[1][], Some(m.args[2][]))
                } else if m.args.len() == 2 {
                    Command::MODE(m.args[0][], m.args[1][], None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SERVICE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 5 { return Err(invalid_input()) }
                    Command::SERVICE(m.args[0][], m.args[1][], m.args[2][], m.args[3][],
                                     m.args[4][], suffix[])
                },
                None => {
                    if m.args.len() != 6 { return Err(invalid_input()) }
                    Command::SERVICE(m.args[0][], m.args[1][], m.args[2][], m.args[3][],
                                     m.args[4][], m.args[5][])
                }
            }
        } else if let "QUIT" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::QUIT(Some(suffix[])),
                None => Command::QUIT(None)
            }
        } else if let "SQUIT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUIT(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUIT(m.args[0][], m.args[1][])
                }
            }
        } else if let "JOIN" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::JOIN(suffix[], None)
                } else if m.args.len() == 1 {
                    Command::JOIN(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::JOIN(m.args[0][], None)
                } else if m.args.len() == 2 {
                    Command::JOIN(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PART" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PART(suffix[], None)
                } else if m.args.len() == 1 {
                    Command::PART(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PART(m.args[0][], None)
                } else if m.args.len() == 2 {
                    Command::PART(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TOPIC" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::TOPIC(suffix[], None)
                } else if m.args.len() == 1 {
                    Command::TOPIC(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::TOPIC(m.args[0][], None)
                } else if m.args.len() == 2 {
                    Command::TOPIC(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "NAMES" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::NAMES(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::NAMES(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::NAMES(None, None)
                } else if m.args.len() == 1 {
                    Command::NAMES(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::NAMES(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LIST" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LIST(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::LIST(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LIST(None, None)
                } else if m.args.len() == 1 {
                    Command::LIST(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::LIST(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "INVITE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::INVITE(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::INVITE(m.args[0][], m.args[1][])
                }
            }
        } else if let "KICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(m.args[0][], m.args[1][], Some(suffix[]))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(m.args[0][], m.args[1][], None)
                },
            }
        } else if let "PRIVMSG" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::PRIVMSG(m.args[0][], suffix[])
                },
                None => return Err(invalid_input())
            }
        } else if let "NOTICE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NOTICE(m.args[0][], suffix[])
                },
                None => return Err(invalid_input())
            }
        } else if let "MOTD" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::MOTD(Some(suffix[])),
                None => Command::MOTD(None)
            }
        } else if let "LUSERS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LUSERS(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::LUSERS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LUSERS(None, None)
                } else if m.args.len() == 1 {
                    Command::LUSERS(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::LUSERS(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "VERSION" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::VERSION(Some(suffix[])),
                None => Command::VERSION(None)
            }
        } else if let "STATS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::STATS(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::STATS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::STATS(None, None)
                } else if m.args.len() == 1 {
                    Command::STATS(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::STATS(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LINKS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::LINKS(None, Some(suffix[]))
                } else if m.args.len() == 1 {
                    Command::LINKS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::LINKS(None, None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TIME" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::TIME(Some(suffix[])),
                None => Command::TIME(None)
            }
        } else if let "CONNECT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(m.args[0][], m.args[1][], Some(suffix[]))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(m.args[0][], m.args[1][], None)
                }
            }
        } else if let "TRACE" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::TRACE(Some(suffix[])),
                None => Command::TRACE(None)
            }
        } else if let "ADMIN" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::ADMIN(Some(suffix[])),
                None => Command::ADMIN(None)
            }
        } else if let "INFO" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => Command::INFO(Some(suffix[])),
                None => Command::INFO(None)
            }
        } else if let "SERVLIST" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::SERVLIST(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::SERVLIST(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::SERVLIST(None, None)
                } else if m.args.len() == 1 {
                    Command::SERVLIST(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::SERVLIST(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SQUERY" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUERY(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUERY(m.args[0][], m.args[1][])
                }
            }
        } else if let "WHO" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHO(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    Command::WHO(Some(m.args[0][]), Some(suffix[] == "o"))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    Command::WHO(None, None)
                } else if m.args.len() == 1 {
                    Command::WHO(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    Command::WHO(Some(m.args[0][]), Some(m.args[1][] == "o"))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOIS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHOIS(None, suffix[])
                } else if m.args.len() == 1 {
                    Command::WHOIS(Some(m.args[0][]), suffix[])
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::WHOIS(None, m.args[0][])
                } else if m.args.len() == 2 {
                    Command::WHOIS(Some(m.args[0][]), m.args[1][])
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOWAS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::WHOWAS(suffix[], None, None)
                } else if m.args.len() == 1 {
                    Command::WHOWAS(m.args[0][], None, Some(suffix[]))
                } else if m.args.len() == 2 {
                    Command::WHOWAS(m.args[0][], Some(m.args[1][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::WHOWAS(m.args[0][], None, None)
                } else if m.args.len() == 2 {
                    Command::WHOWAS(m.args[0][], None, Some(m.args[1][]))
                } else if m.args.len() == 3 {
                    Command::WHOWAS(m.args[0][], Some(m.args[1][]), Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "KILL" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::KILL(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::KILL(m.args[0][], m.args[1][])
                }
            }
        } else if let "PING" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PING(suffix[], None)
                } else if m.args.len() == 1 {
                    Command::PING(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PING(m.args[0][], None)
                } else if m.args.len() == 2 {
                    Command::PING(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PONG" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::PONG(suffix[], None)
                } else if m.args.len() == 1 {
                    Command::PONG(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::PONG(m.args[0][], None)
                } else if m.args.len() == 2 {
                    Command::PONG(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "ERROR" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::ERROR(suffix[])
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "AWAY" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::AWAY(Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "REHASH" = m.command[] {
            if m.args.len() == 0 {
                Command::REHASH
            } else {
                return Err(invalid_input())
            }
        } else if let "DIE" = m.command[] {
            if m.args.len() == 0 {
                Command::DIE
            } else {
                return Err(invalid_input())
            }
        } else if let "RESTART" = m.command[] {
            if m.args.len() == 0 {
                Command::RESTART
            } else {
                return Err(invalid_input())
            }
        } else if let "SUMMON" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    Command::SUMMON(suffix[], None, None)
                } else if m.args.len() == 1 {
                    Command::SUMMON(m.args[0][], Some(suffix[]), None)
                } else if m.args.len() == 2 {
                    Command::SUMMON(m.args[0][], Some(m.args[1][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::SUMMON(m.args[0][], None, None)
                } else if m.args.len() == 2 {
                    Command::SUMMON(m.args[0][], Some(m.args[1][]), None)
                } else if m.args.len() == 3 {
                    Command::SUMMON(m.args[0][], Some(m.args[1][]), Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "USERS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::USERS(Some(suffix[]))
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::USERS(Some(m.args[0][]))
                }
            }
        } else if let "WALLOPS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::WALLOPS(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::WALLOPS(m.args[0][])
                }
            }
        } else if let "USERHOST" = m.command[] {
            if m.suffix.is_none() {
                Command::USERHOST(m.args.iter().map(|s| s[]).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "ISON" = m.command[] {
            if m.suffix.is_none() {
                Command::USERHOST(m.args.iter().map(|s| s[]).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "SAJOIN" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAJOIN(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAJOIN(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAMODE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 1 {
                    Command::SAMODE(m.args[0][], suffix[], None)
                } else if m.args.len() == 2 {
                    Command::SAMODE(m.args[0][], m.args[1][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 2 {
                    Command::SAMODE(m.args[0][], m.args[1][], None)
                } else if m.args.len() == 3 {
                    Command::SAMODE(m.args[0][], m.args[1][], Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SANICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SANICK(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SANICK(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAPART" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAPART(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAPART(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAQUIT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::SAQUIT(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    Command::SAQUIT(m.args[0][], m.args[1][])
                }
            }
        } else if let "NICKSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::NICKSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::NICKSERV(m.args[0][])
                }
            }
        } else if let "CHANSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::CHANSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::CHANSERV(m.args[0][])
                }
            }
        } else if let "OPERSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::OPERSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::OPERSERV(m.args[0][])
                }
            }
        } else if let "BOTSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::BOTSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::BOTSERV(m.args[0][])
                }
            }
        } else if let "HOSTSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::HOSTSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::HOSTSERV(m.args[0][])
                }
            }
        } else if let "MEMOSERV" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    Command::MEMOSERV(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    Command::MEMOSERV(m.args[0][])
                }
            }
        } else if let "CAP" = m.command[] {
            if m.args.len() != 1 { return Err(invalid_input()) }
            if let Some(cmd) = m.args[0].parse() {
                match m.suffix {
                    Some(ref suffix) => Command::CAP(cmd, Some(suffix[])),
                    None => Command::CAP(cmd, None),
                }
            } else {
                return Err(invalid_input())
            }
        } else {
            return Err(invalid_input())
        })
    }
}

/// A list of all of the subcommands for the capabilities extension.
#[stable]
#[deriving(Copy, Show, PartialEq)]
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
    /// Requests that the server clears the capabilities of this client.
    CLEAR,
    /// Ends the capability negotiation before registration.
    END
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
            &CapSubCommand::CLEAR => "CLEAR",
            &CapSubCommand::END   => "END",
        }
    }
}

impl FromStr for CapSubCommand {
    fn from_str(s: &str) -> Option<CapSubCommand> {
        match s {
            "LS"    => Some(CapSubCommand::LS),
            "LIST"  => Some(CapSubCommand::LIST),
            "REQ"   => Some(CapSubCommand::REQ),
            "ACK"   => Some(CapSubCommand::ACK),
            "NAK"   => Some(CapSubCommand::NAK),
            "CLEAR" => Some(CapSubCommand::CLEAR),
            "END"   => Some(CapSubCommand::END),
            _       => None,
        }
    }
}

/// Produces an invalid_input IoError.
#[stable]
fn invalid_input() -> IoError {
    IoError {
        kind: InvalidInput,
        desc: "Failed to parse malformed message as command.",
        detail: None
    }
}
