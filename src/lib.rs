use worker::*;

mod config;
mod routes;
mod transform;
mod models;
mod utils;

use config::Config;

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let config = Config::from_env(&env)?;
    let url = req.url()?;
    let method = req.method();
    
    match (url.path(), method) {
        ("/", Method::Get) => routes::static_pages::home().await,
        ("/terms", Method::Get) => routes::static_pages::terms().await,
        ("/privacy", Method::Get) => routes::static_pages::privacy().await,
        ("/install.sh", Method::Get) => routes::static_pages::install_sh().await,
        ("/v1/messages", Method::Post) => routes::proxy::handle_messages(req, &config).await,
        _ => Response::error("Not Found", 404),
    }
}
