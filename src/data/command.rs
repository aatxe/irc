//! Enumeration of all available client commands.
#![stable]
use std::io::{InvalidInput, IoError, IoResult};
use data::message::Message;

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812).
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
}

impl<'a> Command<'a> {
    /// Converts a Command into a Message.
    #[stable]
    pub fn to_message(self) -> Message {
        match self {
            PASS(p) => Message::new(None, "PASS", None, Some(p)),
            NICK(n) => Message::new(None, "NICK", None, Some(n)),
            USER(u, m, r) => Message::new(None, "USER", Some(vec![u, m, "*"]), Some(r)),
            OPER(u, p) => Message::new(None, "OPER", Some(vec![u]), Some(p)),
            MODE(t, m, Some(p)) => Message::new(None, "MODE", Some(vec![t, m, p]), None),
            MODE(t, m, None) => Message::new(None, "MODE", Some(vec![t, m]), None),
            SERVICE(n, r, d, t, re, i) => Message::new(None, "SERVICE",
                                          Some(vec![n, r, d, t, re]), Some(i)),
            QUIT(Some(m)) => Message::new(None, "QUIT", None, Some(m)),
            QUIT(None) => Message::new(None, "QUIT", None, None),
            SQUIT(s, c) => Message::new(None, "SQUIT", Some(vec![s]), Some(c)),
            JOIN(c, Some(k)) => Message::new(None, "JOIN", Some(vec![c, k]), None),
            JOIN(c, None) => Message::new(None, "JOIN", Some(vec![c]), None),
            PART(c, Some(m)) => Message::new(None, "PART", Some(vec![c]), Some(m)),
            PART(c, None) => Message::new(None, "PART", Some(vec![c]), None),
            TOPIC(c, Some(t)) => Message::new(None, "TOPIC", Some(vec![c]), Some(t)),
            TOPIC(c, None) => Message::new(None, "TOPIC", Some(vec![c]), None),
            NAMES(Some(c), Some(t)) => Message::new(None, "NAMES", Some(vec![c]), Some(t)),
            NAMES(Some(c), None) => Message::new(None, "NAMES", Some(vec![c]), None),
            NAMES(None, _) => Message::new(None, "NAMES", None, None),
            LIST(Some(c), Some(t)) => Message::new(None, "LIST", Some(vec![c]), Some(t)),
            LIST(Some(c), None) => Message::new(None, "LIST", Some(vec![c]), None),
            LIST(None, _) => Message::new(None, "LIST", None, None),
            INVITE(n, c) => Message::new(None, "INVITE", Some(vec![n, c]), None),
            KICK(c, n, Some(r)) => Message::new(None, "KICK", Some(vec![c, n]), Some(r)),
            KICK(c, n, None) => Message::new(None, "KICK", Some(vec![c, n]), None),
            PRIVMSG(t, m) => Message::new(None, "PRIVMSG", Some(vec![t]), Some(m)),
            NOTICE(t, m) => Message::new(None, "NOTICE", Some(vec![t]), Some(m)),
            MOTD(Some(t)) => Message::new(None, "MOTD", None, Some(t)),
            MOTD(None) => Message::new(None, "MOTD", None, None),
            LUSERS(Some(m), Some(t)) => Message::new(None, "LUSERS", Some(vec![m]), Some(t)),
            LUSERS(Some(m), None) => Message::new(None, "LUSERS", Some(vec![m]), None),
            LUSERS(None, _) => Message::new(None, "LUSERS", None, None),
            VERSION(Some(t)) => Message::new(None, "VERSION", None, Some(t)),
            VERSION(None) => Message::new(None, "VERSION", None, None),
            STATS(Some(q), Some(t)) => Message::new(None, "STATS", Some(vec![q]), Some(t)),
            STATS(Some(q), None) => Message::new(None, "STATS", Some(vec![q]), None),
            STATS(None, _) => Message::new(None, "STATS", None, None),
            LINKS(Some(r), Some(s)) => Message::new(None, "LINKS", Some(vec![r]), Some(s)),
            LINKS(None, Some(s)) => Message::new(None, "LINKS", None, Some(s)),
            LINKS(_, None) => Message::new(None, "LINKS", None, None),
            TIME(Some(t)) => Message::new(None, "TIME", None, Some(t)),
            TIME(None) => Message::new(None, "TIME", None, None),
            CONNECT(t, p, Some(r)) => Message::new(None, "CONNECT", Some(vec![t, p]), Some(r)),
            CONNECT(t, p, None) => Message::new(None, "CONNECT", Some(vec![t, p]), None),
            TRACE(Some(t)) => Message::new(None, "TRACE", None, Some(t)),
            TRACE(None) => Message::new(None, "TRACE", None, None),
            ADMIN(Some(t)) => Message::new(None, "ADMIN", None, Some(t)),
            ADMIN(None) => Message::new(None, "ADMIN", None, None),
            INFO(Some(t)) => Message::new(None, "INFO", None, Some(t)),
            INFO(None) => Message::new(None, "INFO", None, None),
            SERVLIST(Some(m), Some(t)) => Message::new(None, "SERVLIST", Some(vec![m]), Some(t)),
            SERVLIST(Some(m), None) => Message::new(None, "SERVLIST", Some(vec![m]), None),
            SERVLIST(None, _) => Message::new(None, "SERVLIST", None, None),
            SQUERY(s, t) => Message::new(None, "SQUERY", Some(vec![s, t]), None),
            WHO(Some(s), Some(true)) => Message::new(None, "WHO", Some(vec![s, "o"]), None),
            WHO(Some(s), _) => Message::new(None, "WHO", Some(vec![s]), None),
            WHO(None, _) => Message::new(None, "WHO", None, None),
            WHOIS(Some(t), m) => Message::new(None, "WHOIS", Some(vec![t, m]), None),
            WHOIS(None, m) => Message::new(None, "WHOIS", Some(vec![m]), None),
            WHOWAS(n, Some(c), Some(t)) => Message::new(None, "WHOWAS", Some(vec![n, c]), Some(t)),
            WHOWAS(n, Some(c), None) => Message::new(None, "WHOWAS", Some(vec![n, c]), None),
            WHOWAS(n, None, _) => Message::new(None, "WHOWAS", Some(vec![n]), None),
            KILL(n, c) => Message::new(None, "KILL", Some(vec![n]), Some(c)),
            PING(s, Some(t)) => Message::new(None, "PING", Some(vec![s]), Some(t)),
            PING(s, None) => Message::new(None, "PING", None, Some(s)),
            PONG(s, Some(t)) => Message::new(None, "PONG", Some(vec![s]), Some(t)),
            PONG(s, None) => Message::new(None, "PONG", None, Some(s)),
            ERROR(m) => Message::new(None, "ERROR", None, Some(m)),
            AWAY(Some(m)) => Message::new(None, "AWAY", None, Some(m)),
            AWAY(None) => Message::new(None, "AWAY", None, None),
            REHASH => Message::new(None, "REHASH", None, None),
            DIE => Message::new(None, "DIE", None, None),
            RESTART => Message::new(None, "RESTART", None, None),
            SUMMON(u, Some(t), Some(c)) => Message::new(None, "SUMMON", Some(vec![u, t]), Some(c)),
            SUMMON(u, Some(t), None) => Message::new(None, "SUMMON", Some(vec![u, t]), None),
            SUMMON(u, None, _) => Message::new(None, "SUMMON", Some(vec![u]), None),
            USERS(Some(t)) => Message::new(None, "USERS", None, Some(t)),
            USERS(None) => Message::new(None, "USERS", None, None),
            WALLOPS(t) => Message::new(None, "WALLOPS", None, Some(t)),
            USERHOST(u) => Message::new(None, "USERHOST", Some(u), None),
            ISON(u) => Message::new(None, "ISON", Some(u), None),
            SAJOIN(n, c) => Message::new(None, "SAJOIN", Some(vec![n, c]), None),
            SAMODE(t, m, Some(p)) => Message::new(None, "SAMODE", Some(vec![t, m, p]), None),
            SAMODE(t, m, None) => Message::new(None, "SAMODE", Some(vec![t, m]), None),
            SANICK(o, n) => Message::new(None, "SANICK", Some(vec![o, n]), None),
            SAPART(c, r) => Message::new(None, "SAPART", Some(vec![c]), Some(r)),
            SAQUIT(c, r) => Message::new(None, "SAQUIT", Some(vec![c]), Some(r)),
        }
    }

    /// Converts a Message into a Command.
    #[stable]
    pub fn from_message(m: &'a Message) -> IoResult<Command<'a>> {
        Ok(if let "PASS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    PASS(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    PASS(m.args[0][])
                }
            }
        } else if let "NICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    NICK(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    NICK(m.args[0][])
                }
            }
        } else if let "USER" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    USER(m.args[0][], m.args[1][], suffix[])
                },
                None => {
                    if m.args.len() != 3 { return Err(invalid_input()) }
                    USER(m.args[0][], m.args[1][], m.args[2][])
                }
            }
        } else if let "OPER" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    OPER(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    OPER(m.args[0][], m.args[1][])
                }
            }
        } else if let "MODE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    MODE(m.args[0][], m.args[1][], Some(suffix[]))
                }
                None => if m.args.len() == 3 {
                    MODE(m.args[0][], m.args[1][], Some(m.args[2][]))
                } else if m.args.len() == 2 {
                    MODE(m.args[0][], m.args[1][], None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SERVICE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 5 { return Err(invalid_input()) }
                    SERVICE(m.args[0][], m.args[1][], m.args[2][], m.args[3][], m.args[4][], suffix[])
                },
                None => {
                    if m.args.len() != 6 { return Err(invalid_input()) }
                    SERVICE(m.args[0][], m.args[1][], m.args[2][], m.args[3][], m.args[4][], m.args[5][])
                }
            }
        } else if let "QUIT" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => QUIT(Some(suffix[])),
                None => QUIT(None)
            }
        } else if let "SQUIT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SQUIT(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SQUIT(m.args[0][], m.args[1][])
                }
            }
        } else if let "JOIN" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    JOIN(suffix[], None)
                } else if m.args.len() == 1 {
                    JOIN(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    JOIN(m.args[0][], None)
                } else if m.args.len() == 2 {
                    JOIN(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PART" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    PART(suffix[], None)
                } else if m.args.len() == 1 {
                    PART(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    PART(m.args[0][], None)
                } else if m.args.len() == 2 {
                    PART(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TOPIC" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    TOPIC(suffix[], None)
                } else if m.args.len() == 1 {
                    TOPIC(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    TOPIC(m.args[0][], None)
                } else if m.args.len() == 2 {
                    TOPIC(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "NAMES" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    NAMES(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    NAMES(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    NAMES(None, None)
                } else if m.args.len() == 1 {
                    NAMES(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    NAMES(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LIST" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    LIST(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    LIST(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    LIST(None, None)
                } else if m.args.len() == 1 {
                    LIST(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    LIST(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "INVITE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    INVITE(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    INVITE(m.args[0][], m.args[1][])
                }
            }
        } else if let "KICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    KICK(m.args[0][], m.args[1][], Some(suffix[]))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    KICK(m.args[0][], m.args[1][], None)
                },
            }
        } else if let "PRIVMSG" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    PRIVMSG(m.args[0][], suffix[])
                },
                None => return Err(invalid_input())
            }
        } else if let "NOTICE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    NOTICE(m.args[0][], suffix[])
                },
                None => return Err(invalid_input())
            }
        } else if let "MOTD" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => MOTD(Some(suffix[])),
                None => MOTD(None)
            }
        } else if let "LUSERS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    LUSERS(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    LUSERS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    LUSERS(None, None)
                } else if m.args.len() == 1 {
                    LUSERS(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    LUSERS(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "VERSION" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => VERSION(Some(suffix[])),
                None => VERSION(None)
            }
        } else if let "STATS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    STATS(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    STATS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    STATS(None, None)
                } else if m.args.len() == 1 {
                    STATS(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    STATS(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LINKS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    LINKS(None, Some(suffix[]))
                } else if m.args.len() == 1 {
                    LINKS(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    LINKS(None, None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TIME" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => TIME(Some(suffix[])),
                None => TIME(None)
            }
        } else if let "CONNECT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    CONNECT(m.args[0][], m.args[1][], Some(suffix[]))
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    CONNECT(m.args[0][], m.args[1][], None)
                }
            }
        } else if let "TRACE" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => TRACE(Some(suffix[])),
                None => TRACE(None)
            }
        } else if let "ADMIN" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => ADMIN(Some(suffix[])),
                None => ADMIN(None)
            }
        } else if let "INFO" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            match m.suffix {
                Some(ref suffix) => INFO(Some(suffix[])),
                None => INFO(None)
            }
        } else if let "SERVLIST" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    SERVLIST(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    SERVLIST(Some(m.args[0][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    SERVLIST(None, None)
                } else if m.args.len() == 1 {
                    SERVLIST(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    SERVLIST(Some(m.args[0][]), Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SQUERY" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SQUERY(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SQUERY(m.args[0][], m.args[1][])
                }
            }
        } else if let "WHO" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    WHO(Some(suffix[]), None)
                } else if m.args.len() == 1 {
                    WHO(Some(m.args[0][]), Some(suffix[] == "o"))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 0 {
                    WHO(None, None)
                } else if m.args.len() == 1 {
                    WHO(Some(m.args[0][]), None)
                } else if m.args.len() == 2 {
                    WHO(Some(m.args[0][]), Some(m.args[1][] == "o"))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOIS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    WHOIS(None, suffix[])
                } else if m.args.len() == 1 {
                    WHOIS(Some(m.args[0][]), suffix[])
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    WHOIS(None, m.args[0][])
                } else if m.args.len() == 2 {
                    WHOIS(Some(m.args[0][]), m.args[1][])
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOWAS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    WHOWAS(suffix[], None, None)
                } else if m.args.len() == 1 {
                    WHOWAS(m.args[0][], None, Some(suffix[]))
                } else if m.args.len() == 2 {
                    WHOWAS(m.args[0][], Some(m.args[1][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    WHOWAS(m.args[0][], None, None)
                } else if m.args.len() == 2 {
                    WHOWAS(m.args[0][], None, Some(m.args[1][]))
                } else if m.args.len() == 3 {
                    WHOWAS(m.args[0][], Some(m.args[1][]), Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "KILL" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    KILL(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    KILL(m.args[0][], m.args[1][])
                }
            }
        } else if let "PING" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    PING(suffix[], None)
                } else if m.args.len() == 1 {
                    PING(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    PING(m.args[0][], None)
                } else if m.args.len() == 2 {
                    PING(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PONG" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    PONG(suffix[], None)
                } else if m.args.len() == 1 {
                    PONG(m.args[0][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    PONG(m.args[0][], None)
                } else if m.args.len() == 2 {
                    PONG(m.args[0][], Some(m.args[1][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "ERROR" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    ERROR(suffix[])
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "AWAY" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    AWAY(Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "REHASH" = m.command[] {
            if m.args.len() == 0 {
                REHASH
            } else {
                return Err(invalid_input())
            }
        } else if let "DIE" = m.command[] {
            if m.args.len() == 0 {
                DIE
            } else {
                return Err(invalid_input())
            }
        } else if let "RESTART" = m.command[] {
            if m.args.len() == 0 {
                RESTART
            } else {
                return Err(invalid_input())
            }
        } else if let "SUMMON" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 0 {
                    SUMMON(suffix[], None, None)
                } else if m.args.len() == 1 {
                    SUMMON(m.args[0][], Some(suffix[]), None)
                } else if m.args.len() == 2 {
                    SUMMON(m.args[0][], Some(m.args[1][]), Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    SUMMON(m.args[0][], None, None)
                } else if m.args.len() == 2 {
                    SUMMON(m.args[0][], Some(m.args[1][]), None)
                } else if m.args.len() == 3 {
                    SUMMON(m.args[0][], Some(m.args[1][]), Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "USERS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    USERS(Some(suffix[]))
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    USERS(Some(m.args[0][]))
                }
            }
        } else if let "WALLOPS" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 0 { return Err(invalid_input()) }
                    WALLOPS(suffix[])
                },
                None => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    WALLOPS(m.args[0][])
                }
            }
        } else if let "USERHOST" = m.command[] {
            if m.suffix.is_none() {
                USERHOST(m.args.iter().map(|s| s[]).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "ISON" = m.command[] {
            if m.suffix.is_none() {
                USERHOST(m.args.iter().map(|s| s[]).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "SAJOIN" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SAJOIN(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SAJOIN(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAMODE" = m.command[] {
            match m.suffix {
                Some(ref suffix) => if m.args.len() == 1 {
                    SAMODE(m.args[0][], suffix[], None)
                } else if m.args.len() == 2 {
                    SAMODE(m.args[0][], m.args[1][], Some(suffix[]))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 2 {
                    SAMODE(m.args[0][], m.args[1][], None)
                } else if m.args.len() == 3 {
                    SAMODE(m.args[0][], m.args[1][], Some(m.args[2][]))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SANICK" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SANICK(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SANICK(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAPART" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SAPART(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SAPART(m.args[0][], m.args[1][])
                }
            }
        } else if let "SAQUIT" = m.command[] {
            match m.suffix {
                Some(ref suffix) => {
                    if m.args.len() != 1 { return Err(invalid_input()) }
                    SAQUIT(m.args[0][], suffix[])
                },
                None => {
                    if m.args.len() != 2 { return Err(invalid_input()) }
                    SAQUIT(m.args[0][], m.args[1][])
                }
            }
        } else {
            return Err(invalid_input())
        })
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
