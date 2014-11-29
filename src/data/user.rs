//! Data for tracking user information.
#![unstable]
use std::str::FromStr;

/// IRC User data.
#[unstable]
#[deriving(Clone, Show)]
pub struct User {
    /// The user's nickname.
    name: String,
    /// The user's access level.
    /// For simplicity, this is not used to determine the equality of two users.
    /// That is a user is equal if and only if their nickname is the same.
    access_level: AccessLevel,
}

impl User {
    /// Creates a new User.
    #[stable]
    pub fn new(name: &str) -> User {
        let rank = from_str(name);
        User {
            name: if let Some(AccessLevel::Member) = rank {
                name.into_string()
            } else {
                name[1..].into_string()
            },
            access_level: rank.unwrap(),
        }
    }

    /// Gets the nickname of the user.
    #[stable]
    pub fn get_name(&self) -> &str {
        self.name[]
    }

    /// Gets the user's access level.
    #[experimental]
    pub fn access_level(&self) -> AccessLevel {
        self.access_level
    }

    /// Updates the user's access level.
    #[unstable]
    pub fn update_access_level(&mut self, mode: &str) {
        self.access_level = match mode {
            "+q" => AccessLevel::Owner,
            "-q" => AccessLevel::Member,
            "+a" => AccessLevel::Admin,
            "-a" => AccessLevel::Member,
            "+o" => AccessLevel::Oper,
            "-o" => AccessLevel::Member,
            "+h" => AccessLevel::HalfOp,
            "-h" => AccessLevel::Member,
            "+v" => AccessLevel::Voice,
            "-v" => AccessLevel::Member,
            _ => self.access_level,
        }
    }
}

impl PartialEq for User {
    fn eq(&self, other: &User) -> bool {
        self.name == other.name
    }
}

/// The user's access level.
#[stable]
#[deriving(PartialEq, Clone, Show)]
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

impl FromStr for AccessLevel {
    fn from_str(s: &str) -> Option<AccessLevel> {
        if s.len() == 0 { Some(AccessLevel::Member) } else {
            Some(match s.char_at(0) {
                '~' => AccessLevel::Owner,
                '&' => AccessLevel::Admin,
                '@' => AccessLevel::Oper,
                '%' => AccessLevel::HalfOp,
                '+' => AccessLevel::Voice,
                 _  => AccessLevel::Member,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AccessLevel, User};
    use super::AccessLevel::{Admin, HalfOp, Member, Oper, Owner, Voice};

    #[test]
    fn access_level_from_str() {
        assert_eq!(from_str::<AccessLevel>("member").unwrap(), Member);
        assert_eq!(from_str::<AccessLevel>("~owner").unwrap(), Owner);
        assert_eq!(from_str::<AccessLevel>("&admin").unwrap(), Admin);
        assert_eq!(from_str::<AccessLevel>("@oper").unwrap(), Oper);
        assert_eq!(from_str::<AccessLevel>("%halfop").unwrap(), HalfOp);
        assert_eq!(from_str::<AccessLevel>("+voice").unwrap(), Voice);
        assert_eq!(from_str::<AccessLevel>("").unwrap(), Member);
    }

    #[test]
    fn create_user() {
        let user = User::new("~owner");
        let exp = User {
            name: format!("owner"),
            access_level: Owner,
        };
        assert_eq!(user, exp);
        assert_eq!(user.access_level, exp.access_level);
    }

    #[test]
    fn get_name() {
        let user = User::new("~owner");
        assert_eq!(user.get_name(), "owner");
    }

    #[test]
    fn access_level() {
        let user = User::new("~owner");
        assert_eq!(user.access_level(), Owner);
    }

    #[test]
    fn update_user_rank() {
        let mut user = User::new("user");
        assert_eq!(user.access_level, Member);
        user.update_access_level("+q");
        assert_eq!(user.access_level, Owner);
        user.update_access_level("-q");
        assert_eq!(user.access_level, Member);
        user.update_access_level("+a");
        assert_eq!(user.access_level, Admin);
        user.update_access_level("-a");
        assert_eq!(user.access_level, Member);
        user.update_access_level("+o");
        assert_eq!(user.access_level, Oper);
        user.update_access_level("-o");
        assert_eq!(user.access_level, Member);
        user.update_access_level("+h");
        assert_eq!(user.access_level, HalfOp);
        user.update_access_level("-h");
        assert_eq!(user.access_level, Member);
        user.update_access_level("+v");
        assert_eq!(user.access_level, Voice);
        user.update_access_level("-v");
        assert_eq!(user.access_level, Member);
    }
}
