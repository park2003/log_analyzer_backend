use crate::domain::{
    models::{CurationJob, CurationStatus, ImageFeedback, ImageForFeedback},
    repositories::{CurationJobRepository, ImageEmbeddingRepository},
};
use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

// Service interfaces for infrastructure dependencies
#[async_trait::async_trait]
pub trait StorageService: Send + Sync {
    async fn list_images(&self, uri: &str) -> Result<Vec<String>>;
    async fn download_image(&self, uri: &str) -> Result<Vec<u8>>;
    async fn upload_dataset(&self, data: Vec<String>, uri: &str) -> Result<()>;
}

#[async_trait::async_trait]
pub trait EmbeddingService: Send + Sync {
    async fn generate_embedding(&self, image_data: &[u8]) -> Result<Vec<f32>>;
}

pub struct StartCurationUseCase<C, E, S, EM>
where
    C: CurationJobRepository + 'static,
    E: ImageEmbeddingRepository + 'static,
    S: StorageService + 'static,
    EM: EmbeddingService + 'static,
{
    job_repository: Arc<C>,
    embedding_repository: Arc<E>,
    storage_service: Arc<S>,
    embedding_service: Arc<EM>,
}

impl<C, E, S, EM> StartCurationUseCase<C, E, S, EM>
where
    C: CurationJobRepository + 'static,
    E: ImageEmbeddingRepository + 'static,
    S: StorageService + 'static,
    EM: EmbeddingService + 'static,
{
    pub fn new(
        job_repository: Arc<C>,
        embedding_repository: Arc<E>,
        storage_service: Arc<S>,
        embedding_service: Arc<EM>,
    ) -> Self {
        Self {
            job_repository,
            embedding_repository,
            storage_service,
            embedding_service,
        }
    }

    pub async fn execute(&self, project_id: String, raw_data_uri: String) -> Result<Uuid> {
        // Create new curation job
        let job = CurationJob::new(project_id, raw_data_uri);
        let job_id = job.id;

        self.job_repository.create(&job).await?;

        // Start async embedding generation process
        let job_repo = Arc::clone(&self.job_repository);
        let embedding_repo = Arc::clone(&self.embedding_repository);
        let storage = Arc::clone(&self.storage_service);
        let embedder = Arc::clone(&self.embedding_service);

        tokio::spawn(async move {
            // This would be better as a separate background task/worker
            if let Err(e) =
                Self::process_embeddings(job_id, job_repo, embedding_repo, storage, embedder).await
            {
                tracing::error!("Failed to process embeddings: {}", e);
            }
        });

        Ok(job_id)
    }

    async fn process_embeddings(
        job_id: Uuid,
        job_repo: Arc<C>,
        embedding_repo: Arc<E>,
        storage: Arc<S>,
        embedder: Arc<EM>,
    ) -> Result<()> {
        // Update job status to Embedding
        if let Some(mut job) = job_repo.get_by_id(&job_id).await? {
            job.transition_to_embedding();
            job_repo.update(&job).await?;

            // List and process images
            let image_uris = storage.list_images(&job.raw_data_uri).await?;

            for uri in image_uris {
                let image_data = storage.download_image(&uri).await?;
                let embedding = embedder.generate_embedding(&image_data).await?;

                let image_embedding = crate::domain::models::ImageEmbedding {
                    id: Uuid::new_v4(),
                    project_id: job.project_id.clone(),
                    image_uri: uri,
                    embedding,
                    created_at: chrono::Utc::now(),
                };

                embedding_repo.save(&image_embedding).await?;
            }

            // After embeddings are generated, select images for feedback
            let selected_images = embedding_repo
                .find_cluster_boundaries(&job.project_id, 20)
                .await?;

            let images_for_feedback: Vec<ImageForFeedback> = selected_images
                .into_iter()
                .map(|e| ImageForFeedback {
                    image_id: e.id.to_string(),
                    image_uri: e.image_uri,
                })
                .collect();

            // Update job to awaiting feedback
            let mut job = job_repo.get_by_id(&job_id).await?.unwrap();
            job.transition_to_awaiting_feedback(images_for_feedback);
            job_repo.update(&job).await?;
        }

        Ok(())
    }
}

pub struct GetCurationStatusUseCase<C>
where
    C: CurationJobRepository,
{
    job_repository: Arc<C>,
}

impl<C> GetCurationStatusUseCase<C>
where
    C: CurationJobRepository,
{
    pub fn new(job_repository: Arc<C>) -> Self {
        Self { job_repository }
    }

    pub async fn execute(&self, job_id: Uuid) -> Result<Option<CurationJob>> {
        self.job_repository.get_by_id(&job_id).await
    }
}

pub struct SubmitFeedbackUseCase<C, E, S>
where
    C: CurationJobRepository,
    E: ImageEmbeddingRepository,
    S: StorageService,
{
    job_repository: Arc<C>,
    embedding_repository: Arc<E>,
    storage_service: Arc<S>,
}

impl<C, E, S> SubmitFeedbackUseCase<C, E, S>
where
    C: CurationJobRepository,
    E: ImageEmbeddingRepository,
    S: StorageService,
{
    pub fn new(
        job_repository: Arc<C>,
        embedding_repository: Arc<E>,
        storage_service: Arc<S>,
    ) -> Self {
        Self {
            job_repository,
            embedding_repository,
            storage_service,
        }
    }

    pub async fn execute(&self, job_id: Uuid, feedback: Vec<ImageFeedback>) -> Result<bool> {
        if let Some(mut job) = self.job_repository.get_by_id(&job_id).await? {
            // Filter accepted images
            let accepted_image_ids: Vec<String> = feedback
                .into_iter()
                .filter(|f| f.accepted)
                .map(|f| f.image_id)
                .collect();

            // Get the accepted image URIs
            let embeddings = self
                .embedding_repository
                .get_by_project(&job.project_id)
                .await?;

            let accepted_uris: Vec<String> = embeddings
                .into_iter()
                .filter(|e| accepted_image_ids.contains(&e.id.to_string()))
                .map(|e| e.image_uri)
                .collect();

            // Create curated dataset URI
            let curated_uri = format!("{}/curated", job.raw_data_uri.trim_end_matches("/raw"));

            // Upload curated dataset
            self.storage_service
                .upload_dataset(accepted_uris, &curated_uri)
                .await?;

            // Update job to completed
            job.complete(curated_uri);
            self.job_repository.update(&job).await?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
