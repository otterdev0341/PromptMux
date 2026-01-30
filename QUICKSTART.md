# PromptMux Quick Start Guide

## Installation & Setup

1. **Install Dependencies**
   ```bash
   npm install
   ```

2. **Configure LLM (Optional)**
   
   Create the settings file for AI-powered prompt refinement:
   ```bash
   mkdir -p ~/.promptmux
   cp examples/settings.json ~/.promptmux/settings.json
   ```
   
   Edit `~/.promptmux/settings.json` and add your API key.
   ```json
   {
     "provider": "openai",
     "apiKey": "sk-your-actual-api-key",
     "baseUrl": "https://api.openai.com/v1",
     "model": "gpt-4"
   }
   ```

    **For Z.ai (GLM Models):**
    ```json
    {
    "provider": "openai",
    "apiKey": "your-zai-api-key",
    "baseUrl": "https://api.z.ai/api/coding/paas/v4",
    "model": "glm-4.5-air"
    }
    ```
    
    > **Tip:** Z.ai offers GLM-4.5-air (faster responses, recommended) and GLM-4.7 (for complex tasks). Both are OpenAI-compatible.


3. **Run the Application**
   ```bash
   npm run tauri dev
   ```

## First Time Using PromptMux

### Step 1: Create Your First Section
1. Click the **"New Section"** button in the top toolbar
2. Enter a name (e.g., "Project Overview")
3. Press **Enter** or click **Create**

### Step 2: Add Topics to Your Section
1. Click on your new section in the sidebar to select it
2. Press **`n`** to create a new topic
3. The topic will be created and you can immediately start typing

### Step 3: Write Your Prompt
- The editor is now focused - start writing your prompt content
- **Auto-save is enabled** - your work saves every 500ms
- Use the **Merged Output** pane on the right to see your complete prompt in real-time

### Step 4: Create More Topics
1. Press **`n`** while in the sidebar to create another topic
2. The editor automatically focuses on the new topic
3. Continue building your prompt structure

## Keyboard Shortcuts (Essential)

### Navigation
- **`Ctrl+b` then `s`** - Focus sidebar
- **`Ctrl+b` then `e`** - Focus editor  
- **`Ctrl+b` then `o`** - Focus merged output
- **`Ctrl+b` then `?`** - Show help

### In Sidebar (when focused)
- **`j` / `k`** - Move down/up
- **`h` / `l`** - Collapse/expand sections
- **`Enter`** - Select item
- **`n`** - Create new topic in current section
- **`d`** - Delete selected item

### In Editor
- **`Ctrl+s`** - Manual save
- **`Ctrl+r`** - Refine with AI (requires LLM setup)

## Tips for New Users

1. **Start Simple**: Create one section with 2-3 topics to get familiar
2. **Use the Toolbar**: The "New Section" button is the easiest way to start
3. **Watch the Output Pane**: See how your topics merge together in real-time
4. **Keyboard First**: Try to use keyboard shortcuts - they're much faster
5. **Auto-Save**: Don't worry about saving - it happens automatically

## Common Workflows

### Creating a New Prompt Structure
1. Click "New Section" → Name it (e.g., "Requirements")
2. Click the section → Press `n` → Create topic "User Stories"
3. Press `n` again → Create topic "Acceptance Criteria"
4. Press `Ctrl+b e` → Focus editor and start writing

### Refining Your Prompts (with AI)
1. Select a topic you want to improve
2. Press `Ctrl+r` in the editor
3. Review the refined version
4. Click "Accept Refined Version" to replace your content

### Building a Large Prompt
1. Create multiple sections: "Context", "Requirements", "Constraints"
2. Add relevant topics to each section
3. Use the merged output to see the complete prompt
4. Copy the merged output for use in other tools

## Troubleshooting

### Keyboard shortcuts not working?
- Make sure the sidebar is focused (click it first)
- Press `Ctrl+b` then `s` to focus the sidebar

### Can't type in the editor?
- Click on a topic in the sidebar to select it
- The editor will auto-focus when a topic is selected

### LLM refinement not working?
- Make sure you've configured `~/.promptmux/settings.json`
- Check that your API key is valid
- Check the console for error messages

## Next Steps

- Explore all keyboard shortcuts (`Ctrl+b ?`)
- Set up your LLM configuration for AI refinement
- Check out the [Z.ai Integration Guide](docs/Z_AI_SETUP.md) for using GLM models
- Check the full README.md for advanced features
- Build your production app: `npm run tauri build`

## Getting Help

- Press `Ctrl+b ?` anytime to see keyboard shortcuts
- Check README.md for detailed documentation
- Review example settings files in the `examples/` directory
