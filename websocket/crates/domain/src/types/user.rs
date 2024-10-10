use crate::utils::generate_id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub tenant_id: String,
}

impl User {
    pub fn new(email: String, name: String, tenant_id: String) -> Self {
        Self {
            id: generate_id(14, "user"),
            email,
            name,
            tenant_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub email: String,
    pub name: String,
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
            "test@example.com".to_string(),
            "Test User".to_string(),
            "tenant123".to_string(),
        );

        assert!(user.id.starts_with("user"));
        assert_eq!(user.email, "test@example.com");
        assert_eq!(user.name, "Test User");
        assert_eq!(user.tenant_id, "tenant123");
    }

    #[test]
    fn test_user_claims_conversion() {
        let user = User::new(
            "test@example.com".to_string(),
            "Test User".to_string(),
            "tenant123".to_string(),
        );

        let claims: UserClaims = user.clone().into();
        assert_eq!(claims.sub, user.id);
        assert_eq!(claims.email, user.email);
        assert_eq!(claims.name, user.name);
        assert_eq!(claims.tenant_id, user.tenant_id);
    }
}
