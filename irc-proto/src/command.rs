//! Enumeration of all available client commands.
use std::str::FromStr;

use chan::ChannelExt;
use error::MessageParseError;
use mode::{ChannelMode, Mode, UserMode};
use response::Response;

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
    ///
    /// ## Responding to a `PRIVMSG`
    ///
    /// When responding to a message, it is not sufficient to simply copy the message target
    /// (msgtarget). This will work just fine for responding to messages in channels where the
    /// target is the same for all participants. However, when the message is sent directly to a
    /// user, this target will be that client's username, and responding to that same target will
    /// actually mean sending itself a response. In such a case, you should instead respond to the
    /// user sending the message as specified in the message prefix. Since this is a common
    /// pattern, there is a utility function
    /// [`Message::response_target`](../message/struct.Message.html#method.response_target)
    /// which is used for this exact purpose.
    PRIVMSG(String, String),
    /// NOTICE msgtarget :message
    ///
    /// ## Responding to a `NOTICE`
    ///
    /// When responding to a notice, it is not sufficient to simply copy the message target
    /// (msgtarget). This will work just fine for responding to messages in channels where the
    /// target is the same for all participants. However, when the message is sent directly to a
    /// user, this target will be that client's username, and responding to that same target will
    /// actually mean sending itself a response. In such a case, you should instead respond to the
    /// user sending the message as specified in the message prefix. Since this is a common
    /// pattern, there is a utility function
    /// [`Message::response_target`](../message/struct.Message.html#method.response_target)
    /// which is used for this exact purpose.
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
    CAP(
        Option<String>,
        CapSubCommand,
        Option<String>,
        Option<String>,
    ),

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
    METADATA(
        String,
        Option<MetadataSubCommand>,
        Option<Vec<String>>,
        Option<String>,
    ),
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

fn stringify(cmd: &str, args: &[&str], suffix: Option<&str>) -> String {
    let args = args.join(" ");
    let sp = if args.is_empty() { "" } else { " " };
    match suffix {
        Some(suffix) => format!("{}{}{} :{}", cmd, sp, args, suffix),
        None => format!("{}{}{}", cmd, sp, args),
    }
}

impl<'a> From<&'a Command> for String {
    fn from(cmd: &'a Command) -> String {
        match *cmd {
            Command::PASS(ref p) => stringify("PASS", &[], Some(p)),
            Command::NICK(ref n) => stringify("NICK", &[], Some(n)),
            Command::USER(ref u, ref m, ref r) => stringify("USER", &[u, m, "*"], Some(r)),
            Command::OPER(ref u, ref p) => stringify("OPER", &[u], Some(p)),
            Command::UserMODE(ref u, ref m) => format!(
                "MODE {}{}",
                u,
                m.iter().fold(String::new(), |mut acc, mode| {
                    acc.push_str(" ");
                    acc.push_str(&mode.to_string());
                    acc
                })
            ),
            Command::SERVICE(ref n, ref r, ref d, ref t, ref re, ref i) => {
                stringify("SERVICE", &[n, r, d, t, re], Some(i))
            }
            Command::QUIT(Some(ref m)) => stringify("QUIT", &[], Some(m)),
            Command::QUIT(None) => stringify("QUIT", &[], None),
            Command::SQUIT(ref s, ref c) => stringify("SQUIT", &[s], Some(c)),
            Command::JOIN(ref c, Some(ref k), Some(ref n)) => stringify("JOIN", &[c, k], Some(n)),
            Command::JOIN(ref c, Some(ref k), None) => stringify("JOIN", &[c, k], None),
            Command::JOIN(ref c, None, Some(ref n)) => stringify("JOIN", &[c], Some(n)),
            Command::JOIN(ref c, None, None) => stringify("JOIN", &[c], None),
            Command::PART(ref c, Some(ref m)) => stringify("PART", &[c], Some(m)),
            Command::PART(ref c, None) => stringify("PART", &[c], None),
            Command::ChannelMODE(ref u, ref m) => format!(
                "MODE {}{}",
                u,
                m.iter().fold(String::new(), |mut acc, mode| {
                    acc.push_str(" ");
                    acc.push_str(&mode.to_string());
                    acc
                })
            ),
            Command::TOPIC(ref c, Some(ref t)) => stringify("TOPIC", &[c], Some(t)),
            Command::TOPIC(ref c, None) => stringify("TOPIC", &[c], None),
            Command::NAMES(Some(ref c), Some(ref t)) => stringify("NAMES", &[c], Some(t)),
            Command::NAMES(Some(ref c), None) => stringify("NAMES", &[c], None),
            Command::NAMES(None, _) => stringify("NAMES", &[], None),
            Command::LIST(Some(ref c), Some(ref t)) => stringify("LIST", &[c], Some(t)),
            Command::LIST(Some(ref c), None) => stringify("LIST", &[c], None),
            Command::LIST(None, _) => stringify("LIST", &[], None),
            Command::INVITE(ref n, ref c) => stringify("INVITE", &[n, c], None),
            Command::KICK(ref c, ref n, Some(ref r)) => stringify("KICK", &[c, n], Some(r)),
            Command::KICK(ref c, ref n, None) => stringify("KICK", &[c, n], None),
            Command::PRIVMSG(ref t, ref m) => stringify("PRIVMSG", &[t], Some(m)),
            Command::NOTICE(ref t, ref m) => stringify("NOTICE", &[t], Some(m)),
            Command::MOTD(Some(ref t)) => stringify("MOTD", &[], Some(t)),
            Command::MOTD(None) => stringify("MOTD", &[], None),
            Command::LUSERS(Some(ref m), Some(ref t)) => stringify("LUSERS", &[m], Some(t)),
            Command::LUSERS(Some(ref m), None) => stringify("LUSERS", &[m], None),
            Command::LUSERS(None, _) => stringify("LUSERS", &[], None),
            Command::VERSION(Some(ref t)) => stringify("VERSION", &[], Some(t)),
            Command::VERSION(None) => stringify("VERSION", &[], None),
            Command::STATS(Some(ref q), Some(ref t)) => stringify("STATS", &[q], Some(t)),
            Command::STATS(Some(ref q), None) => stringify("STATS", &[q], None),
            Command::STATS(None, _) => stringify("STATS", &[], None),
            Command::LINKS(Some(ref r), Some(ref s)) => stringify("LINKS", &[r], Some(s)),
            Command::LINKS(None, Some(ref s)) => stringify("LINKS", &[], Some(s)),
            Command::LINKS(_, None) => stringify("LINKS", &[], None),
            Command::TIME(Some(ref t)) => stringify("TIME", &[], Some(t)),
            Command::TIME(None) => stringify("TIME", &[], None),
            Command::CONNECT(ref t, ref p, Some(ref r)) => stringify("CONNECT", &[t, p], Some(r)),
            Command::CONNECT(ref t, ref p, None) => stringify("CONNECT", &[t, p], None),
            Command::TRACE(Some(ref t)) => stringify("TRACE", &[], Some(t)),
            Command::TRACE(None) => stringify("TRACE", &[], None),
            Command::ADMIN(Some(ref t)) => stringify("ADMIN", &[], Some(t)),
            Command::ADMIN(None) => stringify("ADMIN", &[], None),
            Command::INFO(Some(ref t)) => stringify("INFO", &[], Some(t)),
            Command::INFO(None) => stringify("INFO", &[], None),
            Command::SERVLIST(Some(ref m), Some(ref t)) => stringify("SERVLIST", &[m], Some(t)),
            Command::SERVLIST(Some(ref m), None) => stringify("SERVLIST", &[m], None),
            Command::SERVLIST(None, _) => stringify("SERVLIST", &[], None),
            Command::SQUERY(ref s, ref t) => stringify("SQUERY", &[s, t], None),
            Command::WHO(Some(ref s), Some(true)) => stringify("WHO", &[s, "o"], None),
            Command::WHO(Some(ref s), _) => stringify("WHO", &[s], None),
            Command::WHO(None, _) => stringify("WHO", &[], None),
            Command::WHOIS(Some(ref t), ref m) => stringify("WHOIS", &[t, m], None),
            Command::WHOIS(None, ref m) => stringify("WHOIS", &[m], None),
            Command::WHOWAS(ref n, Some(ref c), Some(ref t)) => {
                stringify("WHOWAS", &[n, c], Some(t))
            }
            Command::WHOWAS(ref n, Some(ref c), None) => stringify("WHOWAS", &[n, c], None),
            Command::WHOWAS(ref n, None, _) => stringify("WHOWAS", &[n], None),
            Command::KILL(ref n, ref c) => stringify("KILL", &[n], Some(c)),
            Command::PING(ref s, Some(ref t)) => stringify("PING", &[s], Some(t)),
            Command::PING(ref s, None) => stringify("PING", &[], Some(s)),
            Command::PONG(ref s, Some(ref t)) => stringify("PONG", &[s], Some(t)),
            Command::PONG(ref s, None) => stringify("PONG", &[], Some(s)),
            Command::ERROR(ref m) => stringify("ERROR", &[], Some(m)),
            Command::AWAY(Some(ref m)) => stringify("AWAY", &[], Some(m)),
            Command::AWAY(None) => stringify("AWAY", &[], None),
            Command::REHASH => stringify("REHASH", &[], None),
            Command::DIE => stringify("DIE", &[], None),
            Command::RESTART => stringify("RESTART", &[], None),
            Command::SUMMON(ref u, Some(ref t), Some(ref c)) => {
                stringify("SUMMON", &[u, t], Some(c))
            }
            Command::SUMMON(ref u, Some(ref t), None) => stringify("SUMMON", &[u, t], None),
            Command::SUMMON(ref u, None, _) => stringify("SUMMON", &[u], None),
            Command::USERS(Some(ref t)) => stringify("USERS", &[], Some(t)),
            Command::USERS(None) => stringify("USERS", &[], None),
            Command::WALLOPS(ref t) => stringify("WALLOPS", &[], Some(t)),
            Command::USERHOST(ref u) => stringify(
                "USERHOST",
                &u.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                None,
            ),
            Command::ISON(ref u) => {
                stringify("ISON", &u.iter().map(|s| &s[..]).collect::<Vec<_>>(), None)
            }

            Command::SAJOIN(ref n, ref c) => stringify("SAJOIN", &[n, c], None),
            Command::SAMODE(ref t, ref m, Some(ref p)) => stringify("SAMODE", &[t, m, p], None),
            Command::SAMODE(ref t, ref m, None) => stringify("SAMODE", &[t, m], None),
            Command::SANICK(ref o, ref n) => stringify("SANICK", &[o, n], None),
            Command::SAPART(ref c, ref r) => stringify("SAPART", &[c], Some(r)),
            Command::SAQUIT(ref c, ref r) => stringify("SAQUIT", &[c], Some(r)),

            Command::NICKSERV(ref m) => stringify("NICKSERV", &[m], None),
            Command::CHANSERV(ref m) => stringify("CHANSERV", &[m], None),
            Command::OPERSERV(ref m) => stringify("OPERSERV", &[m], None),
            Command::BOTSERV(ref m) => stringify("BOTSERV", &[m], None),
            Command::HOSTSERV(ref m) => stringify("HOSTSERV", &[m], None),
            Command::MEMOSERV(ref m) => stringify("MEMOSERV", &[m], None),

            Command::CAP(None, ref s, None, Some(ref p)) => {
                stringify("CAP", &[s.to_str()], Some(p))
            }
            Command::CAP(None, ref s, None, None) => stringify("CAP", &[s.to_str()], None),
            Command::CAP(Some(ref k), ref s, None, Some(ref p)) => {
                stringify("CAP", &[k, s.to_str()], Some(p))
            }
            Command::CAP(Some(ref k), ref s, None, None) => {
                stringify("CAP", &[k, s.to_str()], None)
            }
            Command::CAP(None, ref s, Some(ref c), Some(ref p)) => {
                stringify("CAP", &[s.to_str(), c], Some(p))
            }
            Command::CAP(None, ref s, Some(ref c), None) => {
                stringify("CAP", &[s.to_str(), c], None)
            }
            Command::CAP(Some(ref k), ref s, Some(ref c), Some(ref p)) => {
                stringify("CAP", &[k, s.to_str(), c], Some(p))
            }
            Command::CAP(Some(ref k), ref s, Some(ref c), None) => {
                stringify("CAP", &[k, s.to_str(), c], None)
            }

            Command::AUTHENTICATE(ref d) => stringify("AUTHENTICATE", &[d], None),
            Command::ACCOUNT(ref a) => stringify("ACCOUNT", &[a], None),

            Command::METADATA(ref t, Some(ref c), None, Some(ref p)) => {
                stringify("METADATA", &[&t[..], c.to_str()], Some(p))
            }
            Command::METADATA(ref t, Some(ref c), None, None) => {
                stringify("METADATA", &[&t[..], c.to_str()], None)
            }

            Command::METADATA(ref t, Some(ref c), Some(ref a), Some(ref p)) => stringify(
                "METADATA",
                &vec![t, &c.to_str().to_owned()]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                Some(p),
            ),
            Command::METADATA(ref t, Some(ref c), Some(ref a), None) => stringify(
                "METADATA",
                &vec![t, &c.to_str().to_owned()]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                None,
            ),
            Command::METADATA(ref t, None, None, Some(ref p)) => {
                stringify("METADATA", &[t], Some(p))
            }
            Command::METADATA(ref t, None, None, None) => stringify("METADATA", &[t], None),
            Command::METADATA(ref t, None, Some(ref a), Some(ref p)) => stringify(
                "METADATA",
                &vec![t]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                Some(p),
            ),
            Command::METADATA(ref t, None, Some(ref a), None) => stringify(
                "METADATA",
                &vec![t]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                None,
            ),
            Command::MONITOR(ref c, Some(ref t)) => stringify("MONITOR", &[c, t], None),
            Command::MONITOR(ref c, None) => stringify("MONITOR", &[c], None),
            Command::BATCH(ref t, Some(ref c), Some(ref a)) => stringify(
                "BATCH",
                &vec![t, &c.to_str().to_owned()]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                None,
            ),
            Command::BATCH(ref t, Some(ref c), None) => stringify("BATCH", &[t, c.to_str()], None),
            Command::BATCH(ref t, None, Some(ref a)) => stringify(
                "BATCH",
                &vec![t]
                    .iter()
                    .map(|s| &s[..])
                    .chain(a.iter().map(|s| &s[..]))
                    .collect::<Vec<_>>(),
                None,
            ),
            Command::BATCH(ref t, None, None) => stringify("BATCH", &[t], None),
            Command::CHGHOST(ref u, ref h) => stringify("CHGHOST", &[u, h], None),

            Command::Response(ref resp, ref a, Some(ref s)) => stringify(
                &format!("{:03}", *resp as u16),
                &a.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                Some(s),
            ),
            Command::Response(ref resp, ref a, None) => stringify(
                &format!("{:03}", *resp as u16),
                &a.iter().map(|s| &s[..]).collect::<Vec<_>>(),
                None,
            ),
            Command::Raw(ref c, ref a, Some(ref s)) => {
                stringify(c, &a.iter().map(|s| &s[..]).collect::<Vec<_>>(), Some(s))
            }
            Command::Raw(ref c, ref a, None) => {
                stringify(c, &a.iter().map(|s| &s[..]).collect::<Vec<_>>(), None)
            }
        }
    }
}

impl Command {
    /// Constructs a new Command.
    pub fn new(
        cmd: &str,
        args: Vec<&str>,
        suffix: Option<&str>,
    ) -> Result<Command, MessageParseError> {
        Ok(if cmd.eq_ignore_ascii_case("PASS") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::PASS(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::PASS(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("NICK") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::NICK(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::NICK(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("USER") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 3 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::USER(args[0].to_owned(), args[1].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 4 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::USER(args[0].to_owned(), args[1].to_owned(), args[3].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("OPER") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::OPER(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::OPER(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("MODE") {
            match suffix {
                Some(suffix) => raw(cmd, args, Some(suffix)),
                None => {
                    if args[0].is_channel_name() {
                        let arg = args[1..].join(" ");
                        Command::ChannelMODE(args[0].to_owned(), Mode::as_channel_modes(&arg)?)
                    } else {
                        let arg = args[1..].join(" ");
                        Command::UserMODE(args[0].to_owned(), Mode::as_user_modes(&arg)?)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SERVICE") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 5 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SERVICE(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            args[2].to_owned(),
                            args[3].to_owned(),
                            args[4].to_owned(),
                            suffix.to_owned(),
                        )
                    }
                }
                None => {
                    if args.len() != 6 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SERVICE(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            args[2].to_owned(),
                            args[3].to_owned(),
                            args[4].to_owned(),
                            args[5].to_owned(),
                        )
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("QUIT") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::QUIT(Some(suffix.to_owned())),
                    None => Command::QUIT(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("SQUIT") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SQUIT(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SQUIT(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("JOIN") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::JOIN(suffix.to_owned(), None, None)
                    } else if args.len() == 1 {
                        Command::JOIN(args[0].to_owned(), Some(suffix.to_owned()), None)
                    } else if args.len() == 2 {
                        Command::JOIN(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(suffix.to_owned()),
                        )
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::JOIN(args[0].to_owned(), None, None)
                    } else if args.len() == 2 {
                        Command::JOIN(args[0].to_owned(), Some(args[1].to_owned()), None)
                    } else if args.len() == 3 {
                        Command::JOIN(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(args[2].to_owned()),
                        )
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("PART") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::PART(suffix.to_owned(), None)
                    } else if args.len() == 1 {
                        Command::PART(args[0].to_owned(), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::PART(args[0].to_owned(), None)
                    } else if args.len() == 2 {
                        Command::PART(args[0].to_owned(), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("TOPIC") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::TOPIC(suffix.to_owned(), None)
                    } else if args.len() == 1 {
                        Command::TOPIC(args[0].to_owned(), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::TOPIC(args[0].to_owned(), None)
                    } else if args.len() == 2 {
                        Command::TOPIC(args[0].to_owned(), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("NAMES") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::NAMES(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::NAMES(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::NAMES(None, None)
                    } else if args.len() == 1 {
                        Command::NAMES(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::NAMES(Some(args[0].to_owned()), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("LIST") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::LIST(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::LIST(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::LIST(None, None)
                    } else if args.len() == 1 {
                        Command::LIST(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::LIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("INVITE") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::INVITE(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::INVITE(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("KICK") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 2 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::KICK(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            Some(suffix.to_owned()),
                        )
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::KICK(args[0].to_owned(), args[1].to_owned(), None)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("PRIVMSG") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::PRIVMSG(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => raw(cmd, args, suffix),
            }
        } else if cmd.eq_ignore_ascii_case("NOTICE") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::NOTICE(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => raw(cmd, args, suffix),
            }
        } else if cmd.eq_ignore_ascii_case("MOTD") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::MOTD(Some(suffix.to_owned())),
                    None => Command::MOTD(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("LUSERS") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::LUSERS(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::LUSERS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::LUSERS(None, None)
                    } else if args.len() == 1 {
                        Command::LUSERS(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::LUSERS(Some(args[0].to_owned()), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("VERSION") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::VERSION(Some(suffix.to_owned())),
                    None => Command::VERSION(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("STATS") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::STATS(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::STATS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::STATS(None, None)
                    } else if args.len() == 1 {
                        Command::STATS(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::STATS(Some(args[0].to_owned()), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("LINKS") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::LINKS(None, Some(suffix.to_owned()))
                    } else if args.len() == 1 {
                        Command::LINKS(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::LINKS(None, None)
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("TIME") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::TIME(Some(suffix.to_owned())),
                    None => Command::TIME(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("CONNECT") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 2 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::CONNECT(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            Some(suffix.to_owned()),
                        )
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::CONNECT(args[0].to_owned(), args[1].to_owned(), None)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("TRACE") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::TRACE(Some(suffix.to_owned())),
                    None => Command::TRACE(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("ADMIN") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::ADMIN(Some(suffix.to_owned())),
                    None => Command::ADMIN(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("INFO") {
            if !args.is_empty() {
                raw(cmd, args, suffix)
            } else {
                match suffix {
                    Some(suffix) => Command::INFO(Some(suffix.to_owned())),
                    None => Command::INFO(None),
                }
            }
        } else if cmd.eq_ignore_ascii_case("SERVLIST") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::SERVLIST(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::SERVLIST(Some(args[0].to_owned()), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::SERVLIST(None, None)
                    } else if args.len() == 1 {
                        Command::SERVLIST(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::SERVLIST(Some(args[0].to_owned()), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SQUERY") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SQUERY(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SQUERY(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("WHO") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::WHO(Some(suffix.to_owned()), None)
                    } else if args.len() == 1 {
                        Command::WHO(Some(args[0].to_owned()), Some(&suffix[..] == "o"))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.is_empty() {
                        Command::WHO(None, None)
                    } else if args.len() == 1 {
                        Command::WHO(Some(args[0].to_owned()), None)
                    } else if args.len() == 2 {
                        Command::WHO(Some(args[0].to_owned()), Some(&args[1][..] == "o"))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("WHOIS") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::WHOIS(None, suffix.to_owned())
                    } else if args.len() == 1 {
                        Command::WHOIS(Some(args[0].to_owned()), suffix.to_owned())
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::WHOIS(None, args[0].to_owned())
                    } else if args.len() == 2 {
                        Command::WHOIS(Some(args[0].to_owned()), args[1].to_owned())
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("WHOWAS") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::WHOWAS(suffix.to_owned(), None, None)
                    } else if args.len() == 1 {
                        Command::WHOWAS(args[0].to_owned(), None, Some(suffix.to_owned()))
                    } else if args.len() == 2 {
                        Command::WHOWAS(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(suffix.to_owned()),
                        )
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::WHOWAS(args[0].to_owned(), None, None)
                    } else if args.len() == 2 {
                        Command::WHOWAS(args[0].to_owned(), None, Some(args[1].to_owned()))
                    } else if args.len() == 3 {
                        Command::WHOWAS(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(args[2].to_owned()),
                        )
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("KILL") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::KILL(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::KILL(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("PING") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::PING(suffix.to_owned(), None)
                    } else if args.len() == 1 {
                        Command::PING(args[0].to_owned(), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::PING(args[0].to_owned(), None)
                    } else if args.len() == 2 {
                        Command::PING(args[0].to_owned(), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("PONG") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::PONG(suffix.to_owned(), None)
                    } else if args.len() == 1 {
                        Command::PONG(args[0].to_owned(), Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::PONG(args[0].to_owned(), None)
                    } else if args.len() == 2 {
                        Command::PONG(args[0].to_owned(), Some(args[1].to_owned()))
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("ERROR") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::ERROR(suffix.to_owned())
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => raw(cmd, args, suffix),
            }
        } else if cmd.eq_ignore_ascii_case("AWAY") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::AWAY(Some(suffix.to_owned()))
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => raw(cmd, args, suffix),
            }
        } else if cmd.eq_ignore_ascii_case("REHASH") {
            if args.is_empty() {
                Command::REHASH
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("DIE") {
            if args.is_empty() {
                Command::DIE
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("RESTART") {
            if args.is_empty() {
                Command::RESTART
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("SUMMON") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::SUMMON(suffix.to_owned(), None, None)
                    } else if args.len() == 1 {
                        Command::SUMMON(args[0].to_owned(), Some(suffix.to_owned()), None)
                    } else if args.len() == 2 {
                        Command::SUMMON(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(suffix.to_owned()),
                        )
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::SUMMON(args[0].to_owned(), None, None)
                    } else if args.len() == 2 {
                        Command::SUMMON(args[0].to_owned(), Some(args[1].to_owned()), None)
                    } else if args.len() == 3 {
                        Command::SUMMON(
                            args[0].to_owned(),
                            Some(args[1].to_owned()),
                            Some(args[2].to_owned()),
                        )
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("USERS") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::USERS(Some(suffix.to_owned()))
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::USERS(Some(args[0].to_owned()))
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("WALLOPS") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::WALLOPS(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::WALLOPS(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("USERHOST") {
            if suffix.is_none() {
                Command::USERHOST(args.into_iter().map(|s| s.to_owned()).collect())
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("ISON") {
            if suffix.is_none() {
                Command::USERHOST(args.into_iter().map(|s| s.to_owned()).collect())
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("SAJOIN") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SAJOIN(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SAJOIN(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SAMODE") {
            match suffix {
                Some(suffix) => {
                    if args.len() == 1 {
                        Command::SAMODE(args[0].to_owned(), suffix.to_owned(), None)
                    } else if args.len() == 2 {
                        Command::SAMODE(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            Some(suffix.to_owned()),
                        )
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 2 {
                        Command::SAMODE(args[0].to_owned(), args[1].to_owned(), None)
                    } else if args.len() == 3 {
                        Command::SAMODE(
                            args[0].to_owned(),
                            args[1].to_owned(),
                            Some(args[2].to_owned()),
                        )
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SANICK") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SANICK(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SANICK(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SAPART") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SAPART(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SAPART(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("SAQUIT") {
            match suffix {
                Some(suffix) => {
                    if args.len() != 1 {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::SAQUIT(args[0].to_owned(), suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 2 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::SAQUIT(args[0].to_owned(), args[1].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("NICKSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::NICKSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::NICKSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("CHANSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::CHANSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::CHANSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("OPERSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::OPERSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::OPERSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("BOTSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::BOTSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::BOTSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("HOSTSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::HOSTSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::HOSTSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("MEMOSERV") {
            match suffix {
                Some(suffix) => {
                    if !args.is_empty() {
                        raw(cmd, args, Some(suffix))
                    } else {
                        Command::MEMOSERV(suffix.to_owned())
                    }
                }
                None => {
                    if args.len() != 1 {
                        raw(cmd, args, suffix)
                    } else {
                        Command::MEMOSERV(args[0].to_owned())
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("CAP") {
            if args.len() == 1 {
                if let Ok(cmd) = args[0].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(None, cmd, None, Some(suffix.to_owned())),
                        None => Command::CAP(None, cmd, None, None),
                    }
                } else {
                    raw(cmd, args, suffix)
                }
            } else if args.len() == 2 {
                if let Ok(cmd) = args[0].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(
                            None,
                            cmd,
                            Some(args[1].to_owned()),
                            Some(suffix.to_owned()),
                        ),
                        None => Command::CAP(None, cmd, Some(args[1].to_owned()), None),
                    }
                } else if let Ok(cmd) = args[1].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(
                            Some(args[0].to_owned()),
                            cmd,
                            None,
                            Some(suffix.to_owned()),
                        ),
                        None => Command::CAP(Some(args[0].to_owned()), cmd, None, None),
                    }
                } else {
                    raw(cmd, args, suffix)
                }
            } else if args.len() == 3 {
                if let Ok(cmd) = args[1].parse() {
                    match suffix {
                        Some(suffix) => Command::CAP(
                            Some(args[0].to_owned()),
                            cmd,
                            Some(args[2].to_owned()),
                            Some(suffix.to_owned()),
                        ),
                        None => Command::CAP(
                            Some(args[0].to_owned()),
                            cmd,
                            Some(args[2].to_owned()),
                            None,
                        ),
                    }
                } else {
                    raw(cmd, args, suffix)
                }
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("AUTHENTICATE") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::AUTHENTICATE(suffix.to_owned())
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::AUTHENTICATE(args[0].to_owned())
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("ACCOUNT") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::ACCOUNT(suffix.to_owned())
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::ACCOUNT(args[0].to_owned())
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("METADATA") {
            if args.len() == 2 {
                match suffix {
                    Some(_) => raw(cmd, args, suffix),
                    None => match args[1].parse() {
                        Ok(c) => Command::METADATA(args[0].to_owned(), Some(c), None, None),
                        Err(_) => raw(cmd, args, suffix),
                    },
                }
            } else if args.len() > 2 {
                match args[1].parse() {
                    Ok(c) => Command::METADATA(
                        args[0].to_owned(),
                        Some(c),
                        Some(args.into_iter().skip(1).map(|s| s.to_owned()).collect()),
                        suffix.map(|s| s.to_owned()),
                    ),
                    Err(_) => {
                        if args.len() == 3 && suffix.is_some() {
                            Command::METADATA(
                                args[0].to_owned(),
                                None,
                                Some(args.into_iter().skip(1).map(|s| s.to_owned()).collect()),
                                suffix.map(|s| s.to_owned()),
                            )
                        } else {
                            raw(cmd, args, suffix)
                        }
                    }
                }
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("MONITOR") {
            if args.len() == 1 {
                Command::MONITOR(args[0].to_owned(), suffix.map(|s| s.to_owned()))
            } else {
                raw(cmd, args, suffix)
            }
        } else if cmd.eq_ignore_ascii_case("BATCH") {
            match suffix {
                Some(suffix) => {
                    if args.is_empty() {
                        Command::BATCH(suffix.to_owned(), None, None)
                    } else if args.len() == 1 {
                        Command::BATCH(args[0].to_owned(), Some(suffix.parse().unwrap()), None)
                    } else if args.len() > 1 {
                        Command::BATCH(
                            args[0].to_owned(),
                            Some(args[1].parse().unwrap()),
                            Some(
                                vec![suffix.to_owned()]
                                    .into_iter()
                                    .chain(args.into_iter().skip(2).map(|s| s.to_owned()))
                                    .collect(),
                            ),
                        )
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 1 {
                        Command::BATCH(args[0].to_owned(), None, None)
                    } else if args.len() == 2 {
                        Command::BATCH(args[0].to_owned(), Some(args[1].parse().unwrap()), None)
                    } else if args.len() > 2 {
                        Command::BATCH(
                            args[0].to_owned(),
                            Some(args[1].parse().unwrap()),
                            Some(args.iter().skip(2).map(|&s| s.to_owned()).collect()),
                        )
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if cmd.eq_ignore_ascii_case("CHGHOST") {
            match suffix {
                Some(suffix) => {
                    if args.len() == 1 {
                        Command::CHGHOST(args[0].to_owned(), suffix.to_owned())
                    } else {
                        raw(cmd, args, Some(suffix))
                    }
                }
                None => {
                    if args.len() == 2 {
                        Command::CHGHOST(args[0].to_owned(), args[1].to_owned())
                    } else {
                        raw(cmd, args, suffix)
                    }
                }
            }
        } else if let Ok(resp) = cmd.parse() {
            Command::Response(
                resp,
                args.into_iter().map(|s| s.to_owned()).collect(),
                suffix.map(|s| s.to_owned()),
            )
        } else {
            raw(cmd, args, suffix)
        })
    }
}

/// Makes a raw message from the specified command, arguments, and suffix.
fn raw(cmd: &str, args: Vec<&str>, suffix: Option<&str>) -> Command {
    Command::Raw(
        cmd.to_owned(),
        args.into_iter().map(|s| s.to_owned()).collect(),
        suffix.map(|s| s.to_owned()),
    )
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
        match *self {
            BatchSubCommand::NETSPLIT => "NETSPLIT",
            BatchSubCommand::NETJOIN => "NETJOIN",
            BatchSubCommand::CUSTOM(ref s) => s,
        }
    }
}

impl FromStr for BatchSubCommand {
    type Err = MessageParseError;

    fn from_str(s: &str) -> Result<BatchSubCommand, Self::Err> {
        if s.eq_ignore_ascii_case("NETSPLIT") {
            Ok(BatchSubCommand::NETSPLIT)
        } else if s.eq_ignore_ascii_case("NETJOIN") {
            Ok(BatchSubCommand::NETJOIN)
        } else {
            Ok(BatchSubCommand::CUSTOM(s.to_uppercase()))
        }
    }
}

#[cfg(test)]
mod test {
    use super::Command;
    use super::Response;
    use crate::Message;

    #[test]
    fn format_response() {
        assert!(
            String::from(&Command::Response(
                Response::RPL_WELCOME,
                vec!["foo".into()],
                None
            )) == "001 foo"
        );
    }

    #[test]
    fn user_round_trip() {
        let cmd = Command::USER("a".to_string(), "b".to_string(), "c".to_string());
        let line = Message::from(cmd.clone()).to_string();
        let returned_cmd = line.parse::<Message>().unwrap().command;
        assert_eq!(cmd, returned_cmd);
    }

    #[test]
    fn parse_user_message() {
        let cmd = "USER a 0 * b".parse::<Message>().unwrap().command;
        assert_eq!(
            Command::USER("a".to_string(), "0".to_string(), "b".to_string()),
            cmd
        );
    }
}
