use anyhow::Result;
use uuid::Uuid;
use crate::domain::{
    models::{CurationJob, CurationStatus},
    repositories::{CurationRepository, EmbeddingRepository},
};

pub struct StartCurationUseCase<C: CurationRepository, E: EmbeddingRepository> {
    curation_repository: C,
    embedding_repository: E,
}

impl<C: CurationRepository, E: EmbeddingRepository> StartCurationUseCase<C, E> {
    pub fn new(curation_repository: C, embedding_repository: E) -> Self {
        Self {
            curation_repository,
            embedding_repository,
        }
    }

    pub async fn execute(&self, project_id: &str, raw_data_uri: &str) -> Result<String> {
        let job = CurationJob {
            id: Uuid::new_v4(),
            project_id: project_id.to_string(),
            status: CurationStatus::Pending,
            raw_data_uri: raw_data_uri.to_string(),
            curated_data_uri: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.curation_repository.create_job(&job).await?;
        
        // TODO: Trigger async processing
        
        Ok(job.id.to_string())
    }
}