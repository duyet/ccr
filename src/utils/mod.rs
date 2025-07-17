pub fn map_model(anthropic_model: &str) -> String {
    // If model already contains '/', it's an OpenRouter model ID - return as-is
    if anthropic_model.contains('/') {
        return anthropic_model.to_string();
    }
    
    if anthropic_model.contains("haiku") {
        "anthropic/claude-3.5-haiku".to_string()
    } else if anthropic_model.contains("sonnet") {
        "anthropic/claude-sonnet-4".to_string()
    } else if anthropic_model.contains("opus") {
        "anthropic/claude-opus-4".to_string()
    } else {
        anthropic_model.to_string()
    }
}