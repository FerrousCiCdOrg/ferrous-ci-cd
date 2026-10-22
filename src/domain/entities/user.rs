//! User entity - Represents a system user

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::user_id::UserId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier
    id: UserId,

    /// Username
    username: String,

    /// Email address
    email: String,

    /// Full name
    full_name: Option<String>,

    /// Password hash (bcrypt)
    #[serde(skip_serializing)]
    password_hash: String,

    /// User role
    role: UserRole,

    /// Whether the user is active
    active: bool,

    /// Last login time
    last_login: Option<DateTime<Utc>>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Domain events
    events: Vec<DomainEvent>,
}

/// User roles
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    /// Administrator with full access
    Admin,
    /// Developer with build and deploy permissions
    Developer,
    /// Viewer with read-only access
    Viewer,
    /// Service account for API access
    Service,
}

impl User {
    /// Create a new user
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        role: UserRole,
    ) -> crate::Result<Self> {
        // Validate username
        if username.is_empty() || username.len() < 3 {
            return Err(crate::Error::validation(
                "Username must be at least 3 characters",
            ));
        }

        // Validate email (basic check)
        if !email.contains('@') {
            return Err(crate::Error::validation("Invalid email address"));
        }

        let now = Utc::now();
        let id = UserId::new();

        let mut user = Self {
            id: id.clone(),
            username: username.clone(),
            email: email.clone(),
            full_name: None,
            password_hash,
            role: role.clone(),
            active: true,
            last_login: None,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        user.events.push(DomainEvent::UserCreated {
            user_id: id,
            username,
            email,
            role,
            created_at: now,
        });

        Ok(user)
    }

    /// Get the user ID
    pub fn id(&self) -> &UserId {
        &self.id
    }

    /// Get the username
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Get the email
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Get the user role
    pub fn role(&self) -> &UserRole {
        &self.role
    }

    /// Check if user is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Check if user has admin privileges
    pub fn is_admin(&self) -> bool {
        self.role == UserRole::Admin
    }

    /// Get the password hash
    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    /// Update the password hash
    pub fn update_password(&mut self, new_password_hash: String) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::UserPasswordChanged {
            user_id: self.id.clone(),
            changed_at: self.updated_at,
        });
    }

    /// Update the user's email
    pub fn update_email(&mut self, new_email: String) -> crate::Result<()> {
        if !new_email.contains('@') {
            return Err(crate::Error::validation("Invalid email address"));
        }

        self.email = new_email;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set the user's full name
    pub fn set_full_name(&mut self, full_name: String) {
        self.full_name = Some(full_name);
        self.updated_at = Utc::now();
    }

    /// Update the user's role
    pub fn update_role(&mut self, new_role: UserRole) {
        self.role = new_role;
        self.updated_at = Utc::now();
    }

    /// Deactivate the user
    pub fn deactivate(&mut self) {
        if self.active {
            self.active = false;
            self.updated_at = Utc::now();

            self.events.push(DomainEvent::UserDeactivated {
                user_id: self.id.clone(),
                deactivated_at: self.updated_at,
            });
        }
    }

    /// Activate the user
    pub fn activate(&mut self) {
        if !self.active {
            self.active = true;
            self.updated_at = Utc::now();
        }
    }

    /// Record a login
    pub fn record_login(&mut self) {
        self.last_login = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Get the domain events and clear them
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_user() {
        let user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hashed_password".to_string(),
            UserRole::Developer,
        )
        .unwrap();

        assert_eq!(user.username(), "testuser");
        assert_eq!(user.email(), "test@example.com");
        assert_eq!(user.role(), &UserRole::Developer);
        assert!(user.is_active());
        assert!(!user.is_admin());
        assert_eq!(user.events.len(), 1);
    }

    #[test]
    fn test_user_validation() {
        // Username too short
        let result = User::new(
            "ab".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::Developer,
        );
        assert!(result.is_err());

        // Invalid email
        let result = User::new(
            "testuser".to_string(),
            "invalid_email".to_string(),
            "hash".to_string(),
            UserRole::Developer,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_user_password_update() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "old_hash".to_string(),
            UserRole::Developer,
        )
        .unwrap();

        user.update_password("new_hash".to_string());
        assert_eq!(user.password_hash(), "new_hash");
    }

    #[test]
    fn test_user_deactivation() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::Developer,
        )
        .unwrap();

        assert!(user.is_active());

        user.deactivate();
        assert!(!user.is_active());

        user.activate();
        assert!(user.is_active());
    }

    #[test]
    fn test_user_role_update() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::Developer,
        )
        .unwrap();

        assert!(!user.is_admin());

        user.update_role(UserRole::Admin);
        assert_eq!(user.role(), &UserRole::Admin);
        assert!(user.is_admin());
    }

    #[test]
    fn test_user_login() {
        let mut user = User::new(
            "testuser".to_string(),
            "test@example.com".to_string(),
            "hash".to_string(),
            UserRole::Developer,
        )
        .unwrap();

        assert!(user.last_login.is_none());

        user.record_login();
        assert!(user.last_login.is_some());
    }
}
