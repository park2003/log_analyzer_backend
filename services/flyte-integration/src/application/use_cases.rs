use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{
    models::{ExecutionStatus, WorkflowDefinition, WorkflowExecution},
    repositories::ExecutionRepository,
};

/// Client trait for interacting with Flyte
#[async_trait::async_trait]
pub trait FlyteClient: Send + Sync {
    /// Submit a workflow to Flyte for execution
    async fn submit_workflow(
        &self,
        workflow_definition: serde_json::Value,
        project: &str,
        domain: &str,
    ) -> Result<String, anyhow::Error>;

    /// Get the status of a Flyte execution
    async fn get_execution_status(&self, execution_id: &str) -> Result<String, anyhow::Error>;
}

/// Use cases for workflow execution management
pub struct WorkflowUseCases<R, F>
where
    R: ExecutionRepository,
    F: FlyteClient,
{
    repository: Arc<R>,
    flyte_client: Arc<F>,
}

impl<R, F> WorkflowUseCases<R, F>
where
    R: ExecutionRepository,
    F: FlyteClient,
{
    /// Create a new instance of WorkflowUseCases
    pub fn new(repository: Arc<R>, flyte_client: Arc<F>) -> Self {
        Self {
            repository,
            flyte_client,
        }
    }

    /// Execute a workflow by submitting it to Flyte
    pub async fn execute_workflow(
        &self,
        workflow_id: String,
        workflow_json: serde_json::Value,
        project: String,
        domain: String,
    ) -> Result<WorkflowExecution, anyhow::Error> {
        // Create a new execution record
        let mut execution = WorkflowExecution::new(workflow_id);

        // Save initial execution state
        self.repository.create(&execution).await?;

        // Submit to Flyte
        match self
            .flyte_client
            .submit_workflow(workflow_json, &project, &domain)
            .await
        {
            Ok(flyte_execution_id) => {
                // Update execution with Flyte ID
                execution.set_flyte_execution_id(flyte_execution_id);
                execution.update_status(ExecutionStatus::Submitted);
                self.repository.update(&execution).await?;
                Ok(execution)
            }
            Err(e) => {
                // Mark execution as failed
                execution.set_error(format!("Failed to submit to Flyte: {}", e));
                self.repository.update(&execution).await?;
                Err(e)
            }
        }
    }

    /// Get the status of a workflow execution
    pub async fn get_execution_status(
        &self,
        execution_id: Uuid,
    ) -> Result<WorkflowExecution, anyhow::Error> {
        // Get execution from repository
        let mut execution = self
            .repository
            .get_by_id(execution_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Execution not found: {}", execution_id))?;

        // If execution is not terminal, query Flyte for updated status
        if !execution.status.is_terminal() {
            if let Some(flyte_id) = &execution.flyte_execution_id {
                match self.flyte_client.get_execution_status(flyte_id).await {
                    Ok(status_str) => {
                        let new_status = match status_str.as_str() {
                            "RUNNING" => ExecutionStatus::Running,
                            "SUCCEEDED" => ExecutionStatus::Succeeded,
                            "FAILED" => ExecutionStatus::Failed,
                            "ABORTED" => ExecutionStatus::Aborted,
                            "TIMED_OUT" => ExecutionStatus::TimedOut,
                            _ => execution.status.clone(),
                        };

                        if new_status != execution.status {
                            execution.update_status(new_status);
                            self.repository.update(&execution).await?;
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get Flyte status for {}: {}", flyte_id, e);
                    }
                }
            }
        }

        Ok(execution)
    }

    /// Monitor and update all active executions
    pub async fn monitor_active_executions(&self) -> Result<(), anyhow::Error> {
        let active_executions = self.repository.list_active().await?;

        for execution in active_executions {
            if let Some(flyte_id) = &execution.flyte_execution_id {
                match self.flyte_client.get_execution_status(flyte_id).await {
                    Ok(status_str) => {
                        let mut updated_execution = execution.clone();
                        let new_status = match status_str.as_str() {
                            "RUNNING" => ExecutionStatus::Running,
                            "SUCCEEDED" => ExecutionStatus::Succeeded,
                            "FAILED" => ExecutionStatus::Failed,
                            "ABORTED" => ExecutionStatus::Aborted,
                            "TIMED_OUT" => ExecutionStatus::TimedOut,
                            _ => execution.status.clone(),
                        };

                        if new_status != execution.status {
                            updated_execution.update_status(new_status.clone());
                            self.repository.update(&updated_execution).await?;
                            tracing::info!(
                                "Updated execution {} to status {:?}",
                                execution.id,
                                new_status
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to get Flyte status for {}: {}", flyte_id, e);
                    }
                }
            }
        }

        Ok(())
    }
}
