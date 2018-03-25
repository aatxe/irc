use std::ascii::AsciiExt;
use std::borrow::Borrow;
use std::str::FromStr;

use chan::ChannelExt;
use error::MessageParseError;
use mode::{ChannelMode, Mode, UserMode};
use response::Response;

macro_rules! new_command_helper {
    // All of the following variants of this macro handle specialization for variants that have
    // weird styles of construction that differ from the general pattern.

    // The Mode API is specialized, and so we have to deal with parsing differently.
    (UserMODE($($argty:ty),*) with $args:expr) => {
        {
            let args = $args;
            if args.len() < 2 { return Ok(None) }

            let target = $args[0];
            let arg = {
                let mut buf = String::new();
                for arg in &args[1..] {
                    buf.push_str(arg);
                    buf.push(' ');
                }
                let len = buf.len() - " ".len();
                buf.truncate(len);
                buf
            };

            if target.is_channel_name() {
                return Ok(Some(
                    Command::ChannelMODE(target, Mode::as_channel_modes(&arg)?)
                ))
            } else {
                return Ok(Some(
                    Command::UserMODE(target, Mode::as_user_modes(&arg)?)
                ))
            }
        }
    };

    // The Mode API is specialized, and so we have to deal with parsing differently.
    // We intentionally do nothing here because it's handled in the `UserMODE` case.
    (ChannelMODE($($argty:ty),*) with $args:expr) => {
        unreachable!()
    };

    (WHO($($argty:ty),*) with $args:expr) => {
        {
            let args = $args;
            if args.is_empty() {
                return Ok(Some(Command::WHO(None, None)))
            } else if args.len() == 1 {
                return Ok(Some(
                    Command::WHO(Some(args[0]), None)
                ))
            } else if args.len() == 2 {
                return Ok(Some(
                    Command::WHO(Some(args[0]), Some(args[1] == "o"))
                ))
            } else {
                return Ok(None)
            }
        }
    };

    // `Response` isn't actually a command, but instead a special family of commands. It's handled
    // separately, but we want to treat any actual commands named `Response` as `Raw` and so we
    // return `None` here.
    (Response($($argty:ty),*) with $args:expr) => {
        return Ok(None)
    };

    // Similar to `Response`, we don't actually want to parse `Raw`.
    (Raw($($argty:ty),*) with $args:expr) => {
        return Ok(None)
    };

    // This is the default construction for `Command` variants.
    ($variant:ident($($argty:ty),*) with $args:expr) => {
        {
            #[allow(unused_mut)]
            let mut args = $args.into_iter();
            return Ok(Some(
                Command::$variant(
                    $(<$argty as FromArgIter>::next_arg(&mut args)?),*
                )
            ))
        }
    };
}

trait FromArgIter<'a>: Sized {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str>;
}

impl<'a> FromArgIter<'a> for &'a str {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        Ok(iter.next().unwrap())
    }
}

impl<'a> FromArgIter<'a> for CapSubCommand {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        match iter.next() {
            Some(str) => CapSubCommand::from_str(str),
            None => panic!(),
        }
    }
}

impl<'a> FromArgIter<'a> for Option<MetadataSubCommand> {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        match iter.next() {
            Some(str) => MetadataSubCommand::from_str(str).map(|cmd| Some(cmd)),
            None => Ok(None),
        }
    }
}

impl<'a> FromArgIter<'a> for Option<BatchSubCommand<'a>> {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        Ok(iter.next().map(|str| {
            BatchSubCommand::from_str(str)
        }))
    }
}

impl<'a> FromArgIter<'a> for Option<&'a str> {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        Ok(iter.next())
    }
}

impl<'a> FromArgIter<'a> for Vec<&'a str> {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        Ok(iter.collect())
    }
}

impl<'a> FromArgIter<'a> for Option<Vec<&'a str>> {
    fn next_arg<I>(iter: &mut I) -> Result<Self, MessageParseError>
    where I: Iterator<Item = &'a str> {
        let res: Vec<_> = iter.collect();
        Ok(if res.len() == 0 {
            None
        } else {
            Some(res)
        })
    }
}

macro_rules! cmd_stringify {
    (UserMODE) => { "MODE" };
    (ChannelMODE) => { "MODE" };
    ($variant:ident) => { stringify!($variant) };
}

macro_rules! make_command {
    ($($(#[$attr:meta])+ $variant:ident($($argty:ty),*)),+) => {
        /// Borrowed version of client commands as defined in [RFC 2812][rfc2812]. This also
        /// includes commands from the [capabilities extension][caps]. Additionally, this includes
        /// some common additional commands from popular IRCds.
        ///
        /// [rfc2812]: http://tools.ietf.org/html/rfc2812
        /// [caps]: https://tools.ietf.org/html/draft-mitchell-irc-capabilities-01
        #[derive(Clone, Debug, PartialEq)]
        pub enum Command<'cmd> {
            $($(#[$attr])+ $variant($($argty),*)),+
        }

        impl<'cmd> Command<'cmd> {
            pub fn new(
                cmd: &'cmd str,
                args: &[&'cmd str],
                suffix: Option<&'cmd str>,
            ) -> Result<Command<'cmd>, MessageParseError> {
                Command::new_impl(cmd, args.to_owned(), suffix).map(|opt| opt.unwrap_or_else(|| {
                    Command::Raw(cmd, args.to_owned(), suffix)
                }))
            }

            fn new_impl(
                cmd: &'cmd str,
                args: Vec<&'cmd str>,
                suffix: Option<&'cmd str>,
            ) -> Result<Option<Command<'cmd>>, MessageParseError> {
                $(
                    if cmd.eq_ignore_ascii_case(cmd_stringify!($variant)) {
                        new_command_helper!($variant($($argty),*) with {
                            let mut tmp = args.clone();
                            if let Some(suffix) = suffix {
                                tmp.push(suffix);
                            }
                            tmp
                        })
                    }
                )+

                if let Ok(resp) = cmd.parse() {
                    Ok(Some(Command::Response(resp, args.to_owned(), suffix)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

make_command! {
    // 3.1 Connection Registration
    /// PASS :password
    PASS(&'cmd str),
    /// NICK :nickname
    NICK(&'cmd str),
    /// USER user mode * :realname
    USER(&'cmd str, &'cmd str, &'cmd str),
    /// OPER name :password
    OPER(&'cmd str, &'cmd str),
    /// MODE nickname modes
    UserMODE(&'cmd str, Vec<Mode<UserMode>>),
    /// SERVICE nickname reserved distribution type reserved :info
    SERVICE(&'cmd str, &'cmd str, &'cmd str, &'cmd str, &'cmd str, &'cmd str),
    /// QUIT :comment
    QUIT(Option<&'cmd str>),
    /// SQUIT server :comment
    SQUIT(&'cmd str, &'cmd str),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys] :[Real name]
    JOIN(&'cmd str, Option<&'cmd str>, Option<&'cmd str>),
    /// PART chanlist :[comment]
    PART(&'cmd str, Option<&'cmd str>),
    /// MODE channel [modes [modeparams]]
    ChannelMODE(&'cmd str, Vec<Mode<ChannelMode>>),
    /// TOPIC channel :[topic]
    TOPIC(&'cmd str, Option<&'cmd str>),
    /// NAMES [chanlist :[target]]
    NAMES(Option<&'cmd str>, Option<&'cmd str>),
    /// LIST [chanlist :[target]]
    LIST(Option<&'cmd str>, Option<&'cmd str>),
    /// INVITE nickname channel
    INVITE(&'cmd str, &'cmd str),
    /// KICK chanlist userlist :[comment]
    KICK(&'cmd str, &'cmd str, Option<&'cmd str>),

    // 3.3 Sending messages
    /// PRIVMSG msgtarget :message
    PRIVMSG(&'cmd str, &'cmd str),
    /// NOTICE msgtarget :message
    NOTICE(&'cmd str, &'cmd str),

    // 3.4 Server queries and commands
    /// MOTD :[target]
    MOTD(Option<&'cmd str>),
    /// LUSERS [mask :[target]]
    LUSERS(Option<&'cmd str>, Option<&'cmd str>),
    /// VERSION :[target]
    VERSION(Option<&'cmd str>),
    /// STATS [query :[target]]
    STATS(Option<&'cmd str>, Option<&'cmd str>),
    /// LINKS [[remote server] server :mask]
    LINKS(Option<&'cmd str>, Option<&'cmd str>),
    /// TIME :[target]
    TIME(Option<&'cmd str>),
    /// CONNECT target server port :[remote server]
    CONNECT(&'cmd str, &'cmd str, Option<&'cmd str>),
    /// TRACE :[target]
    TRACE(Option<&'cmd str>),
    /// ADMIN :[target]
    ADMIN(Option<&'cmd str>),
    /// INFO :[target]
    INFO(Option<&'cmd str>),

    // 3.5 Service Query and Commands
    /// SERVLIST [mask :[type]]
    SERVLIST(Option<&'cmd str>, Option<&'cmd str>),
    /// SQUERY servicename text
    SQUERY(&'cmd str, &'cmd str),

    // 3.6 User based queries
    /// WHO [mask ["o"]]
    WHO(Option<&'cmd str>, Option<bool>),
    /// WHOIS [target] masklist
    WHOIS(Option<&'cmd str>, &'cmd str),
    /// WHOWAS nicklist [count :[target]]
    WHOWAS(&'cmd str, Option<&'cmd str>, Option<&'cmd str>),

    // 3.7 Miscellaneous messages
    /// KILL nickname :comment
    KILL(&'cmd str, &'cmd str),
    /// PING server1 :[server2]
    PING(&'cmd str, Option<&'cmd str>),
    /// PONG server :[server2]
    PONG(&'cmd str, Option<&'cmd str>),
    /// ERROR :message
    ERROR(&'cmd str),

    // 4 Optional Features
    /// AWAY :[message]
    AWAY(Option<&'cmd str>),
    /// REHASH
    REHASH(),
    /// DIE
    DIE(),
    /// RESTART
    RESTART(),
    /// SUMMON user [target :[channel]]
    SUMMON(&'cmd str, Option<&'cmd str>, Option<&'cmd str>),
    /// USERS :[target]
    USERS(Option<&'cmd str>),
    /// WALLOPS :Text to be sent
    WALLOPS(&'cmd str),
    /// USERHOST space-separated nicklist
    USERHOST(Vec<&'cmd str>),
    /// ISON space-separated nicklist
    ISON(Vec<&'cmd str>),

    // Non-RFC commands from InspIRCd
    /// SAJOIN nickname channel
    SAJOIN(&'cmd str, &'cmd str),
    /// SAMODE target modes [modeparams]
    SAMODE(&'cmd str, &'cmd str, Option<&'cmd str>),
    /// SANICK old nickname new nickname
    SANICK(&'cmd str, &'cmd str),
    /// SAPART nickname :comment
    SAPART(&'cmd str, &'cmd str),
    /// SAQUIT nickname :comment
    SAQUIT(&'cmd str, &'cmd str),
    /// NICKSERV message
    NICKSERV(&'cmd str),
    /// CHANSERV message
    CHANSERV(&'cmd str),
    /// OPERSERV message
    OPERSERV(&'cmd str),
    /// BOTSERV message
    BOTSERV(&'cmd str),
    /// HOSTSERV message
    HOSTSERV(&'cmd str),
    /// MEMOSERV message
    MEMOSERV(&'cmd str),

    // IRCv3 support
    /// CAP [*] COMMAND [*] :[param]
    CAP(Option<&'cmd str>, CapSubCommand, Option<&'cmd str>, Option<&'cmd str>),

    // IRCv3.1 extensions
    /// AUTHENTICATE data
    AUTHENTICATE(&'cmd str),
    /// ACCOUNT [account name]
    ACCOUNT(&'cmd str),
    // AWAY is already defined as a send-only message.
    // AWAY(Option<&'cmd str>),
    // JOIN is already defined.
    // JOIN(&'cmd str, Option<&'cmd str>, Option<&'cmd str>),

    // IRCv3.2 extensions
    /// METADATA target COMMAND [params] :[param]
    METADATA(&'cmd str, Option<MetadataSubCommand>, Option<Vec<&'cmd str>>, Option<&'cmd str>),
    /// MONITOR command [nicklist]
    MONITOR(&'cmd str, Option<&'cmd str>),
    /// BATCH (+/-)reference-tag [type [params]]
    BATCH(&'cmd str, Option<BatchSubCommand<'cmd>>, Option<Vec<&'cmd str>>),
    /// CHGHOST user host
    CHGHOST(&'cmd str, &'cmd str),

    // Default option.
    /// An IRC response code with arguments and optional suffix.
    Response(Response, Vec<&'cmd str>, Option<&'cmd str>),
    /// A raw IRC command unknown to the crate.
    Raw(&'cmd str, Vec<&'cmd str>, Option<&'cmd str>)
}

fn stringify(cmd: &str, args: &[&str], suffix: Option<&str>) -> String {
    let args = args.join(" ");
    let sp = if args.is_empty() { "" } else { " " };
    match suffix {
        Some(suffix) => format!("{}{}{} :{}", cmd, sp, args, suffix),
        None => format!("{}{}{}", cmd, sp, args),
    }
}

impl<'a> From<Command<'a>> for String {
    fn from(cmd: Command<'a>) -> String {
        match cmd {
            Command::PASS(p) => stringify("PASS", &[], Some(p)),
            Command::NICK(n) => stringify("NICK", &[], Some(n)),
            Command::USER(u, m, r) => stringify("USER", &[u, m, "*"], Some(r)),
            Command::OPER(u, p) => stringify("OPER", &[u], Some(p)),
            Command::UserMODE(u, m) => {
                format!("MODE {}{}", u, m.iter().fold(String::new(), |mut acc, mode| {
                    acc.push_str(" ");
                    acc.push_str(&mode.to_string());
                    acc
                }))
            }
            Command::SERVICE(n, r, d, t, re, i) => {
                stringify("SERVICE", &[n, r, d, t, re], Some(i))
            }
            Command::QUIT(Some(m)) => stringify("QUIT", &[], Some(m)),
            Command::QUIT(None) => stringify("QUIT", &[], None),
            Command::SQUIT(s, c) => stringify("SQUIT", &[s], Some(c)),
            Command::JOIN(c, Some(k), Some(n)) => stringify("JOIN", &[c, k], Some(n)),
            Command::JOIN(c, Some(k), None) => stringify("JOIN", &[c, k], None),
            Command::JOIN(c, None, Some(n)) => stringify("JOIN", &[c], Some(n)),
            Command::JOIN(c, None, None) => stringify("JOIN", &[c], None),
            Command::PART(c, Some(m)) => stringify("PART", &[c], Some(m)),
            Command::PART(c, None) => stringify("PART", &[c], None),
            Command::ChannelMODE(u, m) => {
                format!("MODE {}{}", u, m.iter().fold(String::new(), |mut acc, mode| {
                    acc.push_str(" ");
                    acc.push_str(&mode.to_string());
                    acc
                }))
            }
            Command::TOPIC(c, Some(t)) => stringify("TOPIC", &[c], Some(t)),
            Command::TOPIC(c, None) => stringify("TOPIC", &[c], None),
            Command::NAMES(Some(c), Some(t)) => stringify("NAMES", &[c], Some(t)),
            Command::NAMES(Some(c), None) => stringify("NAMES", &[c], None),
            Command::NAMES(None, _) => stringify("NAMES", &[], None),
            Command::LIST(Some(c), Some(t)) => stringify("LIST", &[c], Some(t)),
            Command::LIST(Some(c), None) => stringify("LIST", &[c], None),
            Command::LIST(None, _) => stringify("LIST", &[], None),
            Command::INVITE(n, c) => stringify("INVITE", &[n, c], None),
            Command::KICK(c, n, Some(r)) => stringify("KICK", &[c, n], Some(r)),
            Command::KICK(c, n, None) => stringify("KICK", &[c, n], None),
            Command::PRIVMSG(t, m) => stringify("PRIVMSG", &[t], Some(m)),
            Command::NOTICE(t, m) => stringify("NOTICE", &[t], Some(m)),
            Command::MOTD(Some(t)) => stringify("MOTD", &[], Some(t)),
            Command::MOTD(None) => stringify("MOTD", &[], None),
            Command::LUSERS(Some(m), Some(t)) => stringify("LUSERS", &[m], Some(t)),
            Command::LUSERS(Some(m), None) => stringify("LUSERS", &[m], None),
            Command::LUSERS(None, _) => stringify("LUSERS", &[], None),
            Command::VERSION(Some(t)) => stringify("VERSION", &[], Some(t)),
            Command::VERSION(None) => stringify("VERSION", &[], None),
            Command::STATS(Some(q), Some(t)) => stringify("STATS", &[q], Some(t)),
            Command::STATS(Some(q), None) => stringify("STATS", &[q], None),
            Command::STATS(None, _) => stringify("STATS", &[], None),
            Command::LINKS(Some(r), Some(s)) => stringify("LINKS", &[r], Some(s)),
            Command::LINKS(None, Some(s)) => stringify("LINKS", &[], Some(s)),
            Command::LINKS(_, None) => stringify("LINKS", &[], None),
            Command::TIME(Some(t)) => stringify("TIME", &[], Some(t)),
            Command::TIME(None) => stringify("TIME", &[], None),
            Command::CONNECT(t, p, Some(r)) => stringify("CONNECT", &[t, p], Some(r)),
            Command::CONNECT(t, p, None) => stringify("CONNECT", &[t, p], None),
            Command::TRACE(Some(t)) => stringify("TRACE", &[], Some(t)),
            Command::TRACE(None) => stringify("TRACE", &[], None),
            Command::ADMIN(Some(t)) => stringify("ADMIN", &[], Some(t)),
            Command::ADMIN(None) => stringify("ADMIN", &[], None),
            Command::INFO(Some(t)) => stringify("INFO", &[], Some(t)),
            Command::INFO(None) => stringify("INFO", &[], None),
            Command::SERVLIST(Some(m), Some(t)) => stringify("SERVLIST", &[m], Some(t)),
            Command::SERVLIST(Some(m), None) => stringify("SERVLIST", &[m], None),
            Command::SERVLIST(None, _) => stringify("SERVLIST", &[], None),
            Command::SQUERY(s, t) => stringify("SQUERY", &[s, t], None),
            Command::WHO(Some(s), Some(true)) => stringify("WHO", &[s, "o"], None),
            Command::WHO(Some(s), _) => stringify("WHO", &[s], None),
            Command::WHO(None, _) => stringify("WHO", &[], None),
            Command::WHOIS(Some(t), m) => stringify("WHOIS", &[t, m], None),
            Command::WHOIS(None, m) => stringify("WHOIS", &[m], None),
            Command::WHOWAS(n, Some(c), Some(t)) => {
                stringify("WHOWAS", &[n, c], Some(t))
            }
            Command::WHOWAS(n, Some(c), None) => stringify("WHOWAS", &[n, c], None),
            Command::WHOWAS(n, None, _) => stringify("WHOWAS", &[n], None),
            Command::KILL(n, c) => stringify("KILL", &[n], Some(c)),
            Command::PING(s, Some(t)) => stringify("PING", &[s], Some(t)),
            Command::PING(s, None) => stringify("PING", &[], Some(s)),
            Command::PONG(s, Some(t)) => stringify("PONG", &[s], Some(t)),
            Command::PONG(s, None) => stringify("PONG", &[], Some(s)),
            Command::ERROR(m) => stringify("ERROR", &[], Some(m)),
            Command::AWAY(Some(m)) => stringify("AWAY", &[], Some(m)),
            Command::AWAY(None) => stringify("AWAY", &[], None),
            Command::REHASH() => stringify("REHASH", &[], None),
            Command::DIE() => stringify("DIE", &[], None),
            Command::RESTART() => stringify("RESTART", &[], None),
            Command::SUMMON(u, Some(t), Some(c)) => {
                stringify("SUMMON", &[u, t], Some(c))
            }
            Command::SUMMON(u, Some(t), None) => stringify("SUMMON", &[u, t], None),
            Command::SUMMON(u, None, _) => stringify("SUMMON", &[u], None),
            Command::USERS(Some(t)) => stringify("USERS", &[], Some(t)),
            Command::USERS(None) => stringify("USERS", &[], None),
            Command::WALLOPS(t) => stringify("WALLOPS", &[], Some(t)),
            Command::USERHOST(u) => {
                stringify(
                    "USERHOST",
                    &u.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                    None,
                )
            }
            Command::ISON(u) => {
                stringify("ISON", &u.iter().map(|s| &s[..]).collect::<Vec<_>>(), None)
            }

            Command::SAJOIN(n, c) => stringify("SAJOIN", &[n, c], None),
            Command::SAMODE(t, m, Some(p)) => stringify("SAMODE", &[t, m, p], None),
            Command::SAMODE(t, m, None) => stringify("SAMODE", &[t, m], None),
            Command::SANICK(o, n) => stringify("SANICK", &[o, n], None),
            Command::SAPART(c, r) => stringify("SAPART", &[c], Some(r)),
            Command::SAQUIT(c, r) => stringify("SAQUIT", &[c], Some(r)),

            Command::NICKSERV(m) => stringify("NICKSERV", &[m], None),
            Command::CHANSERV(m) => stringify("CHANSERV", &[m], None),
            Command::OPERSERV(m) => stringify("OPERSERV", &[m], None),
            Command::BOTSERV(m) => stringify("BOTSERV", &[m], None),
            Command::HOSTSERV(m) => stringify("HOSTSERV", &[m], None),
            Command::MEMOSERV(m) => stringify("MEMOSERV", &[m], None),

            Command::CAP(None, s, None, Some(p)) => {
                stringify("CAP", &[s.to_str()], Some(p))
            }
            Command::CAP(None, s, None, None) => stringify("CAP", &[s.to_str()], None),
            Command::CAP(Some(k), s, None, Some(p)) => {
                stringify("CAP", &[k, s.to_str()], Some(p))
            }
            Command::CAP(Some(k), s, None, None) => {
                stringify("CAP", &[k, s.to_str()], None)
            }
            Command::CAP(None, s, Some(c), Some(p)) => {
                stringify("CAP", &[s.to_str(), c], Some(p))
            }
            Command::CAP(None, s, Some(c), None) => {
                stringify("CAP", &[s.to_str(), c], None)
            }
            Command::CAP(Some(k), s, Some(c), Some(p)) => {
                stringify("CAP", &[k, s.to_str(), c], Some(p))
            }
            Command::CAP(Some(k), s, Some(c), None) => {
                stringify("CAP", &[k, s.to_str(), c], None)
            }

            Command::AUTHENTICATE(d) => stringify("AUTHENTICATE", &[d], None),
            Command::ACCOUNT(a) => stringify("ACCOUNT", &[a], None),

            Command::METADATA(t, Some(c), None, Some(p)) => {
                stringify("METADATA", &[&t[..], c.to_str()], Some(p))
            }
            Command::METADATA(t, Some(c), None, None) => {
                stringify("METADATA", &[&t[..], c.to_str()], None)
            }

            Command::METADATA(t, Some(c), Some(a), Some(p)) => {
                stringify(
                    "METADATA",
                    &vec![t, &c.to_str().to_owned()]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    Some(p),
                )
            }
            Command::METADATA(t, Some(c), Some(a), None) => {
                stringify(
                    "METADATA",
                    &vec![t, &c.to_str().to_owned()]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    None,
                )
            }
            Command::METADATA(t, None, None, Some(p)) => {
                stringify("METADATA", &[t], Some(p))
            }
            Command::METADATA(t, None, None, None) => stringify("METADATA", &[t], None),
            Command::METADATA(t, None, Some(a), Some(p)) => {
                stringify(
                    "METADATA",
                    &vec![t]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    Some(p),
                )
            }
            Command::METADATA(t, None, Some(a), None) => {
                stringify(
                    "METADATA",
                    &vec![t]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    None,
                )
            }
            Command::MONITOR(c, Some(t)) => stringify("MONITOR", &[c, t], None),
            Command::MONITOR(c, None) => stringify("MONITOR", &[c], None),
            Command::BATCH(t, Some(c), Some(a)) => {
                stringify(
                    "BATCH",
                    &vec![t, &c.to_str().to_owned()]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    None,
                )
            }
            Command::BATCH(t, Some(c), None) => stringify("BATCH", &[t, c.to_str()], None),
            Command::BATCH(t, None, Some(a)) => {
                stringify(
                    "BATCH",
                    &vec![t]
                        .iter()
                        .map(|s| &s[..])
                        .chain(a.iter().map(|s| &s[..]))
                        .collect::<Vec<_>>(),
                    None,
                )
            }
            Command::BATCH(t, None, None) => stringify("BATCH", &[t], None),
            Command::CHGHOST(u, h) => stringify("CHGHOST", &[u, h], None),

            Command::Response(resp, a, Some(s)) => {
                stringify(
                    &format!("{}", resp as u16),
                    &a.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                    Some(s),
                )
            }
            Command::Response(resp, a, None) => {
                stringify(
                    &format!("{}", resp as u16),
                    &a.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                    None,
                )
            }
            Command::Raw(c, a, Some(s)) => {
                stringify(c, &a.iter().map(|s| &s[..]).collect::<Vec<_>>(), Some(s))
            }
            Command::Raw(c, a, None) => {
                stringify(c, &a.iter().map(|s| &s[..]).collect::<Vec<_>>(), None)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BatchSubCommand<'sub> {
    /// [NETSPLIT](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETSPLIT,
    /// [NETJOIN](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETJOIN,
    /// Vendor-specific BATCH subcommands.
    CUSTOM(&'sub str),
}

impl<'sub> BatchSubCommand<'sub> {
    /// Gets the string that corresponds to this subcommand.
    pub fn to_str<'a>(&'a self) -> &'a str {
        match *self {
            BatchSubCommand::NETSPLIT => "NETSPLIT",
            BatchSubCommand::NETJOIN => "NETJOIN",
            BatchSubCommand::CUSTOM(ref s) => s,
        }
    }

    pub fn from_str<'a>(s: &'a str) -> BatchSubCommand<'a> {
        if s.eq_ignore_ascii_case("NETSPLIT") {
            BatchSubCommand::NETSPLIT
        } else if s.eq_ignore_ascii_case("NETJOIN") {
            BatchSubCommand::NETJOIN
        } else {
            BatchSubCommand::CUSTOM(s)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OwnedCommand {
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
    UserMODE(String, Vec<Mode<UserMode>>),
    /// SERVICE nickname reserved distribution type reserved :info
    SERVICE(String, String, String, String, String, String),
    /// QUIT :comment
    QUIT(Option<String>),
    /// SQUIT server :comment
    SQUIT(String, String),

    // 3.2 Channel operations
    /// JOIN chanlist [chankeys] :[Real name]
    JOIN(String, Option<String>, Option<String>),
    /// PART chanlist :[comment]
    PART(String, Option<String>),
    /// MODE channel [modes [modeparams]]
    ChannelMODE(String, Vec<Mode<ChannelMode>>),
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
    BATCH(String, Option<OwnedBatchSubCommand>, Option<Vec<String>>),
    /// CHGHOST user host
    CHGHOST(String, String),

    // Default option.
    /// An IRC response code with arguments and optional suffix.
    Response(Response, Vec<String>, Option<String>),
    /// A raw IRC command unknown to the crate.
    Raw(String, Vec<String>, Option<String>),
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
        match *self {
            CapSubCommand::LS => "LS",
            CapSubCommand::LIST => "LIST",
            CapSubCommand::REQ => "REQ",
            CapSubCommand::ACK => "ACK",
            CapSubCommand::NAK => "NAK",
            CapSubCommand::END => "END",
            CapSubCommand::NEW => "NEW",
            CapSubCommand::DEL => "DEL",
        }
    }
}

impl FromStr for CapSubCommand {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<CapSubCommand, Self::Err> {
        if s.eq_ignore_ascii_case("LS") {
            Ok(CapSubCommand::LS)
        } else if s.eq_ignore_ascii_case("LIST") {
            Ok(CapSubCommand::LIST)
        } else if s.eq_ignore_ascii_case("REQ") {
            Ok(CapSubCommand::REQ)
        } else if s.eq_ignore_ascii_case("ACK") {
            Ok(CapSubCommand::ACK)
        } else if s.eq_ignore_ascii_case("NAK") {
            Ok(CapSubCommand::NAK)
        } else if s.eq_ignore_ascii_case("END") {
            Ok(CapSubCommand::END)
        } else if s.eq_ignore_ascii_case("NEW") {
            Ok(CapSubCommand::NEW)
        } else if s.eq_ignore_ascii_case("DEL") {
            Ok(CapSubCommand::DEL)
        } else {
            Err(MessageParseError::InvalidSubcommand {
                cmd: "CAP",
                sub: s.to_owned(),
            })
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
        match *self {
            MetadataSubCommand::GET => "GET",
            MetadataSubCommand::LIST => "LIST",
            MetadataSubCommand::SET => "SET",
            MetadataSubCommand::CLEAR => "CLEAR",
        }
    }
}

impl FromStr for MetadataSubCommand {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<MetadataSubCommand, Self::Err> {
        if s.eq_ignore_ascii_case("GET") {
            Ok(MetadataSubCommand::GET)
        } else if s.eq_ignore_ascii_case("LIST") {
            Ok(MetadataSubCommand::LIST)
        } else if s.eq_ignore_ascii_case("SET") {
            Ok(MetadataSubCommand::SET)
        } else if s.eq_ignore_ascii_case("CLEAR") {
            Ok(MetadataSubCommand::CLEAR)
        } else {
            Err(MessageParseError::InvalidSubcommand {
                cmd: "METADATA",
                sub: s.to_owned(),
            })
        }
    }
}

/// [batch extension](http://ircv3.net/specs/extensions/batch-3.2.html).
#[derive(Clone, Debug, PartialEq)]
pub enum OwnedBatchSubCommand {
    /// [NETSPLIT](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETSPLIT,
    /// [NETJOIN](http://ircv3.net/specs/extensions/batch/netsplit.html)
    NETJOIN,
    /// Vendor-specific BATCH subcommands.
    CUSTOM(String),
}

impl OwnedBatchSubCommand {
    /// Gets the string that corresponds to this subcommand.
    pub fn to_str(&self) -> &str {
        match *self {
            OwnedBatchSubCommand::NETSPLIT => "NETSPLIT",
            OwnedBatchSubCommand::NETJOIN => "NETJOIN",
            OwnedBatchSubCommand::CUSTOM(ref s) => s,
        }
    }
}

impl FromStr for OwnedBatchSubCommand {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<OwnedBatchSubCommand, Self::Err> {
        if s.eq_ignore_ascii_case("NETSPLIT") {
            Ok(OwnedBatchSubCommand::NETSPLIT)
        } else if s.eq_ignore_ascii_case("NETJOIN") {
            Ok(OwnedBatchSubCommand::NETJOIN)
        } else {
            Ok(OwnedBatchSubCommand::CUSTOM(s.to_uppercase()))
        }
    }
}
