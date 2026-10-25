//! Build integration tests

mod common;

use common::TestFixture;
use ferrous_ci_cd::domain::{
    entities::build::BuildTrigger, value_objects::build_status::BuildStatus,
};

#[tokio::test]
async fn test_complete_build_lifecycle() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    // Create pipeline
    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // Create build
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

    assert_eq!(build.number(), 1);
    assert_eq!(build.status(), &BuildStatus::Pending);

    // Register agent
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

    // Start build
    fixture
        .build_service
        .start_build(build.id(), agent.id().clone())
        .await
        .expect("Failed to start build");

    let running_build = fixture
        .build_service
        .get_build(build.id())
        .await
        .expect("Failed to get build");

    assert_eq!(running_build.status(), &BuildStatus::Running);

    // Complete build
    fixture
        .build_service
        .complete_build(build.id())
        .await
        .expect("Failed to complete build");

    let completed_build = fixture
        .build_service
        .get_build(build.id())
        .await
        .expect("Failed to get build");

    assert_eq!(completed_build.status(), &BuildStatus::Success);
    assert!(completed_build.duration().is_some());
}

#[tokio::test]
async fn test_build_failure() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

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
            BuildTrigger::Push,
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

    // Fail the build
    fixture
        .build_service
        .fail_build(build.id(), "Test compilation failed".to_string())
        .await
        .expect("Failed to fail build");

    let failed_build = fixture
        .build_service
        .get_build(build.id())
        .await
        .expect("Failed to get build");

    assert_eq!(failed_build.status(), &BuildStatus::Failed);
}

#[tokio::test]
async fn test_build_cancellation() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

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

    // Cancel pending build
    fixture
        .build_service
        .cancel_build(build.id())
        .await
        .expect("Failed to cancel build");

    let cancelled_build = fixture
        .build_service
        .get_build(build.id())
        .await
        .expect("Failed to get build");

    assert_eq!(cancelled_build.status(), &BuildStatus::Cancelled);
}

#[tokio::test]
async fn test_multiple_builds_same_pipeline() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // Create multiple builds
    let mut build_ids = Vec::new();
    for i in 0..5 {
        let build = fixture
            .build_service
            .create_build(
                pipeline.id().clone(),
                project.id().clone(),
                format!("commit{}", i),
                "main".to_string(),
                BuildTrigger::Push,
            )
            .await
            .expect("Failed to create build");

        build_ids.push(build.id().clone());
        assert_eq!(build.number(), (i + 1) as u64);
    }

    // Verify all builds exist
    let builds = fixture
        .build_service
        .get_pipeline_builds(pipeline.id())
        .await
        .expect("Failed to get builds");

    assert_eq!(builds.len(), 5);
}

#[tokio::test]
async fn test_concurrent_builds() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // Register multiple agents
    let mut agents = Vec::new();
    for i in 0..3 {
        let platform = TestFixture::create_test_platform();
        let agent = fixture
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
        agents.push(agent);
    }

    // Create and start multiple builds concurrently
    let mut handles = Vec::new();
    for i in 0..3 {
        let build_service = fixture.build_service.clone();
        let agent_service = fixture.agent_service.clone();
        let pipeline_id = pipeline.id().clone();
        let project_id = project.id().clone();
        let agent_id = agents[i].id().clone();

        let handle = tokio::spawn(async move {
            let build = build_service
                .create_build(
                    pipeline_id,
                    project_id,
                    format!("commit{}", i),
                    "main".to_string(),
                    BuildTrigger::Push,
                )
                .await
                .expect("Failed to create build");

            build_service
                .start_build(build.id(), agent_id.clone())
                .await
                .expect("Failed to start build");

            agent_service
                .assign_job(&agent_id)
                .await
                .expect("Failed to assign job");

            build.id().clone()
        });

        handles.push(handle);
    }

    // Wait for all builds to start
    let build_ids: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    assert_eq!(build_ids.len(), 3);

    // Verify all builds are running
    let running_builds = fixture
        .build_service
        .get_running_builds()
        .await
        .expect("Failed to get running builds");

    assert_eq!(running_builds.len(), 3);
}
