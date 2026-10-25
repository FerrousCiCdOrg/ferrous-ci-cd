//! Basic integration tests for Ferrous CI/CD

mod common;

use common::TestFixture;
use ferrous_ci_cd::{init, Config};

#[tokio::test]
async fn test_initialization() {
    let result = init().await;
    assert!(result.is_ok(), "Initialization should succeed");
}

#[tokio::test]
async fn test_default_config() {
    let config = Config::default();
    assert!(config.validate().is_ok(), "Default config should be valid");
}

#[tokio::test]
async fn test_system_startup() {
    // Test that all components can be initialized
    let fixture = TestFixture::new().await;

    // Verify services are available
    assert!(fixture
        .pipeline_service
        .get_project_pipelines(&common::TestFixture::create_test_project().id())
        .await
        .is_ok());
    assert!(fixture.agent_service.find_available_agents().await.is_ok());
}

#[tokio::test]
async fn test_end_to_end_workflow() {
    let fixture = TestFixture::new().await;

    // 1. Create project
    let project = TestFixture::create_test_project();

    // 2. Create pipeline
    let config = TestFixture::create_test_pipeline_config();
    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "e2e-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // 3. Register agent
    let platform = TestFixture::create_test_platform();
    let agent = fixture
        .agent_service
        .register_agent(
            "e2e-agent".to_string(),
            5,
            platform,
            "1.0.0".to_string(),
            "192.168.1.100".to_string(),
        )
        .await
        .expect("Failed to register agent");

    // 4. Create and execute build
    let build = fixture
        .build_service
        .create_build(
            pipeline.id().clone(),
            project.id().clone(),
            "abc123".to_string(),
            "main".to_string(),
            ferrous_ci_cd::domain::entities::build::BuildTrigger::Manual {
                user_id: "admin".to_string(),
            },
        )
        .await
        .expect("Failed to create build");

    // 5. Start build
    fixture
        .build_service
        .start_build(build.id(), agent.id().clone())
        .await
        .expect("Failed to start build");

    // 6. Complete build
    fixture
        .build_service
        .complete_build(build.id())
        .await
        .expect("Failed to complete build");

    // 7. Verify final state
    let final_build = fixture
        .build_service
        .get_build(build.id())
        .await
        .expect("Failed to get build");

    assert_eq!(
        final_build.status(),
        &ferrous_ci_cd::domain::value_objects::build_status::BuildStatus::Success
    );
    assert!(final_build.duration().is_some());

    println!("✅ End-to-end workflow completed successfully!");
}
