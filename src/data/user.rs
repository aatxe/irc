//! Data for tracking user information.
use std::from_str::FromStr;

/// IRC User data.
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
    pub fn new(name: &str) -> User {
        let rank = from_str(name);
        User {
            name: if let Some(Member) = rank {
                name.into_string()
            } else {
                name[1..].into_string()
            },
            access_level: rank.unwrap(),
        }
    }

    /// Gets the user's access level.
    pub fn access_level(&self) -> AccessLevel {
        self.access_level
    }

    /// Updates the user's access level.
    pub fn update_access_level(&mut self, mode: &str) {
        self.access_level = match mode {
            "+q" => Owner,
            "-q" => Member,
            "+a" => Admin,
            "-a" => Member,
            "+o" => Oper,
            "-o" => Member,
            "+h" => HalfOp,
            "-h" => Member,
            "+v" => Voice,
            "-v" => Member,
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
        if s.len() == 0 { Some(Member) } else {
            Some(match s.char_at(0) {
                '~' => Owner,
                '&' => Admin,
                '@' => Oper,
                '%' => HalfOp,
                '+' => Voice,
                 _  => Member,
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::{AccessLevel, Admin, HalfOp, Member, Oper, Owner, User, Voice};

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
