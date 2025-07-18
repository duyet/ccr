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
    // Extract API key from multiple possible headers
    // Claude Code can send API key via different environment variables:
    // - ANTHROPIC_API_KEY -> x-api-key header
    // - ANTHROPIC_AUTH_TOKEN -> x-api-key header (alternative)
    // - Authorization Bearer token (fallback)
    let api_key = if let Some(x_api_key) = req.headers().get("x-api-key")? {
        x_api_key.to_string()
    } else if let Some(auth_header) = req.headers().get("Authorization")? {
        auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| worker::Error::RustError("Invalid Authorization header format".to_string()))?
            .to_string()
    } else {
        return Response::error("No API key found in x-api-key or Authorization header", 401);
    };

    // Parse incoming Anthropic-formatted request
    let anthropic_request: AnthropicRequest = req.json().await?;
    
    // Debug: Log the received model name (only in WASM environment)
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Received model from Claude Code: {}", anthropic_request.model).into());

    // Transform to OpenAI format for OpenRouter API
    let openai_request = anthropic_to_openai(&anthropic_request, config)?;

    // Create HTTP client (timeout handled by Cloudflare Workers runtime)
    let client = reqwest::Client::new();

    let url = format!("{}/chat/completions", config.openrouter_base_url);

    // Add debug logging (only in WASM environment)
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Sending request to: {}", url).into());
        web_sys::console::log_1(&format!("Model: {}", openai_request.model).into());
    }

    // Add more debug logging before sending request
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::console::log_1(&format!("Request URL: {}", url).into());
        web_sys::console::log_1(&format!("Request model: {}", openai_request.model).into());
        web_sys::console::log_1(&format!("Request body: {}", serde_json::to_string(&openai_request).unwrap_or_else(|_| "Failed to serialize".to_string())).into());
        web_sys::console::log_1(&format!("API key length: {}", api_key.len()).into());
    }

    // Send request to OpenRouter API with timeout
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("HTTP-Referer", "https://ccr.duyet.net")
        .header("X-Title", "CCR - Claude Code Router")
        .json(&openai_request)
        .send()
        .await
        .map_err(|e| {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Request error: {}", e).into());
            worker::Error::RustError(format!("Request failed: {}", e))
        })?;

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Response status: {}", response.status()).into());

    // Handle error responses from OpenRouter
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| {
            worker::Error::RustError(format!("Failed to read error response: {}", e))
        })?;
        
        // Log error details for debugging
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&format!("OpenRouter error status: {}", status).into());
            web_sys::console::log_1(&format!("OpenRouter error response: {}", text).into());
        }
        
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
