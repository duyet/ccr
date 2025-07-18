use worker::{Response, Result};

pub async fn home() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>CCR - Claude Code Router</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-50 text-gray-900">
    <div class="min-h-screen py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-4xl mx-auto">
            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
                <h1 class="text-3xl font-bold text-gray-900 mb-4">CCR - Claude Code Router</h1>
                <p class="text-lg text-gray-600 mb-8">A seamless proxy enabling Claude Code to work with OpenRouter's diverse model selection</p>
                
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-8">
                    <h2 class="font-semibold text-gray-900 mb-4">What is CCR?</h2>
                    <p class="text-gray-700 mb-6">
                        This Cloudflare Worker acts as a translation layer between Anthropic's Claude API format and OpenAI-compatible APIs, specifically OpenRouter. It allows Claude Code to access a wide range of models through OpenRouter while maintaining the familiar Claude API interface.
                    </p>
                    
                    <div class="bg-white border border-gray-300 rounded-lg p-4">
                        <h3 class="font-semibold text-gray-900 mb-3 text-center">🔄 How CCR Works</h3>
                        <pre class="text-sm text-gray-800 font-mono leading-relaxed overflow-x-auto">
┌───────────────────┐     ┌───────────────────┐     ┌───────────────────┐
│   Claude Code     │────▶│       CCR         │────▶│   OpenRouter      │
│                   │     │                   │     │                   │
│ ANTHROPIC_BASE_   │     │ API Format        │     │ Multiple Models:  │
│ URL="ccr.duyet.   │     │ Translation       │     │                   │
│ net"              │     │                   │     │ • Anthropic       │
│                   │     │ Model Pass-       │     │ • OpenAI          │
│ ANTHROPIC_API_    │     │ through or        │     │ • Moonshot        │
│ KEY="your-open    │     │ Mapping           │     │ • Google          │
│ router-api-key"   │     │                   │     │ • Meta            │
│                   │     │                   │     │ • DeepSeek        │
│ ANTHROPIC_MODEL=  │     │                   │     │ • & More...       │
│ "kimi-k2"         │     │                   │     │                   │
│                   │◀────│                   │◀────│                   │
│ Anthropic Format  │     │                   │     │                   │
└───────────────────┘     └───────────────────┘     └───────────────────┘

</pre>
                    </div>
                </div>

                <div class="grid md:grid-cols-3 gap-6 mb-8">
                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h3 class="font-semibold text-gray-900 mb-2">🔄 API Translation</h3>
                        <p class="text-gray-600">Seamlessly converts between Anthropic and OpenAI API formats</p>
                    </div>
                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h3 class="font-semibold text-gray-900 mb-2">🌐 OpenRouter Integration</h3>
                        <p class="text-gray-600">Access to multiple AI models through OpenRouter's unified API</p>
                    </div>
                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h3 class="font-semibold text-gray-900 mb-2">⚡ Cloudflare Workers</h3>
                        <p class="text-gray-600">Fast, globally distributed proxy with minimal latency</p>
                    </div>
                </div>

                <h2 class="text-2xl font-bold text-gray-900 mb-6">🛠️ Quick Setup</h2>
                <div class="space-y-4 mb-8">
                    <div class="flex items-start space-x-4">
                        <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">1</div>
                        <div>
                            <h3 class="font-semibold text-gray-900">Set up Claude Code</h3>
                            <p class="text-gray-600 mb-2"><a href="https://docs.anthropic.com/en/docs/claude-code/setup" class="text-blue-600 hover:text-blue-800">Claude Code Setup Guide</a></p>
                            <pre class="bg-gray-800 text-gray-100 p-2 rounded text-sm">npm install -g @anthropic-ai/claude-code</pre>
                        </div>
                    </div>
                    <div class="flex items-start space-x-4">
                        <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">2</div>
                        <div>
                            <h3 class="font-semibold text-gray-900">Get OpenRouter API Key</h3>
                            <p class="text-gray-600">Sign up at <a href="https://openrouter.ai" class="text-blue-600 hover:text-blue-800">openrouter.ai</a> and get your API key</p>
                        </div>
                    </div>
                    <div class="flex items-start space-x-4">
                        <div class="flex-shrink-0 w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">3</div>
                        <div class="flex-1">
                            <h3 class="font-semibold text-gray-900">Using Claude Code with CCR</h3>
                            <div class="space-y-4">
                                <div>
                                    <h4 class="font-semibold text-gray-900 mb-2">Basic Usage</h4>
                                    <p class="text-sm text-gray-600 mb-2">Option 1: Using ANTHROPIC_API_KEY</p>
                                    <pre class="bg-gray-800 text-gray-100 p-3 rounded-lg overflow-x-auto text-sm whitespace-pre-wrap break-all mb-3">ANTHROPIC_BASE_URL="https://ccr.duyet.net" \
ANTHROPIC_API_KEY="your-openrouter-api-key" \
claude</pre>
                                    <p class="text-sm text-gray-600 mb-2">Option 2: Using ANTHROPIC_AUTH_TOKEN</p>
                                    <pre class="bg-gray-800 text-gray-100 p-3 rounded-lg overflow-x-auto text-sm whitespace-pre-wrap break-all">ANTHROPIC_BASE_URL="https://ccr.duyet.net" \
ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key" \
claude</pre>
                                </div>
                                <div>
                                    <h4 class="font-semibold text-gray-900 mb-2">With Custom Models</h4>
                                    <p class="text-sm text-gray-600 mb-2">Using ANTHROPIC_API_KEY</p>
                                    <pre class="bg-gray-800 text-gray-100 p-3 rounded-lg overflow-x-auto text-sm whitespace-pre-wrap break-all mb-3">ANTHROPIC_BASE_URL="https://ccr.duyet.net" \
ANTHROPIC_API_KEY="your-openrouter-api-key" \
ANTHROPIC_MODEL="moonshotai/kimi-k2:free" \
claude</pre>
                                    <p class="text-sm text-gray-600 mb-2">Using ANTHROPIC_AUTH_TOKEN</p>
                                    <pre class="bg-gray-800 text-gray-100 p-3 rounded-lg overflow-x-auto text-sm whitespace-pre-wrap break-all">ANTHROPIC_BASE_URL="https://ccr.duyet.net" \
ANTHROPIC_AUTH_TOKEN="your-openrouter-api-key" \
ANTHROPIC_MODEL="moonshotai/kimi-k2:free" \
claude</pre>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-8">
                    <p class="text-yellow-800">
                        <strong>⚠️ Note:</strong> This is a proxy service. Your API key will be used to make requests to OpenRouter. Make sure to use a secure connection and keep your API key private.
                    </p>
                </div>

                <h2 class="text-2xl font-bold text-gray-900 mb-6">🚀 Quick Actions</h2>
                <div class="grid sm:grid-cols-3 gap-4 mb-8">
                    <a href="/install.sh" class="group bg-gradient-to-r from-blue-600 to-blue-700 text-white p-6 rounded-xl hover:from-blue-700 hover:to-blue-800 transition-all duration-300 transform hover:scale-105 shadow-lg">
                        <div class="text-center">
                            <div class="text-3xl mb-3">🚀</div>
                            <h3 class="font-bold text-lg mb-2">Install Script</h3>
                            <p class="text-blue-100 text-sm">One-click setup with automated environment configuration</p>
                        </div>
                    </a>
                    <a href="https://docs.anthropic.com/en/docs/claude-code/setup" target="_blank" class="group bg-gradient-to-r from-green-600 to-green-700 text-white p-6 rounded-xl hover:from-green-700 hover:to-green-800 transition-all duration-300 transform hover:scale-105 shadow-lg">
                        <div class="text-center">
                            <div class="text-3xl mb-3">📚</div>
                            <h3 class="font-bold text-lg mb-2">Claude Code Guide</h3>
                            <p class="text-green-100 text-sm">Official setup documentation and getting started guide</p>
                        </div>
                    </a>
                    <a href="https://openrouter.ai" target="_blank" class="group bg-gradient-to-r from-purple-600 to-purple-700 text-white p-6 rounded-xl hover:from-purple-700 hover:to-purple-800 transition-all duration-300 transform hover:scale-105 shadow-lg">
                        <div class="text-center">
                            <div class="text-3xl mb-3">🔑</div>
                            <h3 class="font-bold text-lg mb-2">Get API Key</h3>
                            <p class="text-purple-100 text-sm">Sign up for OpenRouter and get your API key</p>
                        </div>
                    </a>
                </div>
                
                <div class="border-t border-gray-200 pt-8 text-center">
                    <div class="flex justify-center space-x-4 text-sm text-gray-600">
                        <a href="https://duyet.net" target="_blank" class="hover:text-blue-600">duyet.net</a>
                        <span>•</span>
                        <a href="/terms" class="hover:text-blue-600">Terms</a>
                        <span>•</span>
                        <a href="/privacy" class="hover:text-blue-600">Privacy</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
    "#;

    Response::from_html(html)
}

pub async fn terms() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Terms of Service - CCR</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-50 text-gray-900">
    <div class="min-h-screen py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-4xl mx-auto">
            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
                <a href="/" class="inline-block bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors mb-6">← Back to Home</a>
                
                <h1 class="text-3xl font-bold text-gray-900 mb-4">📋 Terms of Service</h1>
                <p class="text-gray-600 mb-8"><strong>Effective Date:</strong> July 17, 2025</p>
                
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-8">
                    <p class="text-blue-800">
                        <strong>Important:</strong> By using CCR (Claude Code Router), you agree to these terms and conditions. This service is provided "as is" without warranties.
                    </p>
                </div>

                <div class="space-y-8">
                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">1. Service Description</h2>
                        <p class="text-gray-700 mb-4">CCR is a proxy service that translates API requests between Anthropic's Claude API format and OpenAI-compatible APIs, specifically OpenRouter. This service:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Acts as a middleware layer for API translation</li>
                            <li>Forwards your requests to OpenRouter using your API key</li>
                            <li>Does not store or log your API conversations</li>
                            <li>Operates on Cloudflare Workers infrastructure</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">2. User Responsibilities</h2>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>You must provide a valid OpenRouter API key</li>
                            <li>You are responsible for all charges incurred through your API usage</li>
                            <li>You must comply with OpenRouter's terms of service</li>
                            <li>You must not use the service for illegal or harmful purposes</li>
                            <li>You must keep your API key secure and private</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">3. Service Limitations</h2>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Service availability is not guaranteed</li>
                            <li>Streaming functionality is not currently implemented</li>
                            <li>Rate limits may apply based on Cloudflare Workers limits</li>
                            <li>The service may be discontinued without notice</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">4. Privacy & Data</h2>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>We do not store your API requests or responses</li>
                            <li>Your API key is used only for forwarding requests to OpenRouter</li>
                            <li>Standard Cloudflare Workers logging may apply</li>
                            <li>See our <a href="/privacy" class="text-blue-600 hover:text-blue-800">Privacy Policy</a> for more details</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">5. Disclaimer</h2>
                        <p class="text-gray-700 mb-4">This service is provided "as is" without any warranties. The service provider is not responsible for:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Service interruptions or downtime</li>
                            <li>Data loss or corruption</li>
                            <li>Costs incurred from API usage</li>
                            <li>Any damages arising from service use</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">6. Changes to Terms</h2>
                        <p class="text-gray-700">These terms may be updated without prior notice. Continued use of the service constitutes acceptance of any changes.</p>
                    </div>
                </div>

                <div class="border-t border-gray-200 pt-8 mt-8 text-center">
                    <div class="flex justify-center space-x-4">
                        <a href="/" class="text-blue-600 hover:text-blue-800">← Back to Home</a>
                        <span class="text-gray-400">|</span>
                        <a href="/privacy" class="text-blue-600 hover:text-blue-800">Privacy Policy</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
    "#;

    Response::from_html(html)
}

pub async fn privacy() -> Result<Response> {
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Privacy Policy - CCR</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-50 text-gray-900">
    <div class="min-h-screen py-12 px-4 sm:px-6 lg:px-8">
        <div class="max-w-4xl mx-auto">
            <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-8">
                <a href="/" class="inline-block bg-blue-600 text-white px-4 py-2 rounded-lg hover:bg-blue-700 transition-colors mb-6">← Back to Home</a>
                
                <h1 class="text-3xl font-bold text-gray-900 mb-4">🔒 Privacy Policy</h1>
                <p class="text-gray-600 mb-8"><strong>Effective Date:</strong> July 17, 2025</p>
                
                <div class="bg-green-50 border border-green-200 rounded-lg p-4 mb-8">
                    <p class="text-green-800">
                        <strong>Good News:</strong> CCR is designed with privacy in mind. We don't store your conversations, API keys, or personal data.
                    </p>
                </div>

                <div class="space-y-8">
                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">1. What We Don't Collect</h2>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li><strong>API Conversations:</strong> We do not store or log your API requests or responses</li>
                            <li><strong>Personal Data:</strong> We don't collect names, emails, or other personal information</li>
                            <li><strong>API Keys:</strong> Your OpenRouter API key is used only for request forwarding and not stored</li>
                            <li><strong>Usage Analytics:</strong> We don't track your individual usage patterns</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">2. How the Service Works</h2>
                        <p class="text-gray-700 mb-4">When you use CCR:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Your request is received by our Cloudflare Worker</li>
                            <li>The request format is translated from Anthropic to OpenAI format</li>
                            <li>The translated request is forwarded to OpenRouter with your API key</li>
                            <li>OpenRouter's response is translated back to Anthropic format</li>
                            <li>The response is sent back to you</li>
                            <li><strong>Nothing is stored during this process</strong></li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">3. Third-Party Services</h2>
                        <p class="text-gray-700 mb-4">CCR relies on these third-party services:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2 mb-4">
                            <li><strong>Cloudflare Workers:</strong> Hosting platform that may log basic request metadata (IP addresses, timestamps) as per their privacy policy</li>
                            <li><strong>OpenRouter:</strong> API service that processes your requests according to their privacy policy</li>
                        </ul>
                        <p class="text-gray-700 mb-2">We recommend reviewing their privacy policies:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li><a href="https://www.cloudflare.com/privacy/" target="_blank" class="text-blue-600 hover:text-blue-800">Cloudflare Privacy Policy</a></li>
                            <li><a href="https://openrouter.ai/privacy" target="_blank" class="text-blue-600 hover:text-blue-800">OpenRouter Privacy Policy</a></li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">4. Data Security</h2>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>All communications use HTTPS encryption</li>
                            <li>Your API key is transmitted securely and not stored</li>
                            <li>The service runs on Cloudflare's secure infrastructure</li>
                            <li>No persistent storage of user data</li>
                        </ul>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">5. Logging & Monitoring</h2>
                        <p class="text-gray-700 mb-4">Standard Cloudflare Workers logging may include:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Request timestamps</li>
                            <li>Response status codes</li>
                            <li>IP addresses (for basic DDoS protection)</li>
                            <li>Request sizes</li>
                        </ul>
                        <p class="text-gray-700 font-semibold mt-4">These logs do not contain your API requests, responses, or API keys.</p>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">6. Your Rights</h2>
                        <p class="text-gray-700 mb-4">Since we don't store personal data, there's no personal information to:</p>
                        <ul class="list-disc list-inside text-gray-700 space-y-2">
                            <li>Access or download</li>
                            <li>Correct or update</li>
                            <li>Delete or remove</li>
                        </ul>
                        <p class="text-gray-700 mt-4">Your privacy is protected by design.</p>
                    </div>

                    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6">
                        <h2 class="text-xl font-semibold text-gray-900 mb-4">7. Changes to This Policy</h2>
                        <p class="text-gray-700">We may update this privacy policy to reflect changes in our practices or for other operational, legal, or regulatory reasons. Any changes will be posted on this page with an updated effective date.</p>
                    </div>
                </div>

                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mt-8">
                    <p class="text-blue-800">
                        <strong>Questions?</strong> This service is designed to be transparent and privacy-focused. If you have concerns about privacy, consider reviewing the source code or self-hosting the service.
                    </p>
                </div>

                <div class="border-t border-gray-200 pt-8 mt-8 text-center">
                    <div class="flex justify-center space-x-4">
                        <a href="/" class="text-blue-600 hover:text-blue-800">← Back to Home</a>
                        <span class="text-gray-400">|</span>
                        <a href="/terms" class="text-blue-600 hover:text-blue-800">Terms of Service</a>
                    </div>
                </div>
            </div>
        </div>
    </div>
</body>
</html>
    "#;

    Response::from_html(html)
}

pub async fn install_sh() -> Result<Response> {
    let script = r##"#!/bin/bash
# CCR (Claude Code Router) Installation Script
# This script helps you set up environment variables for using CCR with Claude Code

set -e

echo "🚀 CCR - Claude Code Router Installation"
echo "========================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to detect shell
detect_shell() {
    if [ -n "$ZSH_VERSION" ]; then
        echo "zsh"
    elif [ -n "$BASH_VERSION" ]; then
        echo "bash"
    else
        echo "unknown"
    fi
}

# Function to get shell config file
get_shell_config() {
    local shell_type=$(detect_shell)
    case $shell_type in
        "zsh")
            echo "$HOME/.zshrc"
            ;;
        "bash")
            if [ -f "$HOME/.bashrc" ]; then
                echo "$HOME/.bashrc"
            else
                echo "$HOME/.bash_profile"
            fi
            ;;
        *)
            echo "$HOME/.profile"
            ;;
    esac
}

# Check if Claude Code is installed
echo -e "${BLUE}Checking Claude Code installation...${NC}"
if command -v claude >/dev/null 2>&1; then
    echo -e "${GREEN}✓ Claude Code is installed${NC}"
else
    echo -e "${YELLOW}⚠ Claude Code not found. Please install it first:${NC}"
    echo "  Visit: https://claude.ai/code"
    echo ""
fi

# Get OpenRouter API key
echo -e "${BLUE}Setting up OpenRouter API key...${NC}"
echo "You need an OpenRouter API key to use CCR."
echo "1. Visit: https://openrouter.ai"
echo "2. Sign up/login and get your API key"
echo ""

read -p "Enter your OpenRouter API key: " OPENROUTER_API_KEY

if [ -z "$OPENROUTER_API_KEY" ]; then
    echo -e "${RED}✗ API key cannot be empty${NC}"
    exit 1
fi

# Get shell config file
SHELL_CONFIG=$(get_shell_config)
echo -e "${BLUE}Detected shell config: $SHELL_CONFIG${NC}"

# Create backup of shell config
echo -e "${BLUE}Creating backup of shell config...${NC}"
cp "$SHELL_CONFIG" "$SHELL_CONFIG.backup.$(date +%Y%m%d_%H%M%S)"

# Add environment variables
echo -e "${BLUE}Adding environment variables...${NC}"
echo "" >> "$SHELL_CONFIG"
echo "# CCR - Claude Code Router configuration" >> "$SHELL_CONFIG"
echo "export ANTHROPIC_BASE_URL=\"https://ccr.duyet.net\"" >> "$SHELL_CONFIG"
# Choose one:
echo "export ANTHROPIC_API_KEY=\"$OPENROUTER_API_KEY\"" >> "$SHELL_CONFIG"
# OR
echo "export ANTHROPIC_AUTH_TOKEN=\"$OPENROUTER_API_KEY\"" >> "$SHELL_CONFIG"
echo "" >> "$SHELL_CONFIG"

echo -e "${GREEN}✓ Environment variables added to $SHELL_CONFIG${NC}"

# Instructions
echo ""
echo -e "${GREEN}🎉 Installation complete!${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Reload your shell configuration:"
echo "   source $SHELL_CONFIG"
echo ""
echo "2. Test the setup:"
echo "   claude"
echo ""
echo -e "${BLUE}Note:${NC} Your API key is now configured. CCR will route Claude Code requests through OpenRouter."
echo ""
echo -e "${YELLOW}Troubleshooting:${NC}"
echo "- If you have issues, check that your OpenRouter API key is valid"
echo "- Your shell config backup is saved as: $SHELL_CONFIG.backup.*"
echo "- Visit https://ccr.duyet.net for more information"
echo ""
echo -e "${GREEN}Happy coding! 🚀${NC}"
    "##;

    let mut response = Response::ok(script)?;
    response
        .headers_mut()
        .set("Content-Type", "text/plain; charset=utf-8")?;
    Ok(response)
}
