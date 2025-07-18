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
pub async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    // Add performance monitoring
    let start_time = Date::now().as_millis() as f64;
    
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("🚀 Request started at: {}", start_time).into());
    
    // Set up request monitoring with timeout detection
    let result = handle_request_with_monitoring(req, env, ctx, start_time).await;
    
    let end_time = Date::now().as_millis() as f64;
    let duration = end_time - start_time;
    
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("✅ Request completed in: {}ms", duration).into());
    
    result
}

async fn handle_request_with_monitoring(req: Request, env: Env, _ctx: Context, start_time: f64) -> Result<Response> {
    // Add periodic time checks to detect when we're approaching limits
    let check_time = || {
        let current_time = Date::now().as_millis() as f64;
        let elapsed = current_time - start_time;
        if elapsed > 25000.0 { // 25 seconds - approaching 30s limit
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("⚠️  WARNING: Request running for {}ms, approaching timeout", elapsed).into());
        }
        elapsed
    };
    
    // Load configuration from environment variables
    let _elapsed = check_time();
    let config = Config::from_env(&env)?;
    
    let _elapsed = check_time();
    let url = req.url()?;
    let method = req.method();

    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&format!("🔍 Routing: {} {}", method, url.path()).into());

    // Route requests based on path and method
    let _elapsed = check_time();
    match (url.path(), method) {
        // Static documentation pages
        ("/", Method::Get) => routes::static_pages::home().await,
        ("/terms", Method::Get) => routes::static_pages::terms().await,
        ("/privacy", Method::Get) => routes::static_pages::privacy().await,

        // Main API endpoint - translates Anthropic format to OpenAI format
        ("/v1/messages", Method::Post) => {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&"🔄 Handling /v1/messages request".into());
            
            let _elapsed = check_time();
            
            // Wrap in error handling to catch cancellations
            match routes::proxy::handle_messages(req, &config).await {
                Ok(response) => {
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&"✅ handle_messages completed successfully".into());
                    Ok(response)
                },
                Err(e) => {
                    let current_time = Date::now().as_millis() as f64;
                    let total_elapsed = current_time - start_time;
                    
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&format!("🚨 handle_messages ERROR after {}ms: {}", total_elapsed, e).into());
                    
                    // Check if this looks like a cancellation
                    let error_msg = format!("{}", e);
                    if error_msg.contains("canceled") || error_msg.contains("cancelled") {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("🛑 CANCELLATION DETECTED: Runtime cancelled request after {}ms", total_elapsed).into());
                        
                        // Return a more descriptive error
                        Response::error(format!("Request cancelled by Workers runtime after {}ms. This usually means the request exceeded resource limits (CPU/memory/time).", total_elapsed), 500)
                    } else {
                        Err(e)
                    }
                }
            }
        },

        // 404 for all other routes
        _ => Response::error("Not Found", 404),
    }
}
