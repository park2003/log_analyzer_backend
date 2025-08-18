use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::domain::models::{CurationJob, ImageEmbedding, ImageFeedback};

#[async_trait]
pub trait CurationJobRepository: Send + Sync {
    async fn create(&self, job: &CurationJob) -> Result<()>;
    async fn get_by_id(&self, job_id: &Uuid) -> Result<Option<CurationJob>>;
    async fn update(&self, job: &CurationJob) -> Result<()>;
    async fn list_by_project(&self, project_id: &str) -> Result<Vec<CurationJob>>;
}

#[async_trait]
pub trait ImageEmbeddingRepository: Send + Sync {
    async fn save(&self, embedding: &ImageEmbedding) -> Result<()>;
    async fn save_batch(&self, embeddings: &[ImageEmbedding]) -> Result<()>;
    async fn get_by_project(&self, project_id: &str) -> Result<Vec<ImageEmbedding>>;
    async fn get_by_image_uri(&self, image_uri: &str) -> Result<Option<ImageEmbedding>>;
    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<ImageEmbedding>>;
    async fn find_cluster_boundaries(&self, project_id: &str, n_samples: usize) -> Result<Vec<ImageEmbedding>>;
}