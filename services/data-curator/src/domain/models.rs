use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurationJob {
    pub id: Uuid,
    pub project_id: String,
    pub status: CurationStatus,
    pub raw_data_uri: String,
    pub curated_data_uri: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurationStatus {
    Pending,
    Embedding,
    AwaitingFeedback,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEmbedding {
    pub id: Uuid,
    pub project_id: String,
    pub image_uri: String,
    pub embedding: Vec<f32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFeedback {
    pub image_id: String,
    pub accepted: bool,
}