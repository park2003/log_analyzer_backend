use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::application::use_cases::FlyteClient as FlyteClientTrait;

/// HTTP client for interacting with Flyte Admin API
/// For local Flyte instances, this uses the REST proxy
pub struct HttpFlyteClient {
    http_client: Client,
    flyte_admin_url: String,
    project: String,
    domain: String,
}

impl HttpFlyteClient {
    /// Create a new Flyte client
    pub fn new(flyte_admin_url: String, project: String, domain: String) -> Self {
        Self {
            http_client: Client::new(),
            flyte_admin_url,
            project,
            domain,
        }
    }

    /// Register workflow tasks and definitions with Flyte
    async fn register_workflow(&self, workflow_definition: &Value) -> Result<String, anyhow::Error> {
        // For local Flyte, we might skip registration if using flytekit-generated workflows
        // This is a simplified version - actual implementation would parse the workflow JSON
        // and create appropriate Flyte entities
        
        // Generate a unique workflow ID
        let workflow_id = format!("workflow_{}", uuid::Uuid::new_v4());
        
        // In a real implementation, this would:
        // 1. Parse the workflow definition
        // 2. Register tasks with Flyte
        // 3. Register the workflow with Flyte
        // 4. Create a launch plan
        
        Ok(workflow_id)
    }
}

#[async_trait]
impl FlyteClientTrait for HttpFlyteClient {
    /// Submit a workflow to Flyte for execution
    async fn submit_workflow(
        &self,
        workflow_definition: Value,
        project: &str,
        domain: &str,
    ) -> Result<String, anyhow::Error> {
        // Step 1: Register the workflow (if needed)
        let workflow_id = self.register_workflow(&workflow_definition).await?;
        
        // Step 2: Create an execution
        let execution_request = serde_json::json!({
            "project": project,
            "domain": domain,
            "name": format!("execution_{}", uuid::Uuid::new_v4()),
            "spec": {
                "launch_plan": {
                    "resource_type": "LAUNCH_PLAN",
                    "project": project,
                    "domain": domain,
                    "name": workflow_id,
                    "version": "v1"
                },
                "inputs": {
                    // Workflow inputs would be extracted from the definition
                    // This is a placeholder
                }
            }
        });
        
        let url = format!("{}/api/v1/executions", self.flyte_admin_url);
        
        let response = self.http_client
            .post(&url)
            .json(&execution_request)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to Flyte: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Flyte API error: {}", error_text));
        }
        
        let response_body: Value = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse Flyte response: {}", e))?;
        
        // Extract execution ID from response
        let execution_id = response_body["id"]["name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No execution ID in response"))?
            .to_string();
        
        tracing::info!("Successfully submitted workflow to Flyte: {}", execution_id);
        
        Ok(execution_id)
    }
    
    /// Get the status of a Flyte execution
    async fn get_execution_status(&self, execution_id: &str) -> Result<String, anyhow::Error> {
        let url = format!(
            "{}/api/v1/executions/{}/{}",
            self.flyte_admin_url,
            self.project,
            execution_id
        );
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to query Flyte execution: {}", e))?;
        
        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(anyhow::anyhow!("Execution not found: {}", execution_id));
            }
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Flyte API error: {}", error_text));
        }
        
        let response_body: Value = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse Flyte response: {}", e))?;
        
        // Extract phase from response
        // Flyte execution phases: UNDEFINED, QUEUED, RUNNING, SUCCEEDING, SUCCEEDED, FAILING, FAILED, ABORTED, TIMED_OUT
        let phase = response_body["closure"]["phase"]
            .as_str()
            .unwrap_or("UNDEFINED");
        
        // Map Flyte phase to our simplified status
        let status = match phase {
            "QUEUED" => "PENDING",
            "RUNNING" => "RUNNING",
            "SUCCEEDED" | "SUCCEEDING" => "SUCCEEDED",
            "FAILED" | "FAILING" => "FAILED",
            "ABORTED" => "ABORTED",
            "TIMED_OUT" => "TIMED_OUT",
            _ => "PENDING",
        };
        
        Ok(status.to_string())
    }
}

/// Configuration for Flyte client
#[derive(Debug, Clone)]
pub struct FlyteConfig {
    pub admin_url: String,
    pub project: String,
    pub domain: String,
    pub insecure: bool,
}

impl Default for FlyteConfig {
    fn default() -> Self {
        Self {
            admin_url: "http://localhost:30080".to_string(), // Default for local Flyte
            project: "flytesnacks".to_string(),
            domain: "development".to_string(),
            insecure: true, // For local development
        }
    }
}

impl FlyteConfig {
    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            admin_url: std::env::var("FLYTE_ADMIN_URL")
                .unwrap_or_else(|_| "http://localhost:30080".to_string()),
            project: std::env::var("FLYTE_PROJECT")
                .unwrap_or_else(|_| "flytesnacks".to_string()),
            domain: std::env::var("FLYTE_DOMAIN")
                .unwrap_or_else(|_| "development".to_string()),
            insecure: std::env::var("FLYTE_INSECURE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}