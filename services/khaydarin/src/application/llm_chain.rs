use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

// Trait for any component that can execute LLM calls
#[async_trait]
pub trait Llm: Send + Sync {
    async fn invoke(&self, prompt: &str) -> Result<String>;
}

// Template for generating prompts with placeholders
pub struct PromptTemplate {
    template: String,
}

impl PromptTemplate {
    pub fn new(template: &str) -> Self {
        Self {
            template: template.to_string(),
        }
    }

    pub fn format(&self, context: &Value) -> String {
        let mut result = self.template.clone();

        // Replace placeholders like {{user_prompt}} with values from context
        if let Value::Object(map) = context {
            for (key, value) in map {
                let placeholder = format!("{{{{{key}}}}}");
                let replacement = match value {
                    Value::String(s) => s.clone(),
                    _ => value.to_string(),
                };
                result = result.replace(&placeholder, &replacement);
            }
        }

        result
    }
}

// Trait for parsing LLM string output into structured format
pub trait OutputParser: Send + Sync {
    fn parse(&self, output: &str) -> Result<Value>;
}

// JSON output parser implementation
pub struct JsonOutputParser;

impl OutputParser for JsonOutputParser {
    fn parse(&self, output: &str) -> Result<Value> {
        // Try to extract JSON from the output
        // Handle cases where LLM wraps JSON in markdown code blocks
        let cleaned = if output.contains("```json") {
            output
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(output)
                .trim()
        } else if output.contains("```") {
            output.split("```").nth(1).unwrap_or(output).trim()
        } else {
            output.trim()
        };

        Ok(serde_json::from_str(cleaned)?)
    }
}

// Chain that ties everything together
pub struct LlmChain<L: Llm, P: OutputParser> {
    llm: L,
    prompt_template: PromptTemplate,
    output_parser: P,
}

impl<L: Llm, P: OutputParser> LlmChain<L, P> {
    pub fn new(llm: L, prompt_template: PromptTemplate, output_parser: P) -> Self {
        Self {
            llm,
            prompt_template,
            output_parser,
        }
    }

    pub async fn run(&self, context: &Value) -> Result<Value> {
        let formatted_prompt = self.prompt_template.format(context);
        let llm_output = self.llm.invoke(&formatted_prompt).await?;
        self.output_parser.parse(&llm_output)
    }
}
