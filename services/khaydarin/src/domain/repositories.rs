use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::{FineTuningJob, ProcessingRequest};

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create(&self, job: &FineTuningJob) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<FineTuningJob>>;
    async fn update(&self, job: &FineTuningJob) -> Result<()>;
}

#[async_trait]
pub trait ProcessingRepository: Send + Sync {
    async fn save_request(&self, request: &ProcessingRequest) -> Result<()>;
    async fn get_request(&self, request_id: &str) -> Result<Option<ProcessingRequest>>;
}