use serde_json::Value;
use async_trait::async_trait;

// LLM 호출을 실행할 수 있는 모든 컴포넌트를 위한 트레이트.
#[async_trait]
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
        // TODO: Implement template substitution
        self.template.clone()
    }
}

// LLM의 문자열 출력을 구조화된 형식으로 파싱하기 위한 트레이트.
#[async_trait]
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