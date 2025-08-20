use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row, postgres::PgRow};
use uuid::Uuid;

use crate::domain::{
    models::{ExecutionStatus, WorkflowExecution},
    repositories::ExecutionRepository,
};

/// PostgreSQL implementation of the ExecutionRepository
pub struct PostgresExecutionRepository {
    pool: PgPool,
}

impl PostgresExecutionRepository {
    /// Create a new PostgreSQL repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workflow_executions (
                id UUID PRIMARY KEY,
                workflow_id VARCHAR(255) NOT NULL,
                flyte_execution_id VARCHAR(255) UNIQUE,
                status VARCHAR(50) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL,
                started_at TIMESTAMPTZ,
                completed_at TIMESTAMPTZ,
                error_message TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id 
            ON workflow_executions(workflow_id)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_workflow_executions_status 
            ON workflow_executions(status)
            "#,
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_workflow_executions_created_at 
            ON workflow_executions(created_at)
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Helper function to convert database row to WorkflowExecution
fn row_to_execution(row: &PgRow) -> Result<WorkflowExecution, sqlx::Error> {
    let status_str: String = row.try_get("status")?;
    let status = match status_str.as_str() {
        "Pending" => ExecutionStatus::Pending,
        "Submitted" => ExecutionStatus::Submitted,
        "Running" => ExecutionStatus::Running,
        "Succeeded" => ExecutionStatus::Succeeded,
        "Failed" => ExecutionStatus::Failed,
        "Aborted" => ExecutionStatus::Aborted,
        "TimedOut" => ExecutionStatus::TimedOut,
        _ => ExecutionStatus::Pending, // Default fallback
    };

    Ok(WorkflowExecution {
        id: row.try_get("id")?,
        workflow_id: row.try_get("workflow_id")?,
        flyte_execution_id: row.try_get("flyte_execution_id")?,
        status,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
        started_at: row.try_get("started_at")?,
        completed_at: row.try_get("completed_at")?,
        error_message: row.try_get("error_message")?,
    })
}

/// Helper function to convert ExecutionStatus to string
fn status_to_string(status: &ExecutionStatus) -> &'static str {
    match status {
        ExecutionStatus::Pending => "Pending",
        ExecutionStatus::Submitted => "Submitted",
        ExecutionStatus::Running => "Running",
        ExecutionStatus::Succeeded => "Succeeded",
        ExecutionStatus::Failed => "Failed",
        ExecutionStatus::Aborted => "Aborted",
        ExecutionStatus::TimedOut => "TimedOut",
    }
}

#[async_trait]
impl ExecutionRepository for PostgresExecutionRepository {
    async fn create(&self, execution: &WorkflowExecution) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            INSERT INTO workflow_executions 
            (id, workflow_id, flyte_execution_id, status, created_at, updated_at, started_at, completed_at, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(execution.id)
        .bind(&execution.workflow_id)
        .bind(&execution.flyte_execution_id)
        .bind(status_to_string(&execution.status))
        .bind(execution.created_at)
        .bind(execution.updated_at)
        .bind(execution.started_at)
        .bind(execution.completed_at)
        .bind(&execution.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, execution: &WorkflowExecution) -> Result<(), anyhow::Error> {
        sqlx::query(
            r#"
            UPDATE workflow_executions 
            SET workflow_id = $2, 
                flyte_execution_id = $3, 
                status = $4, 
                updated_at = $5, 
                started_at = $6, 
                completed_at = $7, 
                error_message = $8
            WHERE id = $1
            "#,
        )
        .bind(execution.id)
        .bind(&execution.workflow_id)
        .bind(&execution.flyte_execution_id)
        .bind(status_to_string(&execution.status))
        .bind(execution.updated_at)
        .bind(execution.started_at)
        .bind(execution.completed_at)
        .bind(&execution.error_message)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, id: Uuid) -> Result<Option<WorkflowExecution>, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM workflow_executions WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_execution(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_by_workflow_id(
        &self,
        workflow_id: &str,
    ) -> Result<Option<WorkflowExecution>, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM workflow_executions 
            WHERE workflow_id = $1 
            ORDER BY created_at DESC 
            LIMIT 1
            "#,
        )
        .bind(workflow_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_execution(&row)?)),
            None => Ok(None),
        }
    }

    async fn get_by_flyte_id(
        &self,
        flyte_id: &str,
    ) -> Result<Option<WorkflowExecution>, anyhow::Error> {
        let row = sqlx::query(
            r#"
            SELECT * FROM workflow_executions WHERE flyte_execution_id = $1
            "#,
        )
        .bind(flyte_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_execution(&row)?)),
            None => Ok(None),
        }
    }

    async fn list_by_status(
        &self,
        status: ExecutionStatus,
    ) -> Result<Vec<WorkflowExecution>, anyhow::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM workflow_executions 
            WHERE status = $1 
            ORDER BY created_at DESC
            "#,
        )
        .bind(status_to_string(&status))
        .fetch_all(&self.pool)
        .await?;

        let mut executions = Vec::new();
        for row in rows {
            executions.push(row_to_execution(&row)?);
        }

        Ok(executions)
    }

    async fn list_active(&self) -> Result<Vec<WorkflowExecution>, anyhow::Error> {
        let rows = sqlx::query(
            r#"
            SELECT * FROM workflow_executions 
            WHERE status IN ('Pending', 'Submitted', 'Running')
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut executions = Vec::new();
        for row in rows {
            executions.push(row_to_execution(&row)?);
        }

        Ok(executions)
    }
}
