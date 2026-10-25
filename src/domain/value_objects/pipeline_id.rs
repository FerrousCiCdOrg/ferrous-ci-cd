//! Pipeline ID value object

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Pipeline ID value object
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PipelineId(Uuid);

impl PipelineId {
    /// Create a new Pipeline ID
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

impl Default for PipelineId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PipelineId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for PipelineId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<PipelineId> for Uuid {
    fn from(id: PipelineId) -> Self {
        id.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_id_creation() {
        let id1 = PipelineId::new();
        let id2 = PipelineId::new();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_pipeline_id_parse() {
        let id = PipelineId::new();
        let id_str = id.to_string();

        let parsed = PipelineId::parse(&id_str).unwrap();
        assert_eq!(id, parsed);
    }
}
