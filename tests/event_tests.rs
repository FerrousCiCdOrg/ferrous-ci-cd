//! Event system integration tests

mod common;

use common::TestFixture;
use ferrous_ci_cd::domain::{entities::build::BuildTrigger, events::DomainEvent};

#[tokio::test]
async fn test_pipeline_events() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    // Clear any existing events
    fixture.event_publisher.clear().await;

    // Create pipeline (should emit PipelineCreated event)
    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    // Find the PipelineCreated event
    let pipeline_created = events
        .iter()
        .find(|e| matches!(e, DomainEvent::PipelineCreated { .. }))
        .expect("Should have PipelineCreated event");

    match pipeline_created {
        DomainEvent::PipelineCreated {
            pipeline_id, name, ..
        } => {
            assert_eq!(pipeline_id, pipeline.id());
            assert_eq!(name, "test-pipeline");
        }
        _ => panic!("Expected PipelineCreated event"),
    }

    // Disable pipeline (should emit PipelineDisabled event)
    fixture.event_publisher.clear().await;

    fixture
        .pipeline_service
        .disable_pipeline(pipeline.id())
        .await
        .expect("Failed to disable pipeline");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let pipeline_disabled = events
        .iter()
        .find(|e| matches!(e, DomainEvent::PipelineDisabled { .. }))
        .expect("Should have PipelineDisabled event");

    match pipeline_disabled {
        DomainEvent::PipelineDisabled { pipeline_id, .. } => {
            assert_eq!(pipeline_id, pipeline.id());
        }
        _ => panic!("Expected PipelineDisabled event"),
    }
}

#[tokio::test]
async fn test_build_events() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    fixture.event_publisher.clear().await;

    // Create build (should emit BuildCreated event)
    let build = fixture
        .build_service
        .create_build(
            pipeline.id().clone(),
            project.id().clone(),
            "abc123".to_string(),
            "main".to_string(),
            BuildTrigger::Push,
        )
        .await
        .expect("Failed to create build");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let build_created = events
        .iter()
        .find(|e| matches!(e, DomainEvent::BuildCreated { .. }))
        .expect("Should have BuildCreated event");

    match build_created {
        DomainEvent::BuildCreated {
            build_id, number, ..
        } => {
            assert_eq!(build_id, build.id());
            assert_eq!(*number, 1);
        }
        _ => panic!("Expected BuildCreated event"),
    }

    // Register agent and start build
    fixture.event_publisher.clear().await;

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

    fixture.event_publisher.clear().await;

    fixture
        .build_service
        .start_build(build.id(), agent.id().clone())
        .await
        .expect("Failed to start build");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let build_started = events
        .iter()
        .find(|e| matches!(e, DomainEvent::BuildStarted { .. }))
        .expect("Should have BuildStarted event");

    match build_started {
        DomainEvent::BuildStarted {
            build_id, agent_id, ..
        } => {
            assert_eq!(build_id, build.id());
            assert_eq!(agent_id, agent.id());
        }
        _ => panic!("Expected BuildStarted event"),
    }

    // Complete build
    fixture.event_publisher.clear().await;

    fixture
        .build_service
        .complete_build(build.id())
        .await
        .expect("Failed to complete build");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let build_completed = events
        .iter()
        .find(|e| matches!(e, DomainEvent::BuildCompleted { .. }))
        .expect("Should have BuildCompleted event");

    match build_completed {
        DomainEvent::BuildCompleted { build_id, .. } => {
            assert_eq!(build_id, build.id());
        }
        _ => panic!("Expected BuildCompleted event"),
    }
}

#[tokio::test]
async fn test_agent_events() {
    let fixture = TestFixture::new().await;
    let platform = TestFixture::create_test_platform();

    fixture.event_publisher.clear().await;

    // Register agent (should emit AgentRegistered event)
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

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let agent_registered = events
        .iter()
        .find(|e| matches!(e, DomainEvent::AgentRegistered { .. }))
        .expect("Should have AgentRegistered event");

    match agent_registered {
        DomainEvent::AgentRegistered { agent_id, name, .. } => {
            assert_eq!(agent_id, agent.id());
            assert_eq!(name, "test-agent");
        }
        _ => panic!("Expected AgentRegistered event"),
    }

    // Disconnect agent (should emit AgentDisconnected event)
    fixture.event_publisher.clear().await;

    fixture
        .agent_service
        .disconnect_agent(agent.id())
        .await
        .expect("Failed to disconnect agent");

    let events = fixture.event_publisher.get_events().await;
    assert!(!events.is_empty(), "Should have at least one event");

    let agent_disconnected = events
        .iter()
        .find(|e| matches!(e, DomainEvent::AgentDisconnected { .. }))
        .expect("Should have AgentDisconnected event");

    match agent_disconnected {
        DomainEvent::AgentDisconnected { agent_id, .. } => {
            assert_eq!(agent_id, agent.id());
        }
        _ => panic!("Expected AgentDisconnected event"),
    }
}

#[tokio::test]
async fn test_event_collection() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    fixture.event_publisher.clear().await;

    // Perform a series of operations
    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    let build = fixture
        .build_service
        .create_build(
            pipeline.id().clone(),
            project.id().clone(),
            "abc123".to_string(),
            "main".to_string(),
            BuildTrigger::Manual {
                user_id: "user123".to_string(),
            },
        )
        .await
        .expect("Failed to create build");

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

    fixture
        .build_service
        .start_build(build.id(), agent.id().clone())
        .await
        .expect("Failed to start build");

    // Verify all events were collected
    let events = fixture.event_publisher.get_events().await;
    assert!(
        events.len() >= 4,
        "Should have at least 4 events, got {}",
        events.len()
    );

    // Verify we have all expected event types
    let has_pipeline_created = events
        .iter()
        .any(|e| matches!(e, DomainEvent::PipelineCreated { .. }));
    let has_build_created = events
        .iter()
        .any(|e| matches!(e, DomainEvent::BuildCreated { .. }));
    let has_agent_registered = events
        .iter()
        .any(|e| matches!(e, DomainEvent::AgentRegistered { .. }));
    let has_build_started = events
        .iter()
        .any(|e| matches!(e, DomainEvent::BuildStarted { .. }));

    assert!(has_pipeline_created, "Should have PipelineCreated event");
    assert!(has_build_created, "Should have BuildCreated event");
    assert!(has_agent_registered, "Should have AgentRegistered event");
    assert!(has_build_started, "Should have BuildStarted event");

    println!(
        "✅ Event collection verified: {} events with all expected types",
        events.len()
    );
}
