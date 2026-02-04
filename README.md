# PromptMux

A Tmux-inspired prompt design system built with Rust (Tauri) and Svelte. Create, manage, and refine complex prompts in a structured, section-based format with powerful keyboard-driven navigation.

## Features

- **Local-First Architecture**: All data stored locally on your machine
- **Keyboard-Driven Navigation**: Tmux-style leader key (Ctrl+b) for power users
- **Structured Prompt Design**: Organize prompts into sections and topics
- **Real-time Merged Output**: See your complete prompt update as you type
- **AI-Powered Refinement**: Integrate with LLMs to improve your prompts
- **Dark Mode Theme**: Easy on the eyes for long writing sessions
- **Auto-Save**: Never lose your work

## Architecture

### Backend (Rust/Tauri)
- **State Management**: Thread-safe shared state using `Mutex<Project>`
- **Persistence**: Automatic JSON serialization to disk
- **LLM Integration**: Support for OpenAI, Anthropic, and local models
- **Ordering System**: Uses `order_index` for robust reordering

### Frontend (Svelte)
- **Three-Pane Layout**: Sidebar, Topic Editor, Merged Output
- **Reactive Stores**: Svelte stores sync with backend state
- **Keyboard System**: Leader key with tmux-style shortcuts
- **Tailwind CSS**: Utility-first styling with dark mode

## Installation

### Prerequisites
- Rust (latest stable)
- Node.js 18+
- npm or yarn

### Build from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/promptmux.git
cd promptmux
```

2. Install frontend dependencies:
```bash
npm install
```

3. Build and run:
```bash
npm run tauri dev
```

### Production Build

```bash
npm run tauri build
```

## Configuration

### LLM Settings

Create a settings file at `~/.promptmux/settings.json`:

```json
{
  "provider": "openai",
  "apiKey": "your-api-key-here",
  "baseUrl": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

> **New!** 🎉 We now support Z.ai GLM models! Check out [Z.ai Integration Guide](docs/Z_AI_SETUP.md) for details.

#### Supported Providers

**OpenAI:**
```json
{
  "provider": "openai",
  "apiKey": "sk-...",
  "baseUrl": "https://api.openai.com/v1",
  "model": "gpt-4"
}
```

**Anthropic:**
```json
{
  "provider": "anthropic",
  "apiKey": "sk-ant-...",
  "baseUrl": "https://api.anthropic.com/v1",
  "model": "claude-3-sonnet-20240229"
}
```

**Local Model (e.g., Ollama):**
```json
{
  "provider": "openai",
  "apiKey": "any-key",
  "baseUrl": "http://localhost:11434/v1",
  "model": "llama2"
}
```

**Z.ai (GLM Models):**
```json
{
  "provider": "openai",
  "apiKey": "your-zai-api-key",
  "baseUrl": "https://api.z.ai/api/coding/paas/v4",
  "model": "glm-4.5-air"
}
```

> **Note:** Z.ai provides GLM-4.5-air (lightweight/faster response, recommended) and GLM-4.7 (standard/complex tasks). Both models are OpenAI-compatible and require a Z.ai API key.

## Keyboard Shortcuts

### Global Shortcuts (Press Ctrl+b, then...)

| Key | Action |
|-----|--------|
| `q` | Show/hide help modal |
| `s` | Focus sidebar |
| `e` | Focus topic editor |
| `o` | Focus merged output pane |
| `r` | Refine merged output with LLM |
| `i` | Generate diagram (ER, UML, etc.) |

### Sidebar Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate down/up |
| `h` / `l` | Collapse/expand sections |
| `Enter` | Select item |
| `n` | Create new item |
| `d` | Delete highlighted item |
| `Ctrl+Shift+j/k` | Move item down/up |

### Topic Editor

| Key | Action |
|-----|--------|
| `Ctrl+s` | Manual save |
| `Ctrl+r` | Refine current topic with LLM |

### Merged Output

| Key | Action |
|-----|--------|
| `Ctrl+c` | Copy to clipboard (click in pane first) |

## Usage

1. **Create a Section**: 
   - Click "New Section" button in toolbar, OR
   - Press `Ctrl+b` then `s` to focus sidebar, then press `n` to create a new section
   
2. **Add Topics**: 
   - Navigate to a section in the sidebar (click it)
   - Press `n` to create a new topic in that section
   
3. **Write Content**: 
   - The editor auto-focuses when you select/create a topic
   - Start typing - content auto-saves every 500ms
   
4. **View Merged Output**: The right pane updates in real-time as you type

5. **Refine with AI**: 
   - Press `Ctrl+r` while in editor to refine current topic
   - Press `Ctrl+b` then `r` to refine entire merged output

6. **Navigate with Keyboard**:
   - `Ctrl+b` then `s` - Focus sidebar
   - `Ctrl+b` then `e` - Focus editor
   - `Ctrl+b` then `o` - Focus output pane
   - In sidebar: `j/k` to navigate, `n` for new topic, `Enter` to select

## Data Storage

Projects are stored in:
- **Linux**: `~/.local/share/com.promptmux.dev/`
- **macOS**: `~/Library/Application Support/com.promptmux.dev/`
- **Windows**: `%APPDATA%\com.promptmux.dev\`

The project file (`project.json`) contains all sections, topics, and metadata.

## Development

### Project Structure

```
promptmux/
├── src-tauri/
│   ├── src/
│   │   ├── models.rs      # Data structures
│   │   ├── state.rs       # App state management
│   │   ├── commands.rs    # Tauri commands
│   │   ├── lib.rs         # Entry point
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── components/        # Svelte components
│   ├── stores/           # Svelte stores
│   ├── styles/           # Global styles
│   ├── utils/            # Utilities
│   ├── App.svelte
│   └── main.ts
├── package.json
├── vite.config.ts
└── tailwind.config.js
```

### Backend Development

- All mutations go through Tauri commands
- State is single-source-of-truth in Rust
- Auto-saves on every change

### Frontend Development

- Svelte stores mirror backend state
- Reactive updates across components
- Debounced input for performance

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by tmux's keyboard navigation
- Built with [Tauri](https://tauri.app/) and [Svelte](https://svelte.dev/)
- Uses [Tailwind CSS](https://tailwindcss.com/) for styling

## Roadmap

- [ ] Multiple project support
- [ ] Cloud sync (optional)
- [ ] Prompt templates
- [ ] Export to different formats
- [ ] Version history for topics
- [ ] Search across all topics
- [ ] Tags and metadata
- [ ] Plugin system
