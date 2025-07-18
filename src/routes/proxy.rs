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
        let error_text = response
            .text()
            .await
            .map_err(|e| worker::Error::RustError(format!("Failed to read error response: {e}")))?;

        // Log error details for debugging
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("OpenRouter Error {}: {}", status, error_text).into());

        // Transform OpenRouter error to Anthropic format with request context
        let anthropic_error = transform_openrouter_error(&error_text, status, &anthropic_request);
        
        // Create response with JSON and proper status code
        let response = Response::from_json(&anthropic_error)?.with_status(status);
        return Ok(response);
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

/// Transform OpenRouter error response to Anthropic format with comprehensive diagnostics and request context
fn transform_openrouter_error(error_text: &str, status_code: u16, request: &AnthropicRequest) -> serde_json::Value {
    let mut comprehensive_message = String::new();
    let mut error_code = None;
    let mut param_info = None;
    let mut suggestions = Vec::new();
    
    // Add request context information
    comprehensive_message.push_str(&format!("OpenRouter API Error (HTTP {})\n", status_code));
    comprehensive_message.push_str(&format!("Request Model: {}\n", request.model));
    comprehensive_message.push_str(&format!("Message Count: {}\n", request.messages.len()));
    if let Some(max_tokens) = request.max_tokens {
        comprehensive_message.push_str(&format!("Max Tokens: {}\n", max_tokens));
    }
    if let Some(temperature) = request.temperature {
        comprehensive_message.push_str(&format!("Temperature: {}\n", temperature));
    }
    if let Some(stream) = request.stream {
        comprehensive_message.push_str(&format!("Streaming: {}\n", stream));
    }
    if request.tools.is_some() {
        comprehensive_message.push_str("Tools: Enabled\n");
    }
    comprehensive_message.push_str("\n");
    
    // Try to parse OpenRouter error JSON for detailed information
    if let Ok(openrouter_error) = serde_json::from_str::<serde_json::Value>(error_text) {
        // Extract comprehensive error information from OpenRouter
        if let Some(error_obj) = openrouter_error.get("error") {
            // Base error message
            if let Some(message) = error_obj.get("message").and_then(|m| m.as_str()) {
                comprehensive_message.push_str(&format!("Error Details: {}\n", message));
            }
            
            // Error code/type from OpenRouter
            if let Some(code) = error_obj.get("code") {
                error_code = Some(code.clone());
                comprehensive_message.push_str(&format!("Error Code: {}\n", code));
            }
            
            // Parameter-specific errors
            if let Some(param) = error_obj.get("param").and_then(|p| p.as_str()) {
                param_info = Some(param.to_string());
                comprehensive_message.push_str(&format!("Invalid Parameter: '{}'\n", param));
            }
            
            // Additional details or validation errors
            if let Some(details) = error_obj.get("details") {
                if let Some(details_str) = details.as_str() {
                    comprehensive_message.push_str(&format!("Additional Details: {}\n", details_str));
                } else if details.is_object() || details.is_array() {
                    comprehensive_message.push_str(&format!("Validation Details: {}\n", 
                        serde_json::to_string_pretty(details).unwrap_or_else(|_| details.to_string())));
                }
            }
            
            // Model-specific errors
            if let Some(model) = error_obj.get("model").and_then(|m| m.as_str()) {
                comprehensive_message.push_str(&format!("Model Context: {}\n", model));
            }
        }
        
        // Include user_id if present for debugging
        if let Some(user_id) = openrouter_error.get("user_id").and_then(|u| u.as_str()) {
            comprehensive_message.push_str(&format!("User ID: {}\n", user_id));
        }
        
        // Include request_id if present for debugging
        if let Some(request_id) = openrouter_error.get("request_id").and_then(|r| r.as_str()) {
            comprehensive_message.push_str(&format!("Request ID: {}\n", request_id));
        }
        
        // Add original OpenRouter response for complete context
        comprehensive_message.push_str(&format!("\nOriginal OpenRouter Response:\n{}\n", 
            serde_json::to_string_pretty(&openrouter_error).unwrap_or_else(|_| error_text.to_string())));
    } else {
        // Handle non-JSON errors
        comprehensive_message.push_str(&format!("Raw Error Response: {}\n", error_text));
    }
    
    // Add troubleshooting suggestions based on status code
    match status_code {
        400 => {
            suggestions.push("Check your request parameters and format".to_string());
            suggestions.push("Verify the model name is correct for OpenRouter".to_string());
            suggestions.push("Ensure message content is properly formatted".to_string());
            if request.max_tokens.is_some() && request.max_tokens.unwrap() > 32000 {
                suggestions.push("Try reducing max_tokens (some models have lower limits)".to_string());
            }
        },
        401 => {
            suggestions.push("Verify your OpenRouter API key is correct".to_string());
            suggestions.push("Check if your API key has necessary permissions".to_string());
            suggestions.push("Ensure ANTHROPIC_API_KEY environment variable is set".to_string());
        },
        403 => {
            suggestions.push("Your API key doesn't have access to this model".to_string());
            suggestions.push("Check your OpenRouter account permissions".to_string());
            suggestions.push("Verify the model is available in your OpenRouter plan".to_string());
        },
        404 => {
            suggestions.push("The specified model was not found".to_string());
            suggestions.push("Check available models at https://openrouter.ai/models".to_string());
            suggestions.push("Verify the model name format (e.g., 'anthropic/claude-3.5-sonnet')".to_string());
        },
        429 => {
            suggestions.push("You've exceeded the rate limit".to_string());
            suggestions.push("Wait before making another request".to_string());
            suggestions.push("Consider upgrading your OpenRouter plan".to_string());
        },
        500..=599 => {
            suggestions.push("OpenRouter is experiencing server issues".to_string());
            suggestions.push("Try again in a few moments".to_string());
            suggestions.push("Check OpenRouter status page for outages".to_string());
        },
        _ => {
            suggestions.push("Check OpenRouter documentation for this error".to_string());
            suggestions.push("Verify your request format matches OpenRouter API spec".to_string());
        }
    }
    
    // Add suggestions to the message
    if !suggestions.is_empty() {
        comprehensive_message.push_str("\nTroubleshooting Suggestions:\n");
        for (i, suggestion) in suggestions.iter().enumerate() {
            comprehensive_message.push_str(&format!("{}. {}\n", i + 1, suggestion));
        }
    }
    
    // Add debugging information
    comprehensive_message.push_str(&format!("\nDebugging Information:\n"));
    comprehensive_message.push_str(&format!("- CCR Proxy: https://ccr.duyet.net\n"));
    comprehensive_message.push_str(&format!("- OpenRouter Dashboard: https://openrouter.ai/activity\n"));
    comprehensive_message.push_str(&format!("- Request Time: {}\n", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)));
    
    // Create comprehensive Anthropic-formatted error response
    let mut anthropic_error = serde_json::json!({
        "type": "error",
        "error": {
            "type": match status_code {
                400 => "invalid_request_error",
                401 => "authentication_error", 
                403 => "permission_error",
                404 => "not_found_error",
                429 => "rate_limit_error",
                500..=599 => "api_error",
                _ => "api_error"
            },
            "message": comprehensive_message
        }
    });
    
    // Add additional diagnostic fields if available
    if let Some(code) = error_code {
        anthropic_error["error"]["code"] = code;
    }
    
    if let Some(param) = param_info {
        anthropic_error["error"]["param"] = serde_json::Value::String(param);
    }
    
    // Add request context for debugging
    anthropic_error["error"]["request_context"] = serde_json::json!({
        "model": request.model,
        "messages_count": request.messages.len(),
        "max_tokens": request.max_tokens,
        "temperature": request.temperature,
        "stream": request.stream,
        "has_tools": request.tools.is_some(),
        "has_system": request.system.is_some()
    });
    
    anthropic_error
}
