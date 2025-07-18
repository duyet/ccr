use worker::*;

// Module declarations
pub mod config;
pub mod models;
mod routes;
pub mod transform;
pub mod utils;

use config::Config;

/// Main entry point for the Cloudflare Worker
///
/// This function handles all incoming HTTP requests and routes them to appropriate handlers
/// based on the URL path and HTTP method. It acts as a proxy between Anthropic's Claude API
/// and OpenAI-compatible APIs (specifically OpenRouter).
#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Load configuration from environment variables
    let config = Config::from_env(&env)?;
    let url = req.url()?;
    let method = req.method();

    // Route requests based on path and method
    match (url.path(), method) {
        // Static documentation pages
        ("/", Method::Get) => routes::static_pages::home().await,
        ("/terms", Method::Get) => routes::static_pages::terms().await,
        ("/privacy", Method::Get) => routes::static_pages::privacy().await,
        ("/install.sh", Method::Get) => routes::static_pages::install_sh().await,

        // Main API endpoint - translates Anthropic format to OpenAI format
        ("/v1/messages", Method::Post) => routes::proxy::handle_messages(req, &config).await,

        // 404 for all other routes
        _ => Response::error("Not Found", 404),
    }
}
