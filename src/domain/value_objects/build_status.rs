//! Build Status value object

use serde::{Deserialize, Serialize};
use std::fmt;

/// Build Status value object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildStatus {
    /// Build is waiting to start
    Pending,
    /// Build is currently running
    Running,
    /// Build completed successfully
    Success,
    /// Build failed
    Failed,
    /// Build was cancelled
    Cancelled,
}

impl BuildStatus {
    /// Check if the build is in a terminal state (completed)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            BuildStatus::Success | BuildStatus::Failed | BuildStatus::Cancelled
        )
    }

    /// Check if the build is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(self, BuildStatus::Running)
    }

    /// Check if the build is successful
    pub fn is_success(&self) -> bool {
        matches!(self, BuildStatus::Success)
    }

    /// Check if the build failed
    pub fn is_failed(&self) -> bool {
        matches!(self, BuildStatus::Failed)
    }

    /// Get a human-readable description
    pub fn description(&self) -> &str {
        match self {
            BuildStatus::Pending => "Waiting to start",
            BuildStatus::Running => "Running",
            BuildStatus::Success => "Completed successfully",
            BuildStatus::Failed => "Failed",
            BuildStatus::Cancelled => "Cancelled",
        }
    }

    /// Get the emoji representation
    pub fn emoji(&self) -> &str {
        match self {
            BuildStatus::Pending => "⏸️",
            BuildStatus::Running => "🏃",
            BuildStatus::Success => "✅",
            BuildStatus::Failed => "❌",
            BuildStatus::Cancelled => "🚫",
        }
    }
}

impl fmt::Display for BuildStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildStatus::Pending => write!(f, "Pending"),
            BuildStatus::Running => write!(f, "Running"),
            BuildStatus::Success => write!(f, "Success"),
            BuildStatus::Failed => write!(f, "Failed"),
            BuildStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl Default for BuildStatus {
    fn default() -> Self {
        BuildStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_status_terminal() {
        assert!(!BuildStatus::Pending.is_terminal());
        assert!(!BuildStatus::Running.is_terminal());
        assert!(BuildStatus::Success.is_terminal());
        assert!(BuildStatus::Failed.is_terminal());
        assert!(BuildStatus::Cancelled.is_terminal());
    }

    #[test]
    fn test_build_status_in_progress() {
        assert!(!BuildStatus::Pending.is_in_progress());
        assert!(BuildStatus::Running.is_in_progress());
        assert!(!BuildStatus::Success.is_in_progress());
        assert!(!BuildStatus::Failed.is_in_progress());
        assert!(!BuildStatus::Cancelled.is_in_progress());
    }

    #[test]
    fn test_build_status_display() {
        assert_eq!(BuildStatus::Success.to_string(), "Success");
        assert_eq!(BuildStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_build_status_description() {
        assert_eq!(BuildStatus::Running.description(), "Running");
        assert_eq!(BuildStatus::Success.description(), "Completed successfully");
    }
}
