//! Agent integration tests

mod common;

use common::TestFixture;
use ferrous_ci_cd::domain::entities::agent::AgentStatus;

#[tokio::test]
async fn test_agent_registration_and_heartbeat() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    // Register agent
    let agent = fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            5,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    assert_eq!(agent.name(), "test-agent");
    assert_eq!(agent.status(), &AgentStatus::Online);
    assert!(agent.can_accept_job());

    // Send heartbeat
    fixture
        .agent_service
        .heartbeat(agent.id())
        .await
        .expect("Failed to send heartbeat");

    // Verify agent is still online
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 1);
}

#[tokio::test]
async fn test_agent_job_assignment() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    let agent = fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            3,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    // Assign jobs up to capacity
    for i in 0..3 {
        fixture
            .agent_service
            .assign_job(agent.id())
            .await
            .expect(&format!("Failed to assign job {}", i));
    }

    // Agent should be busy now
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 0);

    // Cannot assign more jobs
    let result = fixture.agent_service.assign_job(agent.id()).await;
    assert!(result.is_err());

    // Release a job
    fixture
        .agent_service
        .release_job(agent.id())
        .await
        .expect("Failed to release job");

    // Agent should be available again
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 1);
}

#[tokio::test]
async fn test_agent_disconnection() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    let agent = fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            5,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    // Disconnect agent
    fixture
        .agent_service
        .disconnect_agent(agent.id())
        .await
        .expect("Failed to disconnect agent");

    // Agent should not be available
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 0);
}

#[tokio::test]
async fn test_multiple_agents() {
    let fixture = TestFixture::new().await;

    // Register multiple agents
    for i in 0..5 {
        let platform = TestFixture::create_test_platform();
        fixture
            .agent_service
            .register_agent(
                format!("agent-{}", i),
                5,
                platform,
                "1.0.0".to_string(),
                format!("192.168.1.{}", 100 + i),
            )
            .await
            .expect("Failed to register agent");
    }

    // All agents should be available
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 5);
}

#[tokio::test]
async fn test_agent_cleanup_mechanism() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    // Register agent
    let _agent = fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            5,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    // Agent should be online initially
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 1);

    // Test cleanup with generous timeout (agent should not be dead)
    let cleaned = fixture
        .agent_service
        .cleanup_dead_agents(300) // 5 minute timeout
        .await
        .expect("Failed to cleanup dead agents");

    // Agent was just registered, so it should not be cleaned up
    assert_eq!(cleaned, 0);

    // Verify agent is still available
    let available_agents = fixture
        .agent_service
        .find_available_agents()
        .await
        .expect("Failed to find available agents");

    assert_eq!(available_agents.len(), 1);
}

#[tokio::test]
async fn test_duplicate_agent_registration() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    // Register agent
    fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            5,
            platform.clone(),
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    // Try to register agent with same name
    let result = fixture
        .agent_service
        .register_agent(
            "test-agent".to_string(),
            5,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await;

    assert!(result.is_err(), "Should fail with duplicate agent name");
}
