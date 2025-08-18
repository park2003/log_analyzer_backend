use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use crate::domain::models::{KhaydarinLogEntry, ProcessingRequest};

// Repository trait for Khaydarin service persistence
#[async_trait]
pub trait KhaydarinRepository: Send + Sync {
    // Log a processing request and its result
    async fn log_request(&self, entry: &KhaydarinLogEntry) -> Result<()>;
    
    // Retrieve a log entry by request ID
    async fn get_by_request_id(&self, request_id: &str) -> Result<Option<KhaydarinLogEntry>>;
    
    // Retrieve history for a specific user
    async fn get_user_history(&self, user_id: &str, limit: usize) -> Result<Vec<KhaydarinLogEntry>>;
    
    // Update processing result for a request
    async fn update_processing_result(&self, request_id: &str, entry: &KhaydarinLogEntry) -> Result<()>;
    
    // Check if a request ID already exists (for idempotency)
    async fn request_exists(&self, request_id: &str) -> Result<bool>;
}