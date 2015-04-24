//! Enumeration of all the possible server responses.
#![stable]
#![allow(non_camel_case_types)]
use std::mem::transmute;
use std::str::FromStr;
use client::data::message::Message;

/// List of all server responses as defined in [RFC 2812](http://tools.ietf.org/html/rfc2812).
/// All commands are documented with their expected form from the RFC.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u16)]
#[stable]
pub enum Response {
    // Expected replies
    /// 001 Welcome to the Internet Relay Network <nick>!<user>@<host>
    #[stable]
    RPL_WELCOME         = 001,
    /// 002 Your host is <servername>, running version <ver>
    #[stable]
    RPL_YOURHOST        = 002,
    /// 003 This server was created <date>
    #[stable]
    RPL_CREATED         = 003,
    /// 004 <servername> <version> <available user modes> available channel modes>
    #[stable]
    RPL_MYINFO          = 004,
    /// 005 Try server <server name>, port <port number>
    #[stable]
    RPL_BOUNCE          = 005,
    /// 302 :*1<reply> *( " " <reply> )
    #[stable]
    RPL_USERHOST        = 302,
    /// 303 :*1<nick> *( " " <nick> )
    #[stable]
    RPL_ISON            = 303,
    /// 301 <nick> :<away message>
    #[stable]
    RPL_AWAY            = 301,
    /// 305 :You are no longer marked as being away
    #[stable]
    RPL_UNAWAY          = 305,
    /// 306 :You have been marked as being away
    #[stable]
    RPL_NOWAWAY         = 306,
    /// 311 <nick> <user> <host> * :<real name>
    #[stable]
    RPL_WHOISUSER       = 311,
    /// 312 <nick> <server> :<server info>
    #[stable]
    RPL_WHOISSERVER     = 312,
    /// 313 <nick> :is an IRC operator
    #[stable]
    RPL_WHOISOPERATOR   = 313,
    /// 317 <nick> <integer> :seconds idle
    #[stable]
    RPL_WHOISIDLE       = 317,
    /// 318 <nick> :End of WHOIS list
    #[stable]
    RPL_ENDOFWHOIS      = 318,
    /// 319 <nick> :*( ( "@" / "+" ) <channel> " " )
    #[stable]
    RPL_WHOISCHANNELS   = 319,
    /// 314 <nick> <user> <host> * :<real name>
    #[stable]
    RPL_WHOWASUSER      = 314,
    /// 369 <nick> :End of WHOWAS
    #[stable]
    RPL_ENDOFWHOWAS     = 369,
    /// Obsolete. Not used.
    #[stable]
    RPL_LISTSTART       = 321,
    /// 322 <channel> <# visible> :<topic>
    #[stable]
    RPL_LIST            = 322,
    /// 323 :End of LIST
    #[stable]
    RPL_LISTEND         = 323,
    /// 325 <channel> <nickname>
    #[stable]
    RPL_UNIQOPIS        = 325,
    /// 324 <channel> <mode> <mode params>
    #[stable]
    RPL_CHANNELMODEIS   = 324,
    /// 331 <channel> :No topic is set
    #[stable]
    RPL_NOTOPIC         = 331,
    /// 332 <channel> :<topic>
    #[stable]
    RPL_TOPIC           = 332,
    /// 341 <channel> <nick>
    #[stable]
    RPL_INVITING        = 341,
    /// 342 <user> :Summoning user to IRC
    #[stable]
    RPL_SUMMONING       = 342,
    /// 346 <channel> <invitemask>
    #[stable]
    RPL_INVITELIST      = 346,
    /// 347 <channel> :End of channel invite list
    #[stable]
    RPL_ENDOFINVITELIST = 347,
    /// 348 <channel> <exceptionmask>
    #[stable]
    RPL_EXCEPTLIST      = 348,
    /// 349 <channel> :End of channel exception list
    #[stable]
    RPL_ENDOFEXECPTLIST = 349,
    /// 351 <version>.<debuglevel> <server> :<comments>
    #[stable]
    RPL_VERSION         = 351,
    /// 352 <channel> <user> <host> <server> <nick> ( "H" / "G" > ["*"] [ ( "@" / "+" ) ] 
    #[stable]
    /// :<hopcount> <real name>
    #[stable]
    RPL_WHOREPLY        = 352,
    /// 315 <name> :End of WHO list
    #[stable]
    RPL_ENDOFWHO        = 315,
    /// 353 ( "=" / "*" / "@" ) <channel> :[ "@" / "+" ] <nick> *( " " [ "@" / "+" ] <nick> )
    #[stable]
    RPL_NAMREPLY        = 353,
    /// 366 <channel> :End of NAMES list
    #[stable]
    RPL_ENDOFNAMES      = 366,
    /// 364 <mask> <server> :<hopcount> <server info>
    #[stable]
    RPL_LINKS           = 364,
    /// 365 <mask> :End of LINKS list
    #[stable]
    RPL_ENDOFLINKS      = 365,
    /// 367 <channel> <banmask>
    #[stable]
    RPL_BANLIST         = 367,
    /// 368 <channel> :End of channel ban list
    #[stable]
    RPL_ENDOFBANLIST    = 368,
    /// 371 :<string>
    #[stable]
    RPL_INFO            = 371,
    /// 374 :End of INFO list
    #[stable]
    RPL_ENDOFINFO       = 374,
    /// 375 :- <server> Message of the day - 
    #[stable]
    RPL_MOTDSTART       = 375,
    /// 372 :- <text>
    #[stable]
    RPL_MOTD            = 372,
    /// 376 :End of MOTD command
    #[stable]
    RPL_ENDOFMOTD       = 376,
    /// 381 :You are now an IRC operator
    #[stable]
    RPL_YOUREOPER       = 381,
    /// 382 <config file> :Rehashing
    #[stable]
    RPL_REHASHING       = 382,
    /// 383 You are service <servicename>
    #[stable]
    RPL_YOURESERVICE    = 383,
    /// 391 <server> :<string showing server's local time>
    #[stable]
    RPL_TIME            = 391,
    /// 392 :UserID   Terminal  Host
    #[stable]
    RPL_USERSSTART      = 392,
    /// 393 :<username> <ttyline> <hostname>
    #[stable]
    RPL_USERS           = 393,
    /// 394 :End of users
    #[stable]
    RPL_ENDOFUSERS      = 394,
    /// 395 :Nobody logged in
    #[stable]
    RPL_NOUSERS         = 395,
    /// 200 Link <version & debug level> <destination> <next server> V<protocol version>
    /// <link uptime in seconds> <backstream sendq> <upstream sendq>
    #[stable]
    RPL_TRACELINK       = 200,
    /// 201 Try. <class> <server>
    #[stable]
    RPL_TRACECONNECTING = 201,
    /// 202 H.S. <class> <server>
    #[stable]
    RPL_TRACEHANDSHAKE  = 202,
    /// 203 ???? <class> [<client IP address in dot form>]
    #[stable]
    RPL_TRACEUKNOWN     = 203,
    /// 204 Oper <class> <nick>
    #[stable]
    RPL_TRACEOPERATOR   = 204,
    /// 205 User <class> <nick>
    #[stable]
    RPL_TRACEUSER       = 205,
    /// 206 Serv <class> <int>S <int>C <server> <nick!user|*!*>@<host|server> V<protocol version>
    #[stable]
    RPL_TRACESERVER     = 206,
    /// 207 Service <class> <name> <type> <active type>
    #[stable]
    RPL_TRACESERVICE    = 207,
    /// 208 <newtype> 0 <client name>
    #[stable]
    RPL_TRACENEWTYPE    = 208,
    /// 209 Class <class> <count>
    #[stable]
    RPL_TRACECLASS      = 209,
    /// Unused.
    RPL_TRACERECONNECT  = 210,
    /// 261 File <logfile> <debug level>
    #[stable]
    RPL_TRACELOG        = 261,
    /// 262 <server name> <version & debug level> :End of TRACE
    #[stable]
    RPL_TRACEEND        = 262,
    /// 211 <linkname> <sendq> <sent messages> <sent Kbytes> <received messages> <received Kbytes>
    /// <time open>
    #[stable]
    RPL_STATSLINKINFO   = 211,
    /// 212 <command> <count> <byte count> <remote count>
    #[stable]
    RPL_STATSCOMMANDS   = 212,
    /// 219 <stats letter> :End of STATS report
    #[stable]
    RPL_ENDOFSTATS      = 219,
    /// 242 :Server Up %d days %d:%02d:%02d
    #[stable]
    RPL_STATSUPTIME     = 242,
    /// O <hostmask> * <name>
    #[stable]
    RPL_STATSOLINE      = 243,
    /// 221 <user mode string>
    #[stable]
    RPL_UMODEIS         = 221,
    /// 234 <name> <server> <mask> <type> <hopcount> <info>
    #[stable]
    RPL_SERVLIST        = 234,
    /// 235 <mask> <type> :End of service listing
    #[stable]
    RPL_SERVLISTEND     = 235,
    /// 251 :There are <integer> users and <integer> services on <integer> servers
    #[stable]
    RPL_LUSERCLIENT     = 251,
    /// 252 <integer> :operator(s) online
    #[stable]
    RPL_LUSEROP         = 252,
    /// 253 <integer> :unknown connection(s)
    #[stable]
    RPL_LUSERUNKNOWN    = 253,
    /// 254 <integer> :channels formed
    #[stable]
    RPL_LUSERCHANNELS   = 254,
    /// 255 :I have <integer> clients and <integer> servers
    #[stable]
    RPL_LUSERME         = 255,
    /// 256 <server> :Administrative info
    #[stable]
    RPL_ADMINME         = 256,
    /// 257 :<admin info>
    #[stable]
    RPL_ADMINLOC1       = 257,
    /// 258 :<admin info>
    #[stable]
    RPL_ADMINLOC2       = 258,
    /// 259 :<admin info>
    #[stable]
    RPL_ADMINEMAIL      = 259,
    /// 263 <command> :Please wait a while and try again.
    #[stable]
    RPL_TRYAGAIN        = 263,

    // Error replies
    /// 401 <nickname> :No such nick/channel
    #[stable]
    ERR_NOSUCHNICK          = 401,
    /// 402 <server name> :No such server
    #[stable]
    ERR_NOSUCHSERVER        = 402,
    /// 403 <channel name> :No such channel
    #[stable]
    ERR_NOSUCHCHANNEL       = 403,
    /// 404 <channel name> :Cannot send to channel
    #[stable]
    ERR_CANNOTSENDTOCHAN    = 404,
    /// 405 <channel name> :You have joined too many channels
    #[stable]
    ERR_TOOMANYCHANNELS     = 405,
    /// 406 <nickname> :There was no such nickname
    #[stable]
    ERR_WASNOSUCHNICK       = 406,
    /// 407 <target> :<error code> recipients. <abort message>
    #[stable]
    ERR_TOOMANYTARGETS      = 407,
    /// 408 <service name> :No such service
    #[stable]
    ERR_NOSUCHSERVICE       = 408,
    /// 409 :No origin specified
    #[stable]
    ERR_NOORIGIN            = 409,
    /// 411 :No recipient given (<command>)
    #[stable]
    ERR_NORECIPIENT         = 411,
    /// 412 :No text to send
    #[stable]
    ERR_NOTEXTTOSEND        = 412,
    /// 413 <mask> :No toplevel domain specified
    #[stable]
    ERR_NOTOPLEVEL          = 413,
    /// 414 <mask> :Wildcard in toplevel domain
    #[stable]
    ERR_WILDTOPLEVEL        = 414,
    /// 415 <mask> :Bad Server/host mask
    #[stable]
    ERR_BADMASK             = 415,
    /// 421 <command> :Unknown command
    #[stable]
    ERR_UNKNOWNCOMMAND      = 421,
    /// 422 :MOTD File is missing
    #[stable]
    ERR_NOMOTD              = 422,
    /// 423 <server> :No administrative info available
    #[stable]
    ERR_NOADMININFO         = 423,
    /// 424 :File error doing <file op> on <file>
    #[stable]
    ERR_FILEERROR           = 424,
    /// 431 :No nickname given
    #[stable]
    ERR_NONICKNAMEGIVEN     = 431,
    /// 432 <nick> :Erroneous nickname"
    #[stable]
    ERR_ERRONEOUSNICKNAME   = 432,
    /// 433 <nick> :Nickname is already in use
    #[stable]
    ERR_NICKNAMEINUSE       = 433,
    /// 436 <nick> :Nickname collision KILL from <user>@<host>
    #[stable]
    ERR_NICKCOLLISION       = 436,
    /// 437 <nick/channel> :Nick/channel is temporarily unavailable
    #[stable]
    ERR_UNAVAILRESOURCE     = 437,
    /// 441 <nick> <channel> :They aren't on that channel
    #[stable]
    ERR_USERNOTINCHANNEL    = 441,
    /// 442 <channel> :You're not on that channel
    #[stable]
    ERR_NOTONCHANNEL        = 442,
    /// 443 <user> <channel> :is already on channel
    #[stable]
    ERR_USERONCHANNEL       = 443,
    /// 444 <user> :User not logged in
    #[stable]
    ERR_NOLOGIN             = 444,
    /// 445 :SUMMON has been disabled
    #[stable]
    ERR_SUMMONDISABLED      = 445,
    /// 446 :USERS has been disabled
    #[stable]
    ERR_USERSDISABLED       = 446,
    /// 451 :You have not registered
    #[stable]
    ERR_NOTREGISTERED       = 451,
    /// 461 <command> :Not enough parameters
    #[stable]
    ERR_NEEDMOREPARAMS      = 461,
    /// 462 :Unauthorized command (already registered)
    #[stable]
    ERR_ALREADYREGISTRED    = 462,
    /// 463 :Your host isn't among the privileged
    #[stable]
    ERR_NOPERMFORHOST       = 463,
    /// 464 :Password incorrect
    #[stable]
    ERR_PASSWDMISMATCH      = 464,
    /// 465 :You are banned from this server
    #[stable]
    ERR_YOUREBANNEDCREEP    = 465,
    /// 466
    #[stable]
    ERR_YOUWILLBEBANNED     = 466,
    /// 467 <channel> :Channel key already set
    #[stable]
    ERR_KEYSET              = 467,
    /// 471 <channel> :Cannot join channel (+l)
    #[stable]
    ERR_CHANNELISFULL       = 471,
    /// 472 <char> :is unknown mode char to me for <channel>
    #[stable]
    ERR_UNKNOWNMODE         = 472,
    /// 473 <channel> :Cannot join channel (+i)
    #[stable]
    ERR_INVITEONLYCHAN      = 473,
    /// 474 <channel> :Cannot join channel (+b)
    #[stable]
    ERR_BANNEDFROMCHAN      = 474,
    /// 475 <channel> :Cannot join channel (+k)
    #[stable]
    ERR_BADCHANNELKEY       = 475,
    /// 476 <channel> :Bad Channel Mask
    #[stable]
    ERR_BADCHANMASK         = 476,
    /// 477 <channel> :Channel doesn't support modes
    #[stable]
    ERR_NOCHANMODES         = 477,
    /// 478 <channel> <char> :Channel list is full
    #[stable]
    ERR_BANLISTFULL         = 478,
    /// 481 :Permission Denied- You're not an IRC operator
    #[stable]
    ERR_NOPRIVILEGES        = 481,
    /// 482 <channel> :You're not channel operator
    #[stable]
    ERR_CHANOPRIVSNEEDED    = 482,
    /// 483 :You can't kill a server!
    #[stable]
    ERR_CANTKILLSERVER      = 483,
    /// 484 :Your connection is restricted!
    #[stable]
    ERR_RESTRICTED          = 484,
    /// 485 :You're not the original channel operator
    #[stable]
    ERR_UNIQOPPRIVSNEEDED   = 485,
    /// 491 :No O-lines for your host
    #[stable]
    ERR_NOOPERHOST          = 491,
    /// 501 :Unknown MODE flag
    #[stable]
    ERR_UMODEUNKNOWNFLAG    = 501,
    /// 502 :Cannot change mode for other users
    #[stable]
    ERR_USERSDONTMATCH      = 502,
}

#[stable]
impl Response {
    /// Gets a response from a message.
    #[stable]
    pub fn from_message(m: &Message) -> Option<Response> { 
        m.command.parse().ok()
    }

    /// Determines whether or not this response is an error response.
    #[stable]
    pub fn is_error(&self) -> bool {
        *self as u16 >= 400
    }
}

impl FromStr for Response {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Response, &'static str> {
        if let Ok(rc) = s.parse::<u16>() {
            // This wall of text was brought to you by the removal of FromPrimitive.
            if (rc > 0 && rc < 5) || (rc > 200 && rc < 213) || rc == 219 || rc == 221 || rc == 234
               || rc == 235 || rc == 242 || rc == 243 || (rc > 250 && rc < 260) || 
               (rc > 260 && rc < 264) || (rc > 300 && rc < 307) || 
               (rc > 310 && rc < 326 && rc != 320) || rc == 331 || rc == 332 || rc == 341 || 
               rc == 342 || (rc > 345 && rc < 354 && rc != 350) || 
               (rc > 363 && rc < 377 && rc != 370) || (rc > 380 && rc < 384) || 
               (rc > 390 && rc < 396) || (rc > 400 && rc < 415 && rc != 410) || 
               (rc > 420 && rc < 425) || (rc > 430 && rc < 434) || rc == 436 || rc == 437 ||
               (rc > 440 && rc < 447) || rc == 451 || (rc > 460 && rc < 468) ||
               (rc > 470 && rc < 479) || (rc > 480 && rc < 486) || rc == 491 || rc == 501 ||
               rc == 502 {
                Ok(unsafe { transmute(rc) })
            } else {
                Err("Failed to parse due to unknown response code.")
            }
        } else {
            Err("Failed to parse response code.")
        }   
    }
}

#[cfg(test)]
mod test {
    use super::Response;
    use client::data::message::ToMessage;

    #[test]
    fn from_message() {
        assert_eq!(Response::from_message(
            &":irc.test.net 353 test = #test :test\r\n".to_message()
        ).unwrap(), Response::RPL_NAMREPLY);
        assert_eq!(Response::from_message(
            &":irc.test.net 433 <nick> :Nickname is already in use\r\n".to_message()
        ).unwrap(), Response::ERR_NICKNAMEINUSE);
    }

    #[test]
    fn is_error() {
        assert!(!Response::RPL_NAMREPLY.is_error());
        assert!(Response::ERR_NICKNAMEINUSE.is_error());
    }
}
