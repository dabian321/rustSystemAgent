use crate::config::Config;
use reqwest::Client;
use serde::{Deserialize, Serialize};

// ── Request types ──────────────────────────────────────────────

#[derive(Debug, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    tools: Vec<ToolDefinition>,
    temperature: f32,
}

// ── Response types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    pub choices: Option<Vec<Choice>>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ResponseMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: String,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

// ── LLM Client ─────────────────────────────────────────────────

pub struct LlmClient {
    client: Client,
    config: Config,
}

impl LlmClient {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    pub async fn chat(
        &self,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<ChatResponse, String> {
        let url = format!("{}/chat/completions", self.config.base_url);

        let request = ChatRequest {
            model: self.config.model.clone(),
            messages: messages.to_vec(),
            tools: tools.iter().map(|t| ToolDefinition {
                tool_type: t.tool_type.clone(),
                function: FunctionDefinition {
                    name: t.function.name.clone(),
                    description: t.function.description.clone(),
                    parameters: t.function.parameters.clone(),
                },
            }).collect(),
            temperature: self.config.temperature,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {e}"))?;

        if !status.is_success() {
            return Err(format!("API returned {status}: {body}"));
        }

        serde_json::from_str::<ChatResponse>(&body)
            .map_err(|e| format!("Failed to parse API response: {e}\nBody: {body}"))
    }
}
