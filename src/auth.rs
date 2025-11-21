use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

pub const SESSION_ID: &str = "session_id";
pub struct SessionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Auth {
    Unauthenticated,
    Authenticated(AuthenticatedUser),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub subject: String,
    pub name: String,
    pub roles: HashSet<Role>,
}
impl Auth {
    pub fn is_authenticated(&self) -> bool {
        if let Auth::Authenticated(_) = self {
            true
        } else {
            false
        }
    }
    pub fn is_authenticated_guest(&self) -> bool {
        if let Auth::Authenticated(user) = self {
            tracing::info!("is guest: {:?}",&user.roles);
            user.roles.is_empty() || !user.has_any_role(&[Role::User,Role::Admin])
        } else {
            false
        }
    }
    pub fn is_authenticated_admin(&self) -> bool {
        if let Auth::Authenticated(user) = self {
            user.has_role(&Role::Admin)
        } else {
            false
        }
    }
}

impl AuthenticatedUser {
    /// Check if user has a specific role
    pub fn has_role(&self, role: &Role) -> bool {
        self.roles.contains(role)
    }

    /// Check if user has any of the specified roles
    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|role| self.roles.contains(role))
    }

    /// Check if user has all the specified roles
    pub fn has_all_roles(&self, roles: &[Role]) -> bool {
        roles.iter().all(|role| self.roles.contains(role))
    }

    /// Check if user is admin
    pub fn is_admin(&self) -> bool {
        self.has_role(&Role::Admin)
    }
}
impl Display for AuthenticatedUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User(sub:{}, name:{}, roles:{:?})",
            self.subject, self.name, self.roles
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Admin,
    User,
    Custom(String),
}
impl Role {
    pub fn from_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "admin" => Role::Admin,
            "user" => Role::User,
            _ => Role::Custom(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Role::Admin => "admin",
            Role::User => "user",
            Role::Custom(s) => s.as_str(),
        }
    }
}
