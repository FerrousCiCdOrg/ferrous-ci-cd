//! Pipeline integration tests

mod common;

use common::TestFixture;
use ferrous_ci_cd::domain::value_objects::pipeline_config::{Job, PipelineConfig, Stage, Trigger};

#[tokio::test]
async fn test_create_and_retrieve_pipeline() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    // Create pipeline
    let pipeline = fixture
        .pipeline_service
        .create_pipeline(
            project.id().clone(),
            "test-pipeline".to_string(),
            config.clone(),
        )
        .await
        .expect("Failed to create pipeline");

    assert_eq!(pipeline.name(), "test-pipeline");
    assert!(pipeline.is_enabled());

    // Retrieve pipeline
    let retrieved = fixture
        .pipeline_service
        .get_pipeline(pipeline.id())
        .await
        .expect("Failed to retrieve pipeline");

    assert_eq!(retrieved.id(), pipeline.id());
    assert_eq!(retrieved.name(), pipeline.name());
}

#[tokio::test]
async fn test_enable_disable_pipeline() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // Disable pipeline
    fixture
        .pipeline_service
        .disable_pipeline(pipeline.id())
        .await
        .expect("Failed to disable pipeline");

    let disabled = fixture
        .pipeline_service
        .get_pipeline(pipeline.id())
        .await
        .expect("Failed to retrieve pipeline");

    assert!(!disabled.is_enabled());

    // Enable pipeline
    fixture
        .pipeline_service
        .enable_pipeline(pipeline.id())
        .await
        .expect("Failed to enable pipeline");

    let enabled = fixture
        .pipeline_service
        .get_pipeline(pipeline.id())
        .await
        .expect("Failed to retrieve pipeline");

    assert!(enabled.is_enabled());
}

#[tokio::test]
async fn test_update_pipeline_config() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    let pipeline = fixture
        .pipeline_service
        .create_pipeline(project.id().clone(), "test-pipeline".to_string(), config)
        .await
        .expect("Failed to create pipeline");

    // Create new config
    let mut new_job = Job::new("deploy".to_string());
    new_job.add_command("./deploy.sh".to_string());

    let new_stage = Stage::new("deploy".to_string(), vec![new_job]);
    let new_config = PipelineConfig::new(vec![new_stage], vec![Trigger::Manual]);

    // Update config
    fixture
        .pipeline_service
        .update_config(pipeline.id(), new_config)
        .await
        .expect("Failed to update config");

    let updated = fixture
        .pipeline_service
        .get_pipeline(pipeline.id())
        .await
        .expect("Failed to retrieve pipeline");

    assert_eq!(updated.config().stages.len(), 1);
    assert_eq!(updated.config().stages[0].name, "deploy");
}

#[tokio::test]
async fn test_list_project_pipelines() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();
    let config = TestFixture::create_test_pipeline_config();

    // Create multiple pipelines
    for i in 0..3 {
        fixture
            .pipeline_service
            .create_pipeline(
                project.id().clone(),
                format!("pipeline-{}", i),
                config.clone(),
            )
            .await
            .expect("Failed to create pipeline");
    }

    // List pipelines
    let pipelines = fixture
        .pipeline_service
        .get_project_pipelines(project.id())
        .await
        .expect("Failed to list pipelines");

    assert_eq!(pipelines.len(), 3);
}

#[tokio::test]
async fn test_pipeline_validation() {
    let fixture = TestFixture::new().await;
    let project = TestFixture::create_test_project();

    // Create invalid config (no stages)
    let invalid_config = PipelineConfig::new(vec![], vec![Trigger::Manual]);

    let result = fixture
        .pipeline_service
        .create_pipeline(
            project.id().clone(),
            "invalid-pipeline".to_string(),
            invalid_config,
        )
        .await;

    assert!(result.is_err(), "Should fail with invalid config");
}
