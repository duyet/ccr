# Technology Stack

## Runtime Environment
- **Platform**: Cloudflare Workers
- **Language**: Rust
- **Edition**: 2021
- **Crate Type**: cdylib (for WebAssembly compilation)

## Key Dependencies
- `worker` (0.6.0) - Cloudflare Workers runtime bindings
- `reqwest` (0.12.22) - HTTP client with JSON and streaming support
- `serde` + `serde_json` - JSON serialization/deserialization
- `bytes` - Byte buffer utilities
- `futures` - Async programming primitives

## Build System
- **Build Tool**: Wrangler (Cloudflare's CLI)
- **Output**: WebAssembly module deployed to Cloudflare Workers
- **Compatibility**: nodejs_compat flag enabled

## Common Commands
```bash
# Development
wrangler dev                    # Local development server
wrangler deploy                 # Deploy to Cloudflare Workers
wrangler tail                   # View live logs

# Rust toolchain
cargo check                     # Check compilation
cargo clippy                    # Linting
cargo fmt                       # Code formatting
```

## Configuration
- Environment variables managed via `wrangler.toml`
- Observability enabled with head sampling
- OpenRouter API base URL configurable via environment