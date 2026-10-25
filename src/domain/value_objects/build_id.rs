//! Build ID value object

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Build ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BuildId(Uuid);

impl BuildId {
    /// Create a new Build ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Parse from a string
    pub fn parse(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Get the inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for BuildId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for BuildId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<BuildId> for Uuid {
    fn from(id: BuildId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_id_creation() {
        let id1 = BuildId::new();
        let id2 = BuildId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_build_id_parse() {
        let id = BuildId::new();
        let id_str = id.to_string();

        let parsed = BuildId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_build_id_display() {
        let id = BuildId::new();
        let displayed = format!("{}", id);

        assert!(!displayed.is_empty());
        assert_eq!(displayed.len(), 36); // UUID string length
    }
}
