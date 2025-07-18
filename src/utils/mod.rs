use crate::config::Config;

/// Maps Claude model names to OpenRouter model identifiers
///
/// This function handles the model name passed from Claude Code. It:
/// - Passes through OpenRouter model IDs (containing '/') unchanged
/// - Maps common Claude short names to full OpenRouter model IDs
/// - Returns unknown models as-is
///
/// # Arguments
/// * `anthropic_model` - The model name from the Anthropic API request
/// * `_config` - Configuration (unused but kept for API compatibility)
///
/// # Returns
/// The OpenRouter-compatible model identifier
pub fn map_model(anthropic_model: &str, _config: &Config) -> String {
    // Debug logging (only in WASM environment)
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("map_model input: '{}'", anthropic_model).into());
    
    // If model already contains '/', it's an OpenRouter model ID - return as-is
    if anthropic_model.contains('/') {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("Found '/' in model, returning as-is: '{}'", anthropic_model).into());
        return anthropic_model.to_string();
    }

    let model_lower = anthropic_model.to_lowercase();
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("Model lowercased: '{}'", model_lower).into());

    // Map common Claude short names to full OpenRouter model IDs
    // Only match exact names or standard Claude model patterns
    let result = if model_lower == "haiku" || model_lower.starts_with("claude-3") && model_lower.contains("haiku") {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Mapping haiku -> anthropic/claude-3.5-haiku".into());
        "anthropic/claude-3.5-haiku".to_string()
    } else if model_lower == "sonnet" || model_lower.starts_with("claude-3") && model_lower.contains("sonnet") {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Mapping sonnet -> anthropic/claude-sonnet-4".into());
        "anthropic/claude-sonnet-4".to_string()
    } else if model_lower == "opus" || model_lower.starts_with("claude-3") && model_lower.contains("opus") {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&"Mapping opus -> anthropic/claude-opus-4".into());
        "anthropic/claude-opus-4".to_string()
    } else {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!("No mapping found, returning unchanged: '{}'", anthropic_model).into());
        // Return unknown models unchanged - Claude Code will set ANTHROPIC_MODEL
        anthropic_model.to_string()
    };
    
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("map_model output: '{}'", result).into());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config {
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
            default_max_tokens: 4096,
        }
    }

    #[test]
    fn test_map_model_haiku() {
        let config = default_config();
        assert_eq!(
            map_model("claude-3-haiku-20240307", &config),
            "anthropic/claude-3.5-haiku"
        );
        assert_eq!(map_model("claude-3.5-haiku", &config), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("haiku", &config), "anthropic/claude-3.5-haiku");
    }

    #[test]
    fn test_map_model_sonnet() {
        let config = default_config();
        assert_eq!(
            map_model("claude-3-sonnet-20240229", &config),
            "anthropic/claude-sonnet-4"
        );
        assert_eq!(map_model("claude-3.5-sonnet", &config), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("sonnet", &config), "anthropic/claude-sonnet-4");
    }

    #[test]
    fn test_map_model_opus() {
        let config = default_config();
        assert_eq!(
            map_model("claude-3-opus-20240229", &config),
            "anthropic/claude-opus-4"
        );
        assert_eq!(map_model("claude-3.5-opus", &config), "anthropic/claude-opus-4");
        assert_eq!(map_model("opus", &config), "anthropic/claude-opus-4");
    }

    #[test]
    fn test_map_model_openrouter_id() {
        let config = default_config();
        assert_eq!(
            map_model("anthropic/claude-3.5-sonnet", &config),
            "anthropic/claude-3.5-sonnet"
        );
        assert_eq!(map_model("openai/gpt-4", &config), "openai/gpt-4");
        assert_eq!(
            map_model("meta-llama/llama-3.1-8b", &config),
            "meta-llama/llama-3.1-8b"
        );
    }

    #[test]
    fn test_map_model_unknown() {
        let config = default_config();
        assert_eq!(map_model("unknown-model", &config), "unknown-model");
        assert_eq!(map_model("gpt-4", &config), "gpt-4");
        assert_eq!(map_model("", &config), "");
    }

    #[test]
    fn test_map_model_case_sensitivity() {
        let config = default_config();
        assert_eq!(
            map_model("Claude-3-Haiku-20240307", &config),
            "anthropic/claude-3.5-haiku"
        );
        assert_eq!(map_model("SONNET", &config), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("Opus", &config), "anthropic/claude-opus-4");
    }

    #[test]
    fn test_map_model_partial_matches() {
        let config = default_config();
        // These should now NOT match because they don't follow Claude naming pattern
        assert_eq!(map_model("my-haiku-model", &config), "my-haiku-model");
        assert_eq!(map_model("sonnet-variant", &config), "sonnet-variant");
        assert_eq!(map_model("opus-custom", &config), "opus-custom");
        
        // But these should match because they follow Claude patterns
        assert_eq!(map_model("claude-3-haiku-20240307", &config), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("claude-3-sonnet-20240229", &config), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("claude-3-opus-20240229", &config), "anthropic/claude-opus-4");
    }


    #[test]
    fn test_map_model_passthrough() {
        let config = default_config();
        
        // Should pass through OpenRouter model IDs unchanged
        assert_eq!(map_model("moonshotai/kimi-k2:free", &config), "moonshotai/kimi-k2:free");
        assert_eq!(map_model("google/gemini-2.5-flash", &config), "google/gemini-2.5-flash");
        
        // Should pass through unknown models unchanged
        assert_eq!(map_model("unknown-model", &config), "unknown-model");
        assert_eq!(map_model("custom-model-name", &config), "custom-model-name");
    }

    #[test]
    fn test_anthropic_model_env_var_simulation() {
        let config = default_config();
        
        // Test the exact models a user might set in ANTHROPIC_MODEL
        assert_eq!(map_model("moonshotai/kimi-k2:free", &config), "moonshotai/kimi-k2:free");
        assert_eq!(map_model("google/gemini-2.5-flash", &config), "google/gemini-2.5-flash");
        assert_eq!(map_model("openai/gpt-4o-mini", &config), "openai/gpt-4o-mini");
        assert_eq!(map_model("anthropic/claude-3-opus-20240229", &config), "anthropic/claude-3-opus-20240229");
        
        // Test that short names still get mapped for backward compatibility
        assert_eq!(map_model("haiku", &config), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("sonnet", &config), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("opus", &config), "anthropic/claude-opus-4");
        
        // Ensure no false matches
        assert_eq!(map_model("not-haiku-model", &config), "not-haiku-model");
        assert_eq!(map_model("some-sonnet-variant", &config), "some-sonnet-variant");
    }
}
