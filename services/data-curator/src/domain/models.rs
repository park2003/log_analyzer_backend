use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurationJob {
    pub id: Uuid,
    pub project_id: String,
    pub status: CurationStatus,
    pub raw_data_uri: String,
    pub curated_data_uri: Option<String>,
    pub images_for_feedback: Vec<ImageForFeedback>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl CurationJob {
    pub fn new(project_id: String, raw_data_uri: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            project_id,
            status: CurationStatus::Pending,
            raw_data_uri,
            curated_data_uri: None,
            images_for_feedback: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn transition_to_embedding(&mut self) {
        self.status = CurationStatus::Embedding;
        self.updated_at = Utc::now();
    }

    pub fn transition_to_awaiting_feedback(&mut self, images: Vec<ImageForFeedback>) {
        self.status = CurationStatus::AwaitingFeedback;
        self.images_for_feedback = images;
        self.updated_at = Utc::now();
    }

    pub fn complete(&mut self, curated_data_uri: String) {
        self.status = CurationStatus::Completed;
        self.curated_data_uri = Some(curated_data_uri);
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = CurationStatus::Failed;
        self.updated_at = Utc::now();
    }
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
pub struct ImageForFeedback {
    pub image_id: String,
    pub image_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageFeedback {
    pub image_id: String,
    pub accepted: bool,
}
