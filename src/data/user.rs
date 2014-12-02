//! Data for tracking user information.
#![unstable]
use std::str::FromStr;

/// IRC User data.
#[unstable]
#[deriving(Clone, Show)]
pub struct User {
    /// The user's nickname.
    /// This is the only detail used in determining the equality of two users.
    name: String,
    /// The user's highest access level.
    highest_access_level: AccessLevel,
    access_levels: Vec<AccessLevel>,
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
            highest_access_level: rank.unwrap(),
            access_levels: if let Some(AccessLevel::Member) = rank {
                vec![rank.unwrap()]
            } else {
                vec![rank.unwrap(), AccessLevel::Member]
            }
        }
    }

    /// Gets the nickname of the user.
    #[stable]
    pub fn get_name(&self) -> &str {
        self.name[]
    }

    /// Gets the user's highest access level.
    #[experimental]
    pub fn highest_access_level(&self) -> AccessLevel {
        self.highest_access_level
    }

    /// Gets all the user's access levels.
    #[experimental]
    pub fn access_levels(&self) -> Vec<AccessLevel> {
        self.access_levels.clone()
    }

    /// Updates the user's access level.
    #[unstable]
    pub fn update_access_level(&mut self, mode: &str) {
        match mode {
            "+q" => self.add_access_level(AccessLevel::Owner),
            "-q" => self.sub_access_level(AccessLevel::Owner),
            "+a" => self.add_access_level(AccessLevel::Admin),
            "-a" => self.sub_access_level(AccessLevel::Admin),
            "+o" => self.add_access_level(AccessLevel::Oper),
            "-o" => self.sub_access_level(AccessLevel::Oper),
            "+h" => self.add_access_level(AccessLevel::HalfOp),
            "-h" => self.sub_access_level(AccessLevel::HalfOp),
            "+v" => self.add_access_level(AccessLevel::Voice),
            "-v" => self.sub_access_level(AccessLevel::Voice),
            _    => {},
       }
    }

    fn add_access_level(&mut self, level: AccessLevel) {
        if level > self.highest_access_level() {
            self.highest_access_level = level   
        }
        self.access_levels.push(level.clone())
    }

    fn sub_access_level(&mut self, level: AccessLevel) {
        if let Some(n) = self.access_levels[].position_elem(&level) {
            self.access_levels.swap_remove(n);
        }
        if level == self.highest_access_level() {
            self.highest_access_level = {
                let mut max = AccessLevel::Member;
                for level in self.access_levels.iter() {
                    if level > &max {
                        max = level.clone()
                    }
                }
                max
            }
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

impl PartialOrd for AccessLevel {
    fn partial_cmp(&self, other: &AccessLevel) -> Option<Ordering> {
        if self == other { return Some(Equal) }
        match self {
            &AccessLevel::Owner => Some(Greater),
            &AccessLevel::Admin => {
                if other == &AccessLevel::Owner {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            },
            &AccessLevel::Oper => {
                if other == &AccessLevel::Owner || other == &AccessLevel::Admin {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            },
            &AccessLevel::HalfOp => {
                if other == &AccessLevel::Voice || other == &AccessLevel::Member {
                    Some(Greater)
                } else {
                    Some(Less)
                }
            },
            &AccessLevel::Voice => {
                if other == &AccessLevel::Member {
                    Some(Greater)
                } else {
                    Some(Less)
                }
            },
            &AccessLevel::Member => Some(Less),
        }    
    }
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
            highest_access_level: Owner,
            access_levels: vec![Owner, Member],
        };
        assert_eq!(user, exp);
        assert_eq!(user.highest_access_level, exp.highest_access_level);
    }

    #[test]
    fn get_name() {
        let user = User::new("~owner");
        assert_eq!(user.get_name(), "owner");
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
        user.update_access_level("+q");
        assert_eq!(user.highest_access_level, Owner);
        user.update_access_level("-q");
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level("+a");
        assert_eq!(user.highest_access_level, Admin);
        user.update_access_level("-a");
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level("+o");
        assert_eq!(user.highest_access_level, Oper);
        user.update_access_level("-o");
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level("+h");
        assert_eq!(user.highest_access_level, HalfOp);
        user.update_access_level("-h");
        assert_eq!(user.highest_access_level, Member);
        user.update_access_level("+v");
        assert_eq!(user.highest_access_level, Voice);
        user.update_access_level("-v");
        assert_eq!(user.highest_access_level, Member);
    }

    #[test]
    fn derank_user_in_full() {
        let mut user = User::new("user");
        user.update_access_level("+q");
        user.update_access_level("+a");
        user.update_access_level("+o");
        user.update_access_level("+h");
        user.update_access_level("+v");
        assert_eq!(user.highest_access_level, Owner);
        assert_eq!(user.access_levels, vec![Member, Owner, Admin, Oper, HalfOp, Voice]);
        user.update_access_level("-h");
        assert_eq!(user.highest_access_level, Owner);
        assert_eq!(user.access_levels, vec![Member, Owner, Admin, Oper, Voice]);
        user.update_access_level("-q");
        assert_eq!(user.highest_access_level, Admin);
        assert_eq!(user.access_levels, vec![Member, Voice, Admin, Oper]);
        user.update_access_level("-a");
        assert_eq!(user.highest_access_level, Oper);
        assert_eq!(user.access_levels, vec![Member, Voice, Oper]);
        user.update_access_level("-o");
        assert_eq!(user.highest_access_level, Voice);
        assert_eq!(user.access_levels, vec![Member, Voice]);
        user.update_access_level("-v");
        assert_eq!(user.highest_access_level, Member);
        assert_eq!(user.access_levels, vec![Member]);
    }
}
