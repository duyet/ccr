# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CCR (Claude Code Router) is a Cloudflare Worker written in Rust that acts as a proxy between Anthropic's Claude API and OpenAI-compatible APIs (specifically OpenRouter). It translates API requests between the two formats, enabling Claude Code to work with OpenRouter's diverse model selection.

## Architecture

The codebase follows a modular structure with clear separation of concerns:

- **`src/lib.rs`**: Main entry point with request routing based on URL paths and HTTP methods
- **`src/config.rs`**: Configuration management reading from environment variables
- **`src/routes/`**: Request handlers split by functionality
  - `proxy.rs`: Core API translation logic for `/v1/messages` endpoint
  - `static_pages.rs`: Static HTML responses for documentation pages
- **`src/models/`**: Data structures for Anthropic and OpenAI API formats
- **`src/transform/`**: Core transformation logic between API formats
- **`src/utils/`**: Utility functions including model name mapping

## Key Components

### API Translation Flow
1. Receive Anthropic-formatted request at `/v1/messages`
2. Transform to OpenAI format using `anthropic_to_openai()`
3. Forward to OpenRouter API with bearer token
4. Transform OpenRouter response back to Anthropic format using `openai_to_anthropic()`
5. Return to client

### Model Mapping
The `map_model()` function in `src/utils/mod.rs` handles Claude model name translation:
- `haiku` → `anthropic/claude-3.5-haiku`
- `sonnet` → `anthropic/claude-sonnet-4`
- `opus` → `anthropic/claude-opus-4`
- Models with `/` are passed through as OpenRouter model IDs

## Development Commands

### Building and Testing
```bash
# Check for compilation errors
cargo check

# Build the project
cargo build

# Build for release
cargo build --release
```

### Cloudflare Worker Deployment
```bash
# Deploy to Cloudflare Workers
wrangler publish

# Test locally (requires wrangler)
wrangler dev
```

### Development Workflow
1. Make changes to Rust code
2. Run `cargo check` to verify compilation
3. Test locally with `wrangler dev`
4. Deploy with `wrangler publish`

## Important Notes

### Current Limitations
- **Streaming not implemented**: The `/v1/messages` endpoint returns a 501 error for streaming requests
- **Hardcoded token**: Bearer token is currently hardcoded as "test-token" in `src/routes/proxy.rs:11`
- **Basic error handling**: Error responses are passed through without detailed transformation

### Environment Variables
- `OPENROUTER_BASE_URL`: OpenRouter API base URL (defaults to "https://openrouter.ai/api/v1")
- Additional environment variables should be added to `wrangler.toml` under `[vars]`

### Key Files to Modify
- `src/routes/proxy.rs`: Main API proxy logic and authentication
- `src/transform/mod.rs`: API format transformation logic
- `src/utils/mod.rs`: Model mapping and utility functions
- `wrangler.toml`: Cloudflare Worker configuration and environment variables

### Testing Strategy
Since this is a Cloudflare Worker, testing should focus on:
1. Local development with `wrangler dev`
2. Manual API testing with curl/Postman
3. Integration testing with actual Claude Code client
4. Monitoring logs through Cloudflare dashboard

## Dependencies
- `worker`: Cloudflare Workers runtime and utilities
- `reqwest`: HTTP client for making requests to OpenRouter
- `serde`/`serde_json`: JSON serialization/deserialization
- `futures`: Async utilities (prepared for streaming implementation)
- `bytes`: Byte manipulation utilities