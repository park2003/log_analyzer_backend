use async_trait::async_trait;
use anyhow::Result;
use crate::domain::models::WorkflowExecution;

#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn create(&self, execution: &WorkflowExecution) -> Result<()>;
    async fn get_by_id(&self, id: &str) -> Result<Option<WorkflowExecution>>;
    async fn get_by_execution_id(&self, execution_id: &str) -> Result<Option<WorkflowExecution>>;
    async fn update_status(&self, id: &str, status: &str) -> Result<()>;
}