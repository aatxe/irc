//! Enumeration of all available client commands.
#![stable]
use std::old_io::{InvalidInput, IoError, IoResult};
use std::str::FromStr;
use client::data::message::{Message, ToMessage};

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812). This
/// also includes commands from the
/// [capabilities extension](https://tools.ietf.org/html/draft-mitchell-irc-capabilities-01).
/// Additionally, this includes some common additional commands from popular IRCds.
#[stable]
#[derive(Debug, PartialEq)]
pub enum Command {
    // 3.1 Connection Registration
    /// PASS :password
    #[stable]
    PASS(String),
    /// NICK :nickname
    #[stable]
    NICK(String),
    /// USER user mode * :realname
    #[stable]
    USER(String, String, String),
    /// OPER name :password
    #[stable]
    OPER(String, String),
    /// MODE nickname modes
    /// MODE channel modes [modeparams]
    #[stable]
    MODE(String, String, Option<String>),
    /// SERVICE nickname reserved distribution type reserved :info
    #[stable]
    SERVICE(String, String, String, String, String, String),
    /// QUIT :comment
    #[stable]
    QUIT(Option<String>),
    /// SQUIT server :comment
    #[stable]
    SQUIT(String, String),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys]
    #[stable]
    JOIN(String, Option<String>),
    /// PART chanlist :[comment]
    #[stable]
    PART(String, Option<String>),
    // MODE is already defined.
    // MODE(String, String, Option<String>),
    /// TOPIC channel :[topic]
    #[stable]
    TOPIC(String, Option<String>),
    /// NAMES [chanlist :[target]]
    #[stable]
    NAMES(Option<String>, Option<String>),
    /// LIST [chanlist :[target]]
    #[stable]
    LIST(Option<String>, Option<String>),
    /// INVITE nickname channel
    #[stable]
    INVITE(String, String),
    /// KICK chanlist userlist :[comment]
    #[stable]
    KICK(String, String, Option<String>),

    // 3.3 Sending messages
    /// PRIVMSG msgtarget :message
    #[stable]
    PRIVMSG(String, String),
    /// NOTICE msgtarget :message
    #[stable]
    NOTICE(String, String),

    // 3.4 Server queries and commands
    /// MOTD :[target]
    #[stable]
    MOTD(Option<String>),
    /// LUSERS [mask :[target]]
    #[stable]
    LUSERS(Option<String>, Option<String>),
    /// VERSION :[target]
    #[stable]
    VERSION(Option<String>),
    /// STATS [query :[target]]
    #[stable]
    STATS(Option<String>, Option<String>),
    /// LINKS [[remote server] server :mask]
    #[stable]
    LINKS(Option<String>, Option<String>),
    /// TIME :[target]
    #[stable]
    TIME(Option<String>),
    /// CONNECT target server port :[remote server]
    #[stable]
    CONNECT(String, String, Option<String>),
    /// TRACE :[target]
    #[stable]
    TRACE(Option<String>),
    /// ADMIN :[target]
    #[stable]
    ADMIN(Option<String>),
    /// INFO :[target]
    #[stable]
    INFO(Option<String>),

    // 3.5 Service Query and Commands
    /// SERVLIST [mask :[type]]
    #[stable]
    SERVLIST(Option<String>, Option<String>),
    /// SQUERY servicename text
    #[stable]
    SQUERY(String, String),

    // 3.6 User based queries
    /// WHO [mask ["o"]]
    #[stable]
    WHO(Option<String>, Option<bool>),
    /// WHOIS [target] masklist
    #[stable]
    WHOIS(Option<String>, String),
    /// WHOWAS nicklist [count :[target]]
    #[stable]
    WHOWAS(String, Option<String>, Option<String>),

    // 3.7 Miscellaneous messages
    /// KILL nickname :comment
    #[stable]
    KILL(String, String),
    /// PING server1 :[server2]
    #[stable]
    PING(String, Option<String>),
    /// PONG server :[server2]
    #[stable]
    PONG(String, Option<String>),
    /// ERROR :message
    #[stable]
    ERROR(String),


    // 4 Optional Features
    /// AWAY :[message]
    #[stable]
    AWAY(Option<String>),
    /// REHASH
    #[stable]
    REHASH,
    /// DIE
    #[stable]
    DIE,
    /// RESTART
    #[stable]
    RESTART,
    /// SUMMON user [target :[channel]]
    #[stable]
    SUMMON(String, Option<String>, Option<String>),
    /// USERS :[target]
    #[stable]
    USERS(Option<String>),
    /// WALLOPS :Text to be sent
    #[stable]
    WALLOPS(String),
    /// USERHOST space-separated nicklist
    #[stable]
    USERHOST(Vec<String>),
    /// ISON space-separated nicklist
    #[stable]
    ISON(Vec<String>),

    // Non-RFC commands from InspIRCd
    /// SAJOIN nickname channel
    #[stable]
    SAJOIN(String, String),
    /// SAMODE target modes [modeparams]
    #[stable]
    SAMODE(String, String, Option<String>),
    /// SANICK old nickname new nickname
    #[stable]
    SANICK(String, String),
    /// SAPART nickname :comment
    #[stable]
    SAPART(String, String),
    /// SAQUIT nickname :comment
    #[stable]
    SAQUIT(String, String),
    /// NICKSERV message
    #[stable]
    NICKSERV(String),
    /// CHANSERV message
    #[stable]
    CHANSERV(String),
    /// OPERSERV message
    #[stable]
    OPERSERV(String),
    /// BOTSERV message
    #[stable]
    BOTSERV(String),
    /// HOSTSERV message
    #[stable]
    HOSTSERV(String),
    /// MEMOSERV message
    #[stable]
    MEMOSERV(String),

    // Capabilities extension to IRCv3
    /// CAP [*] COMMAND [*] :[param]
    #[unstable = "Feature recently changed to hopefully be specification-compliant."]
    CAP(Option<String>, CapSubCommand, Option<String>, Option<String>),
}

impl ToMessage for Command {
    /// Converts a Command into a Message.
    fn to_message(&self) -> Message {
        match *self {
            Command::PASS(ref p) => Message::new(None, "PASS", None, Some(&p[..])),

            Command::NICK(ref n) => Message::new(None, "NICK", None, Some(&n[..])),

            Command::USER(ref u, ref m, ref r) => Message::new(None, "USER",
                                                               Some(vec![&u[..], &m[..], "*"]),
                                                               Some(&r[..])),

            Command::OPER(ref u, ref p) => Message::new(None, "OPER", Some(vec![&u[..]]),
                                                        Some(&p[..])),

            Command::MODE(ref t, ref m, Some(ref p)) =>
                Message::new(None, "MODE", Some(vec![&t[..], &m[..],&p[..]]), None),

            Command::MODE(ref t, ref m, None) => Message::new(None, "MODE",
                                                              Some(vec![&t[..], &m[..]]), None),

            Command::SERVICE(ref n, ref r, ref d, ref t, ref re, ref i) =>
                Message::new(None, "SERVICE", Some(vec![&n[..], &r[..], &d[..], &t[..], &re[..]]),
                             Some(i)),

            Command::QUIT(Some(ref m)) => Message::new(None, "QUIT", None, Some(&m[..])),

            Command::QUIT(None) => Message::new(None, "QUIT", None, None),

            Command::SQUIT(ref s, ref c) => Message::new(None, "SQUIT", Some(vec![&s[..]]),
                                                         Some(&c[..])),

            Command::JOIN(ref c, Some(ref k)) => Message::new(None, "JOIN",
                                                              Some(vec![&c[..], &k[..]]), None),

            Command::JOIN(ref c, None) => Message::new(None, "JOIN", Some(vec![&c[..]]), None),

            Command::PART(ref c, Some(ref m)) => Message::new(None, "PART", Some(vec![&c[..]]),
                                                              Some(&m[..])),

            Command::PART(ref c, None) => Message::new(None, "PART", Some(vec![&c[..]]), None),

            Command::TOPIC(ref c, Some(ref t)) => Message::new(None, "TOPIC", Some(vec![&c[..]]),
                                                               Some(&t[..])),

            Command::TOPIC(ref c, None) => Message::new(None, "TOPIC", Some(vec![&c[..]]), None),

            Command::NAMES(Some(ref c), Some(ref t)) =>
                Message::new(None, "NAMES", Some(vec![&c[..]]), Some(&t[..])),

            Command::NAMES(Some(ref c), None) => Message::new(None, "NAMES", Some(vec![&c[..]]),
                                                              None),

            Command::NAMES(None, _) => Message::new(None, "NAMES", None, None),

            Command::LIST(Some(ref c), Some(ref t)) => Message::new(None, "LIST",
                                                                    Some(vec![&c[..]]), Some(t)),

            Command::LIST(Some(ref c), None) => Message::new(None, "LIST",
                                                             Some(vec![&c[..]]), None),

            Command::LIST(None, _) => Message::new(None, "LIST", None, None),

            Command::INVITE(ref n, ref c) => Message::new(None, "INVITE",
                                                          Some(vec![&n[..], &c[..]]), None),

            Command::KICK(ref c, ref n, Some(ref r)) => Message::new(None, "KICK",
                                                                     Some(vec![&c[..], &n[..]]),
                                                                     Some(r)),

            Command::KICK(ref c, ref n, None) => Message::new(None, "KICK",
                                                              Some(vec![&c[..], &n[..]]), None),

            Command::PRIVMSG(ref t, ref m) => Message::new(None, "PRIVMSG", Some(vec![&t[..]]),
                                                           Some(&m[..])),

            Command::NOTICE(ref t, ref m) => Message::new(None, "NOTICE", Some(vec![&t[..]]),
                                                          Some(&m[..])),

            Command::MOTD(Some(ref t)) => Message::new(None, "MOTD", None, Some(&t[..])),

            Command::MOTD(None) => Message::new(None, "MOTD", None, None),

            Command::LUSERS(Some(ref m), Some(ref t)) => Message::new(None, "LUSERS",
                                                                      Some(vec![&m[..]]), Some(t)),

            Command::LUSERS(Some(ref m), None) => Message::new(None, "LUSERS", Some(vec![&m[..]]),
                                                               None),

            Command::LUSERS(None, _) => Message::new(None, "LUSERS", None, None),

            Command::VERSION(Some(ref t)) => Message::new(None, "VERSION", None, Some(&t[..])),

            Command::VERSION(None) => Message::new(None, "VERSION", None, None),

            Command::STATS(Some(ref q), Some(ref t)) => Message::new(None, "STATS",
                                                                     Some(vec![&q[..]]), Some(t)),

            Command::STATS(Some(ref q), None) => Message::new(None, "STATS", Some(vec![&q[..]]),
                                                              None),

            Command::STATS(None, _) => Message::new(None, "STATS", None, None),

            Command::LINKS(Some(ref r), Some(ref s)) => Message::new(None, "LINKS",
                                                                     Some(vec![&r[..]]),
                                                                     Some(&s[..])),

            Command::LINKS(None, Some(ref s)) => Message::new(None, "LINKS", None, Some(&s[..])),

            Command::LINKS(_, None) => Message::new(None, "LINKS", None, None),

            Command::TIME(Some(ref t)) => Message::new(None, "TIME", None, Some(&t[..])),

            Command::TIME(None) => Message::new(None, "TIME", None, None),

            Command::CONNECT(ref t, ref p, Some(ref r)) => Message::new(None, "CONNECT",
                                                                        Some(vec![&t[..], &p[..]]),
                                                                        Some(&r[..])),

            Command::CONNECT(ref t, ref p, None) => Message::new(None, "CONNECT",
                                                                 Some(vec![&t[..], &p[..]]), None),

            Command::TRACE(Some(ref t)) => Message::new(None, "TRACE", None, Some(&t[..])),

            Command::TRACE(None) => Message::new(None, "TRACE", None, None),

            Command::ADMIN(Some(ref t)) => Message::new(None, "ADMIN", None, Some(&t[..])),

            Command::ADMIN(None) => Message::new(None, "ADMIN", None, None),

            Command::INFO(Some(ref t)) => Message::new(None, "INFO", None, Some(&t[..])),

            Command::INFO(None) => Message::new(None, "INFO", None, None),

            Command::SERVLIST(Some(ref m), Some(ref t)) => Message::new(None, "SERVLIST",
                                                                        Some(vec![&m[..]]),
                                                                        Some(&t[..])),

            Command::SERVLIST(Some(ref m), None) => Message::new(None, "SERVLIST",
                                                                 Some(vec![&m[..]]), None),

            Command::SERVLIST(None, _) => Message::new(None, "SERVLIST", None, None),

            Command::SQUERY(ref s, ref t) => Message::new(None, "SQUERY",
                                                          Some(vec![&s[..], &t[..]]), None),

            Command::WHO(Some(ref s), Some(true)) => Message::new(None, "WHO",
                                                                  Some(vec![&s[..], "o"]), None),

            Command::WHO(Some(ref s), _) => Message::new(None, "WHO", Some(vec![&s[..]]), None),

            Command::WHO(None, _) => Message::new(None, "WHO", None, None),

            Command::WHOIS(Some(ref t), ref m) => Message::new(None, "WHOIS",
                                                               Some(vec![&t[..], &m[..]]), None),

            Command::WHOIS(None, ref m) => Message::new(None, "WHOIS", Some(vec![&m[..]]), None),

            Command::WHOWAS(ref n, Some(ref c), Some(ref t)) =>
                Message::new(None, "WHOWAS", Some(vec![&n[..], &c[..]]), Some(t)),

            Command::WHOWAS(ref n, Some(ref c), None) => Message::new(None, "WHOWAS",
                                                                      Some(vec![&n[..], &c[..]]),
                                                                      None),

            Command::WHOWAS(ref n, None, _) => Message::new(None, "WHOWAS", Some(vec![&n[..]]),
                                                            None),

            Command::KILL(ref n, ref c) => Message::new(None, "KILL", Some(vec![&n[..]]),
                                                        Some(&c[..])),

            Command::PING(ref s, Some(ref t)) => Message::new(None, "PING", Some(vec![&s[..]]),
                                                              Some(&t[..])),

            Command::PING(ref s, None) => Message::new(None, "PING", None, Some(&s[..])),

            Command::PONG(ref s, Some(ref t)) => Message::new(None, "PONG", Some(vec![&s[..]]),
                                                              Some(&t[..])),

            Command::PONG(ref s, None) => Message::new(None, "PONG", None, Some(&s[..])),

            Command::ERROR(ref m) => Message::new(None, "ERROR", None, Some(&m[..])),

            Command::AWAY(Some(ref m)) => Message::new(None, "AWAY", None, Some(&m[..])),

            Command::AWAY(None) => Message::new(None, "AWAY", None, None),

            Command::REHASH => Message::new(None, "REHASH", None, None),

            Command::DIE => Message::new(None, "DIE", None, None),

            Command::RESTART => Message::new(None, "RESTART", None, None),

            Command::SUMMON(ref u, Some(ref t), Some(ref c)) =>
                Message::new(None, "SUMMON", Some(vec![&u[..], &t[..]]), Some(&c[..])),

            Command::SUMMON(ref u, Some(ref t), None) => Message::new(None, "SUMMON",
                                                                      Some(vec![&u[..], &t[..]]),
                                                                      None),

            Command::SUMMON(ref u, None, _) => Message::new(None, "SUMMON",
                                                            Some(vec![&u[..]]), None),

            Command::USERS(Some(ref t)) => Message::new(None, "USERS", None, Some(&t[..])),

            Command::USERS(None) => Message::new(None, "USERS", None, None),

            Command::WALLOPS(ref t) => Message::new(None, "WALLOPS", None, Some(&t[..])),


            Command::USERHOST(ref u) => Message::new(None, "USERHOST",
                                                     Some(u.iter().map(|s| &s[..]).collect()),
                                                     None),

            Command::ISON(ref u) => Message::new(None, "ISON",
                                                 Some(u.iter().map(|s| &s[..]).collect()),
                                                 None),

            Command::SAJOIN(ref n, ref c) => Message::new(None, "SAJOIN",
                                                          Some(vec![&n[..], &c[..]]), None),

            Command::SAMODE(ref t, ref m, Some(ref p)) =>
                Message::new(None, "SAMODE", Some(vec![&t[..], &m[..], &p[..]]), None),

            Command::SAMODE(ref t, ref m, None) => Message::new(None, "SAMODE", Some(vec![t, m]),
                                                                None),

            Command::SANICK(ref o, ref n) => Message::new(None, "SANICK",
                                                          Some(vec![&o[..], &n[..]]), None),

            Command::SAPART(ref c, ref r) => Message::new(None, "SAPART", Some(vec![&c[..]]),
                                                          Some(&r[..])),

            Command::SAQUIT(ref c, ref r) => Message::new(None, "SAQUIT", Some(vec![&c[..]]),
                                                          Some(&r[..])),

            Command::NICKSERV(ref m) => Message::new(None, "NICKSERV", Some(vec![&m[..]]), None),

            Command::CHANSERV(ref m) => Message::new(None, "CHANSERV", Some(vec![&m[..]]), None),

            Command::OPERSERV(ref m) => Message::new(None, "OPERSERV", Some(vec![&m[..]]), None),

            Command::BOTSERV(ref m) => Message::new(None, "BOTSERV", Some(vec![&m[..]]), None),

            Command::HOSTSERV(ref m) => Message::new(None, "HOSTSERV", Some(vec![&m[..]]), None),

            Command::MEMOSERV(ref m) => Message::new(None, "MEMOSERV", Some(vec![&m[..]]), None),

            Command::CAP(None, ref s, None, ref p) => Message::new(None, "CAP", 
                                                                   Some(vec![s.to_str()]),
                                                      p.as_ref().map(|m| m.as_slice())),
            Command::CAP(Some(ref k), ref s, None, ref  p) => Message::new(None, "CAP", 
                                                              Some(vec![&k, s.to_str()]),
                                                              p.as_ref().map(|m| m.as_slice())),
            Command::CAP(None, ref s, Some(ref c), ref p) => Message::new(None, "CAP", 
                                                             Some(vec![s.to_str(), &c]),
                                                             p.as_ref().map(|m| m.as_slice())),
            Command::CAP(Some(ref k), ref s, Some(ref c), ref p) => Message::new(None, "CAP", 
                                                             Some(vec![&k, s.to_str(), &c]),
                                                             p.as_ref().map(|m| m.as_slice())),
        }
    }
}

#[stable]
impl Command {
    /// Converts a Message into a Command.
    #[stable]
    pub fn from_message(m: &Message) -> IoResult<Command> {
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
                    Command::JOIN(suffix.clone(), None)
                } else if m.args.len() == 1 {
                    Command::JOIN(m.args[0].clone(), Some(suffix.clone()))
                } else {
                    return Err(invalid_input())
                },
                None => if m.args.len() == 1 {
                    Command::JOIN(m.args[0].clone(), None)
                } else if m.args.len() == 2 {
                    Command::JOIN(m.args[0].clone(), Some(m.args[1].clone()))
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
        } else {
            return Err(invalid_input())
        })
    }

    /// Converts a potential Message result into a potential Command result.
    #[unstable = "This feature is still relatively new."]
    pub fn from_message_io(m: IoResult<Message>) -> IoResult<Command> {
        m.and_then(|msg| Command::from_message(&msg))
    }
}

/// A list of all of the subcommands for the capabilities extension.
#[stable]
#[derive(Copy, Debug, PartialEq)]
pub enum CapSubCommand {
    /// Requests a list of the server's capabilities.
    #[stable]
    LS,
    /// Requests a list of the server's capabilities.
    #[stable]
    LIST,
    /// Requests specific capabilities blindly.
    #[stable]
    REQ,
    /// Acknowledges capabilities.
    #[stable]
    ACK,
    /// Does not acknowledge certain capabilities.
    #[stable]
    NAK,
    /// Requests that the server clears the capabilities of this client.
    #[stable]
    CLEAR,
    /// Ends the capability negotiation before registration.
    #[stable]
    END
}

#[stable]
impl CapSubCommand {
    /// Gets the string that corresponds to this subcommand.
    #[stable]
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
    type Err = &'static str;
    fn from_str(s: &str) -> Result<CapSubCommand, &'static str> {
        match s {
            "LS"    => Ok(CapSubCommand::LS),
            "LIST"  => Ok(CapSubCommand::LIST),
            "REQ"   => Ok(CapSubCommand::REQ),
            "ACK"   => Ok(CapSubCommand::ACK),
            "NAK"   => Ok(CapSubCommand::NAK),
            "CLEAR" => Ok(CapSubCommand::CLEAR),
            "END"   => Ok(CapSubCommand::END),
            _       => Err("Failed to parse CAP subcommand."),
        }
    }
}

/// Produces an invalid_input IoError.
fn invalid_input() -> IoError {
    IoError {
        kind: InvalidInput,
        desc: "Failed to parse malformed message as command.",
        detail: None
    }
}
