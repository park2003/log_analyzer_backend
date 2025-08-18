use reqwest::Client;
use serde_json::Value;
use anyhow::Result;

// Flyte의 Admin 서비스 API는 gRPC이지만, 단순화를 위해
// REST 프록시와 상호작용하거나 라이브러리를 사용할 수 있습니다.
// 이 의사코드는 설명을 위해 일반적인 HTTP 클라이언트를 사용합니다.

pub struct FlyteClient {
    http_client: Client,
    flyte_admin_url: String,
    // 인증 토큰은 여기서 관리됩니다
}

impl FlyteClient {
    pub fn new(flyte_admin_url: String) -> Self {
        Self {
            http_client: Client::new(),
            flyte_admin_url,
        }
    }

    pub async fn submit_workflow(
        &self,
        workflow_definition: Value, // JSON 그래프
        project: &str,
        domain: &str,
    ) -> Result<String> {
        // 1단계: Flyte Admin에 워크플로우 작업/정의 등록
        // 이는 아마도 런치 플랜 생성을 포함할 것입니다.
        // let registration_response = self.http_client.post(...)
        //     .bearer_auth(...)
        //     .json(&workflow_definition)
        //     .send().await?;

        // 2단계: 등록된 워크플로우/런치 플랜을 사용하여 실행 생성
        let execution_request = serde_json::json!({
            "project": project,
            "domain": domain,
            // "launch_plan_id": from_registration_response,
            "inputs": { /* 워크플로우 입력 */ }
        });

        let response = self.http_client
            .post(format!("{}/api/v1/executions", self.flyte_admin_url))
            .bearer_auth("token") // 인증 처리, 예: OAuth2 클라이언트 자격 증명
            .json(&execution_request)
            .send()
            .await?;

        let response_body: Value = response.json().await?;
        let execution_id = response_body["id"]["name"].as_str().unwrap_or("").to_string();

        Ok(execution_id)
    }

    pub async fn get_execution_status(&self, execution_id: &str) -> Result<String> {
        // Flyte의 /api/v1/executions/{id} 엔드포인트를 폴링하는 로직
        // TODO: Implement actual status polling
        // "RUNNING", "SUCCEEDED", "FAILED"와 같은 상태 문자열 반환
        Ok("RUNNING".to_string())
    }
}