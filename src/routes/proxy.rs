use crate::config::Config;
use crate::models::AnthropicRequest;
use crate::transform::{anthropic_to_openai, openai_to_anthropic, stream_openai_to_anthropic};
use worker::{Request, Response, Result, Date};

/// Handles POST requests to /v1/messages endpoint
///
/// This function acts as the core proxy logic:
/// 1. Receives Anthropic-formatted request from client
/// 2. Transforms it to OpenAI format
/// 3. Forwards to OpenRouter API
/// 4. Transforms response back to Anthropic format
/// 5. Returns to client
pub async fn handle_messages(mut req: Request, config: &Config) -> Result<Response> {
    let start_time = Date::now().as_millis() as f64;
    
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("🎯 handle_messages started at: {}", start_time).into());
    
    let check_time = |_step: &str| {
        let current_time = Date::now().as_millis() as f64;
        let elapsed = current_time - start_time;
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("⏱️  {}: {}ms", _step, elapsed).into());
        elapsed
    };
    // Extract API key from multiple possible headers
    let _elapsed = check_time("API key extraction start");
    let api_key = if let Some(x_api_key) = req.headers().get("x-api-key")? {
        x_api_key.to_string()
    } else if let Some(auth_header) = req.headers().get("Authorization")? {
        auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| {
                worker::Error::RustError("Invalid Authorization header format".to_string())
            })?
            .to_string()
    } else {
        return Response::error("No API key found in x-api-key or Authorization header", 401);
    };
    
    let _elapsed = check_time("API key extraction complete");

    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("API key: {}...", &api_key[..8.min(api_key.len())]).into());

    // Parse incoming Anthropic-formatted request
    let _elapsed = check_time("Request parsing start");
    let anthropic_request: AnthropicRequest = req.json().await?;
    let _elapsed = check_time("Request parsing complete");

    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Request: {} | {} msgs", anthropic_request.model, anthropic_request.messages.len()).into());

    // Transform to OpenAI format for OpenRouter API
    let _elapsed = check_time("Transform start");
    let openai_request = anthropic_to_openai(&anthropic_request, config)?;
    let _elapsed = check_time("Transform complete");

    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Mapped: {}", openai_request.model).into());

    // Create HTTP client (timeout handled by Cloudflare Workers runtime)
    let client = reqwest::Client::new();

    let url = format!("{}/chat/completions", config.openrouter_base_url);

    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("→ OpenRouter: {}", openai_request.model).into());

    // Send request to OpenRouter API with timeout
    let _elapsed = check_time("HTTP request start");
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .header("HTTP-Referer", "https://ccr.duyet.net")
        .header("X-Title", "CCR - Claude Code Router")
        .json(&openai_request)
        .send()
        .await
        .map_err(|e| {
            let _elapsed = check_time("HTTP request ERROR");
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("🚨 HTTP Error: {} (timeout: {}, request: {})", e, e.is_timeout(), e.is_request()).into());
            worker::Error::RustError(format!("Request failed: {e}"))
        })?;
    
    let _elapsed = check_time("HTTP request complete");
    
    // Minimal debug logging
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Response: {}", response.status()).into());

    // Handle error responses from OpenRouter
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response
            .text()
            .await
            .map_err(|e| worker::Error::RustError(format!("Failed to read error response: {e}")))?;

        // Log error details for debugging
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("Error {}: {}", status, text).into());

        return Response::error(text, status);
    }

    // Handle streaming vs non-streaming responses
    if anthropic_request.stream.unwrap_or(false) {
        // Handle streaming response
        stream_openai_to_anthropic(response, &anthropic_request.model).await
    } else {
        // Parse OpenRouter response
        let openai_response: serde_json::Value = response.json().await.map_err(|e| {
            worker::Error::RustError(format!("Failed to parse OpenAI response: {e}"))
        })?;

        // Debug logging removed for performance

        // Transform back to Anthropic format
        let anthropic_response = openai_to_anthropic(&openai_response, &anthropic_request.model)?;

        // Debug logging removed for performance

        // Return Anthropic-formatted response to client
        Response::from_json(&anthropic_response)
    }
}
