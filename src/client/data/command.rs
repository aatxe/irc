//! Enumeration of all available client commands.
use std::io::{Error, ErrorKind, Result};
use std::result::Result as StdResult;
use std::str::FromStr;
use client::data::Response;

/// List of all client commands as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812). This
/// also includes commands from the
/// [capabilities extension](https://tools.ietf.org/html/draft-mitchell-irc-capabilities-01).
/// Additionally, this includes some common additional commands from popular IRCds.
#[derive(Clone, Debug, PartialEq)]
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
    /// AUTHENTICATE data
    AUTHENTICATE(String),
    /// ACCOUNT [account name]
    ACCOUNT(String),
    // AWAY is already defined as a send-only message.
    // AWAY(Option<String>),
    // JOIN is already defined.
    // JOIN(String, Option<String>, Option<String>),

    // IRCv3.2 extensions
    /// METADATA target COMMAND [params] :[param]
    METADATA(String, Option<MetadataSubCommand>, Option<Vec<String>>, Option<String>),
    /// MONITOR command [nicklist]
    MONITOR(String, Option<String>),
    /// BATCH (+/-)reference-tag [type [params]]
    BATCH(String, Option<BatchSubCommand>, Option<Vec<String>>),
    /// CHGHOST user host
    CHGHOST(String, String),

    // Default option.
    /// An IRC response code with arguments and optional suffix.
    Response(Response, Vec<String>, Option<String>),
    /// A raw IRC command unknown to the crate.
    Raw(String, Vec<String>, Option<String>),
}

fn stringify(cmd: &str, args: Vec<&str>, suffix: Option<&str>) -> String {
    let args = args.join(" ");
    let sp = if args.len() > 0 { " " } else { "" };
    match suffix {
        Some(suffix) => format!("{}{}{} :{}", cmd, sp, args, suffix),
        None => format!("{}{}{}", cmd, sp, args),
    }

}

impl<'a> From<&'a Command> for String {
    fn from(cmd: &'a Command) -> String {
        match *cmd {
            Command::PASS(ref p) => stringify("PASS", vec![], Some(p)),
            Command::NICK(ref n) => stringify("NICK", vec![], Some(n)),
            Command::USER(ref u, ref m, ref r) =>
                stringify("USER", vec![u, m, "*"], Some(r)),
            Command::OPER(ref u, ref p) =>
                stringify("OPER", vec![u], Some(p)),
            Command::MODE(ref t, ref m, Some(ref p)) =>
                stringify("MODE", vec![t, m, p], None),
            Command::MODE(ref t, ref m, None) =>
                stringify("MODE", vec![t, m], None),
            Command::SERVICE(ref n, ref r, ref d, ref t, ref re, ref i) =>
                stringify("SERVICE", vec![n, r, d, t, re], Some(i)),
            Command::QUIT(Some(ref m)) => stringify("QUIT", vec![], Some(m)),
            Command::QUIT(None) => stringify("QUIT", vec![], None),
            Command::SQUIT(ref s, ref c) =>
                stringify("SQUIT", vec![s], Some(c)),
            Command::JOIN(ref c, Some(ref k), Some(ref n)) =>
                stringify("JOIN", vec![c, k], Some(n)),
            Command::JOIN(ref c, Some(ref k), None) =>
                stringify("JOIN", vec![c, k], None),
            Command::JOIN(ref c, None, Some(ref n)) =>
                stringify("JOIN", vec![c], Some(n)),
            Command::JOIN(ref c, None, None) =>
                stringify("JOIN", vec![c], None),
            Command::PART(ref c, Some(ref m)) =>
                stringify("PART", vec![c], Some(m)),
            Command::PART(ref c, None) =>
                stringify("PART", vec![c], None),
            Command::TOPIC(ref c, Some(ref t)) =>
                stringify("TOPIC", vec![c], Some(t)),
            Command::TOPIC(ref c, None) =>
                stringify("TOPIC", vec![c], None),
            Command::NAMES(Some(ref c), Some(ref t)) =>
                stringify("NAMES", vec![c], Some(t)),
            Command::NAMES(Some(ref c), None) =>
                stringify("NAMES", vec![c], None),
            Command::NAMES(None, _) => stringify("NAMES", vec![], None),
            Command::LIST(Some(ref c), Some(ref t)) =>
                stringify("LIST", vec![c], Some(t)),
            Command::LIST(Some(ref c), None) =>
                stringify("LIST", vec![c], None),
            Command::LIST(None, _) => stringify("LIST", vec![], None),
            Command::INVITE(ref n, ref c) =>
                stringify("INVITE", vec![n, c], None),
            Command::KICK(ref c, ref n, Some(ref r)) =>
                stringify("KICK", vec![c, n], Some(r)),
            Command::KICK(ref c, ref n, None) =>
                stringify("KICK", vec![c, n], None),
            Command::PRIVMSG(ref t, ref m) =>
                stringify("PRIVMSG", vec![t], Some(m)),
            Command::NOTICE(ref t, ref m) =>
                stringify("NOTICE", vec![t], Some(m)),
            Command::MOTD(Some(ref t)) => stringify("MOTD", vec![], Some(t)),
            Command::MOTD(None) => stringify("MOTD", vec![], None),
            Command::LUSERS(Some(ref m), Some(ref t)) =>
                stringify("LUSERS", vec![m], Some(t)),
            Command::LUSERS(Some(ref m), None) =>
                stringify("LUSERS", vec![m], None),
            Command::LUSERS(None, _) => stringify("LUSERS", vec![], None),
            Command::VERSION(Some(ref t)) =>
                stringify("VERSION", vec![], Some(t)),
            Command::VERSION(None) => stringify("VERSION", vec![], None),
            Command::STATS(Some(ref q), Some(ref t)) =>
                stringify("STATS", vec![q], Some(t)),
            Command::STATS(Some(ref q), None) =>
                stringify("STATS", vec![q], None),
            Command::STATS(None, _) => stringify("STATS", vec![], None),
            Command::LINKS(Some(ref r), Some(ref s)) =>
                stringify("LINKS", vec![r], Some(s)),
            Command::LINKS(None, Some(ref s)) =>
                stringify("LINKS", vec![], Some(s)),
            Command::LINKS(_, None) => stringify("LINKS", vec![], None),
            Command::TIME(Some(ref t)) => stringify("TIME", vec![], Some(t)),
            Command::TIME(None) => stringify("TIME", vec![], None),
            Command::CONNECT(ref t, ref p, Some(ref r)) =>
                stringify("CONNECT", vec![t, p], Some(r)),
            Command::CONNECT(ref t, ref p, None) =>
                stringify("CONNECT", vec![t, p], None),
            Command::TRACE(Some(ref t)) => stringify("TRACE", vec![], Some(t)),
            Command::TRACE(None) => stringify("TRACE", vec![], None),
            Command::ADMIN(Some(ref t)) => stringify("ADMIN", vec![], Some(t)),
            Command::ADMIN(None) => stringify("ADMIN", vec![], None),
            Command::INFO(Some(ref t)) => stringify("INFO", vec![], Some(t)),
            Command::INFO(None) => stringify("INFO", vec![], None),
            Command::SERVLIST(Some(ref m), Some(ref t)) =>
                stringify("SERVLIST", vec![m], Some(t)),
            Command::SERVLIST(Some(ref m), None) =>
                stringify("SERVLIST", vec![m], None),
            Command::SERVLIST(None, _) =>
                stringify("SERVLIST", vec![], None),
            Command::SQUERY(ref s, ref t) =>
                stringify("SQUERY", vec![s, t], None),
            Command::WHO(Some(ref s), Some(true)) =>
                stringify("WHO", vec![s, "o"], None),
            Command::WHO(Some(ref s), _) =>
                stringify("WHO", vec![s], None),
            Command::WHO(None, _) => stringify("WHO", vec![], None),
            Command::WHOIS(Some(ref t), ref m) =>
                stringify("WHOIS", vec![t, m], None),
            Command::WHOIS(None, ref m) =>
                stringify("WHOIS", vec![m], None),
            Command::WHOWAS(ref n, Some(ref c), Some(ref t)) =>
                stringify("WHOWAS", vec![n, c], Some(t)),
            Command::WHOWAS(ref n, Some(ref c), None) =>
                stringify("WHOWAS", vec![n, c], None),
            Command::WHOWAS(ref n, None, _) =>
                stringify("WHOWAS", vec![n], None),
            Command::KILL(ref n, ref c) =>
                stringify("KILL", vec![n], Some(c)),
            Command::PING(ref s, Some(ref t)) =>
                stringify("PING", vec![s], Some(t)),
            Command::PING(ref s, None) => stringify("PING", vec![], Some(s)),
            Command::PONG(ref s, Some(ref t)) =>
                stringify("PONG", vec![s], Some(t)),
            Command::PONG(ref s, None) => stringify("PONG", vec![], Some(s)),
            Command::ERROR(ref m) => stringify("ERROR", vec![], Some(m)),
            Command::AWAY(Some(ref m)) => stringify("AWAY", vec![], Some(m)),
            Command::AWAY(None) => stringify("AWAY", vec![], None),
            Command::REHASH => stringify("REHASH", vec![], None),
            Command::DIE => stringify("DIE", vec![], None),
            Command::RESTART => stringify("RESTART", vec![], None),
            Command::SUMMON(ref u, Some(ref t), Some(ref c)) =>
                stringify("SUMMON", vec![u, t], Some(c)),
            Command::SUMMON(ref u, Some(ref t), None) =>
                stringify("SUMMON", vec![u, t], None),
            Command::SUMMON(ref u, None, _) =>
                stringify("SUMMON", vec![u], None),
            Command::USERS(Some(ref t)) => stringify("USERS", vec![], Some(t)),
            Command::USERS(None) => stringify("USERS", vec![], None),
            Command::WALLOPS(ref t) => stringify("WALLOPS", vec![], Some(t)),
            Command::USERHOST(ref u) => stringify("USERHOST", u.iter().map(|s| &s[..]).collect(), None),
            Command::ISON(ref u) => stringify("ISON", u.iter().map(|s| &s[..]).collect(), None),

            Command::SAJOIN(ref n, ref c) =>
                stringify("SAJOIN", vec![n, c], None),
            Command::SAMODE(ref t, ref m, Some(ref p)) =>
                stringify("SAMODE", vec![t, m, p], None),
            Command::SAMODE(ref t, ref m, None) =>
                stringify("SAMODE", vec![t, m], None),
            Command::SANICK(ref o, ref n) =>
                stringify("SANICK", vec![o, n], None),
            Command::SAPART(ref c, ref r) =>
                stringify("SAPART", vec![c], Some(r)),
            Command::SAQUIT(ref c, ref r) =>
                stringify("SAQUIT", vec![c], Some(r)),

            Command::NICKSERV(ref m) =>
                stringify("NICKSERV", vec![m], None),
            Command::CHANSERV(ref m) =>
                stringify("CHANSERV", vec![m], None),
            Command::OPERSERV(ref m) =>
                stringify("OPERSERV", vec![m], None),
            Command::BOTSERV(ref m) =>
                stringify("BOTSERV", vec![m], None),
            Command::HOSTSERV(ref m) =>
                stringify("HOSTSERV", vec![m], None),
            Command::MEMOSERV(ref m) =>
                stringify("MEMOSERV", vec![m], None),

            Command::CAP(None, ref s, None, Some(ref p)) =>
                stringify("CAP", vec![s.to_str()], Some(p)),
            Command::CAP(None, ref s, None, None) =>
                stringify("CAP", vec![s.to_str()], None),
            Command::CAP(Some(ref k), ref s, None,  Some(ref p)) =>
                stringify("CAP", vec![k, s.to_str()], Some(p)),
            Command::CAP(Some(ref k), ref s, None,  None) =>
                stringify("CAP", vec![k, s.to_str()], None),
            Command::CAP(None, ref s, Some(ref c), Some(ref p)) =>
                stringify("CAP", vec![s.to_str(), c], Some(p)),
            Command::CAP(None, ref s, Some(ref c), None) =>
                stringify("CAP", vec![s.to_str(), c], None),
            Command::CAP(Some(ref k), ref s, Some(ref c), Some(ref p)) =>
                stringify("CAP", vec![k, s.to_str(), c], Some(p)),
            Command::CAP(Some(ref k), ref s, Some(ref c), None) =>
                stringify("CAP", vec![k, s.to_str(), c], None),

            Command::AUTHENTICATE(ref d) =>
                stringify("AUTHENTICATE", vec![d], None),
            Command::ACCOUNT(ref a) =>
                stringify("ACCOUNT", vec![a], None),

            Command::METADATA(ref t, Some(ref c), None, Some(ref p)) =>
                stringify("METADATA", vec![&t[..], c.to_str()], Some(p)),
            Command::METADATA(ref t, Some(ref c), None, None) =>
                stringify("METADATA", vec![&t[..], c.to_str()], None),

            Command::METADATA(ref t, Some(ref c), Some(ref a), Some(ref p)) => stringify(
                "METADATA",
                vec![t, &c.to_str().to_owned()].iter().map(|s| &s[..])
                                               .chain(a.iter().map(|s| &s[..])).collect(),
                Some(p)),
            Command::METADATA(ref t, Some(ref c), Some(ref a), None) =>
                stringify("METADATA",
                vec![t, &c.to_str().to_owned()].iter().map(|s| &s[..])
                                               .chain(a.iter().map(|s| &s[..])).collect(),
                None),
            Command::METADATA(ref t, None, None, Some(ref p)) =>
                stringify("METADATA", vec![t], Some(p)),
            Command::METADATA(ref t, None, None, None) =>
                stringify("METADATA", vec![t], None),
            Command::METADATA(ref t, None, Some(ref a), Some(ref p)) =>
                stringify("METADATA", vec![t].iter().map(|s| &s[..]).chain(a.iter().map(|s| &s[..])).collect(), Some(p)),
            Command::METADATA(ref t, None, Some(ref a), None) =>
                stringify("METADATA", vec![t].iter().map(|s| &s[..]).chain(a.iter().map(|s| &s[..])).collect(), None),
            Command::MONITOR(ref c, Some(ref t)) =>
                stringify("MONITOR", vec![c, t], None),
            Command::MONITOR(ref c, None) =>
                stringify("MONITOR", vec![c], None),
            Command::BATCH(ref t, Some(ref c), Some(ref a)) => stringify(
                "BATCH", vec![t, &c.to_str().to_owned()].iter().map(|s| &s[..]).chain(a.iter().map(|s| &s[..])).collect(),
                None
            ),
            Command::BATCH(ref t, Some(ref c), None) =>
                stringify("BATCH", vec![t, c.to_str()], None),
            Command::BATCH(ref t, None, Some(ref a)) =>
                stringify("BATCH",
                                    vec![t].iter().map(|s| &s[..]).chain(a.iter().map(|s| &s[..])).collect(), None),
            Command::BATCH(ref t, None, None) =>
                stringify("BATCH", vec![t], None),
            Command::CHGHOST(ref u, ref h) =>
                stringify("CHGHOST", vec![u, h], None),

            Command::Response(ref resp, ref a, Some(ref s)) =>
                stringify(&format!("{}", *resp as u16), a.iter().map(|s| &s[..]).collect(), Some(s)),
            Command::Response(ref resp, ref a, None) =>
                stringify(&format!("{}", *resp as u16), a.iter().map(|s| &s[..]).collect(), None),
            Command::Raw(ref c, ref a, Some(ref s)) =>
                stringify(c, a.iter().map(|s| &s[..]).collect(), Some(s)),
            Command::Raw(ref c, ref a, None) =>
                stringify(c, a.iter().map(|s| &s[..]).collect(), None),
        }
    }
}

impl Command {
    /// Constructs a new Command.
    pub fn new(cmd: &str, args: Vec<&str>, suffix: Option<&str>) -> Result<Command> {
        Ok(if let "PASS" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::PASS(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::PASS(args[0].to_owned())
                }
            }
        } else if let "NICK" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::NICK(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::NICK(args[0].to_owned())
                }
            }
        } else if let "USER" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::USER(args[0].to_owned(), args[1].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 3 { return Err(invalid_input()) }
                    Command::USER(args[0].to_owned(), args[1].to_owned(), args[2].to_owned())
                }
            }
        } else if let "OPER" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::OPER(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::OPER(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "MODE" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 2 {
                    Command::MODE(args[0].to_owned(), args[1].to_owned(), Some(suffix.to_owned()))
                } else if args.len() == 1 {
                    Command::MODE(args[0].to_owned(), suffix.to_owned(), None)
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 3 {
                    Command::MODE(args[0].to_owned(), args[1].to_owned(), Some(args[2].to_owned()))
                } else if args.len() == 2 {
                    Command::MODE(args[0].to_owned(), args[1].to_owned(), None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SERVICE" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 5 { return Err(invalid_input()) }
                    Command::SERVICE(args[0].to_owned(), args[1].to_owned(), args[2].to_owned(),
                                     args[3].to_owned(), args[4].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 6 { return Err(invalid_input()) }
                    Command::SERVICE(args[0].to_owned(), args[1].to_owned(), args[2].to_owned(),
                                     args[3].to_owned(), args[4].to_owned(), args[5].to_owned())
                }
            }
        } else if let "QUIT" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::QUIT(Some(suffix.to_owned())),
                None => Command::QUIT(None)
            }
        } else if let "SQUIT" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUIT(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUIT(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "JOIN" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::JOIN(suffix.to_owned(), None, None)
                } else if args.len() == 1 {
                    Command::JOIN(args[0].to_owned(), Some(suffix.to_owned()), None)
                } else if args.len() == 2 {
                    Command::JOIN(args[0].to_owned(), Some(args[1].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::JOIN(args[0].to_owned(), None, None)
                } else if args.len() == 2 {
                    Command::JOIN(args[0].to_owned(), Some(args[1].to_owned()), None)
                } else if args.len() == 3 {
                    Command::JOIN(args[0].to_owned(), Some(args[1].to_owned()),
                                  Some(args[2].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PART" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::PART(suffix.to_owned(), None)
                } else if args.len() == 1 {
                    Command::PART(args[0].to_owned(), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::PART(args[0].to_owned(), None)
                } else if args.len() == 2 {
                    Command::PART(args[0].to_owned(), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TOPIC" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::TOPIC(suffix.to_owned(), None)
                } else if args.len() == 1 {
                    Command::TOPIC(args[0].to_owned(), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::TOPIC(args[0].to_owned(), None)
                } else if args.len() == 2 {
                    Command::TOPIC(args[0].to_owned(), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "NAMES" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::NAMES(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::NAMES(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::NAMES(None, None)
                } else if args.len() == 1 {
                    Command::NAMES(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::NAMES(Some(args[0].to_owned()), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LIST" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::LIST(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::LIST(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::LIST(None, None)
                } else if args.len() == 1 {
                    Command::LIST(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::LIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "INVITE" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::INVITE(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::INVITE(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "KICK" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(args[0].to_owned(), args[1].to_owned(), Some(suffix.to_owned()))
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::KICK(args[0].to_owned(), args[1].to_owned(), None)
                },
            }
        } else if let "PRIVMSG" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::PRIVMSG(args[0].to_owned(), suffix.to_owned())
                },
                None => return Err(invalid_input())
            }
        } else if let "NOTICE" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::NOTICE(args[0].to_owned(), suffix.to_owned())
                },
                None => return Err(invalid_input())
            }
        } else if let "MOTD" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::MOTD(Some(suffix.to_owned())),
                None => Command::MOTD(None)
            }
        } else if let "LUSERS" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::LUSERS(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::LUSERS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::LUSERS(None, None)
                } else if args.len() == 1 {
                    Command::LUSERS(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::LUSERS(Some(args[0].to_owned()), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "VERSION" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::VERSION(Some(suffix.to_owned())),
                None => Command::VERSION(None)
            }
        } else if let "STATS" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::STATS(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::STATS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::STATS(None, None)
                } else if args.len() == 1 {
                    Command::STATS(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::STATS(Some(args[0].to_owned()), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "LINKS" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::LINKS(None, Some(suffix.to_owned()))
                } else if args.len() == 1 {
                    Command::LINKS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::LINKS(None, None)
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "TIME" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::TIME(Some(suffix.to_owned())),
                None => Command::TIME(None)
            }
        } else if let "CONNECT" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(args[0].to_owned(), args[1].to_owned(), Some(suffix.to_owned()))
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::CONNECT(args[0].to_owned(), args[1].to_owned(), None)
                }
            }
        } else if let "TRACE" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::TRACE(Some(suffix.to_owned())),
                None => Command::TRACE(None)
            }
        } else if let "ADMIN" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::ADMIN(Some(suffix.to_owned())),
                None => Command::ADMIN(None)
            }
        } else if let "INFO" = cmd {
            if args.len() != 0 { return Err(invalid_input()) }
            match suffix {
                Some(suffix) => Command::INFO(Some(suffix.to_owned())),
                None => Command::INFO(None)
            }
        } else if let "SERVLIST" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::SERVLIST(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::SERVLIST(Some(args[0].to_owned()), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::SERVLIST(None, None)
                } else if args.len() == 1 {
                    Command::SERVLIST(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::SERVLIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SQUERY" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SQUERY(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SQUERY(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "WHO" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::WHO(Some(suffix.to_owned()), None)
                } else if args.len() == 1 {
                    Command::WHO(Some(args[0].to_owned()), Some(&suffix[..] == "o"))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 0 {
                    Command::WHO(None, None)
                } else if args.len() == 1 {
                    Command::WHO(Some(args[0].to_owned()), None)
                } else if args.len() == 2 {
                    Command::WHO(Some(args[0].to_owned()), Some(&args[1][..] == "o"))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOIS" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::WHOIS(None, suffix.to_owned())
                } else if args.len() == 1 {
                    Command::WHOIS(Some(args[0].to_owned()), suffix.to_owned())
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::WHOIS(None, args[0].to_owned())
                } else if args.len() == 2 {
                    Command::WHOIS(Some(args[0].to_owned()), args[1].to_owned())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "WHOWAS" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::WHOWAS(suffix.to_owned(), None, None)
                } else if args.len() == 1 {
                    Command::WHOWAS(args[0].to_owned(), None, Some(suffix.to_owned()))
                } else if args.len() == 2 {
                    Command::WHOWAS(args[0].to_owned(), Some(args[1].to_owned()),
                                    Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::WHOWAS(args[0].to_owned(), None, None)
                } else if args.len() == 2 {
                    Command::WHOWAS(args[0].to_owned(), None, Some(args[1].to_owned()))
                } else if args.len() == 3 {
                    Command::WHOWAS(args[0].to_owned(), Some(args[1].to_owned()),
                                    Some(args[2].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "KILL" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::KILL(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::KILL(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "PING" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::PING(suffix.to_owned(), None)
                } else if args.len() == 1 {
                    Command::PING(args[0].to_owned(), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::PING(args[0].to_owned(), None)
                } else if args.len() == 2 {
                    Command::PING(args[0].to_owned(), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "PONG" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::PONG(suffix.to_owned(), None)
                } else if args.len() == 1 {
                    Command::PONG(args[0].to_owned(), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::PONG(args[0].to_owned(), None)
                } else if args.len() == 2 {
                    Command::PONG(args[0].to_owned(), Some(args[1].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "ERROR" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::ERROR(suffix.to_owned())
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "AWAY" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::AWAY(Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => return Err(invalid_input())
            }
        } else if let "REHASH" = cmd {
            if args.len() == 0 {
                Command::REHASH
            } else {
                return Err(invalid_input())
            }
        } else if let "DIE" = cmd {
            if args.len() == 0 {
                Command::DIE
            } else {
                return Err(invalid_input())
            }
        } else if let "RESTART" = cmd {
            if args.len() == 0 {
                Command::RESTART
            } else {
                return Err(invalid_input())
            }
        } else if let "SUMMON" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::SUMMON(suffix.to_owned(), None, None)
                } else if args.len() == 1 {
                    Command::SUMMON(args[0].to_owned(), Some(suffix.to_owned()), None)
                } else if args.len() == 2 {
                    Command::SUMMON(args[0].to_owned(), Some(args[1].to_owned()),
                                    Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::SUMMON(args[0].to_owned(), None, None)
                } else if args.len() == 2 {
                    Command::SUMMON(args[0].to_owned(), Some(args[1].to_owned()), None)
                } else if args.len() == 3 {
                    Command::SUMMON(args[0].to_owned(), Some(args[1].to_owned()),
                                    Some(args[2].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "USERS" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::USERS(Some(suffix.to_owned()))
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::USERS(Some(args[0].to_owned()))
                }
            }
        } else if let "WALLOPS" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::WALLOPS(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::WALLOPS(args[0].to_owned())
                }
            }
        } else if let "USERHOST" = cmd {
            if suffix.is_none() {
                Command::USERHOST(args.into_iter().map(|s| s.to_owned()).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "ISON" = cmd {
            if suffix.is_none() {
                Command::USERHOST(args.into_iter().map(|s| s.to_owned()).collect())
            } else {
                return Err(invalid_input())
            }
        } else if let "SAJOIN" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SAJOIN(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SAJOIN(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "SAMODE" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 1 {
                    Command::SAMODE(args[0].to_owned(), suffix.to_owned(), None)
                } else if args.len() == 2 {
                    Command::SAMODE(args[0].to_owned(), args[1].to_owned(), Some(suffix.to_owned()))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 2 {
                    Command::SAMODE(args[0].to_owned(), args[1].to_owned(), None)
                } else if args.len() == 3 {
                    Command::SAMODE(args[0].to_owned(), args[1].to_owned(), Some(args[2].to_owned()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "SANICK" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SANICK(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SANICK(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "SAPART" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SAPART(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SAPART(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "SAQUIT" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::SAQUIT(args[0].to_owned(), suffix.to_owned())
                },
                None => {
                    if args.len() != 2 { return Err(invalid_input()) }
                    Command::SAQUIT(args[0].to_owned(), args[1].to_owned())
                }
            }
        } else if let "NICKSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::NICKSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::NICKSERV(args[0].to_owned())
                }
            }
        } else if let "CHANSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::CHANSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::CHANSERV(args[0].to_owned())
                }
            }
        } else if let "OPERSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::OPERSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::OPERSERV(args[0].to_owned())
                }
            }
        } else if let "BOTSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::BOTSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::BOTSERV(args[0].to_owned())
                }
            }
        } else if let "HOSTSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::HOSTSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::HOSTSERV(args[0].to_owned())
                }
            }
        } else if let "MEMOSERV" = cmd {
            match suffix {
                Some(suffix) => {
                    if args.len() != 0 { return Err(invalid_input()) }
                    Command::MEMOSERV(suffix.to_owned())
                },
                None => {
                    if args.len() != 1 { return Err(invalid_input()) }
                    Command::MEMOSERV(args[0].to_owned())
                }
            }
        } else if let "CAP" = cmd {
            if args.len() == 1 {
                if let Ok(cmd) = args[0].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(None, cmd, None, Some(suffix.to_owned())),
                        None => Command::CAP(None, cmd, None, None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else if args.len() == 2 {
                if let Ok(cmd) = args[0].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(None, cmd, Some(args[1].to_owned()),
                                                         Some(suffix.to_owned())),
                        None => Command::CAP(None, cmd, Some(args[1].to_owned()), None),
                    }
                } else if let Ok(cmd) = args[1].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(Some(args[0].to_owned()), cmd, None,
                                                         Some(suffix.to_owned())),
                        None => Command::CAP(Some(args[0].to_owned()), cmd, None, None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else if args.len() == 3 {
                if let Ok(cmd) = args[1].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(Some(args[0].to_owned()), cmd,
                                                         Some(args[2].to_owned()),
                                                         Some(suffix.to_owned())),
                        None => Command::CAP(Some(args[0].to_owned()), cmd, Some(args[2].to_owned()),
                                             None),
                    }
                } else {
                    return Err(invalid_input())
                }
            } else {
                return Err(invalid_input())
            }
        } else if let "AUTHENTICATE" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::AUTHENTICATE(suffix.to_owned())
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::AUTHENTICATE(args[0].to_owned())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "ACCOUNT" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::ACCOUNT(suffix.to_owned())
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::ACCOUNT(args[0].to_owned())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "METADATA" = cmd {
            if args.len() == 2 {
                match suffix {
                    Some(_) => return Err(invalid_input()),
                    None => match args[1].parse() {
                        Ok(c) => Command::METADATA(args[0].to_owned(), Some(c), None, None),
                        Err(_) => return Err(invalid_input()),
                    },
                }
            } else if args.len() > 2 {
                match args[1].parse() {
                    Ok(c) => Command::METADATA(
                        args[0].to_owned(), Some(c),
                        Some(args.into_iter().skip(1).map(|s| s.to_owned()).collect()),
                        suffix.map(|s| s.to_owned())
                    ),
                    Err(_) => if args.len() == 3 && suffix.is_some() {
                        Command::METADATA(
                            args[0].to_owned(), None,
                            Some(args.into_iter().skip(1).map(|s| s.to_owned()).collect()),
                            suffix.map(|s| s.to_owned())
                        )
                    } else {
                        return Err(invalid_input())
                    },
                }
            } else {
                return Err(invalid_input())
            }
        } else if let "MONITOR" = cmd {
            if args.len() == 1 {
                Command::MONITOR(args[0].to_owned(), suffix.map(|s| s.to_owned()))
            } else {
                return Err(invalid_input())
            }
        } else if let "BATCH" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 0 {
                    Command::BATCH(suffix.to_owned(), None, None)
                } else if args.len() == 1 {
                    Command::BATCH(args[0].to_owned(), Some(
                        suffix.parse().unwrap()
                    ), None)
                } else if args.len() > 1 {
                    Command::BATCH(args[0].to_owned(), Some(
                        args[1].parse().unwrap()
                    ), Some(
                        vec![suffix.to_owned()].into_iter().chain(
                            args.into_iter().skip(2).map(|s| s.to_owned())
                        ).collect()
                    ))
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 1 {
                    Command::BATCH(args[0].to_owned(), None, None)
                } else if args.len() == 2 {
                    Command::BATCH(args[0].to_owned(), Some(
                        args[1].parse().unwrap()
                    ), None)
                } else if args.len() > 2 {
                    Command::BATCH(args[0].to_owned(), Some(
                        args[1].parse().unwrap()
                    ), Some(args.iter().skip(2).map(|&s| s.to_owned()).collect()))
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let "CHGHOST" = cmd {
            match suffix {
                Some(suffix) => if args.len() == 1 {
                    Command::CHGHOST(args[0].to_owned(), suffix.to_owned())
                } else {
                    return Err(invalid_input())
                },
                None => if args.len() == 2 {
                    Command::CHGHOST(args[0].to_owned(), args[1].to_owned())
                } else {
                    return Err(invalid_input())
                }
            }
        } else if let Ok(resp) = cmd.parse() {
            Command::Response(
                resp, args.into_iter().map(|s| s.to_owned()).collect(),
                suffix.map(|s| s.to_owned())
            )
        } else {
            Command::Raw(
                cmd.to_owned(), args.into_iter().map(|s| s.to_owned()).collect(),
                suffix.map(|s| s.to_owned())
            )
        })
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

/// A list of all the subcommands for the
/// [metadata extension](http://ircv3.net/specs/core/metadata-3.2.html).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MetadataSubCommand {
    /// Looks up the value for some keys.
    GET,
    /// Lists all of the metadata keys and values.
    LIST,
    /// Sets the value for some key.
    SET,
    /// Removes all metadata.
    CLEAR,
}

impl MetadataSubCommand {
    /// Gets the string that corresponds to this subcommand.
    pub fn to_str(&self) -> &str {
        match self {
            &MetadataSubCommand::GET   => "GET",
            &MetadataSubCommand::LIST  => "LIST",
            &MetadataSubCommand::SET   => "SET",
            &MetadataSubCommand::CLEAR => "CLEAR",
        }
    }
}

impl FromStr for MetadataSubCommand {
    type Err = &'static str;
    fn from_str(s: &str) -> StdResult<MetadataSubCommand, &'static str> {
        match s {
            "GET"   => Ok(MetadataSubCommand::GET),
            "LIST"  => Ok(MetadataSubCommand::LIST),
            "SET"   => Ok(MetadataSubCommand::SET),
            "CLEAR" => Ok(MetadataSubCommand::CLEAR),
            _       => Err("Failed to parse METADATA subcommand."),
        }
    }
}

/// [batch extension](http://ircv3.net/specs/extensions/batch-3.2.html).
#[derive(Clone, Debug, PartialEq)]
pub enum BatchSubCommand {
    /// [NETSPLIT](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETSPLIT,
    /// [NETJOIN](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETJOIN,
    /// Vendor-specific BATCH subcommands.
    CUSTOM(String),
}

impl BatchSubCommand {
    /// Gets the string that corresponds to this subcommand.
    pub fn to_str(&self) -> &str {
        match self {
            &BatchSubCommand::NETSPLIT      => "NETSPLIT",
            &BatchSubCommand::NETJOIN       => "NETJOIN",
            &BatchSubCommand::CUSTOM(ref s) => &s,
        }
    }
}

impl FromStr for BatchSubCommand {
    type Err = &'static str;
    fn from_str(s: &str) -> StdResult<BatchSubCommand, &'static str> {
        match s {
            "NETSPLIT" => Ok(BatchSubCommand::NETSPLIT),
            "NETJOIN"  => Ok(BatchSubCommand::NETJOIN),
            _          => Ok(BatchSubCommand::CUSTOM(s.to_owned())),
        }
    }
}

/// Produces an invalid_input IoError.
fn invalid_input() -> Error {
    Error::new(ErrorKind::InvalidInput, "Failed to parse malformed message as command.")
}
