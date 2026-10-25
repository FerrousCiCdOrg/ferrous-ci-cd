//! User repository interface

use crate::domain::entities::user::{User, UserRole};
use crate::domain::value_objects::user_id::UserId;
use async_trait::async_trait;

/// User repository interface
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Save a user
    async fn save(&self, user: &User) -> crate::Result<()>;

    /// Find a user by ID
    async fn find_by_id(&self, id: &UserId) -> crate::Result<Option<User>>;

    /// Find a user by username
    async fn find_by_username(&self, username: &str) -> crate::Result<Option<User>>;

    /// Find a user by email
    async fn find_by_email(&self, email: &str) -> crate::Result<Option<User>>;

    /// Find all users
    async fn find_all(&self) -> crate::Result<Vec<User>>;

    /// Find users by role
    async fn find_by_role(&self, role: &UserRole) -> crate::Result<Vec<User>>;

    /// Find active users
    async fn find_active(&self) -> crate::Result<Vec<User>>;

    /// Update a user
    async fn update(&self, user: &User) -> crate::Result<()>;

    /// Delete a user
    async fn delete(&self, id: &UserId) -> crate::Result<()>;

    /// Check if a user exists
    async fn exists(&self, id: &UserId) -> crate::Result<bool>;

    /// Check if a username is taken
    async fn username_exists(&self, username: &str) -> crate::Result<bool>;

    /// Check if an email is taken
    async fn email_exists(&self, email: &str) -> crate::Result<bool>;
}
