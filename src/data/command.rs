//! Enumeration of all available client commands
#![stable]
use std::io::{InvalidInput, IoError, IoResult};
use data::message::Message;

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812)
#[stable]
#[deriving(Show, PartialEq)]
pub enum Command {
    // 3.1 Connection Registration
    /// PASS password
    PASS(String),
    /// NICK nickname
    NICK(String),
    /// USER user mode * realname
    USER(String, String, String),
    /// OPER name password
    OPER(String, String),
    /// MODE nickname modes
    /// MODE channel modes [modeparams]
    MODE(String, String, Option<String>),
    /// SERVICE nickname reserved distribution type reserved info
    SERVICE(String, String, String, String, String, String),
    /// QUIT Quit Message
    QUIT(Option<String>),
    /// SQUIT server comment
    SQUIT(String, String),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys]
    JOIN(String, Option<String>),
    /// PART chanlist [Part Message]
    PART(String, Option<String>),
    // MODE is already defined.
    // MODE(String, String, Option<String>),
    /// TOPIC channel [topic]
    TOPIC(String, Option<String>),
    /// NAMES [chanlist [target]]
    NAMES(Option<String>, Option<String>),
    /// LIST [chanlist [target]]
    LIST(Option<String>, Option<String>),
    /// INVITE nickname channel
    INVITE(String, String),
    /// KICK chanlist userlist [comment]
    KICK(String, String, Option<String>),

    // 3.3 Sending messages
    /// PRIVMSG msgtarget text to be sent
    PRIVMSG(String, String),
    /// NOTICE msgtarget text
    NOTICE(String, String),

    // 3.4 Server queries and commands
    /// MOTD [target]
    MOTD(Option<String>),
    /// LUSERS [mask [target]]
    LUSERS(Option<String>, Option<String>),
    /// VERSION [target]
    VERSION(Option<String>),
    /// STATS [query [target]]
    STATS(Option<String>, Option<String>),
    /// LINKS [[remote server] server mask]
    LINKS(Option<String>, Option<String>),
    /// TIME [target]
    TIME(Option<String>),
    /// CONNECT target server port [remote server]
    CONNECT(String, String, Option<String>),
    /// TRACE [target]
    TRACE(Option<String>),
    /// ADMIN [target]
    ADMIN(Option<String>),
    /// INFO [target]
    INFO(Option<String>),

    // 3.5 Service Query and Commands
    /// SERVLIST [mask [type]]
    SERVLIST(Option<String>, Option<String>),
    /// SQUERY servicename text
    SQUERY(String, String),

    // 3.6 User based queries
    /// WHO [mask ["o"]]
    WHO(Option<String>, Option<bool>),
    /// WHOIS [target] masklist
    WHOIS(Option<String>, String),
    /// WHOWAS nicklist [count [target]]
    WHOWAS(String, Option<String>, Option<String>),

    // 3.7 Miscellaneous messages
    /// KILL nickname comment
    KILL(String, String),
    /// PING server1 [server2]
    PING(String, Option<String>),
    /// PONG server [server2]
    PONG(String, Option<String>),
    /// ERROR error message
    ERROR(String),


    // 4 Optional Features
    /// AWAY [text]
    AWAY(Option<String>),
    /// REHASH
    REHASH,
    /// DIE
    DIE,
    /// RESTART
    RESTART,
    /// SUMMON user [target [channel]]
    SUMMON(String, Option<String>, Option<String>),
    /// USERS [target]
    USERS(Option<String>),
    /// WALLOPS Text to be sent
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
    /// SAPART nickname reason
    SAPART(String, String),
    /// SAQUIT nickname reason
    SAQUIT(String, String),
}

impl Command {
    /// Converts a Command into a Message
    #[stable]
    pub fn to_message(self) -> Message {
        match self {
            PASS(p) => Message::new(None, "PASS", None, Some(p[])),
            NICK(n) => Message::new(None, "NICK", None, Some(n[])),
            USER(u, m, r) => Message::new(None, "USER", Some(vec![u[], m[], "*"]), Some(r[])),
            OPER(u, p) => Message::new(None, "OPER", Some(vec![u[]]), Some(p[])),
            MODE(t, m, Some(p)) => Message::new(None, "MODE", Some(vec![t[], m[], p[]]), None),
            MODE(t, m, None) => Message::new(None, "MODE", Some(vec![t[], m[]]), None),
            SERVICE(n, r, d, t, re, i) => Message::new(None, "SERVICE",
                                          Some(vec![n[], r[], d[], t[], re[]]), Some(i[])),
            QUIT(Some(m)) => Message::new(None, "QUIT", None, Some(m[])),
            QUIT(None) => Message::new(None, "QUIT", None, None),
            SQUIT(s, c) => Message::new(None, "SQUIT", Some(vec![s[]]), Some(c[])),
            JOIN(c, Some(k)) => Message::new(None, "JOIN", Some(vec![c[], k[]]), None),
            JOIN(c, None) => Message::new(None, "JOIN", Some(vec![c[]]), None),
            PART(c, Some(m)) => Message::new(None, "PART", Some(vec![c[]]), Some(m[])),
            PART(c, None) => Message::new(None, "PART", Some(vec![c[]]), None),
            TOPIC(c, Some(t)) => Message::new(None, "TOPIC", Some(vec![c[]]), Some(t[])),
            TOPIC(c, None) => Message::new(None, "TOPIC", Some(vec![c[]]), None),
            NAMES(Some(c), Some(t)) => Message::new(None, "NAMES", Some(vec![c[]]), Some(t[])),
            NAMES(Some(c), None) => Message::new(None, "NAMES", Some(vec![c[]]), None),
            NAMES(None, _) => Message::new(None, "NAMES", None, None),
            LIST(Some(c), Some(t)) => Message::new(None, "LIST", Some(vec![c[]]), Some(t[])),
            LIST(Some(c), None) => Message::new(None, "LIST", Some(vec![c[]]), None),
            LIST(None, _) => Message::new(None, "LIST", None, None),
            INVITE(n, c) => Message::new(None, "INVITE", Some(vec![n[], c[]]), None),
            KICK(c, n, Some(r)) => Message::new(None, "KICK", Some(vec![c[], n[]]), Some(r[])),
            KICK(c, n, None) => Message::new(None, "KICK", Some(vec![c[], n[]]), None),
            PRIVMSG(t, m) => Message::new(None, "PRIVMSG", Some(vec![t[]]), Some(m[])),
            NOTICE(t, m) => Message::new(None, "NOTICE", Some(vec![t[]]), Some(m[])),
            MOTD(Some(t)) => Message::new(None, "MOTD", None, Some(t[])),
            MOTD(None) => Message::new(None, "MOTD", None, None),
            LUSERS(Some(m), Some(t)) => Message::new(None, "LUSERS", Some(vec![m[]]), Some(t[])),
            LUSERS(Some(m), None) => Message::new(None, "LUSERS", Some(vec![m[]]), None),
            LUSERS(None, _) => Message::new(None, "LUSERS", None, None),
            VERSION(Some(t)) => Message::new(None, "VERSION", None, Some(t[])),
            VERSION(None) => Message::new(None, "VERSION", None, None),
            STATS(Some(q), Some(t)) => Message::new(None, "STATS", Some(vec![q[]]), Some(t[])),
            STATS(Some(q), None) => Message::new(None, "STATS", Some(vec![q[]]), None),
            STATS(None, _) => Message::new(None, "STATS", None, None),
            LINKS(Some(r), Some(s)) => Message::new(None, "LINKS", Some(vec![r[]]), Some(s[])),
            LINKS(None, Some(s)) => Message::new(None, "LINKS", None, Some(s[])),
            LINKS(_, None) => Message::new(None, "LINKS", None, None),
            TIME(Some(t)) => Message::new(None, "TIME", None, Some(t[])),
            TIME(None) => Message::new(None, "TIME", None, None),
            CONNECT(t, p, Some(r)) => Message::new(None, "CONNECT", Some(vec![t[], p[]]), Some(r[])),
            CONNECT(t, p, None) => Message::new(None, "CONNECT", Some(vec![t[], p[]]), None),
            TRACE(Some(t)) => Message::new(None, "TRACE", None, Some(t[])),
            TRACE(None) => Message::new(None, "TRACE", None, None),
            ADMIN(Some(t)) => Message::new(None, "ADMIN", None, Some(t[])),
            ADMIN(None) => Message::new(None, "ADMIN", None, None),
            INFO(Some(t)) => Message::new(None, "INFO", None, Some(t[])),
            INFO(None) => Message::new(None, "INFO", None, None),
            SERVLIST(Some(m), Some(t)) => Message::new(None, "SERVLIST", Some(vec![m[]]), Some(t[])),
            SERVLIST(Some(m), None) => Message::new(None, "SERVLIST", Some(vec![m[]]), None),
            SERVLIST(None, _) => Message::new(None, "SERVLIST", None, None),
            SQUERY(s, t) => Message::new(None, "SQUERY", Some(vec![s[], t[]]), None),
            WHO(Some(s), Some(true)) => Message::new(None, "WHO", Some(vec![s[], "o"]), None),
            WHO(Some(s), _) => Message::new(None, "WHO", Some(vec![s[]]), None),
            WHO(None, _) => Message::new(None, "WHO", None, None),
            WHOIS(Some(t), m) => Message::new(None, "WHOIS", Some(vec![t[], m[]]), None),
            WHOIS(None, m) => Message::new(None, "WHOIS", Some(vec![m[]]), None),
            WHOWAS(n, Some(c), Some(t)) => Message::new(None, "WHOWAS", Some(vec![n[], c[]]), Some(t[])),
            WHOWAS(n, Some(c), None) => Message::new(None, "WHOWAS", Some(vec![n[], c[]]), None),
            WHOWAS(n, None, _) => Message::new(None, "WHOWAS", Some(vec![n[]]), None),
            KILL(n, c) => Message::new(None, "KILL", Some(vec![n[]]), Some(c[])),
            PING(s, Some(t)) => Message::new(None, "PING", Some(vec![s[]]), Some(t[])),
            PING(s, None) => Message::new(None, "PING", None, Some(s[])),
            PONG(s, Some(t)) => Message::new(None, "PONG", Some(vec![s[]]), Some(t[])),
            PONG(s, None) => Message::new(None, "PONG", None, Some(s[])),
            ERROR(m) => Message::new(None, "ERROR", None, Some(m[])),
            AWAY(Some(m)) => Message::new(None, "AWAY", None, Some(m[])),
            AWAY(None) => Message::new(None, "AWAY", None, None),
            REHASH => Message::new(None, "REHASH", None, None),
            DIE => Message::new(None, "DIE", None, None),
            RESTART => Message::new(None, "RESTART", None, None),
            SUMMON(u, Some(t), Some(c)) => Message::new(None, "SUMMON", Some(vec![u[], t[]]), Some(c[])),
            SUMMON(u, Some(t), None) => Message::new(None, "SUMMON", Some(vec![u[], t[]]), None),
            SUMMON(u, None, _) => Message::new(None, "SUMMON", Some(vec![u[]]), None),
            USERS(Some(t)) => Message::new(None, "USERS", None, Some(t[])),
            USERS(None) => Message::new(None, "USERS", None, None),
            WALLOPS(t) => Message::new(None, "WALLOPS", None, Some(t[])),
            USERHOST(u) => Message::new(None, "USERHOST", Some(u.iter().map(|s| s[]).collect()), None),
            ISON(u) => Message::new(None, "ISON", Some(u.iter().map(|s| s[]).collect()), None),
            SAJOIN(n, c) => Message::new(None, "SAJOIN", Some(vec![n[], c[]]), None),
            SAMODE(t, m, Some(p)) => Message::new(None, "SAMODE", Some(vec![t[], m[], p[]]), None),
            SAMODE(t, m, None) => Message::new(None, "SAMODE", Some(vec![t[], m[]]), None),
            SANICK(o, n) => Message::new(None, "SANICK", Some(vec![o[], n[]]), None),
            SAPART(c, r) => Message::new(None, "SAPART", Some(vec![c[]]), Some(r[])),
            SAQUIT(c, r) => Message::new(None, "SAQUIT", Some(vec![c[]]), Some(r[])),
        }
    }

    /// Converts a Message into a Command
    #[stable]
    pub fn from_message(m: Message) -> IoResult<Command> {
        Ok(if let "PASS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 0 { return Err(invalid_input()) }
                PASS(m.suffix.unwrap())
            } else {
                if m.args.len() != 1 { return Err(invalid_input()) }
                PASS(m.args[0].clone())
            }
        } else if let "NICK" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 0 { return Err(invalid_input()) }
                NICK(m.suffix.unwrap())
            } else {
                if m.args.len() != 1 { return Err(invalid_input()) }
                NICK(m.args[0].clone())
            }
        } else if let "USER" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 2 { return Err(invalid_input()) }
                USER(m.args[0].clone(), m.args[1].clone(), m.suffix.unwrap().clone())
            } else {
                if m.args.len() != 3 { return Err(invalid_input()) }
                USER(m.args[0].clone(), m.args[1].clone(), m.args[2].clone())
            }
        } else if let "OPER" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                OPER(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                OPER(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "MODE" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 2 { return Err(invalid_input()) }
                MODE(m.args[0].clone(), m.args[1].clone(), Some(m.suffix.unwrap().clone()))
            } else if m.args.len() == 3 {
                MODE(m.args[0].clone(), m.args[1].clone(), Some(m.args[2].clone()))
            } else if m.args.len() == 2 {
                MODE(m.args[0].clone(), m.args[1].clone(), None)
            } else {
                return Err(invalid_input())
            }
        } else if let "SERVICE" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 5 { return Err(invalid_input()) }
                SERVICE(m.args[0].clone(), m.args[1].clone(), m.args[2].clone(), m.args[3].clone(),
                        m.args[4].clone(), m.suffix.unwrap().clone())
            } else {
                if m.args.len() != 6 { return Err(invalid_input()) }
                SERVICE(m.args[0].clone(), m.args[1].clone(), m.args[2].clone(), m.args[3].clone(),
                        m.args[4].clone(), m.args[5].clone())
            }
        } else if let "QUIT" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                QUIT(Some(m.suffix.unwrap().clone()))
            } else {
                QUIT(None)
            }
        } else if let "SQUIT" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SQUIT(m.args[0].clone(), m.suffix.unwrap().clone())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SQUIT(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "JOIN" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    JOIN(m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 1 {
                    JOIN(m.args[0].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                JOIN(m.args[0].clone(), None)
            } else if m.args.len() == 2 {
                JOIN(m.args[0].clone(), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "PART" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    PART(m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 1 {
                    PART(m.args[0].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                PART(m.args[0].clone(), None)
            } else if m.args.len() == 2 {
                PART(m.args[0].clone(), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "TOPIC" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    TOPIC(m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 1 {
                    TOPIC(m.args[0].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                TOPIC(m.args[0].clone(), None)
            } else if m.args.len() == 2 {
                TOPIC(m.args[0].clone(), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "NAMES" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    NAMES(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    NAMES(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                NAMES(None, None)
            } else if m.args.len() == 1 {
                NAMES(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                NAMES(Some(m.args[0].clone()), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "LIST" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    LIST(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    LIST(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                LIST(None, None)
            } else if m.args.len() == 1 {
                LIST(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                LIST(Some(m.args[0].clone()), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "INVITE" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                INVITE(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                INVITE(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "KICK" = m.command[] {
            if m.args.len() != 2 { return Err(invalid_input()) }
            KICK(m.args[0].clone(), m.args[1].clone(), m.suffix.clone())
        } else if let "PRIVMSG" = m.command[] {
            if !m.suffix.is_some() || m.args.len() != 1 { return Err(invalid_input()) }
            PRIVMSG(m.args[0].clone(), m.suffix.unwrap().clone())
        } else if let "NOTICE" = m.command[] {
            if !m.suffix.is_some() || m.args.len() != 1 { return Err(invalid_input()) }
            NOTICE(m.args[0].clone(), m.suffix.unwrap().clone())
        } else if let "MOTD" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                MOTD(Some(m.suffix.unwrap().clone()))
            } else {
                MOTD(None)
            }
        } else if let "LUSERS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    LUSERS(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    LUSERS(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                LUSERS(None, None)
            } else if m.args.len() == 1 {
                LUSERS(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                LUSERS(Some(m.args[0].clone()), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "VERSION" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                VERSION(Some(m.suffix.unwrap().clone()))
            } else {
                VERSION(None)
            }
        } else if let "STATS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    STATS(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    STATS(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                STATS(None, None)
            } else if m.args.len() == 1 {
                STATS(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                STATS(Some(m.args[0].clone()), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "LINKS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    LINKS(None, Some(m.suffix.unwrap().clone()))
                } else if m.args.len() == 1 {
                    LINKS(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                LINKS(None, None)
            } else {
                return Err(invalid_input())
            }
        } else if let "TIME" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                TIME(Some(m.suffix.unwrap().clone()))
            } else {
                TIME(None)
            }
        } else if let "CONNECT" = m.command[] {
            if m.args.len() != 2 { return Err(invalid_input()) }
            KICK(m.args[0].clone(), m.args[1].clone(), m.suffix.clone())
        } else if let "TRACE" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                TRACE(Some(m.suffix.unwrap().clone()))
            } else {
                TRACE(None)
            }
        } else if let "ADMIN" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                TIME(Some(m.suffix.unwrap().clone()))
            } else {
                TIME(None)
            }
        } else if let "INFO" = m.command[] {
            if m.args.len() != 0 { return Err(invalid_input()) }
            if m.suffix.is_some() {
                TIME(Some(m.suffix.unwrap().clone()))
            } else {
                TIME(None)
            }
        } else if let "SERVLIST" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    SERVLIST(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    SERVLIST(Some(m.args[0].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                SERVLIST(None, None)
            } else if m.args.len() == 1 {
                SERVLIST(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                SERVLIST(Some(m.args[0].clone()), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "SQUERY" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SQUERY(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SQUERY(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "WHO" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    WHO(Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 1 {
                    WHO(Some(m.args[0].clone()), Some(m.suffix.unwrap()[] == "o"))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 0 {
                WHO(None, None)
            } else if m.args.len() == 1 {
                WHO(Some(m.args[0].clone()), None)
            } else if m.args.len() == 2 {
                WHO(Some(m.args[0].clone()), Some(m.args[1][] == "o"))
            } else {
                return Err(invalid_input())
            }
        } else if let "WHOIS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    WHOIS(None, m.suffix.unwrap().clone())
                } else if m.args.len() == 1 {
                    WHOIS(Some(m.args[0].clone()), m.suffix.unwrap().clone())
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                WHOIS(None, m.args[0].clone())
            } else if m.args.len() == 2 {
                WHOIS(Some(m.args[0].clone()), m.args[1].clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "WHOWAS" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    WHOWAS(m.suffix.unwrap().clone(), None, None)
                } else if m.args.len() == 1 {
                    WHOWAS(m.args[0].clone(), None, Some(m.suffix.unwrap().clone()))
                } else if m.args.len() == 2 {
                    WHOWAS(m.args[0].clone(), Some(m.args[1].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                WHOWAS(m.args[0].clone(), None, None)
            } else if m.args.len() == 2 {
                WHOWAS(m.args[0].clone(), None, Some(m.args[1].clone()))
            } else if m.args.len() == 3 {
                WHOWAS(m.args[0].clone(), Some(m.args[1].clone()), Some(m.args[2].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "KILL" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                KILL(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                KILL(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "PING" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    PING(m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 1 {
                    PING(m.args[0].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                PING(m.args[0].clone(), None)
            } else if m.args.len() == 2 {
                PING(m.args[0].clone(), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "PONG" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    PONG(m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 1 {
                    PONG(m.args[0].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                PONG(m.args[0].clone(), None)
            } else if m.args.len() == 2 {
                PONG(m.args[0].clone(), Some(m.args[1].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "ERROR" = m.command[] {
            if m.suffix.is_some() && m.args.len() == 0 {
                ERROR(m.suffix.unwrap().clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "AWAY" = m.command[] {
            if m.args.len() == 0 {
                AWAY(m.suffix.clone())
            } else {
                return Err(invalid_input())
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
            if m.suffix.is_some() {
                if m.args.len() == 0 {
                    SUMMON(m.suffix.unwrap().clone(), None, None)
                } else if m.args.len() == 1 {
                    SUMMON(m.args[0].clone(), Some(m.suffix.unwrap().clone()), None)
                } else if m.args.len() == 2 {
                    SUMMON(m.args[0].clone(), Some(m.args[1].clone()), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 1 {
                SUMMON(m.args[0].clone(), None, None)
            } else if m.args.len() == 2 {
                SUMMON(m.args[0].clone(), Some(m.args[1].clone()), None)
            } else if m.args.len() == 3 {
                SUMMON(m.args[0].clone(), Some(m.args[1].clone()), Some(m.args[2].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "USERS" = m.command[] {
            if m.args.len() == 0 {
                USERS(m.suffix.clone())
            } else if m.args.len() == 1 {
                USERS(Some(m.args[0].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "WALLOPS" = m.command[] {
            if m.suffix.is_some() && m.args.len() == 0 {
                WALLOPS(m.suffix.unwrap().clone())
            } else if m.args.len() == 1 {
                WALLOPS(m.args[0].clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "USERHOST" = m.command[] {
            if m.suffix.is_none() {
                USERHOST(m.args.clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "ISON" = m.command[] {
            if m.suffix.is_none() {
                USERHOST(m.args.clone())
            } else {
                return Err(invalid_input())
            }
        } else if let "SAJOIN" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SAJOIN(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SAJOIN(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "SAMODE" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() == 1 {
                    SAMODE(m.args[0].clone(), m.suffix.unwrap().clone(), None)
                } else if m.args.len() == 2 {
                    SAMODE(m.args[0].clone(), m.args[1].clone(), Some(m.suffix.unwrap().clone()))
                } else {
                    return Err(invalid_input())
                }
            } else if m.args.len() == 2 {
                SAMODE(m.args[0].clone(), m.args[1].clone(), None)
            } else if m.args.len() == 3 {
                SAMODE(m.args[0].clone(), m.args[1].clone(), Some(m.args[2].clone()))
            } else {
                return Err(invalid_input())
            }
        } else if let "SANICK" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SANICK(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SANICK(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "SAPART" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SAPART(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SAPART(m.args[0].clone(), m.args[1].clone())
            }
        } else if let "SAQUIT" = m.command[] {
            if m.suffix.is_some() {
                if m.args.len() != 1 { return Err(invalid_input()) }
                SAQUIT(m.args[0].clone(), m.suffix.unwrap())
            } else {
                if m.args.len() != 2 { return Err(invalid_input()) }
                SAQUIT(m.args[0].clone(), m.args[1].clone())
            }
        } else {
            return Err(invalid_input())
        })
    }
}

/// Produces an invalid_input IoError
#[stable]
fn invalid_input() -> IoError {
    IoError {
        kind: InvalidInput,
        desc: "Failed to parse malformed message as command.",
        detail: None
    }
}
