use crate::config::Config;
use crate::models::AnthropicRequest;
use crate::transform::{anthropic_to_openai, openai_to_anthropic, stream_openai_to_anthropic};
use worker::{Request, Response, Result};

/// Handles POST requests to /v1/messages endpoint
///
/// This function acts as the core proxy logic:
/// 1. Receives Anthropic-formatted request from client
/// 2. Transforms it to OpenAI format
/// 3. Forwards to OpenRouter API
/// 4. Transforms response back to Anthropic format
/// 5. Returns to client
pub async fn handle_messages(mut req: Request, config: &Config) -> Result<Response> {
    // Parse incoming Anthropic-formatted request
    let anthropic_request: AnthropicRequest = req.json().await?;

    // Transform to OpenAI format for OpenRouter API
    let openai_request = anthropic_to_openai(&anthropic_request)?;

    // TODO: Replace with proper authentication from environment or request header
    let bearer_token = "test-token".to_string();

    // Create HTTP client and prepare request to OpenRouter
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.openrouter_base_url);

    // Send request to OpenRouter API
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", bearer_token))
        .json(&openai_request)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("Request failed: {}", e)))?;

    // Handle error responses from OpenRouter
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| {
            worker::Error::RustError(format!("Failed to read error response: {}", e))
        })?;
        return Response::error(text, status);
    }

    // Handle streaming vs non-streaming responses
    if anthropic_request.stream.unwrap_or(false) {
        // Handle streaming response
        stream_openai_to_anthropic(response, &anthropic_request.model).await
    } else {
        // Parse OpenRouter response
        let openai_response: serde_json::Value = response.json().await.map_err(|e| {
            worker::Error::RustError(format!("Failed to parse OpenAI response: {}", e))
        })?;

        // Transform back to Anthropic format
        let anthropic_response = openai_to_anthropic(&openai_response, &anthropic_request.model)?;

        // Return Anthropic-formatted response to client
        Response::from_json(&anthropic_response)
    }
}
