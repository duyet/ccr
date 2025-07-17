use worker::Result;
use crate::models::{AnthropicRequest, AnthropicResponse, OpenAIRequest};
use crate::utils::map_model;

pub fn anthropic_to_openai(req: &AnthropicRequest) -> Result<OpenAIRequest> {
    let mut messages = Vec::new();
    
    // Add system message if present
    if let Some(system) = &req.system {
        messages.push(serde_json::json!({
            "role": "system",
            "content": system
        }));
    }
    
    // Convert messages (simplified version)
    for message in &req.messages {
        messages.push(message.clone());
    }
    
    Ok(OpenAIRequest {
        model: map_model(&req.model),
        messages,
        temperature: req.temperature,
        tools: req.tools.clone(),
        stream: req.stream,
    })
}

pub fn openai_to_anthropic(response: &serde_json::Value, model: &str) -> Result<AnthropicResponse> {
    let message_id = format!("msg_{}", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    
    let choice = response["choices"][0].clone();
    let message = choice["message"].clone();
    
    let content = if let Some(content_str) = message["content"].as_str() {
        vec![serde_json::json!({"text": content_str, "type": "text"})]
    } else if let Some(tool_calls) = message["tool_calls"].as_array() {
        tool_calls.iter().map(|tc| {
            serde_json::json!({
                "type": "tool_use",
                "id": tc["id"],
                "name": tc["function"]["name"],
                "input": tc["function"]["arguments"]
            })
        }).collect()
    } else {
        vec![]
    };
    
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

// Streaming function placeholder - not implemented for now
// pub fn stream_openai_to_anthropic(
//     _stream: impl futures::Stream<Item = Result<bytes::Bytes>>,
//     _model: &str,
// ) -> Result<impl futures::Stream<Item = Result<bytes::Bytes>>> {
//     // For now, return an error to indicate streaming not implemented
//     Err(worker::Error::RustError("Streaming not implemented yet".to_string()))
// }