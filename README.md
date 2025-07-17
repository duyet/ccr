# CCR (Claude Code Router)

A Cloudflare Worker proxy that enables you to use Claude models with OpenAI-compatible APIs. Simply deploy and start using Claude models through OpenRouter with your existing OpenAI client libraries.

## 🚀 What is CCR?

CCR acts as a translation bridge that allows any application built for OpenAI's API to seamlessly work with Claude models without code changes. It automatically converts requests and responses between Anthropic and OpenAI formats.

**Perfect for:**
- Using Claude models with OpenAI client libraries
- Switching between OpenAI and Claude models without code changes
- Accessing Claude models through OpenRouter
- Building applications that support multiple AI providers

## ⚡ Quick Start

### 1. Deploy to Cloudflare Workers

```bash
# Install Wrangler CLI
npm install -g wrangler

# Clone and deploy
git clone <your-repo-url>
cd ccr
wrangler deploy
```

### 2. Configure Authentication

Set your OpenRouter API key:

```bash
wrangler secret put OPENROUTER_API_KEY
# Enter your OpenRouter API key when prompted
```

### 3. Start Using

Replace your OpenAI base URL with your deployed worker URL:

```python
import openai

client = openai.OpenAI(
    api_key="your-openrouter-key",
    base_url="https://your-worker.your-subdomain.workers.dev"
)

# Use Claude models with OpenAI client!
response = client.chat.completions.create(
    model="sonnet",
    messages=[{"role": "user", "content": "Hello Claude!"}]
)
```

## 🔧 Configuration

### Environment Variables

Configure via `wrangler.toml`:

```toml
[vars]
OPENROUTER_BASE_URL = "https://openrouter.ai/api/v1"
```

### Required Secrets

```bash
# Your OpenRouter API key
wrangler secret put OPENROUTER_API_KEY
```

## 🎯 Usage Guide

### Supported Models

CCR automatically maps these model names to OpenRouter:

| Model Name | OpenRouter ID | Description |
|------------|---------------|-------------|
| `haiku` | `anthropic/claude-3.5-haiku` | Fast, lightweight model |
| `sonnet` | `anthropic/claude-sonnet-4` | Balanced performance |
| `opus` | `anthropic/claude-opus-4` | Most capable model |
| `kimi` or `k2` | `moonshot/kimi-k2` | Kimi AI model |

You can also use full OpenRouter model IDs directly:
- `anthropic/claude-3.5-sonnet`
- `openai/gpt-4`
- `meta-llama/llama-3.1-8b`

### API Endpoints

- `POST /v1/messages` - Chat completions (main API endpoint)
- `GET /` - Documentation homepage
- `GET /install.sh` - Installation script
- `GET /terms` - Terms of service
- `GET /privacy` - Privacy policy

### Example Requests

#### Python (OpenAI client)

```python
import openai

client = openai.OpenAI(
    api_key="your-openrouter-key",
    base_url="https://your-worker.workers.dev"
)

# Simple chat
response = client.chat.completions.create(
    model="sonnet",
    messages=[
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Explain quantum computing"}
    ],
    temperature=0.7
)

print(response.choices[0].message.content)

# Streaming example
stream = client.chat.completions.create(
    model="sonnet",
    messages=[{"role": "user", "content": "Count to 10"}],
    stream=True
)

for chunk in stream:
    if chunk.choices[0].delta.content is not None:
        print(chunk.choices[0].delta.content, end="")
```

#### JavaScript/Node.js

```javascript
import OpenAI from 'openai';

const openai = new OpenAI({
  apiKey: 'your-openrouter-key',
  baseURL: 'https://your-worker.workers.dev'
});

const response = await openai.chat.completions.create({
  model: 'sonnet',
  messages: [
    { role: 'user', content: 'Write a haiku about code' }
  ]
});

console.log(response.choices[0].message.content);

// Streaming example
const stream = await openai.chat.completions.create({
  model: 'sonnet',
  messages: [
    { role: 'user', content: 'Count to 10' }
  ],
  stream: true
});

for await (const chunk of stream) {
  if (chunk.choices[0].delta.content) {
    process.stdout.write(chunk.choices[0].delta.content);
  }
}
```

#### cURL

```bash
curl -X POST "https://your-worker.workers.dev/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-openrouter-key" \
  -d '{
    "model": "sonnet",
    "messages": [
      {"role": "user", "content": "Hello Claude!"}
    ]
  }'

# Streaming example
curl -X POST "https://your-worker.workers.dev/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-openrouter-key" \
  -d '{
    "model": "sonnet",
    "messages": [
      {"role": "user", "content": "Count to 10"}
    ],
    "stream": true
  }'
```

### Tool/Function Calling

CCR supports tool calling with Claude models:

```python
tools = [
    {
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get current weather",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                }
            }
        }
    }
]

response = client.chat.completions.create(
    model="sonnet",
    messages=[{"role": "user", "content": "What's the weather in NYC?"}],
    tools=tools
)
```

## 🛠️ Local Development

For testing and development:

```bash
# Start local development server
wrangler dev

# Test locally
curl -X POST "http://localhost:8787/v1/messages" \
  -H "Content-Type: application/json" \
  -d '{"model": "sonnet", "messages": [{"role": "user", "content": "Hello"}]}'
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