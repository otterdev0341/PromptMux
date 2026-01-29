# Linux Troubleshooting Guide

## Issue: Can't Type or Edit Topics

If you're experiencing issues where you can create topics but can't type into them on Linux, here are several solutions to try:

### Solution 1: Use the "Click to Edit" Button
When a topic is selected but you can't type, look for a blue **"Click to Edit"** button in the editor header. Click it to manually focus the editor.

### Solution 2: Click Directly in the Editor
Simply click anywhere in the text area (the large dark area where you type) to give it focus. You should see:
- A purple hint bar saying "✨ Editor ready! Start typing or click here to focus."
- The cursor appearing in the text area

### Solution 3: Check Browser Console
1. Right-click in the PromptMux window
2. Select "Inspect" or "Inspect Element"
3. Go to the "Console" tab
4. Look for any red error messages
5. Report these errors for further diagnosis

Common errors to look for:
- `Failed to save topic: ...` → File permission issue
- `invoke not allowed` → Tauri command not registered properly
- `textareaElement is undefined` → Component mounting issue

### Solution 4: File Permissions (Linux Specific)

The app stores data in `~/.local/share/com.promptmux.dev/`. Ensure you have write permissions:

```bash
# Check if directory exists and permissions
ls -la ~/.local/share/com.promptmux.dev/

# If directory doesn't exist, create it manually
mkdir -p ~/.local/share/com.promptmux.dev/

# Fix permissions if needed
chmod 755 ~/.local/share/com.promptmux.dev/
chmod 644 ~/.local/share/com.promptmux.dev/project.json
```

### Solution 5: Restart the Dev Server

Sometimes hot-reloading doesn't pick up all changes:

```bash
# Stop the current dev server (Ctrl+C in terminal)
# Then restart it:
npm run tauri dev
```

### Solution 6: Try Different Focus Methods

1. **Keyboard**: Press `Ctrl+b` then `e` to focus the editor
2. **Mouse**: Click anywhere in the text editor area
3. **Button**: Click the "Click to Edit" button in the header

### Solution 7: Check if Data is Saving

Even if you can't see what you're typing, the data might be saving:

1. Type something (even if you can't see it)
2. Press `Ctrl+s` to manually save
3. Check the console for "Saved!" message
4. Switch to a different topic and back
5. Your text should appear if saving is working

### Debug Features Added

The latest version includes several debugging features:

1. **Console Logging**: When editor focuses, it logs to console:
   ```
   Auto-focused textarea for topic: [topic name]
   ```

2. **Focus Indicator**: 
   - A purple gradient bar appears when editor is ready
   - "Click to Edit" button appears when editor is not focused

3. **Save Indicator**: 
   - Green "Saved!" notification appears when you press Ctrl+s

### What to Check in the Code

If you're a developer debugging this issue:

**File: `src/components/TopicEditor.svelte`**
- Lines 13-21: Auto-focus logic with `tick()` and `bind:this`
- Lines 90-101: Textarea with `tabindex="0"` for Linux compatibility
- Lines 66-77: Focus/blur handlers that track editor state

**Key Changes for Linux:**
- Added `tabindex="0"` to make textarea explicitly focusable
- Added `autocomplete="off"`, `autocapitalize="off"`, `autocorrect="off"` to prevent browser interference
- Added visual "Click to Edit" button as fallback
- Added focus hint that appears for 2 seconds after topic selection

### Known Linux Quirks

1. **Wayland vs X11**: Some window managers handle focus differently
2. **Input Methods**: IBus, fcitx, or other input methods might interfere
3. **Browser Security**: Some Linux browsers have stricter security policies

### Temporary Workaround

If nothing works, you can edit the project JSON file directly:

```bash
# Edit the project file
nano ~/.local/share/com.promptmux.dev/project.json
```

Then restart PromptMux to see your changes.

### Next Steps

If none of these solutions work, please provide:
1. Your Linux distribution (e.g., Ubuntu 22.04, Fedora 38, etc.)
2. The browser console errors (if any)
3. Whether you can see the text area at all
4. What happens when you click in the editor
