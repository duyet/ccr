use crate::config::Config;
use crate::models::{AnthropicRequest, AnthropicResponse, OpenAIRequest};
use crate::utils::map_model;
use worker::Result;

/// Apply model-specific transformations inspired by claude-code-router
/// Handles model-specific parameter requirements and incompatibilities
fn apply_model_specific_transforms(
    model: &str,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    tools: &Option<Vec<serde_json::Value>>,
    stream: Option<bool>,
) -> (
    Option<f32>,
    Option<u32>,
    Option<Vec<serde_json::Value>>,
    Option<bool>,
) {
    match model {
        // MoonshotAI models (like Kimi K2) have specific requirements
        model_name if model_name.starts_with("moonshotai/") => {
            // Based on claude-code-router config: moonshotai models work better with specific settings
            let adjusted_temp = temperature.map(|t| (t * 0.6).min(1.0));

            // MoonshotAI models don't support complex tools or cache_control - disable them for now
            let adjusted_tools = None; // Disable tools to avoid cache_control issues

            // Set reasonable max_tokens for moonshotai models if not specified
            let adjusted_max_tokens = max_tokens.or(Some(16384)); // Based on their config

            // MoonshotAI supports streaming
            let adjusted_stream = stream;

            (
                adjusted_temp,
                adjusted_max_tokens,
                adjusted_tools,
                adjusted_stream,
            )
        }

        // DeepSeek models
        model_name if model_name.starts_with("deepseek/") || model_name.contains("deepseek") => {
            // DeepSeek models prefer lower temperature
            let adjusted_temp = temperature.map(|t| (t * 0.8).min(1.0));
            (adjusted_temp, max_tokens, tools.clone(), stream)
        }

        // Anthropic Claude models (native)
        model_name if model_name.starts_with("anthropic/") => {
            // Claude models should work well with original parameters
            (temperature, max_tokens, tools.clone(), stream)
        }

        // OpenAI models
        model_name if model_name.starts_with("openai/") => {
            // OpenAI models work well with standard parameters
            (temperature, max_tokens, tools.clone(), stream)
        }

        // Google models
        model_name if model_name.starts_with("google/") => {
            // Google models might have different tool format requirements
            (temperature, max_tokens, tools.clone(), stream)
        }

        // Default case - minimal changes
        _ => (temperature, max_tokens, tools.clone(), stream),
    }
}

/// Validate and clean the OpenAI request to prevent API errors
/// Inspired by claude-code-router's approach to handle API incompatibilities
fn validate_and_clean_request(request: &mut OpenAIRequest) {
    // Ensure all messages have valid content
    for message in &mut request.messages {
        if let Some(content) = message.get("content") {
            if content.is_string() {
                if let Some(content_str) = content.as_str() {
                    if content_str.trim().is_empty() {
                        // Replace empty content with minimal valid content
                        *message.get_mut("content").unwrap() =
                            serde_json::Value::String(" ".to_string());
                    }
                }
            }
        } else {
            // Add content field if missing
            message.as_object_mut().unwrap().insert(
                "content".to_string(),
                serde_json::Value::String(" ".to_string()),
            );
        }
    }

    // Model-specific validation
    match request.model.as_str() {
        model if model.starts_with("moonshotai/") => {
            // MoonshotAI models might not support certain parameters
            // Keep basic parameters only if there are issues

            // Ensure max_tokens is reasonable
            if let Some(max_tokens) = request.max_tokens {
                if max_tokens > 32768 {
                    request.max_tokens = Some(16384); // Safe default
                }
            }

            // Validate temperature range
            if let Some(temp) = request.temperature {
                if !(0.0..=2.0).contains(&temp) {
                    request.temperature = Some(0.6); // MoonshotAI recommended value
                }
            }
        }

        model if model.starts_with("deepseek/") => {
            // DeepSeek specific validations
            if let Some(temp) = request.temperature {
                if temp > 1.5 {
                    request.temperature = Some(1.0); // DeepSeek works better with lower temps
                }
            }
        }

        _ => {
            // General validations for other models
            if let Some(temp) = request.temperature {
                if !(0.0..=2.0).contains(&temp) {
                    request.temperature = Some(1.0); // Safe default
                }
            }
        }
    }
}

/// Transforms an Anthropic API request to OpenAI API format
///
/// This function handles the conversion of request structure, including:
/// - Converting system messages to OpenAI format
/// - Mapping Claude model names to OpenRouter model IDs
/// - Preserving message structure and optional parameters
pub fn anthropic_to_openai(req: &AnthropicRequest, config: &Config) -> Result<OpenAIRequest> {
    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Transform: {} msgs", req.messages.len()).into());

    let mut messages = Vec::new();

    // Add system message if present (OpenAI format uses system role)
    if let Some(system) = &req.system {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }

    // Convert messages from Anthropic format to OpenAI format
    for message in req.messages.iter() {
        let mut openai_message = serde_json::Map::new();

        // Copy role
        if let Some(role) = message.get("role") {
            openai_message.insert("role".to_string(), role.clone());
        }

        // Skip cache_control fields that OpenRouter doesn't support
        // (Claude Code may include these but OpenRouter will reject them)

        // Convert content from Anthropic array format to OpenAI string format
        if let Some(content) = message.get("content") {
            if let Some(content_array) = content.as_array() {
                // Extract text from Anthropic content array
                let mut text_content = String::new();
                for item in content_array {
                    if let Some(text) = item.get("text") {
                        if let Some(text_str) = text.as_str() {
                            text_content.push_str(text_str);
                        }
                    }
                }

                // Ensure content is not empty - OpenRouter rejects empty content
                if text_content.is_empty() {
                    text_content = " ".to_string(); // Use single space as fallback
                }

                openai_message.insert(
                    "content".to_string(),
                    serde_json::Value::String(text_content),
                );
            } else if let Some(content_str) = content.as_str() {
                // Already a string, use as-is but ensure it's not empty
                let final_content = if content_str.trim().is_empty() {
                    " ".to_string() // Use single space as fallback for empty strings
                } else {
                    content_str.to_string()
                };

                openai_message.insert(
                    "content".to_string(),
                    serde_json::Value::String(final_content),
                );
            }
        } else {
            // If no content field exists, add minimal content to prevent 400 error
            openai_message.insert(
                "content".to_string(),
                serde_json::Value::String(" ".to_string()),
            );
        }

        let converted_message = serde_json::Value::Object(openai_message);
        messages.push(converted_message);
    }

    // Only set max_tokens if explicitly provided - let OpenRouter use model defaults
    let max_tokens = req.max_tokens;

    let mapped_model = map_model(&req.model, config);

    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("→ {}", mapped_model).into());

    // Strip cache_control from tools if present (OpenRouter doesn't support it)
    let cleaned_tools = req.tools.as_ref().map(|tools| {
        tools
            .iter()
            .map(|tool| {
                let mut cleaned_tool = tool.clone();
                if let Some(tool_obj) = cleaned_tool.as_object_mut() {
                    tool_obj.remove("cache_control");
                    // Also clean any nested cache_control in input_schema or other fields
                    if let Some(input_schema) = tool_obj.get_mut("input_schema") {
                        if let Some(schema_obj) = input_schema.as_object_mut() {
                            schema_obj.remove("cache_control");
                        }
                    }
                }
                cleaned_tool
            })
            .collect()
    });

    // Apply model-specific transformations (similar to claude-code-router approach)
    let (adjusted_temperature, adjusted_max_tokens, adjusted_tools, adjusted_stream) =
        apply_model_specific_transforms(
            &mapped_model,
            req.temperature,
            max_tokens,
            &cleaned_tools,
            req.stream,
        );

    let mut openai_request = OpenAIRequest {
        model: mapped_model.clone(),
        messages,
        temperature: adjusted_temperature,
        tools: adjusted_tools,
        stream: adjusted_stream,
        max_tokens: adjusted_max_tokens,
    };

    // Validate and clean the request to prevent API errors
    validate_and_clean_request(&mut openai_request);

    // Removed detailed debugging to reduce CPU usage

    Ok(openai_request)
}

/// Transforms an OpenAI API response back to Anthropic API format
///
/// This function handles the conversion of response structure, including:
/// - Converting OpenAI message content to Anthropic format
/// - Handling both text responses and tool calls
/// - Mapping OpenAI finish_reason to Anthropic stop_reason
/// - Generating Anthropic-compatible message IDs
pub fn openai_to_anthropic(response: &serde_json::Value, model: &str) -> Result<AnthropicResponse> {
    // Debug logging removed for performance

    // Generate a timestamp-based message ID in Anthropic format
    let message_id = format!(
        "msg_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| worker::Error::RustError(format!("Time error: {e}")))?
            .as_millis()
    );

    // Safe array access with bounds checking
    let choices = response["choices"]
        .as_array()
        .ok_or_else(|| worker::Error::RustError("Response missing choices array".to_string()))?;

    if choices.is_empty() {
        return Err(worker::Error::RustError(
            "Response has empty choices array".to_string(),
        ));
    }

    let choice = choices[0].clone();
    let message = choice["message"].clone();

    // Debug logging removed for performance

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
            .map_err(|e| worker::Error::RustError(format!("Time error: {e}")))?
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
            cache_control: None,
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
            cache_control: None,
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
            cache_control: None,
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
