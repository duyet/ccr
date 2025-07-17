# Project Structure

## Root Level
- `Cargo.toml` - Rust package configuration and dependencies
- `wrangler.toml` - Cloudflare Workers deployment configuration
- `Cargo.lock` - Dependency lock file (committed)

## Source Organization (`src/`)
```
src/
├── lib.rs          # Main entry point with request routing
├── config.rs       # Environment configuration management
├── models/         # Data structures and API schemas
├── routes/         # HTTP request handlers
├── transform/      # API format conversion logic
└── utils/          # Shared utility functions
```

## Module Responsibilities

### `lib.rs`
- Main event handler for fetch requests
- URL routing and method matching
- Environment setup and configuration loading

### `config.rs`
- Environment variable management
- Configuration struct definitions
- Default value handling

### `models/`
- Request/response data structures
- Serde serialization traits
- API schema definitions for Anthropic and OpenAI formats

### `routes/`
- `static_pages.rs` - Static content endpoints (home, terms, privacy, install script)
- `proxy.rs` - Main API proxy logic for `/v1/messages`

### `transform/`
- API format conversion between Anthropic and OpenAI
- Request/response transformation logic
- Streaming response handling (placeholder)

### `utils/`
- Model name mapping utilities
- Shared helper functions

## Conventions
- Each module has a `mod.rs` file for public interface
- Use `Result<T>` return types for error handling
- Async functions throughout for Worker compatibility
- JSON handling via serde with flexible `serde_json::Value` types