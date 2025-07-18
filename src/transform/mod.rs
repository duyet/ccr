use crate::models::{AnthropicRequest, AnthropicResponse, OpenAIRequest};
use crate::utils::map_model;
use crate::config::Config;
use worker::Result;

/// Transforms an Anthropic API request to OpenAI API format
///
/// This function handles the conversion of request structure, including:
/// - Converting system messages to OpenAI format
/// - Mapping Claude model names to OpenRouter model IDs
/// - Preserving message structure and optional parameters
pub fn anthropic_to_openai(req: &AnthropicRequest, config: &Config) -> Result<OpenAIRequest> {
    let mut messages = Vec::new();

    // Add system message if present (OpenAI format uses system role)
    if let Some(system) = &req.system {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }

    // Convert messages (simplified version - assumes compatible format)
    for message in &req.messages {
        messages.push(message.clone());
    }

    // Set reasonable max_tokens default to avoid credit limit issues
    // If user specified max_tokens, respect it; otherwise use config default
    let max_tokens = req.max_tokens.or(Some(config.default_max_tokens));

    Ok(OpenAIRequest {
        model: map_model(&req.model, config),
        messages,
        temperature: req.temperature,
        tools: req.tools.clone(),
        stream: req.stream,
        max_tokens,
    })
}

/// Transforms an OpenAI API response back to Anthropic API format
///
/// This function handles the conversion of response structure, including:
/// - Converting OpenAI message content to Anthropic format
/// - Handling both text responses and tool calls
/// - Mapping OpenAI finish_reason to Anthropic stop_reason
/// - Generating Anthropic-compatible message IDs
pub fn openai_to_anthropic(response: &serde_json::Value, model: &str) -> Result<AnthropicResponse> {
    // Generate a timestamp-based message ID in Anthropic format
    let message_id = format!(
        "msg_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    let choice = response["choices"][0].clone();
    let message = choice["message"].clone();

    // Convert content based on response type
    let content = if let Some(content_str) = message["content"].as_str() {
        // Regular text response
        vec![serde_json::json!({"text": content_str, "type": "text"})]
    } else if let Some(tool_calls) = message["tool_calls"].as_array() {
        // Tool call response - convert to Anthropic format
        tool_calls
            .iter()
            .map(|tc| {
                serde_json::json!({
                    "type": "tool_use",
                    "id": tc["id"],
                    "name": tc["function"]["name"],
                    "input": tc["function"]["arguments"]
                })
            })
            .collect()
    } else {
        // Empty response
        vec![]
    };

    // Map OpenAI finish_reason to Anthropic stop_reason
    let stop_reason = match choice["finish_reason"].as_str() {
        Some("tool_calls") => Some("tool_use".to_string()),
        _ => Some("end_turn".to_string()),
    };

    Ok(AnthropicResponse {
        id: message_id,
        response_type: "message".to_string(),
        role: "assistant".to_string(),
        content,
        stop_reason,
        stop_sequence: None,
        model: model.to_string(),
    })
}

use std::collections::HashMap;

/// Streaming state to track content blocks and tool calls
#[derive(Debug, Clone)]
struct StreamingState {
    content_block_index: u32,
    has_started_text_block: bool,
    is_tool_use: bool,
    current_tool_call_id: Option<String>,
    tool_call_json_map: HashMap<String, String>,
}

impl StreamingState {
    fn new() -> Self {
        Self {
            content_block_index: 0,
            has_started_text_block: false,
            is_tool_use: false,
            current_tool_call_id: None,
            tool_call_json_map: HashMap::new(),
        }
    }
}

/// Transforms OpenAI streaming response to Anthropic streaming format
///
/// This function converts Server-Sent Events from OpenAI API to Anthropic's
/// streaming event format, handling both text content and tool calls.
pub async fn stream_openai_to_anthropic(
    openai_response: reqwest::Response,
    model: &str,
) -> Result<worker::Response> {
    let message_id = format!(
        "msg_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Create streaming response
    let stream_body = format_streaming_response(openai_response, &message_id, model).await?;

    // Create response with proper headers for SSE
    let mut response = worker::Response::ok(stream_body)?;
    response
        .headers_mut()
        .set("Content-Type", "text/event-stream")?;
    response.headers_mut().set("Cache-Control", "no-cache")?;
    response.headers_mut().set("Connection", "keep-alive")?;

    Ok(response)
}

/// Formats streaming response from OpenAI to Anthropic format
async fn format_streaming_response(
    openai_response: reqwest::Response,
    message_id: &str,
    model: &str,
) -> Result<String> {
    let mut stream = openai_response.bytes_stream();
    let mut buffer = String::new();
    let mut state = StreamingState::new();
    let mut output_lines = Vec::new();

    // Send message_start event
    let message_start = crate::models::MessageStart {
        event_type: "message_start".to_string(),
        message: crate::models::MessageInfo {
            id: message_id.to_string(),
            message_type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![],
            model: model.to_string(),
            stop_reason: None,
            stop_sequence: None,
            usage: crate::models::Usage {
                input_tokens: 1,
                output_tokens: 1,
            },
        },
    };

    output_lines.push(format_sse_event("message_start", &message_start)?);

    // Process streaming chunks
    use futures::StreamExt;
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let chunk_str = String::from_utf8_lossy(&chunk);
                buffer.push_str(&chunk_str);

                // Process complete lines
                let lines: Vec<&str> = buffer.split('\n').collect();
                let new_buffer = lines.last().unwrap_or(&"").to_string();

                for line in &lines[..lines.len() - 1] {
                    if line.trim().starts_with("data: ") {
                        let data = line.trim().strip_prefix("data: ").unwrap_or("");
                        if data == "[DONE]" {
                            break;
                        }

                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(choices) = parsed["choices"].as_array() {
                                if let Some(choice) = choices.first() {
                                    if let Some(delta) = choice.get("delta") {
                                        if let Ok(events) = process_stream_delta(delta, &mut state)
                                        {
                                            output_lines.extend(events);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Update buffer with incomplete line
                buffer = new_buffer;
            }
            Err(_) => break,
        }
    }

    // Close last content block
    if state.is_tool_use || state.has_started_text_block {
        let content_block_stop = crate::models::ContentBlockStop {
            event_type: "content_block_stop".to_string(),
            index: state.content_block_index,
        };
        output_lines.push(format_sse_event("content_block_stop", &content_block_stop)?);
    }

    // Send message_delta and message_stop
    let message_delta = crate::models::MessageDelta {
        event_type: "message_delta".to_string(),
        delta: crate::models::MessageDeltaData {
            stop_reason: Some(if state.is_tool_use {
                "tool_use".to_string()
            } else {
                "end_turn".to_string()
            }),
            stop_sequence: None,
        },
        usage: crate::models::Usage {
            input_tokens: 100,
            output_tokens: 150,
        },
    };
    output_lines.push(format_sse_event("message_delta", &message_delta)?);

    let message_stop = crate::models::MessageStop {
        event_type: "message_stop".to_string(),
    };
    output_lines.push(format_sse_event("message_stop", &message_stop)?);

    // Join all lines and return as String
    let response_text = output_lines.join("");
    Ok(response_text)
}

/// Formats Server-Sent Event
fn format_sse_event<T: serde::Serialize>(event_type: &str, data: &T) -> Result<String> {
    let json_data = serde_json::to_string(data)
        .map_err(|e| worker::Error::RustError(format!("JSON serialization error: {e}")))?;

    Ok(format!("event: {event_type}\ndata: {json_data}\n\n"))
}

/// Processes streaming delta from OpenAI and generates Anthropic events
fn process_stream_delta(
    delta: &serde_json::Value,
    state: &mut StreamingState,
) -> Result<Vec<String>> {
    let mut events = Vec::new();

    // Handle tool calls
    if let Some(tool_calls) = delta["tool_calls"].as_array() {
        for tool_call in tool_calls {
            if let Some(tool_call_id) = tool_call["id"].as_str() {
                if Some(tool_call_id.to_string()) != state.current_tool_call_id {
                    // Close previous content block if needed
                    if state.is_tool_use || state.has_started_text_block {
                        let content_block_stop = crate::models::ContentBlockStop {
                            event_type: "content_block_stop".to_string(),
                            index: state.content_block_index,
                        };
                        events.push(format_sse_event("content_block_stop", &content_block_stop)?);
                    }

                    // Start new tool use block
                    state.is_tool_use = true;
                    state.has_started_text_block = false;
                    state.current_tool_call_id = Some(tool_call_id.to_string());
                    state.content_block_index += 1;
                    state
                        .tool_call_json_map
                        .insert(tool_call_id.to_string(), String::new());

                    let tool_block = serde_json::json!({
                        "type": "tool_use",
                        "id": tool_call_id,
                        "name": tool_call["function"]["name"].as_str().unwrap_or(""),
                        "input": {}
                    });

                    let content_block_start = crate::models::ContentBlockStart {
                        event_type: "content_block_start".to_string(),
                        index: state.content_block_index,
                        content_block: crate::models::ContentBlock {
                            block_type: "tool_use".to_string(),
                            data: tool_block,
                        },
                    };
                    events.push(format_sse_event(
                        "content_block_start",
                        &content_block_start,
                    )?);
                }
            }

            // Handle tool call arguments
            if let Some(arguments) = tool_call["function"]["arguments"].as_str() {
                if let Some(current_id) = &state.current_tool_call_id {
                    let current_json = state
                        .tool_call_json_map
                        .get(current_id)
                        .cloned()
                        .unwrap_or_default();
                    state
                        .tool_call_json_map
                        .insert(current_id.clone(), current_json + arguments);

                    let content_block_delta = crate::models::ContentBlockDelta {
                        event_type: "content_block_delta".to_string(),
                        index: state.content_block_index,
                        delta: crate::models::Delta {
                            delta_type: "input_json_delta".to_string(),
                            data: serde_json::json!({
                                "partial_json": arguments
                            }),
                        },
                    };
                    events.push(format_sse_event(
                        "content_block_delta",
                        &content_block_delta,
                    )?);
                }
            }
        }
    }
    // Handle text content
    else if let Some(content) = delta["content"].as_str() {
        if state.is_tool_use {
            let content_block_stop = crate::models::ContentBlockStop {
                event_type: "content_block_stop".to_string(),
                index: state.content_block_index,
            };
            events.push(format_sse_event("content_block_stop", &content_block_stop)?);
            state.is_tool_use = false;
            state.current_tool_call_id = None;
            state.content_block_index += 1;
        }

        if !state.has_started_text_block {
            let text_block = serde_json::json!({
                "type": "text",
                "text": ""
            });

            let content_block_start = crate::models::ContentBlockStart {
                event_type: "content_block_start".to_string(),
                index: state.content_block_index,
                content_block: crate::models::ContentBlock {
                    block_type: "text".to_string(),
                    data: text_block,
                },
            };
            events.push(format_sse_event(
                "content_block_start",
                &content_block_start,
            )?);
            state.has_started_text_block = true;
        }

        let content_block_delta = crate::models::ContentBlockDelta {
            event_type: "content_block_delta".to_string(),
            index: state.content_block_index,
            delta: crate::models::Delta {
                delta_type: "text_delta".to_string(),
                data: serde_json::json!({
                    "text": content
                }),
            },
        };
        events.push(format_sse_event(
            "content_block_delta",
            &content_block_delta,
        )?);
    }

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn default_config() -> Config {
        Config {
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            default_max_tokens: 4096,
        }
    }

    #[test]
    fn test_anthropic_to_openai_basic() {
        let config = default_config();
        let anthropic_req = AnthropicRequest {
            model: "claude-3-sonnet-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello, world!"
            })],
            system: None,
            temperature: Some(0.7),
            tools: None,
            stream: Some(false),
            max_tokens: None,
        };

        let result = anthropic_to_openai(&anthropic_req, &config).unwrap();

        assert_eq!(result.model, "anthropic/claude-sonnet-4");
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.temperature, Some(0.7));
        assert_eq!(result.stream, Some(false));
    }

    #[test]
    fn test_anthropic_to_openai_with_system() {
        let config = default_config();
        let anthropic_req = AnthropicRequest {
            model: "claude-3-haiku-20240307".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "Hello"
            })],
            system: Some(json!("You are a helpful assistant")),
            temperature: None,
            tools: None,
            stream: None,
            max_tokens: None,
        };

        let result = anthropic_to_openai(&anthropic_req, &config).unwrap();

        assert_eq!(result.model, "anthropic/claude-3.5-haiku");
        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0]["role"], "system");
        assert_eq!(result.messages[0]["content"], "You are a helpful assistant");
        assert_eq!(result.messages[1]["role"], "user");
    }

    #[test]
    fn test_anthropic_to_openai_with_tools() {
        let config = default_config();
        let tools = vec![json!({
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "Get weather information"
            }
        })];

        let anthropic_req = AnthropicRequest {
            model: "claude-3-opus-20240229".to_string(),
            messages: vec![json!({
                "role": "user",
                "content": "What's the weather?"
            })],
            system: None,
            temperature: Some(0.5),
            tools: Some(tools.clone()),
            stream: Some(false),
            max_tokens: None,
        };

        let result = anthropic_to_openai(&anthropic_req, &config).unwrap();

        assert_eq!(result.model, "anthropic/claude-opus-4");
        assert_eq!(result.tools, Some(tools));
    }

    #[test]
    fn test_openai_to_anthropic_text_response() {
        let openai_response = json!({
            "choices": [{
                "message": {
                    "content": "Hello! How can I help you today?",
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }]
        });

        let result = openai_to_anthropic(&openai_response, "claude-3-sonnet-20240229").unwrap();

        assert_eq!(result.response_type, "message");
        assert_eq!(result.role, "assistant");
        assert_eq!(result.model, "claude-3-sonnet-20240229");
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0]["type"], "text");
        assert_eq!(
            result.content[0]["text"],
            "Hello! How can I help you today?"
        );
        assert_eq!(result.stop_reason, Some("end_turn".to_string()));
    }

    #[test]
    fn test_openai_to_anthropic_tool_call() {
        let openai_response = json!({
            "choices": [{
                "message": {
                    "tool_calls": [{
                        "id": "call_123",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"New York\"}"
                        }
                    }],
                    "role": "assistant"
                },
                "finish_reason": "tool_calls"
            }]
        });

        let result = openai_to_anthropic(&openai_response, "claude-3-sonnet-20240229").unwrap();

        assert_eq!(result.response_type, "message");
        assert_eq!(result.role, "assistant");
        assert_eq!(result.content.len(), 1);
        assert_eq!(result.content[0]["type"], "tool_use");
        assert_eq!(result.content[0]["id"], "call_123");
        assert_eq!(result.content[0]["name"], "get_weather");
        assert_eq!(result.stop_reason, Some("tool_use".to_string()));
    }

    #[test]
    fn test_openai_to_anthropic_empty_content() {
        let openai_response = json!({
            "choices": [{
                "message": {
                    "content": null,
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }]
        });

        let result = openai_to_anthropic(&openai_response, "claude-3-sonnet-20240229").unwrap();

        assert_eq!(result.content.len(), 0);
        assert_eq!(result.stop_reason, Some("end_turn".to_string()));
    }

    #[test]
    fn test_openai_to_anthropic_generates_valid_id() {
        let openai_response = json!({
            "choices": [{
                "message": {
                    "content": "Test message",
                    "role": "assistant"
                },
                "finish_reason": "stop"
            }]
        });

        let result = openai_to_anthropic(&openai_response, "claude-3-sonnet-20240229").unwrap();

        assert!(result.id.starts_with("msg_"));
        assert!(result.id.len() > 4);
    }
}
