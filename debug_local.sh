#!/bin/bash

# CCR Local Debug Coordinator Script
# This script helps debug the moonshotai/kimi-k2 400 error locally

set -e

echo "🔬 CCR Local Debug Environment"
echo "================================"
echo ""
echo "This script will help you debug the CCR proxy locally to fix the"
echo "400 'Invalid input' error with moonshotai/kimi-k2 model."
echo ""

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "⚠️  Warning: 'jq' is not installed. JSON responses may not be formatted nicely."
    echo "   Install with: brew install jq"
    echo ""
fi

# Check if claude CLI is installed
if ! command -v claude &> /dev/null; then
    echo "❌ Error: Claude Code CLI is not installed."
    echo "   Install with: npm install -g @anthropic-ai/claude-code"
    exit 1
fi

echo "✅ Prerequisites check passed"
echo ""

echo "📋 Setup Instructions:"
echo "======================"
echo ""
echo "1. Open a NEW terminal window and run:"
echo "   cd $(pwd)"
echo "   wrangler dev --local --port 8787"
echo ""
echo "2. Wait for the message 'Ready on http://localhost:8787'"
echo ""
echo "3. Come back to this terminal and press Enter to continue..."
echo ""
read -p "Press Enter when wrangler dev is running..."

# Test basic connectivity
echo ""
echo "🔍 Step 1: Testing local server connectivity..."
if curl -s http://localhost:8787 > /dev/null; then
    echo "✅ Local server is responding at http://localhost:8787"
else
    echo "❌ Cannot connect to local server. Make sure wrangler dev is running!"
    exit 1
fi

echo ""
echo "🔧 Step 2: Running direct API test..."
echo "This will show us exactly what request/response looks like:"
echo ""

./test_api_direct.sh

echo ""
echo "🤖 Step 3: Ready for Claude Code CLI test!"
echo ""
echo "In another terminal, run:"
echo ""
echo "ANTHROPIC_BASE_URL=\"http://localhost:8787\" \\"
echo "ANTHROPIC_API_KEY=\"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\" \\"
echo "ANTHROPIC_MODEL=\"moonshotai/kimi-k2\" \\"
echo "claude"
echo ""
echo "Then type: test message"
echo ""
echo "📊 Watch the wrangler dev logs to see:"
echo "  🔍 Request JSON: The exact request we send to OpenRouter"
echo "  🚨 OpenRouter Error: The detailed error response"
echo ""
echo "🎯 Goal: Fix the request format until it works!"