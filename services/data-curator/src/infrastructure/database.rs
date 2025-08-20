use crate::domain::{
    models::{CurationJob, CurationStatus, ImageEmbedding, ImageForFeedback},
    repositories::{CurationJobRepository, ImageEmbeddingRepository},
};
use anyhow::Result;
use async_trait::async_trait;
use pgvector::Vector;
use sqlx::{PgPool, Row, postgres::PgPoolOptions, postgres::PgRow};
use uuid::Uuid;

pub struct PostgresCurationJobRepository {
    pool: PgPool,
}

impl PostgresCurationJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CurationJobRepository for PostgresCurationJobRepository {
    async fn create(&self, job: &CurationJob) -> Result<()> {
        let status = serde_json::to_value(&job.status)?;
        let images = serde_json::to_value(&job.images_for_feedback)?;

        sqlx::query(
            r#"
            INSERT INTO curation_jobs (
                id, project_id, status, raw_data_uri, 
                curated_data_uri, images_for_feedback, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(job.id)
        .bind(&job.project_id)
        .bind(&status)
        .bind(&job.raw_data_uri)
        .bind(&job.curated_data_uri)
        .bind(&images)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_by_id(&self, job_id: &Uuid) -> Result<Option<CurationJob>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, status, raw_data_uri, 
                   curated_data_uri, images_for_feedback, created_at, updated_at
            FROM curation_jobs
            WHERE id = $1
            "#
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let status: CurationStatus = serde_json::from_value(r.get("status"))?;
                let images: Vec<ImageForFeedback> = serde_json::from_value(r.get("images_for_feedback"))?;

                Ok(Some(CurationJob {
                    id: r.get("id"),
                    project_id: r.get("project_id"),
                    status,
                    raw_data_uri: r.get("raw_data_uri"),
                    curated_data_uri: r.get("curated_data_uri"),
                    images_for_feedback: images,
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, job: &CurationJob) -> Result<()> {
        let status = serde_json::to_value(&job.status)?;
        let images = serde_json::to_value(&job.images_for_feedback)?;

        sqlx::query(
            r#"
            UPDATE curation_jobs
            SET project_id = $2, status = $3, raw_data_uri = $4,
                curated_data_uri = $5, images_for_feedback = $6, updated_at = $7
            WHERE id = $1
            "#
        )
        .bind(job.id)
        .bind(&job.project_id)
        .bind(&status)
        .bind(&job.raw_data_uri)
        .bind(&job.curated_data_uri)
        .bind(&images)
        .bind(job.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_by_project(&self, project_id: &str) -> Result<Vec<CurationJob>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, status, raw_data_uri, 
                   curated_data_uri, images_for_feedback, created_at, updated_at
            FROM curation_jobs
            WHERE project_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for r in rows {
            let status: CurationStatus = serde_json::from_value(r.get("status"))?;
            let images: Vec<ImageForFeedback> = serde_json::from_value(r.get("images_for_feedback"))?;

            jobs.push(CurationJob {
                id: r.get("id"),
                project_id: r.get("project_id"),
                status,
                raw_data_uri: r.get("raw_data_uri"),
                curated_data_uri: r.get("curated_data_uri"),
                images_for_feedback: images,
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
            });
        }

        Ok(jobs)
    }
}

pub struct PgVectorEmbeddingRepository {
    pool: PgPool,
}

impl PgVectorEmbeddingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ImageEmbeddingRepository for PgVectorEmbeddingRepository {
    async fn save(&self, embedding: &ImageEmbedding) -> Result<()> {
        let vec = Vector::from(embedding.embedding.clone());

        sqlx::query(
            r#"
            INSERT INTO image_embeddings (id, project_id, image_uri, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#
        )
        .bind(embedding.id)
        .bind(&embedding.project_id)
        .bind(&embedding.image_uri)
        .bind(vec)
        .bind(embedding.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn save_batch(&self, embeddings: &[ImageEmbedding]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for embedding in embeddings {
            let vec = Vector::from(embedding.embedding.clone());

            sqlx::query(
                r#"
                INSERT INTO image_embeddings (id, project_id, image_uri, embedding, created_at)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(embedding.id)
            .bind(&embedding.project_id)
            .bind(&embedding.image_uri)
            .bind(&vec)
            .bind(embedding.created_at)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_by_project(&self, project_id: &str) -> Result<Vec<ImageEmbedding>> {
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, image_uri, embedding, created_at
            FROM image_embeddings
            WHERE project_id = $1
            "#
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await?;

        let mut embeddings = Vec::new();
        for r in rows {
            let vec: Vector = r.get("embedding");
            embeddings.push(ImageEmbedding {
                id: r.get("id"),
                project_id: r.get("project_id"),
                image_uri: r.get("image_uri"),
                embedding: vec.to_vec(),
                created_at: r.get("created_at"),
            });
        }

        Ok(embeddings)
    }

    async fn get_by_image_uri(&self, image_uri: &str) -> Result<Option<ImageEmbedding>> {
        let row = sqlx::query(
            r#"
            SELECT id, project_id, image_uri, embedding, created_at
            FROM image_embeddings
            WHERE image_uri = $1
            "#
        )
        .bind(image_uri)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let vec: Vector = r.get("embedding");
                Ok(Some(ImageEmbedding {
                    id: r.get("id"),
                    project_id: r.get("project_id"),
                    image_uri: r.get("image_uri"),
                    embedding: vec.to_vec(),
                    created_at: r.get("created_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_similar(&self, embedding: &[f32], limit: usize) -> Result<Vec<ImageEmbedding>> {
        let vec = Vector::from(embedding.to_vec());

        // Using cosine distance (<=>)
        let rows = sqlx::query(
            r#"
            SELECT id, project_id, image_uri, embedding, created_at
            FROM image_embeddings
            ORDER BY embedding <=> $1
            LIMIT $2
            "#
        )
        .bind(vec)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut embeddings = Vec::new();
        for r in rows {
            let vec: Vector = r.get("embedding");
            embeddings.push(ImageEmbedding {
                id: r.get("id"),
                project_id: r.get("project_id"),
                image_uri: r.get("image_uri"),
                embedding: vec.to_vec(),
                created_at: r.get("created_at"),
            });
        }

        Ok(embeddings)
    }

    async fn find_cluster_boundaries(
        &self,
        project_id: &str,
        n_samples: usize,
    ) -> Result<Vec<ImageEmbedding>> {
        // This is a simplified implementation
        // In production, you'd use a more sophisticated clustering algorithm

        // Get all embeddings for the project
        let all_embeddings = self.get_by_project(project_id).await?;

        if all_embeddings.len() <= n_samples {
            return Ok(all_embeddings);
        }

        // Simple sampling: take evenly distributed samples
        // In production, use k-means or DBSCAN to find cluster boundaries
        let step = all_embeddings.len() / n_samples;
        let mut selected = Vec::new();

        for i in (0..all_embeddings.len()).step_by(step).take(n_samples) {
            selected.push(all_embeddings[i].clone());
        }

        Ok(selected)
    }
}

pub async fn connect() -> Result<PgPool> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/khaydarin".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations - commented out for now as migrations directory doesn't exist
    // sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async fn setup_database(pool: &PgPool) -> Result<()> {
    // Enable pgvector extension
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(pool)
        .await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS curation_jobs (
            id UUID PRIMARY KEY,
            project_id TEXT NOT NULL,
            status JSONB NOT NULL,
            raw_data_uri TEXT NOT NULL,
            curated_data_uri TEXT,
            images_for_feedback JSONB NOT NULL DEFAULT '[]'::jsonb,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS image_embeddings (
            id UUID PRIMARY KEY,
            project_id TEXT NOT NULL,
            image_uri TEXT NOT NULL UNIQUE,
            embedding vector(768) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL
        )
        "#
    )
    .execute(pool)
    .await?;

    // Create indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_curation_jobs_project_id ON curation_jobs(project_id)"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_image_embeddings_project_id ON image_embeddings(project_id)"
    )
    .execute(pool)
    .await?;

    // Create HNSW index for vector similarity search
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_image_embeddings_hnsw ON image_embeddings USING hnsw (embedding vector_cosine_ops)")
        .execute(pool)
        .await?;

    Ok(())
}
