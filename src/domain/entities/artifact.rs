//! Artifact entity - Represents build artifacts

use crate::domain::value_objects::{artifact_id::ArtifactId, build_id::BuildId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Artifact entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier
    id: ArtifactId,

    /// Build this artifact belongs to
    build_id: BuildId,

    /// Artifact name
    name: String,

    /// Artifact path
    path: String,

    /// Artifact type
    artifact_type: ArtifactType,

    /// File size in bytes
    size: u64,

    /// Checksum (SHA256)
    checksum: String,

    /// Content type / MIME type
    content_type: String,

    /// Whether the artifact is expired
    expired: bool,

    /// Expiration date
    expires_at: Option<DateTime<Utc>>,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last access timestamp
    accessed_at: DateTime<Utc>,
}

/// Artifact type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ArtifactType {
    /// Build output (binaries, libraries, etc.)
    BuildOutput,
    /// Test results
    TestResults,
    /// Code coverage reports
    Coverage,
    /// Documentation
    Documentation,
    /// Container image
    ContainerImage,
    /// Archive (zip, tar, etc.)
    Archive,
    /// Log files
    Logs,
    /// Other
    Other,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(
        build_id: BuildId,
        name: String,
        path: String,
        size: u64,
        checksum: String,
        artifact_type: ArtifactType,
    ) -> Self {
        let now = Utc::now();

        Self {
            id: ArtifactId::new(),
            build_id,
            name,
            path,
            artifact_type,
            size,
            checksum,
            content_type: "application/octet-stream".to_string(),
            expired: false,
            expires_at: None,
            created_at: now,
            accessed_at: now,
        }
    }

    /// Get the artifact ID
    pub fn id(&self) -> &ArtifactId {
        &self.id
    }

    /// Get the artifact name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the artifact path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the artifact size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Get the checksum
    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    /// Check if the artifact is expired
    pub fn is_expired(&self) -> bool {
        if self.expired {
            return true;
        }

        if let Some(expires_at) = self.expires_at {
            return Utc::now() > expires_at;
        }

        false
    }

    /// Set content type
    pub fn set_content_type(&mut self, content_type: String) {
        self.content_type = content_type;
    }

    /// Set expiration date
    pub fn set_expiration(&mut self, expires_at: DateTime<Utc>) {
        self.expires_at = Some(expires_at);
    }

    /// Mark artifact as expired
    pub fn expire(&mut self) {
        self.expired = true;
    }

    /// Record access to the artifact
    pub fn record_access(&mut self) {
        self.accessed_at = Utc::now();
    }

    /// Validate the artifact
    pub fn validate(&self) -> crate::Result<()> {
        if self.name.is_empty() {
            return Err(crate::Error::validation("Artifact name cannot be empty"));
        }

        if self.path.is_empty() {
            return Err(crate::Error::validation("Artifact path cannot be empty"));
        }

        if self.checksum.is_empty() {
            return Err(crate::Error::validation(
                "Artifact checksum cannot be empty",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_artifact() -> Artifact {
        Artifact::new(
            BuildId::new(),
            "app.zip".to_string(),
            "/artifacts/123/app.zip".to_string(),
            1024,
            "abcdef1234567890".to_string(),
            ArtifactType::BuildOutput,
        )
    }

    #[test]
    fn test_new_artifact() {
        let artifact = create_test_artifact();

        assert_eq!(artifact.name(), "app.zip");
        assert_eq!(artifact.size(), 1024);
        assert_eq!(artifact.checksum(), "abcdef1234567890");
        assert!(!artifact.is_expired());
    }

    #[test]
    fn test_artifact_expiration() {
        let mut artifact = create_test_artifact();

        // Not expired initially
        assert!(!artifact.is_expired());

        // Set expiration in the past
        let past = Utc::now() - chrono::Duration::days(1);
        artifact.set_expiration(past);
        assert!(artifact.is_expired());

        // Mark as expired
        let mut artifact2 = create_test_artifact();
        artifact2.expire();
        assert!(artifact2.is_expired());
    }

    #[test]
    fn test_artifact_access() {
        let mut artifact = create_test_artifact();
        let initial_accessed = artifact.accessed_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        artifact.record_access();

        assert!(artifact.accessed_at > initial_accessed);
    }

    #[test]
    fn test_artifact_validation() {
        let artifact = create_test_artifact();
        assert!(artifact.validate().is_ok());

        // Empty name
        let invalid = Artifact::new(
            BuildId::new(),
            "".to_string(),
            "/path".to_string(),
            100,
            "checksum".to_string(),
            ArtifactType::BuildOutput,
        );
        assert!(invalid.validate().is_err());
    }
}
