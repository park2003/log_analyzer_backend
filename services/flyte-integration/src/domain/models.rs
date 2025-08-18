use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a workflow execution in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: String,
    pub flyte_execution_id: Option<String>,
    pub status: ExecutionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

/// Status of a workflow execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Pending,
    Submitted,
    Running,
    Succeeded,
    Failed,
    Aborted,
    TimedOut,
}

impl ExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ExecutionStatus::Succeeded
                | ExecutionStatus::Failed
                | ExecutionStatus::Aborted
                | ExecutionStatus::TimedOut
        )
    }
}

/// Represents a workflow definition that can be executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_id: String,
    pub flyte_workflow_json: serde_json::Value,
    pub project: String,
    pub domain: String,
}

impl WorkflowExecution {
    /// Create a new workflow execution
    pub fn new(workflow_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            flyte_execution_id: None,
            status: ExecutionStatus::Pending,
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }

    /// Update the status of the execution
    pub fn update_status(&mut self, status: ExecutionStatus) {
        self.status = status;
        self.updated_at = Utc::now();
        
        if status == ExecutionStatus::Running && self.started_at.is_none() {
            self.started_at = Some(Utc::now());
        }
        
        if status.is_terminal() && self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }

    /// Set the Flyte execution ID after successful submission
    pub fn set_flyte_execution_id(&mut self, flyte_id: String) {
        self.flyte_execution_id = Some(flyte_id);
        self.updated_at = Utc::now();
    }

    /// Set error message for failed executions
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.status = ExecutionStatus::Failed;
        self.updated_at = Utc::now();
        if self.completed_at.is_none() {
            self.completed_at = Some(Utc::now());
        }
    }
}