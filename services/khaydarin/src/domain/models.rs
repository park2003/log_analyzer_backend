use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

// Core domain model for a fine-tuning plan generated from natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningPlan {
    pub base_model: String,
    pub tuning_type: String,
    pub subject_description: String,
    pub style_description: String,
    pub hyperparameters: HashMap<String, Value>,
}

// Result of processing a user prompt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingResult {
    Success {
        plan: FineTuningPlan,
        confidence: f32,
    },
    LlmError {
        message: String,
    },
    ParsingError {
        message: String,
        raw_output: String,
    },
}

// Request log entry for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KhaydarinLogEntry {
    pub id: Uuid,
    pub request_id: String,
    pub user_id: String,
    pub received_at: DateTime<Utc>,
    pub user_prompt: String,
    pub llm_prompt: Option<String>,
    pub llm_raw_response: Option<String>,
    pub structured_plan: Option<Value>,
    pub status: ProcessingStatus,
    pub processing_time_ms: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Success,
    LlmError,
    ParsingError,
}

// Request object for processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingRequest {
    pub request_id: String,
    pub user_id: String,
    pub user_prompt: String,
}
