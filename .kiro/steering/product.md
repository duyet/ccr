# Product Overview

CCR (Claude Compatibility Router) is a Cloudflare Worker-based proxy service that provides OpenAI-compatible API endpoints for Anthropic's Claude models via OpenRouter.

## Core Functionality
- Transforms Anthropic API requests to OpenAI format for compatibility
- Routes requests through OpenRouter for model access
- Provides static pages for installation and documentation
- Handles both streaming and non-streaming responses

## Target Use Case
Enables applications built for OpenAI's API to seamlessly work with Claude models without code changes, acting as a translation layer between the two API formats.