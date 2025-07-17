/// Maps Claude model names to OpenRouter model identifiers
///
/// This function translates commonly used Claude model names to their
/// corresponding OpenRouter API model IDs. It handles:
/// - Claude family models (haiku, sonnet, opus) -> anthropic/* models
/// - Kimi models -> moonshot/kimi-k2
/// - OpenRouter model IDs (containing '/') are passed through unchanged
/// - Unknown models are returned as-is
///
/// # Arguments
/// * `anthropic_model` - The model name from the Anthropic API request
///
/// # Returns
/// The OpenRouter-compatible model identifier
pub fn map_model(anthropic_model: &str) -> String {
    // If model already contains '/', it's an OpenRouter model ID - return as-is
    if anthropic_model.contains('/') {
        return anthropic_model.to_string();
    }

    let model_lower = anthropic_model.to_lowercase();

    if model_lower.contains("haiku") {
        "anthropic/claude-3.5-haiku".to_string()
    } else if model_lower.contains("sonnet") {
        "anthropic/claude-sonnet-4".to_string()
    } else if model_lower.contains("opus") {
        "anthropic/claude-opus-4".to_string()
    } else if model_lower.contains("kimi") || model_lower.contains("k2") {
        "moonshot/kimi-k2".to_string()
    } else {
        // Return unknown models unchanged
        anthropic_model.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_model_haiku() {
        assert_eq!(
            map_model("claude-3-haiku-20240307"),
            "anthropic/claude-3.5-haiku"
        );
        assert_eq!(map_model("claude-3.5-haiku"), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("haiku"), "anthropic/claude-3.5-haiku");
    }

    #[test]
    fn test_map_model_sonnet() {
        assert_eq!(
            map_model("claude-3-sonnet-20240229"),
            "anthropic/claude-sonnet-4"
        );
        assert_eq!(map_model("claude-3.5-sonnet"), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("sonnet"), "anthropic/claude-sonnet-4");
    }

    #[test]
    fn test_map_model_opus() {
        assert_eq!(
            map_model("claude-3-opus-20240229"),
            "anthropic/claude-opus-4"
        );
        assert_eq!(map_model("claude-3.5-opus"), "anthropic/claude-opus-4");
        assert_eq!(map_model("opus"), "anthropic/claude-opus-4");
    }

    #[test]
    fn test_map_model_openrouter_id() {
        assert_eq!(
            map_model("anthropic/claude-3.5-sonnet"),
            "anthropic/claude-3.5-sonnet"
        );
        assert_eq!(map_model("openai/gpt-4"), "openai/gpt-4");
        assert_eq!(
            map_model("meta-llama/llama-3.1-8b"),
            "meta-llama/llama-3.1-8b"
        );
    }

    #[test]
    fn test_map_model_unknown() {
        assert_eq!(map_model("unknown-model"), "unknown-model");
        assert_eq!(map_model("gpt-4"), "gpt-4");
        assert_eq!(map_model(""), "");
    }

    #[test]
    fn test_map_model_case_sensitivity() {
        assert_eq!(
            map_model("Claude-3-Haiku-20240307"),
            "anthropic/claude-3.5-haiku"
        );
        assert_eq!(map_model("SONNET"), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("Opus"), "anthropic/claude-opus-4");
    }

    #[test]
    fn test_map_model_partial_matches() {
        assert_eq!(map_model("my-haiku-model"), "anthropic/claude-3.5-haiku");
        assert_eq!(map_model("sonnet-variant"), "anthropic/claude-sonnet-4");
        assert_eq!(map_model("opus-custom"), "anthropic/claude-opus-4");
    }

    #[test]
    fn test_map_model_kimi() {
        assert_eq!(map_model("kimi"), "moonshot/kimi-k2");
        assert_eq!(map_model("k2"), "moonshot/kimi-k2");
        assert_eq!(map_model("kimi-k2"), "moonshot/kimi-k2");
        assert_eq!(map_model("KIMI"), "moonshot/kimi-k2");
        assert_eq!(map_model("K2"), "moonshot/kimi-k2");
    }
}
