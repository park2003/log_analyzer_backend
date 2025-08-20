use crate::domain::{
    models::{KhaydarinLogEntry, ProcessingStatus},
    repositories::KhaydarinRepository,
};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};

// PostgreSQL implementation of KhaydarinRepository
pub struct PostgresKhaydarinRepository {
    pool: PgPool,
}

impl PostgresKhaydarinRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Initialize database tables
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS khaydarin_logs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                request_id VARCHAR(255) UNIQUE NOT NULL,
                user_id VARCHAR(255) NOT NULL,
                received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_prompt TEXT NOT NULL,
                llm_prompt TEXT,
                llm_raw_response TEXT,
                structured_plan JSONB,
                status VARCHAR(50) NOT NULL,
                processing_time_ms INT
            );
            
            CREATE INDEX IF NOT EXISTS idx_khaydarin_logs_user_id ON khaydarin_logs(user_id);
            CREATE INDEX IF NOT EXISTS idx_khaydarin_logs_received_at ON khaydarin_logs(received_at);
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl KhaydarinRepository for PostgresKhaydarinRepository {
    async fn log_request(&self, entry: &KhaydarinLogEntry) -> Result<()> {
        let status_str = match entry.status {
            ProcessingStatus::Success => "SUCCESS",
            ProcessingStatus::LlmError => "LLM_ERROR",
            ProcessingStatus::ParsingError => "PARSING_ERROR",
        };

        sqlx::query(
            r#"
            INSERT INTO khaydarin_logs (
                id, request_id, user_id, received_at, user_prompt,
                llm_prompt, llm_raw_response, structured_plan, status, processing_time_ms
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(&entry.id)
        .bind(&entry.request_id)
        .bind(&entry.user_id)
        .bind(&entry.received_at)
        .bind(&entry.user_prompt)
        .bind(&entry.llm_prompt)
        .bind(&entry.llm_raw_response)
        .bind(&entry.structured_plan)
        .bind(status_str)
        .bind(&entry.processing_time_ms)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_request_id(&self, request_id: &str) -> Result<Option<KhaydarinLogEntry>> {
        let row = sqlx::query(
            r#"
            SELECT id, request_id, user_id, received_at, user_prompt,
                   llm_prompt, llm_raw_response, structured_plan, status, processing_time_ms
            FROM khaydarin_logs
            WHERE request_id = $1
            "#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let status_str: String = row.get("status");
                let status = match status_str.as_str() {
                    "SUCCESS" => ProcessingStatus::Success,
                    "LLM_ERROR" => ProcessingStatus::LlmError,
                    "PARSING_ERROR" => ProcessingStatus::ParsingError,
                    _ => ProcessingStatus::LlmError,
                };

                Ok(Some(KhaydarinLogEntry {
                    id: row.get("id"),
                    request_id: row.get("request_id"),
                    user_id: row.get("user_id"),
                    received_at: row.get("received_at"),
                    user_prompt: row.get("user_prompt"),
                    llm_prompt: row.get("llm_prompt"),
                    llm_raw_response: row.get("llm_raw_response"),
                    structured_plan: row.get("structured_plan"),
                    status,
                    processing_time_ms: row.get("processing_time_ms"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<KhaydarinLogEntry>> {
        let rows = sqlx::query(
            r#"
            SELECT id, request_id, user_id, received_at, user_prompt,
                   llm_prompt, llm_raw_response, structured_plan, status, processing_time_ms
            FROM khaydarin_logs
            WHERE user_id = $1
            ORDER BY received_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut entries = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "SUCCESS" => ProcessingStatus::Success,
                "LLM_ERROR" => ProcessingStatus::LlmError,
                "PARSING_ERROR" => ProcessingStatus::ParsingError,
                _ => ProcessingStatus::LlmError,
            };

            entries.push(KhaydarinLogEntry {
                id: row.get("id"),
                request_id: row.get("request_id"),
                user_id: row.get("user_id"),
                received_at: row.get("received_at"),
                user_prompt: row.get("user_prompt"),
                llm_prompt: row.get("llm_prompt"),
                llm_raw_response: row.get("llm_raw_response"),
                structured_plan: row.get("structured_plan"),
                status,
                processing_time_ms: row.get("processing_time_ms"),
            });
        }

        Ok(entries)
    }

    async fn update_processing_result(
        &self,
        request_id: &str,
        entry: &KhaydarinLogEntry,
    ) -> Result<()> {
        let status_str = match entry.status {
            ProcessingStatus::Success => "SUCCESS",
            ProcessingStatus::LlmError => "LLM_ERROR",
            ProcessingStatus::ParsingError => "PARSING_ERROR",
        };

        sqlx::query(
            r#"
            UPDATE khaydarin_logs
            SET llm_prompt = $1,
                llm_raw_response = $2,
                structured_plan = $3,
                status = $4,
                processing_time_ms = $5
            WHERE request_id = $6
            "#,
        )
        .bind(&entry.llm_prompt)
        .bind(&entry.llm_raw_response)
        .bind(&entry.structured_plan)
        .bind(status_str)
        .bind(&entry.processing_time_ms)
        .bind(request_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn request_exists(&self, request_id: &str) -> Result<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM khaydarin_logs WHERE request_id = $1")
                .bind(request_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }
}

// Database connection function
pub async fn connect() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/khaydarin".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}
