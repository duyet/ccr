# CCR (Claude Code Router)

> **Built entirely with [Claude Code](https://claude.ai/code)** - An AI-powered development workflow

A Cloudflare Worker proxy that enables **Claude Code** to access OpenRouter's diverse model selection. CCR acts as a translation layer between Anthropic's Claude API format and OpenAI-compatible APIs.

## 🚀 What is CCR?

This Cloudflare Worker acts as a translation layer between Anthropic's Claude API format and OpenAI-compatible APIs, specifically OpenRouter. It allows **Claude Code** to access a wide range of models through OpenRouter while maintaining the familiar Claude API interface.

**Key Features:**
- 🔄 **API Translation**: Seamlessly converts between Anthropic and OpenAI API formats
- 🌐 **OpenRouter Integration**: Access to multiple AI models through OpenRouter's unified API
- ⚡ **Cloudflare Workers**: Fast, globally distributed proxy with minimal latency
- 🎯 **Claude Code Compatible**: Designed specifically for Claude Code users

## ⚡ Quick Start

### 1. Set up Claude Code
Install Claude Code CLI:

```bash
npm install -g @anthropic-ai/claude-code
```

For more details, see: https://docs.anthropic.com/en/docs/claude-code/setup

### 2. Get OpenRouter API Key
Sign up at [openrouter.ai](https://openrouter.ai) and get your API key

### 3. Using Claude Code with CCR

#### Basic Usage
```bash
# Use either ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN (both work the same way)
ANTHROPIC_BASE_URL="https://ccr.duyet.net" ANTHROPIC_API_KEY="your-openrouter-api-key" claude
ANTHROPIC_BASE_URL="https://ccr.duyet.net" ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key" claude
```

#### With Custom Models
```bash
# Use either ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN with custom models
ANTHROPIC_BASE_URL="https://ccr.duyet.net" ANTHROPIC_API_KEY="your-openrouter-api-key" ANTHROPIC_MODEL="moonshotai/kimi-k2:free" ANTHROPIC_SMALL_FAST_MODEL="google/gemini-2.5-flash" claude
ANTHROPIC_BASE_URL="https://ccr.duyet.net" ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key" ANTHROPIC_MODEL="moonshotai/kimi-k2:free" ANTHROPIC_SMALL_FAST_MODEL="google/gemini-2.5-flash" claude
```

#### Environment Setup (Optional)
For permanent setup, add to your shell profile:

```bash
export ANTHROPIC_BASE_URL="https://ccr.duyet.net"
# Use either ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN (both work the same way)
export ANTHROPIC_API_KEY="your-openrouter-api-key"
# OR
export ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key"

# Optional: Set default models
export ANTHROPIC_MODEL="moonshotai/kimi-k2:free"
export ANTHROPIC_SMALL_FAST_MODEL="google/gemini-2.5-flash"

# Then simply run:
claude
```

That's it! Claude Code will now use OpenRouter's models through the CCR proxy.

## 📥 Installation

### Manual Setup
1. Add the environment variables to your shell profile:
   ```bash
   echo 'export ANTHROPIC_BASE_URL="https://ccr.duyet.net"' >> ~/.bashrc
   # Use either ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN (both work the same way)
   echo 'export ANTHROPIC_API_KEY="your-openrouter-api-key"' >> ~/.bashrc
   # OR
   echo 'export ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key"' >> ~/.bashrc
   source ~/.bashrc
   ```

2. Start Claude Code:
   ```bash
   claude
   ```

## 🔧 Self-Hosting

If you want to deploy your own CCR instance:

### Deploy to Cloudflare Workers

```bash
# Install Wrangler CLI
npm install -g wrangler

# Clone and deploy
git clone https://github.com/duyet/ccr.git
cd ccr
wrangler deploy
```

### Configure Authentication

Set your OpenRouter API key:

```bash
wrangler secret put OPENROUTER_API_KEY
# Enter your OpenRouter API key when prompted
```

### Configure Environment Variables

Update `wrangler.toml`:

```toml
[vars]
OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"
```

#### How Model Selection Works

CCR automatically handles model mapping:

1. **OpenRouter Model IDs**: Models with `/` (like `moonshotai/kimi-k2:free`) are passed through directly
2. **Claude Short Names**: `haiku`, `sonnet`, `opus` are mapped to their OpenRouter equivalents
3. **Custom Models**: Any model name set via `ANTHROPIC_MODEL` environment variable is used as-is

Claude Code users can override the default model using the `ANTHROPIC_MODEL` environment variable.

## 🔒 Security & Privacy

⚠️ **Important**: This is a proxy service. Your API key will be used to make requests to OpenRouter. Make sure to:
- Use a secure connection
- Keep your API key private
- Only use trusted CCR instances

## 🛠️ Local Development

For testing and development:

```bash
# Start local development server
wrangler dev

# Test locally with Claude Code
export ANTHROPIC_BASE_URL="http://localhost:8787"
export ANTHROPIC_API_KEY="your-openrouter-api-key"
claude
```

## 🚨 Troubleshooting

### Common Issues

**Authentication Error**
```
Error: Invalid API key
```
- Ensure your OpenRouter API key is set: `wrangler secret put OPENROUTER_API_KEY`
- Verify the key is valid at [OpenRouter](https://openrouter.ai)

**Model Not Found**
```
Error: Model not found
```
- Check supported models list above
- Verify model is available on OpenRouter
- Use full OpenRouter model ID if needed

**Streaming Issues**
```
Error: Connection timeout or malformed events
```
- Ensure your OpenRouter API key supports streaming
- Check that the model you're using supports streaming responses

**Worker Not Responding**
- Check deployment status: `wrangler deployments list`
- View logs: `wrangler tail`
- Verify your worker domain is correct

### Getting Help

1. Check the [OpenRouter documentation](https://openrouter.ai/docs)
2. View worker logs: `wrangler tail`
3. Create an issue in this repository
4. Visit your worker homepage for built-in documentation

## ⚠️ Current Limitations

- **Authentication**: Currently uses hardcoded token (will be fixed)
- **Error Handling**: Basic error responses
- **Rate Limiting**: Not implemented

## 🔗 Links

- [OpenRouter](https://openrouter.ai) - Get your API key
- [Cloudflare Workers](https://workers.cloudflare.com) - Hosting platform
- [Anthropic](https://anthropic.com) - Claude models
- [Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/) - Deployment tool

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Built entirely with [Claude Code](https://claude.ai/code)** - Showcasing AI-powered development workflow