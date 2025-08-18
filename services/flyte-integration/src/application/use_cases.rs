use anyhow::Result;
use uuid::Uuid;
use serde_json::Value;
use crate::domain::{
    models::{WorkflowExecution, ExecutionStatus},
    repositories::WorkflowRepository,
};

pub struct ExecuteWorkflowUseCase<R: WorkflowRepository> {
    workflow_repository: R,
}

impl<R: WorkflowRepository> ExecuteWorkflowUseCase<R> {
    pub fn new(workflow_repository: R) -> Self {
        Self {
            workflow_repository,
        }
    }

    pub async fn execute(
        &self,
        workflow_id: &str,
        workflow_definition: Value,
    ) -> Result<String> {
        let execution = WorkflowExecution {
            id: Uuid::new_v4(),
            workflow_id: workflow_id.to_string(),
            execution_id: Uuid::new_v4().to_string(),
            status: ExecutionStatus::Submitted,
            definition: workflow_definition.to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        self.workflow_repository.create(&execution).await?;
        
        // TODO: Actually submit to Flyte
        
        Ok(execution.execution_id)
    }

    pub async fn get_status(&self, execution_id: &str) -> Result<Option<String>> {
        let execution = self.workflow_repository.get_by_execution_id(execution_id).await?;
        Ok(execution.map(|e| format!("{:?}", e.status)))
    }
}