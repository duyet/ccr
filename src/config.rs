use worker::{Env, Result};

pub struct Config {
    pub openrouter_base_url: String,
}

impl Config {
    pub fn from_env(env: &Env) -> Result<Self> {
        let openrouter_base_url = env
            .var("OPENROUTER_BASE_URL")
            .ok()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());

        Ok(Config {
            openrouter_base_url,
        })
    }

    #[cfg(test)]
    pub fn new(openrouter_base_url: String) -> Self {
        Config {
            openrouter_base_url,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new() {
        let config = Config::new("https://custom.openrouter.ai/api/v1".to_string());
        assert_eq!(
            config.openrouter_base_url,
            "https://custom.openrouter.ai/api/v1"
        );
    }

    #[test]
    fn test_config_default_url() {
        let config = Config::new("".to_string());
        assert_eq!(config.openrouter_base_url, "");

        let config = Config::new("https://openrouter.ai/api/v1".to_string());
        assert_eq!(config.openrouter_base_url, "https://openrouter.ai/api/v1");
    }

    // Note: Testing Config::from_env is difficult without mocking the worker::Env
    // which is tightly coupled to the Cloudflare Workers runtime.
    // In a real-world scenario, you might want to refactor this to accept
    // a trait for environment variable access to make it more testable.
}
