use worker::{Response, Result};

pub async fn home() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>CCR - Claude Code Router</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <h1>CCR - Claude Code Router</h1>
    <p>A simple proxy enabling Claude Code to work with OpenRouter</p>
    <p>This Cloudflare Worker translates between Anthropic's Claude API and OpenAI-compatible APIs.</p>
    <h2>Usage</h2>
    <p>Set your environment variables:</p>
    <pre>
export ANTHROPIC_BASE_URL="https://ccr.duyet.net"
export ANTHROPIC_API_KEY="your-openrouter-api-key"
    </pre>
    <p><a href="/install.sh">Install Script</a></p>
</body>
</html>
    "#;
    
    Response::from_html(html)
}

pub async fn terms() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Terms of Service - CCR</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <h1>Terms of Service</h1>
    <p>Basic terms of service for CCR proxy service.</p>
</body>
</html>
    "#;
    
    Response::from_html(html)
}

pub async fn privacy() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Privacy Policy - CCR</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <h1>Privacy Policy</h1>
    <p>Privacy policy for CCR proxy service.</p>
</body>
</html>
    "#;
    
    Response::from_html(html)
}

pub async fn install_sh() -> Result<Response> {
    let script = r#"#!/bin/bash
# CCR Installation Script
echo "Setting up CCR (Claude Code Router)..."
echo "Add these lines to your shell config (.bashrc or .zshrc):"
echo ""
echo "export ANTHROPIC_BASE_URL=\"https://ccr.duyet.net\""
echo "export ANTHROPIC_API_KEY=\"your-openrouter-api-key\""
echo ""
echo "Then run: source ~/.bashrc (or ~/.zshrc)"
echo "Finally: claude"
    "#;
    
    let mut response = Response::ok(script)?;
    response.headers_mut().set("Content-Type", "text/plain; charset=utf-8")?;
    Ok(response)
}