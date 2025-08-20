use thiserror::Error;

/// Extended error types for the Savassan system
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database operation failed: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("External API call failed: {0}")]
    ExternalApi(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err.to_string())
    }
}

/// Convert ServiceError to tonic Status for gRPC
impl From<ServiceError> for tonic::Status {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::Database(e) => tonic::Status::internal(format!("Database error: {e}")),
            ServiceError::Serialization(e) => {
                tonic::Status::internal(format!("Serialization error: {e}"))
            }
            ServiceError::ExternalApi(e) => {
                tonic::Status::unavailable(format!("External API error: {e}"))
            }
            ServiceError::Validation(e) => tonic::Status::invalid_argument(e),
            ServiceError::NotFound(e) => tonic::Status::not_found(e),
            ServiceError::Unauthorized(e) => tonic::Status::unauthenticated(e),
            ServiceError::RateLimitExceeded => {
                tonic::Status::resource_exhausted("Rate limit exceeded")
            }
            ServiceError::Internal(e) => tonic::Status::internal(e),
        }
    }
}
