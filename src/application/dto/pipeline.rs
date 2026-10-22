//! Pipeline DTOs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineDto {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
}
