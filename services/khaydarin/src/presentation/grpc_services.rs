use crate::application::llm_chain::{Llm, OutputParser};
use crate::application::use_cases::ProcessPromptUseCase;
use crate::domain::{
    models::{ProcessingRequest, ProcessingResult},
    repositories::KhaydarinRepository,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod khaydarin_proto {
    tonic::include_proto!("savassan.khaydarin.v1");
}

use khaydarin_proto::{
    ProcessPromptRequest, ProcessPromptResponse, khaydarin_service_server::KhaydarinService,
};

// gRPC service implementation
pub struct KhaydarinGrpcServer<R, L, P>
where
    R: KhaydarinRepository + Send + Sync + 'static,
    L: Llm + Send + Sync + 'static,
    P: OutputParser + Send + Sync + 'static,
{
    use_case: Arc<ProcessPromptUseCase<R, L, P>>,
}

impl<R, L, P> KhaydarinGrpcServer<R, L, P>
where
    R: KhaydarinRepository + Send + Sync + 'static,
    L: Llm + Send + Sync + 'static,
    P: OutputParser + Send + Sync + 'static,
{
    pub fn new(use_case: ProcessPromptUseCase<R, L, P>) -> Self {
        Self {
            use_case: Arc::new(use_case),
        }
    }
}

#[tonic::async_trait]
impl<R, L, P> KhaydarinService for KhaydarinGrpcServer<R, L, P>
where
    R: KhaydarinRepository + Send + Sync + 'static,
    L: Llm + Send + Sync + 'static,
    P: OutputParser + Send + Sync + 'static,
{
    async fn process_prompt(
        &self,
        request: Request<ProcessPromptRequest>,
    ) -> Result<Response<ProcessPromptResponse>, Status> {
        let req = request.into_inner();

        // Validate request
        if req.request_id.is_empty() {
            return Err(Status::invalid_argument("request_id is required"));
        }
        if req.user_id.is_empty() {
            return Err(Status::invalid_argument("user_id is required"));
        }
        if req.user_prompt.is_empty() {
            return Err(Status::invalid_argument("user_prompt is required"));
        }

        // Create domain request
        let processing_request = ProcessingRequest {
            request_id: req.request_id.clone(),
            user_id: req.user_id,
            user_prompt: req.user_prompt,
        };

        // Execute use case
        match self.use_case.execute(processing_request).await {
            Ok(result) => {
                match result {
                    ProcessingResult::Success {
                        plan,
                        confidence: _,
                    } => {
                        // Convert plan to protobuf Struct
                        let plan_json = serde_json::to_value(&plan).map_err(|e| {
                            Status::internal(format!("Failed to serialize plan: {e}"))
                        })?;

                        let structured_plan = prost_types::Struct {
                            fields: convert_json_to_protobuf_map(plan_json).map_err(|e| {
                                Status::internal(format!("Failed to convert plan: {e}"))
                            })?,
                        };

                        let response = ProcessPromptResponse {
                            structured_plan: Some(structured_plan),
                        };

                        Ok(Response::new(response))
                    }
                    ProcessingResult::LlmError { message } => {
                        Err(Status::internal(format!("LLM error: {message}")))
                    }
                    ProcessingResult::ParsingError { message, .. } => {
                        Err(Status::internal(format!("Parsing error: {message}")))
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error processing prompt: {}", e);
                Err(Status::internal("Internal server error"))
            }
        }
    }
}

// Helper function to convert JSON to protobuf Struct fields
fn convert_json_to_protobuf_map(
    value: serde_json::Value,
) -> Result<std::collections::BTreeMap<String, prost_types::Value>, String> {
    use std::collections::BTreeMap;

    match value {
        serde_json::Value::Object(map) => {
            let mut proto_map = BTreeMap::new();
            for (key, val) in map {
                let proto_value = convert_json_to_protobuf_value(val)?;
                proto_map.insert(key, proto_value);
            }
            Ok(proto_map)
        }
        _ => Err("Expected JSON object".to_string()),
    }
}

// Helper function to convert JSON value to protobuf Value
fn convert_json_to_protobuf_value(value: serde_json::Value) -> Result<prost_types::Value, String> {
    use prost_types::value::Kind;

    let kind = match value {
        serde_json::Value::Null => Kind::NullValue(0),
        serde_json::Value::Bool(b) => Kind::BoolValue(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Kind::NumberValue(i as f64)
            } else if let Some(f) = n.as_f64() {
                Kind::NumberValue(f)
            } else {
                return Err("Unsupported number type".to_string());
            }
        }
        serde_json::Value::String(s) => Kind::StringValue(s),
        serde_json::Value::Array(arr) => {
            let values: Result<Vec<_>, _> = arr
                .into_iter()
                .map(convert_json_to_protobuf_value)
                .collect();
            Kind::ListValue(prost_types::ListValue { values: values? })
        }
        serde_json::Value::Object(obj) => {
            let fields = convert_json_to_protobuf_map(serde_json::Value::Object(obj))?;
            Kind::StructValue(prost_types::Struct { fields })
        }
    };

    Ok(prost_types::Value { kind: Some(kind) })
}
