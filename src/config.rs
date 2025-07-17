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
}