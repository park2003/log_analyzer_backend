use crate::{
    application::{
        EmbeddingService, GetCurationStatusUseCase, StartCurationUseCase, StorageService,
        SubmitFeedbackUseCase,
    },
    domain::{
        models::{
            ImageFeedback as DomainImageFeedback, ImageForFeedback as DomainImageForFeedback,
        },
        repositories::{CurationJobRepository, ImageEmbeddingRepository},
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

// Include the generated proto code
pub mod data_curator_proto {
    tonic::include_proto!("savassan.data_curator.v1");
}

use data_curator_proto::{
    CurationStatus, Feedback, GetCurationStatusRequest, ImageForFeedback, JobStatus,
    StartCurationRequest, StartCurationResponse, SubmitFeedbackRequest, SubmitFeedbackResponse,
    data_curator_service_server::{DataCuratorService, DataCuratorServiceServer},
};

pub struct DataCuratorGrpcService<C, E, S, EM>
where
    C: CurationJobRepository + Send + Sync + 'static,
    E: ImageEmbeddingRepository + Send + Sync + 'static,
    S: StorageService + Send + Sync + 'static,
    EM: EmbeddingService + Send + Sync + 'static,
{
    start_curation_use_case: Arc<StartCurationUseCase<C, E, S, EM>>,
    get_status_use_case: Arc<GetCurationStatusUseCase<C>>,
    submit_feedback_use_case: Arc<SubmitFeedbackUseCase<C, E, S>>,
}

impl<C, E, S, EM> DataCuratorGrpcService<C, E, S, EM>
where
    C: CurationJobRepository + Send + Sync + 'static,
    E: ImageEmbeddingRepository + Send + Sync + 'static,
    S: StorageService + Send + Sync + 'static,
    EM: EmbeddingService + Send + Sync + 'static,
{
    pub fn new(
        job_repository: Arc<C>,
        embedding_repository: Arc<E>,
        storage_service: Arc<S>,
        embedding_service: Arc<EM>,
    ) -> Self {
        let start_curation_use_case = Arc::new(StartCurationUseCase::new(
            Arc::clone(&job_repository),
            Arc::clone(&embedding_repository),
            Arc::clone(&storage_service),
            Arc::clone(&embedding_service),
        ));

        let get_status_use_case =
            Arc::new(GetCurationStatusUseCase::new(Arc::clone(&job_repository)));

        let submit_feedback_use_case = Arc::new(SubmitFeedbackUseCase::new(
            Arc::clone(&job_repository),
            Arc::clone(&embedding_repository),
            Arc::clone(&storage_service),
        ));

        Self {
            start_curation_use_case,
            get_status_use_case,
            submit_feedback_use_case,
        }
    }

    pub fn into_server(self) -> DataCuratorServiceServer<Self> {
        DataCuratorServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl<C, E, S, EM> DataCuratorService for DataCuratorGrpcService<C, E, S, EM>
where
    C: CurationJobRepository + Send + Sync + 'static,
    E: ImageEmbeddingRepository + Send + Sync + 'static,
    S: StorageService + Send + Sync + 'static,
    EM: EmbeddingService + Send + Sync + 'static,
{
    async fn start_curation(
        &self,
        request: Request<StartCurationRequest>,
    ) -> Result<Response<StartCurationResponse>, Status> {
        let req = request.into_inner();

        tracing::info!(
            project_id = %req.project_id,
            raw_data_uri = %req.raw_data_uri,
            "Starting curation process"
        );

        let job_id = self
            .start_curation_use_case
            .execute(req.project_id, req.raw_data_uri)
            .await
            .map_err(|e| {
                tracing::error!("Failed to start curation: {}", e);
                Status::internal(format!("Failed to start curation: {}", e))
            })?;

        Ok(Response::new(StartCurationResponse {
            curation_job_id: job_id.to_string(),
        }))
    }

    async fn get_curation_status(
        &self,
        request: Request<GetCurationStatusRequest>,
    ) -> Result<Response<CurationStatus>, Status> {
        let req = request.into_inner();

        let job_id = Uuid::parse_str(&req.curation_job_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid job ID: {}", e)))?;

        let job = self
            .get_status_use_case
            .execute(job_id)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get curation status: {}", e);
                Status::internal(format!("Failed to get curation status: {}", e))
            })?;

        match job {
            Some(job) => {
                let status = match job.status {
                    crate::domain::models::CurationStatus::Pending => JobStatus::Pending,
                    crate::domain::models::CurationStatus::Embedding => JobStatus::Embedding,
                    crate::domain::models::CurationStatus::AwaitingFeedback => {
                        JobStatus::AwaitingFeedback
                    }
                    crate::domain::models::CurationStatus::Completed => JobStatus::Completed,
                    crate::domain::models::CurationStatus::Failed => JobStatus::Failed,
                };

                let images_for_feedback: Vec<ImageForFeedback> = job
                    .images_for_feedback
                    .into_iter()
                    .map(|img| ImageForFeedback {
                        image_id: img.image_id,
                        image_uri: img.image_uri,
                    })
                    .collect();

                Ok(Response::new(CurationStatus {
                    curation_job_id: job.id.to_string(),
                    status: status as i32,
                    images_for_feedback,
                    curated_dataset_uri: job.curated_data_uri.unwrap_or_default(),
                }))
            }
            None => Err(Status::not_found(format!("Job {} not found", job_id))),
        }
    }

    async fn submit_feedback(
        &self,
        request: Request<SubmitFeedbackRequest>,
    ) -> Result<Response<SubmitFeedbackResponse>, Status> {
        let req = request.into_inner();

        let job_id = Uuid::parse_str(&req.curation_job_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid job ID: {}", e)))?;

        let feedback: Vec<DomainImageFeedback> = req
            .feedback
            .into_iter()
            .map(|f| DomainImageFeedback {
                image_id: f.image_id,
                accepted: f.accepted,
            })
            .collect();

        tracing::info!(
            job_id = %job_id,
            feedback_count = feedback.len(),
            "Submitting feedback"
        );

        let acknowledged = self
            .submit_feedback_use_case
            .execute(job_id, feedback)
            .await
            .map_err(|e| {
                tracing::error!("Failed to submit feedback: {}", e);
                Status::internal(format!("Failed to submit feedback: {}", e))
            })?;

        Ok(Response::new(SubmitFeedbackResponse { acknowledged }))
    }
}
