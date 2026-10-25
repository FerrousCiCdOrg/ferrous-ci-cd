//! Stage ID value object

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Stage ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StageId(Uuid);

impl StageId {
    /// Create a new Stage ID
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

impl Default for StageId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for StageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for StageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<StageId> for Uuid {
    fn from(id: StageId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_id_creation() {
        let id1 = StageId::new();
        let id2 = StageId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_stage_id_parse() {
        let id = StageId::new();
        let id_str = id.to_string();

        let parsed = StageId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }
}
