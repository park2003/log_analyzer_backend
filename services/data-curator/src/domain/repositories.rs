use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::{CurationJob, ImageEmbedding, ImageFeedback};

#[async_trait]
pub trait CurationRepository: Send + Sync {
    async fn create_job(&self, job: &CurationJob) -> Result<()>;
    async fn get_job(&self, job_id: &str) -> Result<Option<CurationJob>>;
    async fn update_job(&self, job: &CurationJob) -> Result<()>;
}

#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
    async fn save_embedding(&self, embedding: &ImageEmbedding) -> Result<()>;
    async fn get_embeddings(&self, project_id: &str) -> Result<Vec<ImageEmbedding>>;
    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<ImageEmbedding>>;
}