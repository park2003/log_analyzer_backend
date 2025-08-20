use crate::application::llm_chain::{Llm, LlmChain, OutputParser, PromptTemplate};
use crate::domain::{
    models::{
        FineTuningPlan, KhaydarinLogEntry, ProcessingRequest, ProcessingResult, ProcessingStatus,
    },
    repositories::KhaydarinRepository,
};
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

// Use case for processing natural language prompts into structured fine-tuning plans
pub struct ProcessPromptUseCase<R: KhaydarinRepository, L: Llm, P: OutputParser> {
    repository: R,
    llm_chain: LlmChain<L, P>,
}

impl<R: KhaydarinRepository, L: Llm, P: OutputParser> ProcessPromptUseCase<R, L, P> {
    pub fn new(repository: R, llm: L, output_parser: P) -> Self {
        // Define the prompt template for fine-tuning plan generation
        let template = r#"
You are an AI assistant specializing in fine-tuning configuration for generative models.
Given a user's natural language request, generate a structured JSON plan for fine-tuning.

User Request: {{user_prompt}}

Generate a JSON response with the following structure:
{
    "base_model": "model name (e.g., StableDiffusion-v1.5, SDXL)",
    "tuning_type": "fine-tuning method (e.g., LoRA, DreamBooth, Full)",
    "subject_description": "what is being trained",
    "style_description": "artistic style or characteristics",
    "hyperparameters": {
        "learning_rate": "suggested learning rate",
        "lora_rank": "LoRA rank if applicable",
        "training_steps": "number of training steps",
        "batch_size": "batch size"
    }
}
        "#;

        let prompt_template = PromptTemplate::new(template);
        let llm_chain = LlmChain::new(llm, prompt_template, output_parser);

        Self {
            repository,
            llm_chain,
        }
    }

    pub async fn execute(&self, request: ProcessingRequest) -> Result<ProcessingResult> {
        let start_time = Instant::now();

        // Check for idempotency
        if self.repository.request_exists(&request.request_id).await?
            && let Some(existing) = self
                .repository
                .get_by_request_id(&request.request_id)
                .await?
            && let Some(plan_json) = existing.structured_plan
        {
            let plan: FineTuningPlan = serde_json::from_value(plan_json)?;
            return Ok(ProcessingResult::Success {
                plan,
                confidence: 1.0,
            });
        }

        // Create initial log entry
        let mut log_entry = KhaydarinLogEntry {
            id: Uuid::new_v4(),
            request_id: request.request_id.clone(),
            user_id: request.user_id.clone(),
            received_at: Utc::now(),
            user_prompt: request.user_prompt.clone(),
            llm_prompt: None,
            llm_raw_response: None,
            structured_plan: None,
            status: ProcessingStatus::Success,
            processing_time_ms: None,
        };

        // Prepare context for LLM chain
        let context = json!({
            "user_prompt": request.user_prompt
        });

        // Execute LLM chain
        match self.llm_chain.run(&context).await {
            Ok(structured_output) => {
                // Parse the structured output into a FineTuningPlan
                let plan: FineTuningPlan = serde_json::from_value(structured_output.clone())?;

                // Update log entry with success
                log_entry.structured_plan = Some(structured_output);
                log_entry.status = ProcessingStatus::Success;
                log_entry.processing_time_ms = Some(start_time.elapsed().as_millis() as i32);

                // Save to repository
                self.repository.log_request(&log_entry).await?;

                Ok(ProcessingResult::Success {
                    plan,
                    confidence: 0.95, // Default confidence for successful parsing
                })
            }
            Err(e) => {
                // Update log entry with error
                log_entry.status = ProcessingStatus::LlmError;
                log_entry.processing_time_ms = Some(start_time.elapsed().as_millis() as i32);

                // Save to repository
                self.repository.log_request(&log_entry).await?;

                Ok(ProcessingResult::LlmError {
                    message: e.to_string(),
                })
            }
        }
    }

    // Get user's processing history
    pub async fn get_user_history(
        &self,
        user_id: &str,
        limit: usize,
    ) -> Result<Vec<KhaydarinLogEntry>> {
        self.repository.get_user_history(user_id, limit).await
    }
}
