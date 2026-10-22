//! Agent entity - Represents a build agent

use crate::domain::events::DomainEvent;
use crate::domain::value_objects::agent_id::AgentId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent entity representing a build worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier
    id: AgentId,

    /// Agent name
    name: String,

    /// Agent description
    description: Option<String>,

    /// Agent status
    status: AgentStatus,

    /// Agent capabilities/labels
    labels: HashMap<String, String>,

    /// Maximum concurrent jobs this agent can handle
    max_concurrent_jobs: usize,

    /// Current number of running jobs
    current_jobs: usize,

    /// Agent platform information
    platform: AgentPlatform,

    /// Last heartbeat time
    last_heartbeat: DateTime<Utc>,

    /// Agent IP address
    ip_address: Option<String>,

    /// Agent version
    version: String,

    /// Creation timestamp
    created_at: DateTime<Utc>,

    /// Last update timestamp
    updated_at: DateTime<Utc>,

    /// Domain events
    events: Vec<DomainEvent>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    /// Agent is online and ready
    Online,
    /// Agent is busy
    Busy,
    /// Agent is offline
    Offline,
    /// Agent is in maintenance mode
    Maintenance,
    /// Agent is disconnected
    Disconnected,
}

/// Agent platform information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPlatform {
    /// Operating system
    pub os: String,

    /// OS version
    pub os_version: String,

    /// Architecture (x86_64, arm64, etc.)
    pub architecture: String,

    /// CPU cores
    pub cpu_cores: u32,

    /// Memory in MB
    pub memory_mb: u64,

    /// Disk space in GB
    pub disk_gb: u64,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        name: String,
        max_concurrent_jobs: usize,
        platform: AgentPlatform,
        version: String,
    ) -> Self {
        let now = Utc::now();
        let id = AgentId::new();

        let mut agent = Self {
            id: id.clone(),
            name: name.clone(),
            description: None,
            status: AgentStatus::Offline,
            labels: HashMap::new(),
            max_concurrent_jobs,
            current_jobs: 0,
            platform,
            last_heartbeat: now,
            ip_address: None,
            version,
            created_at: now,
            updated_at: now,
            events: Vec::new(),
        };

        agent.events.push(DomainEvent::AgentRegistered {
            agent_id: id,
            name,
            created_at: now,
        });

        agent
    }

    /// Get the agent ID
    pub fn id(&self) -> &AgentId {
        &self.id
    }

    /// Get the agent name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the agent status
    pub fn status(&self) -> &AgentStatus {
        &self.status
    }

    /// Check if the agent can accept a new job
    pub fn can_accept_job(&self) -> bool {
        self.status == AgentStatus::Online && self.current_jobs < self.max_concurrent_jobs
    }

    /// Register the agent (mark as online)
    pub fn register(&mut self, ip_address: String) -> crate::Result<()> {
        self.status = AgentStatus::Online;
        self.ip_address = Some(ip_address);
        self.last_heartbeat = Utc::now();
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Update heartbeat
    pub fn heartbeat(&mut self) -> crate::Result<()> {
        self.last_heartbeat = Utc::now();
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Mark agent as offline
    pub fn disconnect(&mut self) {
        self.status = AgentStatus::Offline;
        self.updated_at = Utc::now();

        self.events.push(DomainEvent::AgentDisconnected {
            agent_id: self.id.clone(),
            disconnected_at: self.updated_at,
        });
    }

    /// Set agent to maintenance mode
    pub fn set_maintenance(&mut self) {
        self.status = AgentStatus::Maintenance;
        self.updated_at = Utc::now();
    }

    /// Assign a job to this agent
    pub fn assign_job(&mut self) -> crate::Result<()> {
        if !self.can_accept_job() {
            return Err(crate::Error::agent("Agent cannot accept more jobs"));
        }

        self.current_jobs += 1;
        self.status = if self.current_jobs >= self.max_concurrent_jobs {
            AgentStatus::Busy
        } else {
            AgentStatus::Online
        };
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Release a job from this agent
    pub fn release_job(&mut self) -> crate::Result<()> {
        if self.current_jobs == 0 {
            return Err(crate::Error::agent("No jobs to release"));
        }

        self.current_jobs -= 1;
        self.status = AgentStatus::Online;
        self.updated_at = Utc::now();

        Ok(())
    }

    /// Add a label to the agent
    pub fn add_label(&mut self, key: String, value: String) {
        self.labels.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// Remove a label from the agent
    pub fn remove_label(&mut self, key: &str) {
        self.labels.remove(key);
        self.updated_at = Utc::now();
    }

    /// Check if agent has a label
    pub fn has_label(&self, key: &str, value: &str) -> bool {
        self.labels.get(key).map_or(false, |v| v == value)
    }

    /// Check if agent is considered dead (no heartbeat for a long time)
    pub fn is_dead(&self, timeout_seconds: i64) -> bool {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.last_heartbeat);
        duration.num_seconds() > timeout_seconds
    }

    /// Get the domain events and clear them
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_agent() -> Agent {
        let platform = AgentPlatform {
            os: "Linux".to_string(),
            os_version: "Ubuntu 22.04".to_string(),
            architecture: "x86_64".to_string(),
            cpu_cores: 8,
            memory_mb: 16384,
            disk_gb: 500,
        };

        Agent::new("test-agent".to_string(), 5, platform, "1.0.0".to_string())
    }

    #[test]
    fn test_new_agent() {
        let agent = create_test_agent();

        assert_eq!(agent.name(), "test-agent");
        assert_eq!(agent.status(), &AgentStatus::Offline);
        assert_eq!(agent.max_concurrent_jobs, 5);
        assert_eq!(agent.current_jobs, 0);
        assert!(!agent.can_accept_job());
    }

    #[test]
    fn test_agent_registration() {
        let mut agent = create_test_agent();

        assert!(agent.register("192.168.1.100".to_string()).is_ok());
        assert_eq!(agent.status(), &AgentStatus::Online);
        assert_eq!(agent.ip_address, Some("192.168.1.100".to_string()));
        assert!(agent.can_accept_job());
    }

    #[test]
    fn test_job_assignment() {
        let mut agent = create_test_agent();
        agent.register("192.168.1.100".to_string()).unwrap();

        // Assign jobs up to the limit
        for _ in 0..5 {
            assert!(agent.assign_job().is_ok());
        }

        assert_eq!(agent.current_jobs, 5);
        assert_eq!(agent.status(), &AgentStatus::Busy);
        assert!(!agent.can_accept_job());

        // Cannot assign more jobs
        assert!(agent.assign_job().is_err());
    }

    #[test]
    fn test_job_release() {
        let mut agent = create_test_agent();
        agent.register("192.168.1.100".to_string()).unwrap();

        agent.assign_job().unwrap();
        agent.assign_job().unwrap();

        assert_eq!(agent.current_jobs, 2);

        assert!(agent.release_job().is_ok());
        assert_eq!(agent.current_jobs, 1);
        assert_eq!(agent.status(), &AgentStatus::Online);

        assert!(agent.release_job().is_ok());
        assert_eq!(agent.current_jobs, 0);
    }

    #[test]
    fn test_agent_labels() {
        let mut agent = create_test_agent();

        agent.add_label("os".to_string(), "linux".to_string());
        agent.add_label("arch".to_string(), "x86_64".to_string());

        assert!(agent.has_label("os", "linux"));
        assert!(agent.has_label("arch", "x86_64"));
        assert!(!agent.has_label("os", "windows"));

        agent.remove_label("arch");
        assert!(!agent.has_label("arch", "x86_64"));
    }

    #[test]
    fn test_agent_heartbeat() {
        let mut agent = create_test_agent();
        agent.register("192.168.1.100".to_string()).unwrap();

        let initial_heartbeat = agent.last_heartbeat;

        std::thread::sleep(std::time::Duration::from_millis(10));
        agent.heartbeat().unwrap();

        assert!(agent.last_heartbeat > initial_heartbeat);
    }

    #[test]
    fn test_agent_is_dead() {
        let mut agent = create_test_agent();

        // Agent just created, not dead
        assert!(!agent.is_dead(60));

        // Set last heartbeat to 2 minutes ago
        agent.last_heartbeat = Utc::now() - chrono::Duration::seconds(120);

        // Agent is dead (timeout is 60 seconds)
        assert!(agent.is_dead(60));
        assert!(!agent.is_dead(180)); // But not with 180 second timeout
    }

    #[test]
    fn test_agent_maintenance() {
        let mut agent = create_test_agent();
        agent.register("192.168.1.100".to_string()).unwrap();

        agent.set_maintenance();
        assert_eq!(agent.status(), &AgentStatus::Maintenance);
        assert!(!agent.can_accept_job());
    }
}
