use serde::{Deserialize, Serialize};

use crate::generate_id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub tenant_id: String,
}

impl User {
    pub fn new(id: String, email: Option<String>, name: Option<String>) -> Self {
        Self {
            id,
            email,
            name,
            tenant_id: generate_id!("tenant"),
        }
    }

    pub fn get_display_name(&self) -> String {
        self.name
            .clone()
            .or_else(|| self.email.clone())
            .unwrap_or_else(|| self.id.clone())
    }

    pub fn is_email_valid(&self) -> bool {
        self.email
            .as_ref()
            .map_or(true, |email| email.contains('@') && email.contains('.'))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub tenant_id: String,
}

impl From<User> for UserClaims {
    fn from(user: User) -> Self {
        Self {
            sub: user.id,
            email: user.email,
            name: user.name,
            tenant_id: user.tenant_id,
        }
    }
}

impl From<UserClaims> for User {
    fn from(claims: UserClaims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            name: claims.name,
            tenant_id: claims.tenant_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation() {
        let user = User::new(
            generate_id!("user"),
            Some("test@example.com".to_string()),
            Some("Test User".to_string()),
        );

        assert!(user.id.starts_with("user"));
        assert_eq!(user.email, Some("test@example.com".to_string()));
        assert_eq!(user.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_user_claims_conversion() {
        let user = User::new(
            generate_id!("user"),
            Some("test@example.com".to_string()),
            Some("Test User".to_string()),
        );

        let claims: UserClaims = user.clone().into();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.name, user.name);
        assert_eq!(claims.tenant_id, user.tenant_id);
    }
}
