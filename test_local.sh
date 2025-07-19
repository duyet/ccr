#!/bin/bash

# CCR Local E2E Test Script
# This script tests the CCR proxy locally using the real Claude Code CLI

set -e

LOCAL_URL="http://localhost:8787"
OPENROUTER_TOKEN="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"
MODEL="moonshotai/kimi-k2"

echo "🚀 CCR Local E2E Test"
echo "====================="
echo "Local Server: $LOCAL_URL"
echo "Model: $MODEL"
echo ""

# Step 1: Test basic connectivity
echo "📡 Step 1: Testing basic connectivity..."
if curl -s "$LOCAL_URL" > /dev/null; then
    echo "✅ Local server is responding"
else
    echo "❌ Local server is not responding. Make sure 'wrangler dev' is running!"
    exit 1
fi

# Step 2: Test the home page
echo ""
echo "🏠 Step 2: Testing home page..."
HOME_RESPONSE=$(curl -s "$LOCAL_URL")
if echo "$HOME_RESPONSE" | grep -q "CCR - Claude Code Router"; then
    echo "✅ Home page is working"
else
    echo "❌ Home page is not working properly"
    exit 1
fi

# Step 3: Test Claude Code CLI with the local server
echo ""
echo "🤖 Step 3: Testing Claude Code CLI with local CCR..."
echo "Command: ANTHROPIC_BASE_URL=\"$LOCAL_URL\" ANTHROPIC_API_KEY=\"$OPENROUTER_TOKEN\" ANTHROPIC_MODEL=\"$MODEL\" claude --message=\"test message\""
echo ""

# Run Claude Code CLI with local server
export ANTHROPIC_BASE_URL="$LOCAL_URL"
export ANTHROPIC_API_KEY="$OPENROUTER_TOKEN"
export ANTHROPIC_MODEL="$MODEL"

echo "🔄 Sending test message to Claude Code CLI..."
echo "Message: 'Hello, please respond with just the word SUCCESS if you can read this'"

# Use a simple test message
claude --message="Hello, please respond with just the word SUCCESS if you can read this"

echo ""
echo "🎯 Test completed! Check the wrangler dev logs for detailed request/response info."