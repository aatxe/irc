//! Data for tracking user information.
use std::from_str::FromStr;

/// IRC User data.
#[deriving(PartialEq, Clone, Show)]
pub struct User {
    /// The user's nickname.
    name: String,
    /// The user's access level.
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
}

///
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
        if s.len() == 0 { None } else {
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

}
