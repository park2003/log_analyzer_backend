use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn setup_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "log_analyzer_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Tracing initialized.");
}
 // main 함수에서 호출될 예정

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// 1. 로그의 종류를 정의하는 Enum (온톨로지의 핵심)
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum LogEventType {
    UserLogin,
    ApiCall,
    DatabaseQuery,
    FileAccess,
    Error,
    Unknown,
}

// 2. 수집하고 저장할 로그의 구조체
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogEntry {
    pub id: Uuid, // 각 로그의 고유 ID
    pub trace_id: Uuid, // 여러 로그를 하나로 묶는 트랜잭션 ID
    pub service_name: String, // 로그가 발생한 서비스 (기존 백엔드)
    pub event_type: LogEventType, // 로그의 종류 (온톨로지)
    pub timestamp: DateTime<Utc>,
    pub message: String,
    pub metadata: serde_json::Value, // 기타 추가 정보 (JSON 형태)
}

// 3. 분석 결과를 담을 구조체
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub relationship_type: String, // 예: "Causal Transaction", "Suspicious Pattern"
    pub related_log_ids: Vec<Uuid>,
    pub summary: String,
}

use axum::{
    routing::post,
    Router, Json, http::StatusCode, extract::State,
};
use std::{net::SocketAddr, sync::{Arc, Mutex}};
use reqwest::Client;

// 애플리케이션 전역에서 공유할 상태
// 실제로는 DB Connection Pool 등이 여기에 들어갑니다.
#[derive(Clone)]
struct AppState {
    http_client: Client,
    // 간단한 예제를 위해 메모리 DB 역할을 할 Mutex로 감싼 Vec
    // 추후 데이터베이스로 바꿀것
    log_storage: Arc<Mutex<Vec<LogEntry>>>, 
}

// 기본 main 함수
#[tokio::main]
async fn main() {
    setup_tracing(); // 2단계에서 만든 tracing 초기화 함수

    let shared_state = Arc::new(AppState {
        http_client: Client::new(),
        log_storage: Arc::new(Mutex::new(Vec::new())),
    });

    let app = Router::new()
        .route("/collect-and-analyze", post(collect_and_analyze_handler))
        .with_state(shared_state.clone());

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// API 핸들러: 로그 수집 및 분석 요청을 처리
#[tracing::instrument(skip(state))]
async fn collect_and_analyze_handler(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<AnalysisResult>>) {
    info!("Received request to collect and analyze logs.");

    // 1. 기존 백엔드에서 로그 가져오기 (가상)
    let fetched_logs = match fetch_logs_from_legacy_backend(&state.http_client).await {
        Ok(logs) => {
            info!("Successfully fetched {} logs from legacy backend.", logs.len());
            logs
        },
        Err(e) => {
            tracing::error!("Failed to fetch logs: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![]));
        }
    };

    // 2. 가져온 로그를 우리 백엔드에 저장
    {
        let mut storage = state.log_storage.lock().unwrap();
        storage.extend_from_slice(&fetched_logs);
        info!("Stored {} new logs. Total logs: {}", fetched_logs.len(), storage.len());
    }

    // 3. 온톨로지 기반 분석 실행
    let analysis_results = analyze_log_relationships(&fetched_logs);
    info!("Analysis complete. Found {} relationships.", analysis_results.len());

    (StatusCode::OK, Json(analysis_results))
}

// 기존 백엔드와 통신하여 로그를 가져오는 함수 (시뮬레이션)
#[tracing::instrument(skip(http_client))]
async fn fetch_logs_from_legacy_backend(http_client: &Client) -> Result<Vec<LogEntry>, reqwest::Error> {
    // 실제로는 아래와 같이 기존 백엔드의 API를 호출합니다.
    // let response = http_client.get("http://legacy-backend.example.com/api/logs").send().await?;
    // let logs = response.json::<Vec<LogEntry>>().await?;
    // Ok(logs)

    // 이 예제에서는 더미 데이터를 생성하여 반환합니다.
    info!("Fetching logs from legacy backend...");
    let trace_id = Uuid::new_v4();
    let dummy_logs = vec![
        LogEntry {
            id: Uuid::new_v4(),
            trace_id,
            service_name: "legacy-auth-service".to_string(),
            event_type: LogEventType::UserLogin,
            timestamp: Utc::now(),
            message: "User 'alice' logged in successfully.".to_string(),
            metadata: serde_json::json!({"ip": "192.168.1.10"}),
        },
        LogEntry {
            id: Uuid::new_v4(),
            trace_id,
            service_name: "legacy-api-gateway".to_string(),
            event_type: LogEventType::ApiCall,
            timestamp: Utc::now(),
            message: "API call to /data initiated.".to_string(),
            metadata: serde_json::json!({"user": "alice", "endpoint": "/data"}),
        },
        LogEntry {
            id: Uuid::new_v4(),
            trace_id,
            service_name: "legacy-data-service".to_string(),
            event_type: LogEventType::DatabaseQuery,
            timestamp: Utc::now(),
            message: "Executing SELECT query on 'products' table.".to_string(),
            metadata: serde_json::json!({"query_hash": "a1b2c3d4"}),
        },
    ];
    
    // 네트워크 딜레이 시뮬레이션
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    Ok(dummy_logs)
}

// 온톨로지(규칙) 기반으로 로그 관계를 분석하는 핵심 로직
#[tracing::instrument]
fn analyze_log_relationships(logs: &[LogEntry]) -> Vec<AnalysisResult> {
    let mut results = Vec::new();
    
    // Rule 1: 같은 trace_id를 가진 로그들은 "Causal Transaction" 관계이다.
    use std::collections::HashMap;
    let mut logs_by_trace: HashMap<Uuid, Vec<LogEntry>> = HashMap::new();
    for log in logs {
        logs_by_trace.entry(log.trace_id).or_default().push(log.clone());
    }

    for (trace_id, traced_logs) in logs_by_trace {
        if traced_logs.len() > 1 {
            results.push(AnalysisResult {
                relationship_type: "Causal Transaction".to_string(),
                related_log_ids: traced_logs.iter().map(|l| l.id).collect(),
                summary: format!(
                    "Found a transaction with trace_id {} involving {} steps.",
                    trace_id,
                    traced_logs.len()
                ),
            });
        }
    }
    
    // 여기에 더 복잡한 규칙들을 추가할 수 있습니다.
    // 예: Rule 2: UserLogin 후 1초 안에 Error 로그가 같은 사용자에 의해 발생하면 "Failed Login Attempt" 관계이다.

    results
}