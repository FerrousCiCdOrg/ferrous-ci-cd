//! Common test utilities

use ferrous_ci_cd::{
    domain::{
        entities::{agent::AgentPlatform, project::Project},
        events::InMemoryEventPublisher,
        repositories::{
            agent::AgentRepository, build::BuildRepository, pipeline::PipelineRepository,
        },
        services::{agent::AgentService, build::BuildService, pipeline::PipelineService},
        value_objects::pipeline_config::{Job, PipelineConfig, Stage, Trigger},
    },
    infrastructure::repositories::in_memory::{
        InMemoryAgentRepository, InMemoryBuildRepository, InMemoryPipelineRepository,
    },
    Config,
};
use std::sync::Arc;

/// Test fixture for integration tests
#[allow(dead_code)]
pub struct TestFixture {
    pub pipeline_service: Arc<PipelineService>,
    pub build_service: Arc<BuildService>,
    pub agent_service: Arc<AgentService>,
    pub pipeline_repo: Arc<dyn PipelineRepository>,
    pub build_repo: Arc<dyn BuildRepository>,
    pub agent_repo: Arc<dyn AgentRepository>,
    pub event_publisher: Arc<InMemoryEventPublisher>,
}

impl TestFixture {
    /// Create a new test fixture
    pub async fn new() -> Self {
        let event_publisher = Arc::new(InMemoryEventPublisher::new());
        let pipeline_repo: Arc<dyn PipelineRepository> =
            Arc::new(InMemoryPipelineRepository::new());
        let build_repo: Arc<dyn BuildRepository> = Arc::new(InMemoryBuildRepository::new());
        let agent_repo: Arc<dyn AgentRepository> = Arc::new(InMemoryAgentRepository::new());

        let pipeline_service = Arc::new(PipelineService::new(
            pipeline_repo.clone(),
            event_publisher.clone(),
        ));

        let build_service = Arc::new(BuildService::new(
            build_repo.clone(),
            event_publisher.clone(),
        ));

        let agent_service = Arc::new(AgentService::new(
            agent_repo.clone(),
            event_publisher.clone(),
        ));

        Self {
            pipeline_service,
            build_service,
            agent_service,
            pipeline_repo,
            build_repo,
            agent_repo,
            event_publisher,
        }
    }

    /// Create a test project
    #[allow(dead_code)]
    pub fn create_test_project() -> Project {
        Project::new(
            "test-project".to_string(),
            "https://github.com/test/repo.git".to_string(),
            "main".to_string(),
        )
    }

    /// Create a test pipeline configuration
    #[allow(dead_code)]
    pub fn create_test_pipeline_config() -> PipelineConfig {
        let mut job1 = Job::new("build".to_string());
        job1.set_image("rust:latest".to_string());
        job1.add_command("cargo build --release".to_string());

        let mut job2 = Job::new("test".to_string());
        job2.set_image("rust:latest".to_string());
        job2.add_command("cargo test".to_string());

        let stage = Stage::new("ci".to_string(), vec![job1, job2]);

        PipelineConfig::new(
            vec![stage],
            vec![Trigger::Push {
                branches: vec!["main".to_string()],
            }],
        )
    }

    /// Create a test agent platform
    #[allow(dead_code)]
    pub fn create_test_platform() -> AgentPlatform {
        AgentPlatform {
            os: "Linux".to_string(),
            os_version: "Ubuntu 22.04".to_string(),
            architecture: "x86_64".to_string(),
            cpu_cores: 8,
            memory_mb: 16384,
            disk_gb: 500,
        }
    }

    /// Wait for async operations (helper for tests)
    #[allow(dead_code)]
    pub async fn wait_ms(ms: u64) {
        tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
    }
}

/// Test config helper
#[allow(dead_code)]
pub fn create_test_config() -> Config {
    Config::default()
}
