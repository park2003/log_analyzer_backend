Phase 1: 시스템 전반의 기술 기반 및 프로토콜 정의비즈니스 로직을 한 줄이라도 작성하기 전에, 우리는 시스템의 기반을 정의해야 합니다: 사용할 프레임워크, 작성할 언어, 그리고 서로 통신할 프로토콜. 이러한 결정들은 복합적인 효과를 가지므로 신중하게 내려야 합니다.1.1. 백엔드 서비스 프레임워크: Rust & Axum결정: 모든 백엔드 마이크로서비스는 Rust로 작성하며, Axum 웹 프레임워크를 사용합니다.기술적 근거:이는 유행을 좇는 것이 아닙니다. 장기적인 안정성과 성능을 위한 전략적 선택입니다.안전성과 성능: Rust의 컴파일 타임 보증은 복잡하고 동시적인 상태와 비싼 GPU 자원을 관리할 시스템에 매우 중요한 일반적인 메모리 오류 및 데이터 경쟁을 방지합니다. 이는 단순히 '있으면 좋은 것'이 아니라, 전체 프로덕션 버그 클래스를 예방하는 핵심 기능입니다.원활한 Tokio 생태계 통합: Axum은 비동기 Rust의 사실상 표준인 Tokio 팀에 의해 개발되었습니다.1 이는 데이터베이스 드라이버(sqlx)부터 미들웨어(tower)에 이르기까지 전체 비동기 생태계와 프레임워크와 런타임 간의 임피던스 불일치 없이 일급의 네이티브 통합을 제공한다는 것을 의미합니다.1마법 없는 인체공학 (매크로 없는 API): 매크로에 크게 의존하는 다른 프레임워크와 달리, Axum은 Handler 및 Extractor와 같은 핵심 추상화를 위해 Rust의 강력한 타입 시스템과 트레이트(trait)를 활용합니다.2 이는 더 명시적이고 디버깅하기 쉬운 코드를 낳고, 일반적으로 더 이해하기 쉬운 컴파일러 오류를 발생시켜 개발을 가속화합니다.3엔터프라이즈급 모듈성: Axum의 철학은 "모든 기능 포함(batteries-included)"이 아니라 "가볍고 확장 가능함(lean and extensible)"입니다.1 미들웨어를 위해 tower::Service 트레이트에 의존하는데, 이는 추적, 타임아웃, 권한 부여와 같은 문제에 대해 프레임워크 특정 구현에 얽매이지 않고 견고하고 검증된 생태계를 활용할 수 있음을 의미합니다.4표준 프로젝트 구조 (도메인 주도 설계):모든 서비스에 걸쳐 일관성과 유지보수성을 보장하기 위해, 우리는 도메인 주도 설계(DDD)에서 영감을 받은 계층형 아키텍처를 채택할 것입니다.5 이는 관심사를 깔끔하게 분리합니다.Plaintext.
├── Cargo.toml
└── src
├── main.rs # 컴포지션 루트: 의존성을 초기화하고 연결합니다.
├── domain # 핵심 비즈니스 로직 및 타입. 다른 계층에 대한 의존성 없음.
│ ├── mod.rs
│ ├── models.rs # 예: FineTuningJob, DataSet 엔티티.
│ └── repositories.rs # 영속성을 위한 트레이트(인터페이스), 예: trait JobRepository.
├── application # 도메인 로직을 오케스트레이션합니다. 유스 케이스를 포함합니다.
│ ├── mod.rs
│ └── use_cases.rs # 예: fn start_new_finetuning_job(...).
├── infrastructure # 다른 계층의 트레이트를 구현합니다. 모든 외부 관심사.
│ ├── mod.rs
│ ├── database.rs # 예: PostgresJobRepository 구현.
│ └── external_apis.rs # 예: Flyte API 클라이언트.
└── presentation # API 계층 (예: gRPC 서비스, HTTP 핸들러).
├── mod.rs
└── grpc_services.rs # gRPC 서비스 트레이트를 구현합니다.
보일러플레이트 main.rs (컴포지션 루트):Rust// src/main.rs
use std::net::SocketAddr;
use tracing::info;

// 실제 서비스 구현을 위한 플레이스홀더
mod presentation {
pub mod grpc_services {
// 이것은 tonic-build에 의해 생성될 것입니다
// pub struct MyGrpcServer;
//... 구현...
}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
// 추적 초기화
tracing_subscriber::fmt::init();

    // TODO: 인프라 계층에서 의존성 초기화
    // let db_pool = infrastructure::database::connect().await?;
    // let job_repository = infrastructure::database::PostgresJobRepository::new(db_pool);
    // let use_cases = application::use_cases::UseCases::new(job_repository);
    // let grpc_service = presentation::grpc_services::MyGrpcServer::new(use_cases);

    let addr = "[::1]:50051".parse::<SocketAddr>()?;
    info!("gRPC server listening on {}", addr);

    // 이것은 플레이스홀더입니다. 여기서는 tonic::transport::Server를 사용할 것입니다.
    // tonic::transport::Server::builder()
    //.add_service(grpc_service)
    //.serve(addr)
    //.await?;

    Ok(())

}
1.2. 서비스 통신 매트릭스결정:내부 (서비스 간): Protocol Buffers를 사용한 gRPC.외부 (클라이언트-게이트웨이): HTTP/3 (QUIC).실시간 UI 업데이트: WebSockets.근거:gRPC (내부): 클러스터 내 통신에서는 성능과 신뢰성이 가장 중요합니다.성능: gRPC는 HTTP/2와 바이너리 직렬화(Protobuf)를 활용하여, 특히 큰 페이로드와 높은 처리량 시나리오에서 텍스트 기반 REST/JSON보다 훨씬 빠르고 네트워크 효율적입니다.7 벤치마크는 gRPC가 더 낮은 지연 시간으로 초당 2-3배 더 많은 요청을 처리함을 일관되게 보여줍니다.7강력한 타입의 계약: .proto 파일은 표준적이고 언어에 구애받지 않는 인터페이스 정의 역할을 합니다. 이는 모호함을 없애고 일치하지 않는 JSON 스키마로 인해 발생하는 전체 런타임 오류 클래스를 제거합니다. 자동 코드 생성은 개발을 가속화하고 클라이언트-서버 호환성을 보장합니다.9스트리밍: 양방향 스트리밍에 대한 네이티브 지원은 서비스 간에 로그나 중간 모델 출력을 스트리밍하는 것과 같은 미래의 유스 케이스에 있어 반복적인 연결 오버헤드 없이 킬러 기능입니다.9HTTP/3 (외부): 사용자 대면 API 게이트웨이의 경우, 체감 성능이 중요합니다.Head-of-Line 블로킹 감소: HTTP/3는 QUIC(UDP) 위에 구축되어 HTTP/2(TCP)에 내재된 head-of-line 블로킹 문제를 완화합니다. 이는 단일 패킷 손실이 다른 모든 병렬 요청을 지연시키지 않아, 특히 손실이 많은 네트워크에서 더 부드럽고 반응성이 좋은 사용자 경험을 제공한다는 것을 의미합니다.11더 빠른 연결 설정: QUIC는 전송 및 암호화 핸드셰이크를 결합하여 보안 연결을 설정하는 데 필요한 왕복 횟수를 줄입니다.구현: Rust 생태계에는 훌륭하고 프로덕션 준비가 된 라이브러리가 있습니다. h3 크레이트는 HTTP/3 로직을 제공하며, s2n-quic(AWS 제공) 또는 h3-quinn과 같은 QUIC 구현으로 뒷받침될 수 있습니다.12 우리는 강력한 지원과 보안 및 성능에 대한 집중 때문에 s2n-quic으로 시작할 것입니다.15WebSockets (실시간):비판적 평가: WebRTC는 강력한 기술이지만, 주요 사용 사례는 브라우저 간의 P2P(peer-to-peer) 오디오/비디오 스트리밍입니다.16 우리의 필요는 서버에서 생성된 데이터(로그, 진행률 메트릭)를 클라이언트로 스트리밍하는 것입니다. 이는 전형적인 클라이언트-서버 모델입니다.결정: WebRTC는 이 사용 사례에 대해 시그널링 서버 및 NAT 순회 로직을 포함하여 불필요한 복잡성을 도입합니다.18 WebSockets는 클라이언트-서버, 전이중 통신을 위한 성숙하고 간단하며 더 직접적인 솔루션이며, 텍스트 기반 로그 및 JSON 메트릭 스트리밍에 완벽하게 적합합니다.20 우리는 실시간 UI 업데이트를 위해 WebSockets를 사용할 것이며, 진정한 P2P 요구 사항이 발생하지 않는 한 WebRTC에 대한 고려는 연기할 것입니다. Axum은 WebSockets를 일급으로 지원합니다.4초안 savassan.proto v0.1:Protocol Bufferssyntax = "proto3";

package savassan.v1;

// 인텔리전스 코어에서 제공하며, API 게이트웨이/오케스트레이터가 호출하는 서비스.
service IntelligenceService {
// 자연어 프롬프트를 구조화된 워크플로우 계획으로 처리합니다.
rpc ProcessPrompt(ProcessPromptRequest) returns (ProcessPromptResponse);

// 주어진 프로젝트에 대한 데이터 큐레이션 프로세스를 트리거합니다.
rpc CurateData(CurateDataRequest) returns (CurateDataResponse);
}

// 실행 계층에서 제공하며, 인텔리전스 코어가 호출하는 서비스.
service ExecutionService {
// 생성된 워크플로우 정의를 실행하기 위해 제출합니다.
rpc ExecuteWorkflow(ExecuteWorkflowRequest) returns (ExecuteWorkflowResponse);
}

// --- 메시지 정의 ---

message ProcessPromptRequest {
string session_id = 1;
string user_id = 2;
string natural_language_prompt = 3;
}

message ProcessPromptResponse {
string workflow_id = 1;
// 파인튜닝 계획의 구조화된 표현.
// JSON 문자열 또는 더 상세한 protobuf 메시지가 될 수 있습니다.
string structured_plan_json = 2;
}

message CurateDataRequest {
string project_id = 1;
// 아티팩트 저장소의 원시 데이터에 대한 포인터.
string raw_data_uri = 2;
}

message CurateDataResponse {
string curated_dataset_id = 1;
// 아티팩트 저장소의 큐레이션된 데이터에 대한 포인터.
string curated_data_uri = 2;
}

message ExecuteWorkflowRequest {
string workflow_id = 1;
// Flyte 워크플로우 그래프의 JSON 정의.
string flyte_workflow_definition_json = 2;
}

message ExecuteWorkflowResponse {
string execution_id = 1;
enum Status {
SUBMITTED = 0;
FAILED = 1;
}
Status status = 2;
}
우리의 프로토콜 선택은 공격적이지만 정당합니다. 내부적으로 gRPC를 사용하는 것은 다국어 마이크로서비스 환경에서 명백한 이점입니다. .proto 파일을 관리하는 운영 오버헤드는 성능과 타입 안전성에서 열 배로 보상받습니다.HTTP/3를 사용하기로 한 결정은 미래 지향적입니다. API 게이트웨이/인그레스 설정에 복잡성을 더하지만, 최종 사용자를 위한 성능 이점은 실질적입니다. 이것이 더 새로운 기술이며 더 신중한 모니터링이 필요할 수 있다는 점을 받아들여야 합니다.gRPC가 모든 곳에 필요한가? 요청/응답 패턴의 경우, 그렇습니다. 그러나 실행 후 잊어버리는(fire-and-forget) 이벤트나 비동기 작업 알림의 경우, 간단한 메시지 큐(NATS 또는 Redis Streams 등)가 더 적합할 수 있습니다. 예를 들어, ExecutionService가 작업을 완료하면, IntelligenceCore가 폴링하거나 장기 gRPC 스트림을 유지하도록 요구하는 대신, IntelligenceCore가 구독하는 토픽에 메시지를 게시할 수 있습니다. 우리는 RPC 호출에는 gRPC를 고수하되, 서비스 간의 이벤트 기반 알림을 위해 메시지 큐를 도입하여 더욱 분리시킬 것입니다. 이것은 대체가 아닌 개선입니다.
Phase 2: 상세 컴포넌트 설계 및 API 계약이제 C4 레벨 3 컴포넌트로 깊이 들어가, 그들의 구체적인 계약과 내부 로직을 정의합니다.
2.1. 전체 파일 구조
성찰 (Reflection):이 구조는 명확한 관심사 분리(Separation of Concerns)를 제공합니다. 개발자는 특정 서비스(services/khaydarin)나 공유 계약(protos/savassan/v1)에 집중할 수 있습니다. .proto 파일을 중앙에서 관리하는 것은 서비스 간의 API 불일치를 방지하는 핵심적인 역할을 합니다. 만약 khaydarin 서비스가 API를 변경하면, 해당 .proto 파일의 변경은 이 API를 사용하는 다른 서비스의 컴파일을 실패하게 만들어 문제를 조기에 발견할 수 있습니다. 이 구조는 처음 설정하는 데 약간의 노력이 필요하지만, 프로젝트가 성장함에 따라 그 가치를 증명할 것입니다. 이는 확장성과 유지보수성을 위한 견고한 기반을 마련하는 것입니다.제안된 모노레포 파일 구조:
├──.github/workflows/ # CI/CD 파이프라인 정의 (GitHub Actions)
├── Cargo.toml # Rust 워크스페이스 루트
├── protos/ # 모든 Protobuf 정의의 중앙 저장소
│ └── savassan/
│ └── v1/
│ ├── savassan.proto
│ ├── khaydarin.proto
│ └── data_curator.proto
├── services/ # 각 마이크로서비스
│ ├── khaydarin/ # Khaydarin 서비스
│ │ ├── Cargo.toml
│ │ ├── build.rs # protos 디렉토리에서 proto 파일을 컴파일
│ │ └── src/
│ │ ├── main.rs
│ │ ├── domain/
│ │ ├── application/
│ │ ├── infrastructure/
│ │ └── presentation/
│ ├── data-curator/ # 데이터 큐레이터 서비스
│ │ ├── Cargo.toml
│ │ ├── build.rs
│ │ └── src/
│ └──... # 다른 서비스들
├── libs/ # 공유 Rust 라이브러리 (크레이트)
│ ├── savassan-common/
│ │ ├── Cargo.toml
│ │ └── src/ # 예: 공통 오류 타입, 로깅 설정 등
│ └──...
├── frontend/ # SvelteKit 프론트엔드 애플리케이션
│ ├── package.json
│ ├── svelte.config.js
│ └── src/
├── infra/ # 인프라 관련 코드
│ ├── terraform/ # Terraform 코드
│ └── kubernetes/ # Kubernetes 매니페스트
└──.gitignore
2.2. 인텔리전스 코어 - Khaydarin (자연어 처리기)
gRPC API (khaydarin.proto):
syntax = "proto3";

package savassan.khaydarin.v1;

import "google/protobuf/struct.proto";

service KhaydarinService {
// 원시 사용자 프롬프트를 받아 구조화된 파인튜닝 계획을 반환합니다.
rpc ProcessPrompt(ProcessPromptRequest) returns (ProcessPromptResponse);
}

message ProcessPromptRequest {
string request_id = 1; // 멱등성 및 추적용
string user_id = 2;
string user_prompt = 3;
}

message ProcessPromptResponse {
// 파인튜닝 계획의 구조화된 표현.
google.protobuf.Struct structured_plan = 1;
}

// structured_plan이 포함할 수 있는 내용의 예:
// {
// "base_model": "StableDiffusion-v1.5",
// "tuning_type": "LoRA",
// "subject_description": "사용자의 개 Fluffy",
// "style_description": "수채화 그림",
// "hyperparameters": {
// "learning_rate": "1e-4",
// "lora_rank": 8
// }
// }
Rust 의사코드 (LLM 상호작용 로직):이는 LangChain과 같은 라이브러리의 개념을 반영하여 Rust 코드를 모듈식이고 테스트 가능하게 구성하는 방법을 보여줍니다.Rust// services/khaydarin/src/application/llm_chain.rs

use serde_json::Value;

// LLM 호출을 실행할 수 있는 모든 컴포넌트를 위한 트레이트.
pub trait Llm {
async fn invoke(&self, prompt: &str) -> Result<String, anyhow::Error>;
}

// 플레이스홀더가 있는 프롬프트를 생성하기 위한 템플릿.
pub struct PromptTemplate {
template: String,
}

impl PromptTemplate {
pub fn new(template: &str) -> Self {
Self { template: template.to_string() }
}

    pub fn format(&self, context: &Value) -> String {
        // {{user_prompt}}와 같은 플레이스홀더를 context의 값으로 바꾸는 로직
        //... 포맷된 문자열 반환
    }

}

// LLM의 문자열 출력을 구조화된 형식으로 파싱하기 위한 트레이트.
pub trait OutputParser {
fn parse(&self, output: &str) -> Result<Value, anyhow::Error>;
}

// 모든 것을 하나로 묶는 체인.
pub struct LlmChain<L: Llm, P: OutputParser> {
llm: L,
prompt_template: PromptTemplate,
output_parser: P,
}

impl<L: Llm, P: OutputParser> LlmChain<L, P> {
pub async fn run(&self, context: &Value) -> Result<Value, anyhow::Error> {
let formatted_prompt = self.prompt_template.format(context);
let llm_output = self.llm.invoke(&formatted_prompt).await?;
self.output_parser.parse(&llm_output)
}
}
PostgreSQL 스키마 (khaydarin_logs 테이블):SQLCREATE TABLE khaydarin_logs (
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
request_id VARCHAR(255) UNIQUE NOT NULL,
user_id VARCHAR(255) NOT NULL,
received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
user_prompt TEXT NOT NULL,
llm_prompt TEXT,
llm_raw_response TEXT,
structured_plan JSONB,
status VARCHAR(50) NOT NULL, -- 예: 'SUCCESS', 'LLM_ERROR', 'PARSING_ERROR'
processing_time_ms INT
);

CREATE INDEX idx_khaydarin_logs_user_id ON khaydarin_logs(user_id);
CREATE INDEX idx_khaydarin_logs_received_at ON khaydarin_logs(received_at);
2.3. 인텔리전스 코어 - 자기 학습 데이터 큐레이터gRPC API (data_curator.proto):Protocol Bufferssyntax = "proto3";

package savassan.data_curator.v1;

service DataCuratorService {
// 큐레이션 프로세스를 시작합니다. 이것은 비동기 작업입니다.
rpc StartCuration(StartCurationRequest) returns (StartCurationResponse);

// 큐레이션 작업의 상태와 결과를 가져옵니다.
rpc GetCurationStatus(GetCurationStatusRequest) returns (CurationStatus);

// 능동 학습 루프를 위해 사용자로부터 피드백을 제공합니다.
rpc SubmitFeedback(SubmitFeedbackRequest) returns (SubmitFeedbackResponse);
}

message StartCurationRequest {
string project_id = 1;
string raw_data_uri = 2; // 예: "s3://my-bucket/project-x/raw/"
}

message StartCurationResponse {
string curation_job_id = 1;
}

message GetCurationStatusRequest {
string curation_job_id = 1;
}

enum JobStatus {
PENDING = 0;
EMBEDDING = 1;
AWAITING_FEEDBACK = 2;
COMPLETED = 3;
FAILED = 4;
}

message ImageForFeedback {
string image_id = 1;
string image_uri = 2; // UI에 표시할 URI
}

message CurationStatus {
string curation_job_id = 1;
JobStatus status = 2;
repeated ImageForFeedback images_for_feedback = 3;
string curated_dataset_uri = 4; // COMPLETED일 때 채워짐
}

message Feedback {
string image_id = 1;
bool accepted = 2; // 좋은 예시면 true, 아니면 false
}

message SubmitFeedbackRequest {
string curation_job_id = 1;
repeated Feedback feedback = 2;
}

message SubmitFeedbackResponse {
bool acknowledged = 1;
}
벡터 데이터베이스 스키마 (pgvector 예시):
구현 계획 (pgvector):확장 기능 활성화: 먼저 PostgreSQL 데이터베이스에서 pgvector 확장 기능을 활성화해야 합니다.SQLCREATE EXTENSION IF NOT EXISTS vector;
테이블 스키마: 이미지 임베딩과 관련 메타데이터를 저장할 테이블을 설계합니다.SQLCREATE TABLE image_embeddings (
id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
project_id UUID NOT NULL,
image_uri TEXT NOT NULL,
-- CLIP ViT-L/14 모델의 임베딩 차원은 768입니다.
embedding vector(768) NOT NULL,
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 프로젝트별로 데이터를 빠르게 조회하기 위한 인덱스
CREATE INDEX idx_image_embeddings_project_id ON image_embeddings(project_id);
유사도 검색 인덱스: 효율적인 근사 최근접 이웃(Approximate Nearest Neighbor, ANN) 검색을 위해 HNSW(Hierarchical Navigable Small World) 인덱스를 생성합니다. 코사인 거리는 정규화된 임베딩에 적합하며, <=> 연산자를 사용합니다.SQLCREATE INDEX idx_image_embeddings_hnsw ON image_embeddings
USING hnsw (embedding vector_cosine_ops);
이 인덱스는 대규모 데이터셋에서 ORDER BY embedding <=> $1과 같은 쿼리의 성능을 크게 향상시킵니다.시퀀스 다이어그램: 능동 학습 루프코드 스니펫sequenceDiagram
participant User
participant WebApp
participant DataCuratorService
participant PGVectorDB as PostgreSQL (pgvector)
participant ArtifactStore

    User->>WebApp: 프로젝트 X에 대한 이미지 업로드
    WebApp->>ArtifactStore: s3://.../raw/에 이미지 저장
    WebApp->>DataCuratorService: StartCuration(project_id="X", raw_data_uri="s3://...")
    DataCuratorService->>DataCuratorService: 작업 상태: PENDING -> EMBEDDING
    loop raw_data_uri의 각 이미지에 대해
        DataCuratorService->>ArtifactStore: 이미지 읽기
        DataCuratorService->>DataCuratorService: CLIP 임베딩 생성
        DataCuratorService->>PGVectorDB: INSERT INTO image_embeddings (project_id, image_uri, embedding) VALUES (...)
    end
    DataCuratorService->>DataCuratorService: 프로젝트 X의 벡터에 대해 클러스터링/샘플링 알고리즘 실행
    DataCuratorService->>DataCuratorService: 가장 정보 가치가 높은 샘플 식별
    DataCuratorService->>DataCuratorService: 작업 상태: EMBEDDING -> AWAITING_FEEDBACK
    WebApp->>DataCuratorService: GetCurationStatus() 폴링
    DataCuratorService-->>WebApp: 상태 AWAITING_FEEDBACK 및 검토할 이미지 반환
    WebApp->>User: "수락/거절" 피드백을 위한 이미지 표시
    User->>WebApp: 피드백 제출
    WebApp->>DataCuratorService: SubmitFeedback(...)
    DataCuratorService->>DataCuratorService: 피드백을 사용하여 데이터셋 최종 확정
    DataCuratorService->>ArtifactStore: 최종 큐레이션된 데이터셋을 s3://.../curated/에 쓰기
    DataCuratorService->>DataCuratorService: 작업 상태: AWAITING_FEEDBACK -> COMPLETED

2.4. 실행 계층 - Flyte 통합 서비스책임:이 서비스는 중요하지만 좁게 초점이 맞춰진 브리지입니다. 유일한 책임은 다음과 같습니다:번역 및 제출: 구조화된 워크플로우 정의(예: gRPC 호출로부터)를 수락하고 이를 Flyte에서 워크플로우를 등록하고 시작하는 데 필요한 API 호출로 변환합니다.상태 추상화: 내부 workflow_id로 Flyte 실행 상태를 쿼리하는 간단한 API를 노출하여, Flyte 자체의 실행 식별자의 복잡성을 숨깁니다.콜백 처리: Flyte로부터 콜백/웹훅을 수신하거나(구성된 경우) 최종 상태(완료, 실패)를 폴링하고, 시스템의 나머지 부분에 알립니다(예: 메시지 큐를 통해).Rust 의사코드 (Flyte API 클라이언트):Rust// services/flyte-integration/src/infrastructure/flyte_client.rs

use reqwest::Client;
use serde_json::Value;

// Flyte의 Admin 서비스 API는 gRPC이지만, 단순화를 위해
// REST 프록시와 상호작용하거나 라이브러리를 사용할 수 있습니다.
// 이 의사코드는 설명을 위해 일반적인 HTTP 클라이언트를 사용합니다.

pub struct FlyteClient {
http_client: Client,
flyte_admin_url: String,
// 인증 토큰은 여기서 관리됩니다
}

impl FlyteClient {
pub async fn submit_workflow(
&self,
workflow_definition: Value, // JSON 그래프
project: &str,
domain: &str,
) -> Result<String, anyhow::Error> {
// 1단계: Flyte Admin에 워크플로우 작업/정의 등록
// 이는 아마도 런치 플랜 생성을 포함할 것입니다.
// let registration_response = self.http_client.post(...)
//.bearer_auth(...)
//.json(&workflow_definition)
//.send().await?;

        // 2단계: 등록된 워크플로우/런치 플랜을 사용하여 실행 생성
        let execution_request = serde_json::json!({
            "project": project,
            "domain": domain,
            // "launch_plan_id": from_registration_response,
            "inputs": { /* 워크플로우 입력 */ }
        });

        let response = self.http_client
       .post(format!("{}/api/v1/executions", self.flyte_admin_url))
       .bearer_auth(/*... */) // 인증 처리, 예: OAuth2 클라이언트 자격 증명
       .json(&execution_request)
       .send()
       .await?;

        let response_body: Value = response.json().await?;
        let execution_id = response_body["id"]["name"].as_str().unwrap().to_string();

        Ok(execution_id)
    }

    pub async fn get_execution_status(&self, execution_id: &str) -> Result<String, anyhow::Error> {
        // Flyte의 /api/v1/executions/{id} 엔드포인트를 폴링하는 로직
        //...
        // "RUNNING", "SUCCEEDED", "FAILED"와 같은 상태 문자열 반환
    }

}
서비스 경계는 올바르게 느껴집니다. Khaydarin은 텍스트-계획 변환이라는 한 가지 일을 합니다. DataCurator는 원시 데이터-큐레이션된 데이터 변환이라는 한 가지 일을 합니다. FlyteIntegrationService는 Flyte API의 특정 세부 사항으로부터 우리 도메인을 보호하는 전형적인 부패 방지 계층(Anti-Corruption Layer)입니다. 이 모듈성은 핵심입니다.잠재적인 위험은 Rust FlyteIntegrationService와 Python 기반 Flyte 생태계 사이의 다국어(polyglot) 경계입니다. "계약"은 워크플로우의 JSON 정의입니다. 우리는 이 JSON 스키마를 엄격하게 버전 관리하고 검증해야 합니다. Python 기반 Flyte 작업의 변경(예: 입력 매개변수 이름 변경)은 JSON을 생성하는 Rust 서비스를 망가뜨릴 수 있습니다. 우리는 두 서비스가 CI/CD 파이프라인 동안 검증에 사용하는 공유 스키마 정의(예: JSON 스키마 사용)를 생성하여 이를 완화할 것입니다. 이는 계약이 조용히 깨지지 않도록 보장합니다.Phase 3: 프론트엔드 구현 청사진3.1. Svelte-Kit 프로젝트 구조 및 상태 관리디렉토리 구조:표준적이고 확장 가능한 SvelteKit 구조가 사용될 것입니다.Plaintext.
├── src
│ ├── lib
│ │ ├── api # 타입-세이프 API 클라이언트, 예: api-client.ts
│ │ ├── assets # SVG, 이미지
│ │ ├── components
│ │ │ ├── nodes # 커스텀 Svelte Flow 노드 (예: ModelLoaderNode.svelte)
│ │ │ ├── ui # 일반 UI 컴포넌트 (Button.svelte, Modal.svelte)
│ │ │ └── workflow # 워크플로우 편집기 자체를 위한 컴포넌트 (Toolbar.svelte)
│ │ ├── stores # 상태 관리를 위한 Svelte 스토어 (workflowStore.ts)
│ │ └── utils # 헬퍼 함수
│ ├── routes
│ │ ├── +layout.svelte
│ │ ├── +page.svelte # 랜딩/메인 페이지
│ │ └── project
│ │ └── [id]
│ │ ├── +page.svelte # 메인 프로젝트/워크플로우 뷰
│ │ └── +page.server.ts # 서버 측 데이터 로딩
│ └── app.html
├── static
└── package.json
상태 관리 전략:결정: Svelte의 내장 스토어를 사용하여 시작하고 이를 조합하여 커스텀 상태 관리 솔루션을 만들 것입니다.근거: 노드 그래프의 경우 상태가 복잡할 수 있습니다(노드, 엣지, 위치, 매개변수). XState와 같은 상태 머신 라이브러리는 강력하지만, 핵심적으로 직접 조작하는 기능에 대해 상당한 보일러플레이트와 가파른 학습 곡선을 도입할 수 있습니다. Svelte의 스토어는 가볍고, 반응적이며, 매우 구성 가능합니다. 우리는 노드 추가/제거, 연결 업데이트, 그래프 상태 직렬화 로직을 캡슐화하는 workflowStore를 만들 수 있습니다. 이는 Svelte에 관용적이고, 새로운 개발자가 배우기 쉬우며, 초기 요구 사항에 충분한 솔루션을 제공합니다. 상태 전환과 부수 효과가 스토어로 관리하기에 너무 복잡해질 경우에만 나중에 XState를 재평가하고 도입할 수 있습니다.3.2. API-클라이언트 및 실시간 통신타입-세이프 API 클라이언트:결정: gRPC-Web을 사용할 것입니다.근거: 내부 서비스가 이미 Protobuf로 정의되어 있으므로, gRPC-Web을 사용하는 것이 종단 간 타입 안전성을 위한 가장 직접적인 경로입니다. .proto 파일을 사용하여 TypeScript 클라이언트 스텁을 자동 생성할 것입니다. 이는 OpenAPI와 같은 중간 계층의 필요성을 없애고, 백엔드 서비스 API의 모든 주요 변경 사항이 런타임 실패가 아닌 프론트엔드에서 컴파일 타임 오류를 발생시키도록 보장합니다. 이는 신뢰성 측면에서 큰 이점입니다.실시간 통신 (WebSockets):재사용 가능한 Svelte 스토어를 만들어 WebSocket 연결과 그 상태를 관리할 것입니다.예시 realtimeStore.ts:TypeScript// src/lib/stores/realtimeStore.ts
import { writable, readable } from 'svelte/store';

export const logs = writable<string>();
export const connectionStatus = writable<'connecting' | 'open' | 'closed'>('closed');

let socket: WebSocket | null = null;

export function connectToWorkflowLog(workflowId: string) {
if (socket && socket.readyState === WebSocket.OPEN) {
socket.close();
}

    logs.set();
    connectionStatus.set('connecting');

    const wsUrl = `wss://api.savassan.com/v1/workflows/${workflowId}/logs`;
    socket = new WebSocket(wsUrl);

    socket.onopen = () => {
        connectionStatus.set('open');
    };

    socket.onmessage = (event) => {
        // 서버가 로그 라인을 일반 텍스트로 보낸다고 가정
        logs.update(currentLogs => [...currentLogs, event.data]);
    };

    socket.onclose = () => {
        connectionStatus.set('closed');
        socket = null;
    };

    socket.onerror = (error) => {
        console.error('WebSocket Error:', error);
        connectionStatus.set('closed');
        socket = null;
    };

}

export function disconnect() {
if (socket) {
socket.close();
}
}
이 UI가 "점진적 공개"를 어떻게 지원하는가? 아키텍처는 이를 위해 설계되었습니다. /project/[id] 페이지는 초기에 간단한 "마법사" 컴포넌트를 렌더링할 수 있습니다. 이 마법사의 상태는 간단합니다. 사용자가 "고급 모드로 전환"을 클릭하면, 마법사를 마운트 해제하고 전체 WorkflowEditor.svelte 컴포넌트를 마운트할 수 있으며, 이 컴포넌트는 더 복잡한 workflowStore를 구독합니다. 전환은 UI 모드 플래그에 기반한 간단한 조건부 렌더링입니다. 기본 데이터 모델(워크플로우 그래프)은 마법사에 의해 생성된 다음 고급 편집기에 간단히 노출될 수 있습니다.수백 개의 노드가 있는 Svelte Flow의 성능에 관해서는 타당한 우려입니다. 그러나 Svelte Flow 및 React Flow와 같은 현대적인 라이브러리는 고도로 최적화되어 있으며 일반적으로 뷰포트 내의 노드만 렌더링합니다. 우리는 개발 주기 초기에 500개 이상의 노드로 "스트레스 테스트" 페이지를 구축하여 성능을 검증하고 커스텀 노드 컴포넌트가 가볍도록 보장할 것입니다.Phase 4: 인프라, 배포 및 관찰 가능성4.1. 코드형 인프라(Infrastructure as Code) 및 CI/CD결정: 로컬 개발 및 CI/CD 빌드를 위한 컨테이너 엔진으로 Docker 대신 Podman을 표준으로 채택합니다.
코드형 인프라 (Terraform):일관성과 재현성을 위해 모든 클라우드 리소스를 관리하는 데 Terraform을 사용할 것입니다.
├── environments
│ ├── dev
│ │ └── main.tf
│ └── prod
│ └── main.tf
└── modules
├── kubernetes_cluster
│ ├── main.tf
│ └── variables.tf
├── postgres_db
│ ├── main.tf
│ └── variables.tf
└── s3_buckets
├── main.tf
└── variables.tf
다단계 컨테이너 파일 (Dockerfile 구문):Podman은 Docker와 동일한 Dockerfile 구문을 완벽하게 지원합니다. OCI(Open Container Initiative) 표준을 준수하므로 기존 파일은 변경할 필요가 없습니다.Rust 백엔드 서비스:Dockerfile# --- 빌더 스테이지 ---
FROM rust:1.78 as builder

WORKDIR /usr/src/app
COPY..

# 레이어 캐싱을 활용하기 위해 먼저 의존성 빌드

RUN cargo build --release --locked

# --- 최종 스테이지 ---

FROM debian:bookworm-slim

# SSL 인증서 설치

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/\*

# 빌더 스테이지에서 컴파일된 바이너리 복사

COPY --from=builder /usr/src/app/target/release/my-service /usr/local/bin/my-service

# 바이너리 실행

CMD ["my-service"]
SvelteKit 프론트엔드 앱 (Node.js v24 기준):Dockerfile# --- 빌더 스테이지 ---

# Node.js 24 LTS 버전의 Alpine 이미지를 사용하여 빌드 환경을 구성합니다.

FROM node:24-alpine as builder

WORKDIR /app
COPY package\*.json./
RUN npm install
COPY..
RUN npm run build

# --- 최종 스테이지 ---

# 프로덕션 환경 역시 가벼운 Node.js 24 Alpine 이미지를 사용합니다.

FROM node:24-alpine

WORKDIR /app

# 이 Dockerfile은 SvelteKit용 Node.js 어댑터를 가정합니다.

# 정적 어댑터를 사용하는 경우 Nginx 이미지가 더 적합할 수 있습니다.

COPY --from=builder /app/build./build
COPY --from=builder /app/node_modules./node_modules
COPY package.json.

ENV NODE_ENV=production
CMD ["node", "build"]
CI/CD 파이프라인 (GitHub Actions - Podman 사용):Podman을 사용하도록 CI/CD 파이프라인을 수정합니다. Docker 관련 액션 대신 직접 podman 명령을 실행합니다.YAMLname: CI-CD Pipeline

on:
push:
branches: [ "main" ]

jobs:
test-and-build:
runs-on: ubuntu-latest
steps: - uses: actions/checkout@v4 - name: Setup Rust
uses: actions/configure-rust@v1
with:
rust-version: stable

      - name: Lint (clippy)
        run: cargo clippy -- -D warnings

      - name: Test
        run: cargo test --locked

      - name: Build
        run: cargo build --release --locked

      - name: Install Podman
        run: |
          sudo apt-get update
          sudo apt-get install -y podman

      - name: Login to Container Registry
        run: |
          echo "${{ secrets.DOCKERHUB_TOKEN }}" | podman login docker.io -u "${{ secrets.DOCKERHUB_USERNAME }}" --password-stdin

      - name: Build and push container image
        run: |
          podman build -t my-org/my-service:latest.
          podman push my-org/my-service:latest

deploy:
needs: test-and-build
runs-on: ubuntu-latest
steps: # k8s 구성 리포지토리를 체크아웃하는 단계 # 새 이미지 태그로 Kubernetes 매니페스트 업데이트 # kubectl apply -f... - name: Deploy to Kubernetes
run: echo "Deploying to K8s..."
4.2. 관찰 가능성(Observability) 스택결정: 추적을 위해 OpenTelemetry로 보강된 표준 "PLG" 스택을 채택할 것입니다.메트릭: Prometheus (Kubernetes 어노테이션을 통해 수집).로깅: Loki (Fluentd 또는 유사한 로그 전달자를 통해 수집).시각화: Grafana (메트릭 및 로그 모두).추적: Jaeger, OpenTelemetry 표준을 사용하여 계측된 서비스.Rust 코드 예시 (Axum의 OpenTelemetry):최신 의존성(opentelemetry-otlp = "0.30.0")을 사용한 예시입니다.Rust// src/main.rs (additions)
use axum::{routing::get, Router};
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// OTLP 추적 파이프라인을 초기화하는 함수
fn init_tracing() -> Result<sdktrace::Tracer, opentelemetry::trace::TraceError> {
opentelemetry_otlp::new_pipeline()
.tracing()
.with_exporter(
opentelemetry_otlp::new_exporter()
.tonic() // gRPC/Tonic을 사용하여 OTLP 엔드포인트로 전송
.with_endpoint("http://localhost:4317"), // Jaeger/Collector 주소
)
.with_config(sdktrace::config().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new(
                "service.name",
                "my-axum-service",
            ),
        ])))
.install_batch(opentelemetry_sdk::runtime::Tokio)
}

#[tokio::main]
async fn main() {
// 전역 Propagator 설정 (HTTP 헤더를 통해 컨텍스트 전파)
global::set_text_map_propagator(TraceContextPropagator::new());

    // 추적기 초기화
    let tracer = init_tracing().expect("Failed to initialize tracing pipeline.");

    // Tracing Subscriber 설정
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
       .with(telemetry)
       .with(tracing_subscriber::EnvFilter::from_default_env())
       .with(tracing_subscriber::fmt::layer())
       .init();

    let app = Router::new()
       .route("/", get(handler))
        // 이 레이어는 각 요청에 대해 자동으로 스팬(span)을 생성합니다.
       .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

}

#[tracing::instrument] // 이 함수에 대한 스팬을 자동으로 생성
async fn handler() -> &'static str {
"Hello, World!"
}
이 인프라 계획은 표준적이고, 견고하며, 비용 효율적입니다. 관리형 Kubernetes 및 데이터베이스 서비스를 사용하면 운영 부담이 줄어듭니다. 관찰 가능성 스택은 오픈 소스이며 업계 표준이므로 벤더 종속을 방지합니다.다른 환경을 어떻게 관리할 것인가? Terraform 구조는 이를 위해 설계되었습니다. environments 디렉토리를 사용하면 자체 상태 파일과 변수 재정의(예: dev용 소규모 인스턴스 크기)를 사용하여 dev, staging, prod 작업 공간을 정의할 수 있습니다. CI/CD는 올바른 클러스터에 배포하기 위해 환경별 시크릿으로 구성됩니다.관찰 가능성 스택이 충분한가? 예. 메트릭, 로그, 추적(trace_id로 상관 관계)의 조합은 완전한 그림을 제공합니다. 사용자가 오류를 보고하면, trace_id로 Loki에서 요청을 찾고, Jaeger에서 전체 분산 추적을 확인하여 어떤 서비스가 실패했는지 확인한 다음, 실패 시점 주변의 해당 특정 서비스 및 파드에 대한 Prometheus 메트릭을 드릴다운하여 리소스 압박(CPU/메모리)을 확인할 수 있습니다. 이것은 신속한 근본 원인 분석을 위한 강력하고 검증된 워크플로우입니다.
