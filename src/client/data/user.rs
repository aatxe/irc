//! Data for tracking user information.
use std::borrow::ToOwned;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::str::FromStr;

use crate::proto::{ChannelMode, Mode};

/// IRC User data.
#[derive(Clone, Debug)]
pub struct User {
    /// The user's nickname.
    nickname: String,
    /// The user's username.
    username: Option<String>,
    /// The user's hostname.
    hostname: Option<String>,
    /// The user's highest access level.
    highest_access_level: AccessLevel,
    /// All of the user's current access levels.
    access_levels: Vec<AccessLevel>,
}

impl User {
    /// Creates a new User.
    pub fn new(string: &str) -> User {
        let ranks: Vec<_> = AccessLevelIterator::new(string).collect();
        let mut state = &string[ranks.len()..];
        let nickname = state.find('!').map_or(state, |i| &state[..i]).to_owned();
        state = state.find('!').map_or("", |i| &state[i + 1..]);
        let username = state.find('@').map(|i| state[..i].to_owned());
        let hostname = state.find('@').map(|i| state[i + 1..].to_owned());
        User {
            nickname: nickname,
            username: username,
            hostname: hostname,
            access_levels: {
                let mut ranks = ranks.clone();
                ranks.push(AccessLevel::Member);
                ranks
            },
            highest_access_level: {
                let mut max = AccessLevel::Member;
                for rank in ranks {
                    if rank > max {
                        max = rank
                    }
                }
                max
            },
        }
    }

    /// Gets the nickname of the user.
    pub fn get_nickname(&self) -> &str {
        &self.nickname
    }

    /// Gets the username of the user, if it's known.
    /// This requires the IRCv3.2 extension `userhost-in-name`.
    pub fn get_username(&self) -> Option<&str> {
        self.username.as_ref().map(|s| &s[..])
    }

    /// Gets the hostname of the user, if it's known.
    /// This requires the IRCv3.2 extension `userhost-in-name`.
    pub fn get_hostname(&self) -> Option<&str> {
        self.hostname.as_ref().map(|s| &s[..])
    }

    /// Gets the user's highest access level.
    pub fn highest_access_level(&self) -> AccessLevel {
        self.highest_access_level
    }

    /// Gets all the user's access levels.
    pub fn access_levels(&self) -> Vec<AccessLevel> {
        self.access_levels.clone()
    }

    /// Updates the user's access level.
    pub fn update_access_level(&mut self, mode: &Mode<ChannelMode>) {
        match *mode {
            Mode::Plus(ChannelMode::Founder, _) => self.add_access_level(AccessLevel::Owner),
            Mode::Minus(ChannelMode::Founder, _) => self.sub_access_level(AccessLevel::Owner),
            Mode::Plus(ChannelMode::Admin, _) => self.add_access_level(AccessLevel::Admin),
            Mode::Minus(ChannelMode::Admin, _) => self.sub_access_level(AccessLevel::Admin),
            Mode::Plus(ChannelMode::Oper, _) => self.add_access_level(AccessLevel::Oper),
            Mode::Minus(ChannelMode::Oper, _) => self.sub_access_level(AccessLevel::Oper),
            Mode::Plus(ChannelMode::Halfop, _) => self.add_access_level(AccessLevel::HalfOp),
            Mode::Minus(ChannelMode::Halfop, _) => self.sub_access_level(AccessLevel::HalfOp),
            Mode::Plus(ChannelMode::Voice, _) => self.add_access_level(AccessLevel::Voice),
            Mode::Minus(ChannelMode::Voice, _) => self.sub_access_level(AccessLevel::Voice),
            _ => {}
        }
    }

    /// Adds an access level to the list, and updates the highest level if necessary.
    fn add_access_level(&mut self, level: AccessLevel) {
        if level > self.highest_access_level() {
            self.highest_access_level = level
        }
        self.access_levels.push(level)
    }

    /// Removes an access level from the list, and updates the highest level if necessary.
    fn sub_access_level(&mut self, level: AccessLevel) {
        if let Some(n) = self.access_levels.iter().position(|x| *x == level) {
            self.access_levels.swap_remove(n);
        }
        if level == self.highest_access_level() {
            self.highest_access_level = {
                let mut max = AccessLevel::Member;
                for level in &self.access_levels {
                    if level > &max {
                        max = *level
                    }
                }
                max
            }
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.nickname == other.nickname
            && self.username == other.username
            && self.hostname == other.hostname
    }
}

/// The user's access level.
#[derive(Copy, PartialEq, Clone, Debug)]
pub enum AccessLevel {
    /// The channel owner (~).
    Owner,
    /// A channel administrator (&).
    Admin,
    /// A channel operator (@),
    Oper,
    /// A channel half-oper (%),
    HalfOp,
    /// A user with voice (+),
    Voice,
    /// A normal user,
    Member,
}

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &AccessLevel) -> Option<Ordering> {
        if self == other {
            return Some(Equal);
        }
        match *self {
            AccessLevel::Owner => Some(Greater),
            AccessLevel::Admin => {
                if other == &AccessLevel::Owner {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            }
            AccessLevel::Oper => {
                if other == &AccessLevel::Owner || other == &AccessLevel::Admin {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            }
            AccessLevel::HalfOp => {
                if other == &AccessLevel::Voice || other == &AccessLevel::Member {
                    Some(Greater)
                } else {
                    Some(Less)
                }
            }
            AccessLevel::Voice => {
                if other == &AccessLevel::Member {
                    Some(Greater)
                } else {
                    Some(Less)
                }
            }
            AccessLevel::Member => Some(Less),
        }
    }
}

impl FromStr for AccessLevel {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<AccessLevel, &'static str> {
        match s.chars().next() {
            Some('~') => Ok(AccessLevel::Owner),
            Some('&') => Ok(AccessLevel::Admin),
            Some('@') => Ok(AccessLevel::Oper),
            Some('%') => Ok(AccessLevel::HalfOp),
            Some('+') => Ok(AccessLevel::Voice),
            None => Err("No access level in an empty string."),
            _ => Err("Failed to parse access level."),
        }
    }
}

/// An iterator used to parse access levels from strings.
struct AccessLevelIterator {
    value: String,
}

impl AccessLevelIterator {
    pub fn new(value: &str) -> AccessLevelIterator {
        AccessLevelIterator {
            value: value.to_owned(),
        }
    }
}

impl Iterator for AccessLevelIterator {
    type Item = AccessLevel;
    fn next(&mut self) -> Option<AccessLevel> {
        let ret = self.value.parse();
        if !self.value.is_empty() {
            self.value = self.value.chars().skip(1).collect()
        }
        ret.ok()
    }
}

#[cfg(test)]
mod test {
    use super::AccessLevel::*;
    use super::{AccessLevel, User};
    use crate::proto::ChannelMode as M;
    use crate::proto::Mode::*;

    #[test]
    fn parse_access_level() {
        assert!("member".parse::<AccessLevel>().is_err());
        assert_eq!("~owner".parse::<AccessLevel>().unwrap(), Owner);
        assert_eq!("&admin".parse::<AccessLevel>().unwrap(), Admin);
        assert_eq!("@oper".parse::<AccessLevel>().unwrap(), Oper);
        assert_eq!("%halfop".parse::<AccessLevel>().unwrap(), HalfOp);
        assert_eq!("+voice".parse::<AccessLevel>().unwrap(), Voice);
        assert!("".parse::<AccessLevel>().is_err());
    }

    #[test]
    fn create_user() {
        let user = User::new("~owner");
        let exp = User {
            nickname: format!("owner"),
            username: None,
            hostname: None,
            highest_access_level: Owner,
            access_levels: vec![Owner, Member],
        };
        assert_eq!(user, exp);
        assert_eq!(user.highest_access_level, exp.highest_access_level);
        assert_eq!(user.access_levels, exp.access_levels);
    }

    #[test]
    fn create_user_complex() {
        let user = User::new("~&+user");
        let exp = User {
            nickname: format!("user"),
            username: None,
            hostname: None,
            highest_access_level: Owner,
            access_levels: vec![Owner, Admin, Voice, Member],
        };
        assert_eq!(user, exp);
        assert_eq!(user.highest_access_level, exp.highest_access_level);
        assert_eq!(user.access_levels, exp.access_levels);
    }

    #[test]
    fn get_nickname() {
        let user = User::new("~owner");
        assert_eq!(user.get_nickname(), "owner");
    }

    #[test]
    fn get_username() {
        let user = User::new("user!username@hostname");
        assert_eq!(user.get_username(), Some("username"));
        let user = User::new("user");
        assert_eq!(user.get_username(), None);
    }

    #[test]
    fn get_hostname() {
        let user = User::new("user!username@hostname");
        assert_eq!(user.get_hostname(), Some("hostname"));
        let user = User::new("user");
        assert_eq!(user.get_hostname(), None);
    }

    #[test]
    fn access_level() {
        let user = User::new("~owner");
        assert_eq!(user.highest_access_level(), Owner);
    }

    #[test]
    fn update_user_rank() {
        let mut user = User::new("user");
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level(&Plus(M::Founder, None));
        assert_eq!(user.highest_access_level, Owner);
        user.update_access_level(&Minus(M::Founder, None));
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level(&Plus(M::Admin, None));
        assert_eq!(user.highest_access_level, Admin);
        user.update_access_level(&Minus(M::Admin, None));
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level(&Plus(M::Oper, None));
        assert_eq!(user.highest_access_level, Oper);
        user.update_access_level(&Minus(M::Oper, None));
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level(&Plus(M::Halfop, None));
        assert_eq!(user.highest_access_level, HalfOp);
        user.update_access_level(&Minus(M::Halfop, None));
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level(&Plus(M::Voice, None));
        assert_eq!(user.highest_access_level, Voice);
        user.update_access_level(&Minus(M::Voice, None));
        assert_eq!(user.highest_access_level, Member);
    }

    #[test]
    fn derank_user_in_full() {
        let mut user = User::new("~&@%+user");
        assert_eq!(user.highest_access_level, Owner);
        assert_eq!(
            user.access_levels,
            vec![Owner, Admin, Oper, HalfOp, Voice, Member]
        );
        user.update_access_level(&Minus(M::Halfop, None));
        assert_eq!(user.highest_access_level, Owner);
        assert_eq!(user.access_levels, vec![Owner, Admin, Oper, Member, Voice]);
        user.update_access_level(&Minus(M::Founder, None));
        assert_eq!(user.highest_access_level, Admin);
        assert_eq!(user.access_levels, vec![Voice, Admin, Oper, Member]);
        user.update_access_level(&Minus(M::Admin, None));
        assert_eq!(user.highest_access_level, Oper);
        assert_eq!(user.access_levels, vec![Voice, Member, Oper]);
        user.update_access_level(&Minus(M::Oper, None));
        assert_eq!(user.highest_access_level, Voice);
        assert_eq!(user.access_levels, vec![Voice, Member]);
        user.update_access_level(&Minus(M::Voice, None));
        assert_eq!(user.highest_access_level, Member);
        assert_eq!(user.access_levels, vec![Member]);
    }
}
