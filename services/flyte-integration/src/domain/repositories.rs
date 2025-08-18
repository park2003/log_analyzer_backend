use async_trait::async_trait;
use uuid::Uuid;

use super::models::{WorkflowExecution, ExecutionStatus};

/// Repository trait for persisting workflow executions
#[async_trait]
pub trait ExecutionRepository: Send + Sync {
    /// Save a new workflow execution
    async fn create(&self, execution: &WorkflowExecution) -> Result<(), anyhow::Error>;
    
    /// Update an existing workflow execution
    async fn update(&self, execution: &WorkflowExecution) -> Result<(), anyhow::Error>;
    
    /// Get a workflow execution by ID
    async fn get_by_id(&self, id: Uuid) -> Result<Option<WorkflowExecution>, anyhow::Error>;
    
    /// Get a workflow execution by workflow ID
    async fn get_by_workflow_id(&self, workflow_id: &str) -> Result<Option<WorkflowExecution>, anyhow::Error>;
    
    /// Get a workflow execution by Flyte execution ID
    async fn get_by_flyte_id(&self, flyte_id: &str) -> Result<Option<WorkflowExecution>, anyhow::Error>;
    
    /// List all executions with a specific status
    async fn list_by_status(&self, status: ExecutionStatus) -> Result<Vec<WorkflowExecution>, anyhow::Error>;
    
    /// List all non-terminal executions (for monitoring)
    async fn list_active(&self) -> Result<Vec<WorkflowExecution>, anyhow::Error>;
}