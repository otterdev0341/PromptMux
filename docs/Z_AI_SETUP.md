# Z.ai Integration Guide for PromptMux

This guide explains how to configure PromptMux to use Z.ai's GLM models (GLM-4.7 and GLM-4.5-air) for AI-powered prompt refinement.

## Overview

Z.ai provides powerful GLM (General Language Model) models that are fully compatible with the OpenAI protocol. This means you can use them seamlessly with PromptMux by simply changing the API endpoint and model name.

### Available Models

- **GLM-4.5-air** (Recommended): Lightweight model optimized for faster responses and efficient prompt refinement
- **GLM-4.7**: Standard model for complex tasks, detailed analysis, and advanced code generation

### Why GLM-4.5-air?

We recommend **GLM-4.5-air** for most PromptMux users because:
- ⚡ **Faster response times** - Quick prompt iterations
- 💰 **More cost-effective** - Lower API costs
- 🎯 **Optimized for refinement** - Perfect for prompt improvement tasks
- ✨ **Excellent quality** - Maintains high-quality output

Switch to GLM-4.7 only when you need maximum reasoning power for highly complex tasks.

## Quick Setup

### Step 1: Get Your Z.ai API Key

1. Visit [Z.ai](https://z.ai) and sign up for an account
2. Navigate to the API section in your dashboard
3. Generate a new API key
4. Copy your API key for use in the next step

### Step 2: Configure PromptMux

Create or edit your PromptMux settings file:

```bash
mkdir -p ~/.promptmux
```

Create `~/.promptmux/settings.json` with the following content:

```json
{
  "provider": "openai",
  "apiKey": "your-zai-api-key-here",
  "baseUrl": "https://api.z.ai/api/coding/paas/v4",
  "model": "glm-4.5-air"
}
```

### Step 3: Choose Your Model

The default configuration uses **GLM-4.5-air** (recommended for most users).

**For faster responses (recommended):**
```json
"model": "glm-4.5-air"
```

**For complex tasks:**
```json
"model": "glm-4.7"
```

## Configuration Examples

### Example 1: Using GLM-4.7 for Detailed Refinement

```json
{
  "provider": "openai",
  "apiKey": "zai-sk-xxxxxxxxxxxx",
  "baseUrl": "https://api.z.ai/api/coding/paas/v4",
  "model": "glm-4.7"
}
```

Use this configuration when you need:
- In-depth prompt optimization
- Complex reasoning and analysis
- Detailed content generation
- Code refactoring suggestions

### Example 2: Using GLM-4.5-air for Quick Iterations

```json
{
  "provider": "openai",
  "apiKey": "zai-sk-xxxxxxxxxxxx",
  "baseUrl": "https://api.z.ai/api/coding/paas/v4",
  "model": "glm-4.5-air"
}
```

Use this configuration when you need:
- Fast response times
- Quick prompt tweaks
- Simple text improvements
- Rapid prototyping

## Testing Your Setup

1. **Launch PromptMux:**
   ```bash
   npm run tauri dev
   ```

2. **Create a test topic:**
   - Press `Ctrl+b` then `s` to focus the sidebar
   - Press `n` to create a new topic
   - Enter some test text

3. **Test AI refinement:**
   - Press `Ctrl+r` in the editor
   - The AI should refine your prompt using the Z.ai model

## Troubleshooting

### "API Key Invalid" Error

- Verify your API key is correct
- Check that your Z.ai account is active
- Ensure you have sufficient API credits

### "Connection Failed" Error

- Verify the base URL is correct: `https://api.z.ai/api/coding/paas/v4`
- Check your internet connection
- Ensure Z.ai services are operational

### Slow Response Times

- Consider switching to `glm-4.5-air` for faster responses
- Check your network latency to Z.ai servers
- Reduce the length of prompts being refined

## Advanced Usage

### Switching Between Models

You can easily switch between GLM-4.7 and GLM-4.5-air by editing your `settings.json` file and restarting PromptMux.

### Using with Multiple Projects

If you want to use different models for different projects, you can:

1. Create multiple settings files:
   - `~/.promptmux/settings-glm47.json`
   - `~/.promptmux/settings-glm45air.json`

2. Copy the appropriate one to `settings.json` when switching projects:
   ```bash
   cp ~/.promptmux/settings-glm47.json ~/.promptmux/settings.json
   ```

## Comparison with Other Providers

| Feature | Z.ai GLM-4.7 | OpenAI GPT-4 | Anthropic Claude |
|---------|--------------|--------------|------------------|
| Protocol | OpenAI-compatible | OpenAI-native | Custom |
| Complex Tasks | Excellent | Excellent | Excellent |
| Speed | Fast | Moderate | Fast |
| Cost | Competitive | Higher | Competitive |
| Setup | Easy | Easy | Moderate |

## API Endpoint Details

- **Base URL**: `https://api.z.ai/api/coding/paas/v4`
- **Protocol**: OpenAI-compatible
- **Authentication**: Bearer token (API key)
- **Models**: 
  - `glm-4.7` (standard)
  - `glm-4.5-air` (lightweight)

## Best Practices

1. **Start with GLM-4.5-air**: Use the lightweight model for faster, more cost-effective prompt refinement
2. **Switch to GLM-4.7 for complexity**: Only when you need advanced reasoning or complex analysis
3. **Monitor your usage**: Keep track of API calls to manage costs
4. **Provide context**: The more context you give in your prompts, the better the refinements
5. **Iterate**: Use multiple refinement passes to progressively improve your prompts

## Additional Resources

- [Z.ai Documentation](https://docs.z.ai)
- [Z.ai API Reference](https://docs.z.ai/devpack/tool/others)
- [PromptMux README](../README.md)
- [PromptMux Quick Start](../QUICKSTART.md)

## Support

If you encounter issues with Z.ai integration:

1. Check the [Z.ai status page](https://z.ai/status)
2. Review the [Z.ai documentation](https://docs.z.ai)
3. Check PromptMux troubleshooting in [README.md](../README.md)
4. Report issues on the PromptMux GitHub repository

## Example: Full Workflow

Here's a complete example of setting up and using Z.ai with PromptMux:

```bash
# 1. Install PromptMux
npm install

# 2. Create settings directory
mkdir -p ~/.promptmux

# 3. Copy the Z.ai example settings
cp examples/settings-zai.json ~/.promptmux/settings.json

# 4. Edit the file and add your actual API key
nano ~/.promptmux/settings.json

# 5. Run PromptMux
npm run tauri dev

# 6. In the app:
#    - Press Ctrl+b, then s (focus sidebar)
#    - Press n (create new topic)
#    - Type your prompt
#    - Press Ctrl+r (refine with AI)
```

## Changelog

- **2026-01-30**: Initial documentation for Z.ai GLM integration
