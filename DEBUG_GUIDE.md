# 🔬 CCR Local Debug Guide

This guide helps debug the moonshotai/kimi-k2 "Invalid input" 400 error locally.

## Quick Start

1. **Start the local server** (Terminal 1):
   ```bash
   wrangler dev --local --port 8787
   ```
   Wait for: `Ready on http://localhost:8787`

2. **Run the debug coordinator** (Terminal 2):
   ```bash
   ./debug_local.sh
   ```

3. **Test with Claude Code CLI** (Terminal 3):
   ```bash
   ANTHROPIC_BASE_URL="http://localhost:8787" \
   ANTHROPIC_API_KEY="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" \
   ANTHROPIC_MODEL="moonshotai/kimi-k2" \
   claude
   ```

## Available Test Scripts

| Script | Purpose |
|--------|---------|
| `./debug_local.sh` | Main coordinator script |
| `./test_api_direct.sh` | Direct API test with curl |
| `./test_local.sh` | Full Claude Code CLI test |

## What to Look For in Logs

### 🔍 Request JSON Log
Look for: `🔍 Request JSON: {...}`

This shows the exact request we send to OpenRouter. Check:
- ✅ `model`: Should be "moonshotai/kimi-k2"
- ✅ `messages`: Should have role/content structure
- ✅ `temperature`: Should be adjusted for moonshotai (0.6)
- ✅ No null/empty fields

### 🚨 OpenRouter Error Log
Look for: `OpenRouter Error 400: {...}`

This shows OpenRouter's exact error response. Common issues:
- Missing required fields
- Invalid parameter values
- Wrong message format

## Debug Iteration Process

1. **Run test** → See error in logs
2. **Analyze** request JSON vs OpenRouter requirements
3. **Fix** code in `src/transform/mod.rs` or `src/models/mod.rs`
4. **Hot reload** (wrangler dev auto-reloads)
5. **Test again** → Repeat until success

## Key Files to Modify

- `src/transform/mod.rs` - Request transformation logic
- `src/models/mod.rs` - Request/response structures
- `src/routes/proxy.rs` - HTTP handling and error logging

## Success Criteria

✅ No 400 error from OpenRouter
✅ Valid response from moonshotai/kimi-k2
✅ Claude Code CLI works with local CCR
✅ Ready to deploy to production

## Tips

- Keep wrangler dev running in one terminal
- Use different terminals for testing
- Check logs immediately after each test
- jq helps format JSON responses nicely