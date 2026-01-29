# OS Detection Feature

## What Was Implemented

I've added OS detection to PromptMux that automatically detects whether the app is running on Linux, Windows, or macOS.

## Changes Made

### Backend (Rust)

**File: `src-tauri/src/commands.rs`**
- Added `get_platform()` command that returns the OS using `std::env::consts::OS`

**File: `src-tauri/src/lib.rs`**
- Registered the `get_platform` command in the invoke handler

### Frontend (Svelte)

**File: `src/stores/projectStore.ts`**
- Added `platform` writable store (initialized to 'unknown')
- Added `getPlatform()` function that:
  - Calls the Tauri command
  - Formats the OS name (e.g., "linux" → "Linux", "macos" → "macOS")
  - Has error handling to fallback to "Unknown"

**File: `src/components/Toolbar.svelte`**
- Added platform detection in `onMount()` lifecycle
- Added error handling to prevent app crashes
- Added platform badge display in the toolbar showing:
  - 💻 icon
  - OS name (Linux, Windows, macOS, or Unknown)
  - Styled with a dark badge matching the app theme

## How It Works

1. When the app starts, the Toolbar component's `onMount()` runs
2. It calls `getPlatform()` which invokes the Rust backend
3. Rust returns the OS identifier (linux, windows, macos)
4. The frontend formats it and stores it in the `platform` store
5. The toolbar displays the OS in a badge next to the project name

## Testing

Since npm is broken on your system, you'll need to restart the Tauri dev server to see the changes:

```bash
# If the dev server is still running, it should hot-reload
# If not, restart with:
npm run tauri dev
```

Once running, you should see in the toolbar:
- **PromptMux** (app title)
- **My Prompt Project** (project name)
- **💻 Linux** (platform badge) ← This is the new feature!

## Error Handling

The implementation includes error handling:
- If the Tauri command fails, it logs to console
- The platform defaults to "Unknown" instead of crashing
- The app continues to work normally even if OS detection fails

## Platform Detection Values

The Rust backend returns these values:
- `linux` → displayed as "Linux"
- `windows` → displayed as "Windows"  
- `macos` → displayed as "macOS"
- Any other value → Capitalized and displayed as-is

This feature helps with debugging and ensures you know which platform the app is running on, especially useful when testing keyboard shortcuts that may behave differently on different OSes.
