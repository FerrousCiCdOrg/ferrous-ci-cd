//! In-memory repository implementations for testing

use crate::domain::entities::{
    agent::{Agent, AgentStatus},
    build::Build,
    pipeline::Pipeline,
};
use crate::domain::repositories::{
    agent::AgentRepository,
    build::{BuildQueryOptions, BuildRepository},
    pipeline::PipelineRepository,
};
use crate::domain::value_objects::{
    agent_id::AgentId, build_id::BuildId, build_status::BuildStatus, pipeline_id::PipelineId,
    project_id::ProjectId,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory pipeline repository
pub struct InMemoryPipelineRepository {
    pipelines: Arc<RwLock<HashMap<String, Pipeline>>>,
}

impl InMemoryPipelineRepository {
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl PipelineRepository for InMemoryPipelineRepository {
    async fn save(&self, pipeline: &Pipeline) -> crate::Result<()> {
        let mut pipelines = self.pipelines.write().await;
        pipelines.insert(pipeline.id().to_string(), pipeline.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &PipelineId) -> crate::Result<Option<Pipeline>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines.get(&id.to_string()).cloned())
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Pipeline>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines
            .values()
            .filter(|p| p.project_id() == project_id)
            .cloned()
            .collect())
    }

    async fn find_enabled_by_project(
        &self,
        project_id: &ProjectId,
    ) -> crate::Result<Vec<Pipeline>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines
            .values()
            .filter(|p| p.project_id() == project_id && p.is_enabled())
            .cloned()
            .collect())
    }

    async fn find_all(&self) -> crate::Result<Vec<Pipeline>> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines.values().cloned().collect())
    }

    async fn update(&self, pipeline: &Pipeline) -> crate::Result<()> {
        self.save(pipeline).await
    }

    async fn delete(&self, id: &PipelineId) -> crate::Result<()> {
        let mut pipelines = self.pipelines.write().await;
        pipelines.remove(&id.to_string());
        Ok(())
    }

    async fn exists(&self, id: &PipelineId) -> crate::Result<bool> {
        let pipelines = self.pipelines.read().await;
        Ok(pipelines.contains_key(&id.to_string()))
    }
}

/// In-memory build repository
pub struct InMemoryBuildRepository {
    builds: Arc<RwLock<HashMap<String, Build>>>,
}

impl InMemoryBuildRepository {
    pub fn new() -> Self {
        Self {
            builds: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl BuildRepository for InMemoryBuildRepository {
    async fn save(&self, build: &Build) -> crate::Result<()> {
        let mut builds = self.builds.write().await;
        builds.insert(build.id().to_string(), build.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &BuildId) -> crate::Result<Option<Build>> {
        let builds = self.builds.read().await;
        Ok(builds.get(&id.to_string()).cloned())
    }

    async fn find_by_pipeline(&self, pipeline_id: &PipelineId) -> crate::Result<Vec<Build>> {
        let builds = self.builds.read().await;
        Ok(builds
            .values()
            .filter(|b| b.pipeline_id() == pipeline_id)
            .cloned()
            .collect())
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> crate::Result<Vec<Build>> {
        let builds = self.builds.read().await;
        Ok(builds
            .values()
            .filter(|b| b.project_id() == project_id)
            .cloned()
            .collect())
    }

    async fn query(&self, _options: BuildQueryOptions) -> crate::Result<Vec<Build>> {
        // Simplified implementation
        let builds = self.builds.read().await;
        Ok(builds.values().cloned().collect())
    }

    async fn find_running(&self) -> crate::Result<Vec<Build>> {
        let builds = self.builds.read().await;
        Ok(builds
            .values()
            .filter(|b| b.status().is_in_progress())
            .cloned()
            .collect())
    }

    async fn next_build_number(&self, pipeline_id: &PipelineId) -> crate::Result<u64> {
        let builds = self.builds.read().await;
        let max_number = builds
            .values()
            .filter(|b| b.pipeline_id() == pipeline_id)
            .map(|b| b.number())
            .max()
            .unwrap_or(0);
        Ok(max_number + 1)
    }

    async fn update(&self, build: &Build) -> crate::Result<()> {
        self.save(build).await
    }

    async fn delete(&self, id: &BuildId) -> crate::Result<()> {
        let mut builds = self.builds.write().await;
        builds.remove(&id.to_string());
        Ok(())
    }

    async fn count_by_status(&self, status: &BuildStatus) -> crate::Result<u64> {
        let builds = self.builds.read().await;
        Ok(builds.values().filter(|b| b.status() == status).count() as u64)
    }
}

/// In-memory agent repository
pub struct InMemoryAgentRepository {
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}

impl InMemoryAgentRepository {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl AgentRepository for InMemoryAgentRepository {
    async fn save(&self, agent: &Agent) -> crate::Result<()> {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id().to_string(), agent.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: &AgentId) -> crate::Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.get(&id.to_string()).cloned())
    }

    async fn find_by_name(&self, name: &str) -> crate::Result<Option<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().find(|a| a.name() == name).cloned())
    }

    async fn find_all(&self) -> crate::Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents.values().cloned().collect())
    }

    async fn find_by_status(&self, status: &AgentStatus) -> crate::Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents
            .values()
            .filter(|a| a.status() == status)
            .cloned()
            .collect())
    }

    async fn find_available(&self) -> crate::Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents
            .values()
            .filter(|a| a.can_accept_job())
            .cloned()
            .collect())
    }

    async fn find_by_labels(&self, labels: &[(String, String)]) -> crate::Result<Vec<Agent>> {
        let agents = self.agents.read().await;
        Ok(agents
            .values()
            .filter(|a| labels.iter().all(|(key, value)| a.has_label(key, value)))
            .cloned()
            .collect())
    }

    async fn update(&self, agent: &Agent) -> crate::Result<()> {
        self.save(agent).await
    }

    async fn delete(&self, id: &AgentId) -> crate::Result<()> {
        let mut agents = self.agents.write().await;
        agents.remove(&id.to_string());
        Ok(())
    }

    async fn exists(&self, id: &AgentId) -> crate::Result<bool> {
        let agents = self.agents.read().await;
        Ok(agents.contains_key(&id.to_string()))
    }
}
