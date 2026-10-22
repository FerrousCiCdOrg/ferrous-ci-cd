//! Project ID value object

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Project ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl ProjectId {
    /// Create a new Project ID
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

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ProjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ProjectId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ProjectId> for Uuid {
    fn from(id: ProjectId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_id_creation() {
        let id1 = ProjectId::new();
        let id2 = ProjectId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_project_id_parse() {
        let id = ProjectId::new();
        let id_str = id.to_string();

        let parsed = ProjectId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }
}
