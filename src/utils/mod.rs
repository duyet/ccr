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
    // If model already contains '/', it's an OpenRouter model ID - return as-is
    if anthropic_model.contains('/') {
        return anthropic_model.to_string();
    }

    let model_lower = anthropic_model.to_lowercase();

    // Map common Claude short names to full OpenRouter model IDs
    if model_lower.contains("haiku") {
        "anthropic/claude-3.5-haiku".to_string()
    } else if model_lower.contains("sonnet") {
        "anthropic/claude-sonnet-4".to_string()
    } else if model_lower.contains("opus") {
        "anthropic/claude-opus-4".to_string()
    } else {
        // Return unknown models unchanged - Claude Code will set ANTHROPIC_MODEL
        anthropic_model.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> Config {
        Config {
            openrouter_base_url: "https://openrouter.ai/api/v1".to_string(),
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
        assert_eq!(map_model("my-haiku-model", &config), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("sonnet-variant", &config), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("opus-custom", &config), "anthropic/claude-opus-4");
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
}
