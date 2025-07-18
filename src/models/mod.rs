use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub system: Option<serde_json::Value>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<serde_json::Value>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIRequest {
    pub model: String,
    pub messages: Vec<serde_json::Value>,
    pub temperature: Option<f32>,
    pub tools: Option<Vec<serde_json::Value>>,
    pub stream: Option<bool>,
    pub max_tokens: Option<u32>,
}

/// Streaming event models for Anthropic format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStart {
    #[serde(rename = "type")]
    pub event_type: String,
    pub message: MessageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<serde_json::Value>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockStart {
    #[serde(rename = "type")]
    pub event_type: String,
    pub index: u32,
    pub content_block: ContentBlock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockDelta {
    #[serde(rename = "type")]
    pub event_type: String,
    pub index: u32,
    pub delta: Delta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    #[serde(rename = "type")]
    pub delta_type: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlockStop {
    #[serde(rename = "type")]
    pub event_type: String,
    pub index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    #[serde(rename = "type")]
    pub event_type: String,
    pub delta: MessageDeltaData,
    pub usage: Usage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDeltaData {
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageStop {
    #[serde(rename = "type")]
    pub event_type: String,
}

/// OpenAI streaming delta structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIStreamDelta {
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChoice {
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIDelta {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolCall {
    pub id: Option<String>,
    pub function: Option<OpenAIFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}
