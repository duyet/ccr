use worker::{Request, Response, Result};
use crate::config::Config;
use crate::models::AnthropicRequest;
use crate::transform::{anthropic_to_openai, openai_to_anthropic};

pub async fn handle_messages(mut req: Request, config: &Config) -> Result<Response> {
    let anthropic_request: AnthropicRequest = req.json().await?;
    let openai_request = anthropic_to_openai(&anthropic_request)?;
    
    // Simple hardcoded token for now - should be from env or request header
    let bearer_token = "test-token".to_string();
    
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.openrouter_base_url);
    
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", bearer_token))
        .json(&openai_request)
        .send()
        .await
        .map_err(|e| worker::Error::RustError(format!("Request failed: {}", e)))?;
    
    if !response.status().is_success() {
        let status = response.status().as_u16();
        let text = response.text().await
            .map_err(|e| worker::Error::RustError(format!("Failed to read error response: {}", e)))?;
        return Response::error(text, status);
    }
    
    if anthropic_request.stream.unwrap_or(false) {
        // For now, return error since streaming is not implemented
        return Response::error("Streaming not implemented yet", 501);
    } else {
        let openai_response: serde_json::Value = response.json().await
            .map_err(|e| worker::Error::RustError(format!("Failed to parse OpenAI response: {}", e)))?;
        
        let anthropic_response = openai_to_anthropic(&openai_response, &anthropic_request.model)?;
        
        Response::from_json(&anthropic_response)
    }
}