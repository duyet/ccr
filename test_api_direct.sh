#!/bin/bash

# Direct API Test for CCR Local Debug
# This script tests the /v1/messages endpoint directly with curl

set -e

LOCAL_URL="http://localhost:8787"
OPENROUTER_TOKEN="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"
MODEL="moonshotai/kimi-k2"

echo "🔧 CCR Direct API Test"
echo "======================"
echo "Local Server: $LOCAL_URL"
echo "Model: $MODEL"
echo ""

# Test the /v1/messages endpoint directly
echo "📤 Testing /v1/messages endpoint directly..."
echo "This mimics what Claude Code CLI sends to CCR"
echo ""

# Create the Anthropic-format request
REQUEST_JSON='{
  "model": "'$MODEL'",
  "messages": [
    {
      "role": "user",
      "content": "Hello, respond with just the word SUCCESS"
    }
  ],
  "max_tokens": 100
}'

echo "📋 Request payload:"
echo "$REQUEST_JSON" | jq .
echo ""

echo "🚀 Sending request to $LOCAL_URL/v1/messages..."
echo ""

# Send the request and capture response
RESPONSE=$(curl -s -w "\nHTTP_STATUS:%{http_code}\n" \
  -X POST \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENROUTER_TOKEN" \
  -d "$REQUEST_JSON" \
  "$LOCAL_URL/v1/messages")

# Extract HTTP status
HTTP_STATUS=$(echo "$RESPONSE" | grep "HTTP_STATUS:" | cut -d: -f2)
RESPONSE_BODY=$(echo "$RESPONSE" | sed '/HTTP_STATUS:/d')

echo "📥 Response (HTTP $HTTP_STATUS):"
echo "$RESPONSE_BODY" | jq . 2>/dev/null || echo "$RESPONSE_BODY"
echo ""

if [ "$HTTP_STATUS" = "200" ]; then
    echo "✅ Success! The API is working correctly."
else
    echo "❌ Error: HTTP $HTTP_STATUS"
    echo "Check the wrangler dev logs for detailed error information."
fi

echo ""
echo "💡 Tip: Run 'wrangler dev' in another terminal to see real-time logs"