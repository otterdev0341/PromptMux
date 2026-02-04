<script lang="ts">
  export let onClose: () => void;

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-backdrop" on:click={handleBackdropClick}>
  <div class="modal-content">
    <div class="modal-header">
      <h2>⌨️ Keyboard Shortcuts</h2>
      <button class="close-button" on:click={onClose}>✕</button>
    </div>

    <div class="modal-body">
      <section>
        <h3>Global Shortcuts (Ctrl+b then...)</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>s</kbd></td>
            <td>Create new section</td>
          </tr>
          <tr>
            <td><kbd>q</kbd></td>
            <td>Show/hide this help modal</td>
          </tr>
          <tr>
            <td><kbd>e</kbd></td>
            <td>Focus topic editor</td>
          </tr>
          <tr>
            <td><kbd>o</kbd></td>
            <td>Focus merged output pane</td>
          </tr>
          <tr>
            <td><kbd>r</kbd></td>
            <td>Refine merged output with LLM</td>
          </tr>
          <tr>
            <td><kbd>i</kbd></td>
            <td>Generate diagram (ER, UML, Flowchart, Journey)</td>
          </tr>
          <tr>
            <td><kbd>p</kbd></td>
            <td>Toggle project switcher dropdown</td>
          </tr>
          <tr>
            <td><kbd>n</kbd></td>
            <td>Create new project</td>
          </tr>
          <tr>
            <td><kbd>[</kbd> / <kbd>]</kbd></td>
            <td>Cycle to previous/next project</td>
          </tr>
        </table>
      </section>

      <section>
        <h3>Quick Shortcuts</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>Ctrl+Shift+S</kbd></td>
            <td>Create new section</td>
          </tr>
          <tr>
            <td><kbd>Ctrl+Shift+P</kbd></td>
            <td>Cycle to next project</td>
          </tr>
          <tr>
            <td><kbd>Ctrl+Alt+N</kbd></td>
            <td>Create new project</td>
          </tr>
          <tr>
            <td><kbd>F2</kbd></td>
            <td>Rename current section or topic</td>
          </tr>
        </table>
      </section>

      <section>
        <h3>Item Management</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>F2</kbd></td>
            <td>Rename current section or topic</td>
          </tr>
          <tr>
            <td><kbd>n</kbd> (in sidebar)</td>
            <td>Create new topic (in current section)</td>
          </tr>
        </table>
      </section>

      <section>
        <h3>Sidebar Navigation</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>j</kbd> / <kbd>k</kbd></td>
            <td>Navigate down/up</td>
          </tr>
          <tr>
            <td><kbd>h</kbd> / <kbd>l</kbd></td>
            <td>Collapse/expand sections</td>
          </tr>
          <tr>
            <td><kbd>Enter</kbd></td>
            <td>Select item (topic → editor, section → toggle)</td>
          </tr>
          <tr>
            <td><kbd>n</kbd></td>
            <td>Create new topic (in current section)</td>
          </tr>
          <tr>
            <td><kbd>d</kbd></td>
            <td>Delete highlighted item</td>
          </tr>
          <tr>
            <td><kbd>Ctrl+Shift+j</kbd> / <kbd>Ctrl+Shift+k</kbd></td>
            <td>Move item down/up</td>
          </tr>
          <tr>
            <td><kbd>Mouse drag</kbd></td>
            <td>Drag and drop to reorder sections/topics</td>
          </tr>
        </table>
      </section>

      <section>
        <h3>Topic Editor</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>Ctrl+s</kbd></td>
            <td>Save current topic</td>
          </tr>
          <tr>
            <td><kbd>Ctrl+r</kbd></td>
            <td>Refine current topic with LLM</td>
          </tr>
        </table>
      </section>

      <section>
        <h3>Merged Output</h3>
        <table class="shortcuts-table">
          <tr>
            <td><kbd>Ctrl+c</kbd> (in output pane)</td>
            <td>Copy merged output to clipboard</td>
          </tr>
        </table>
      </section>

      <section class="tips">
        <h3>💡 Tips</h3>
        <ul>
          <li>The merged output updates automatically as you type</li>
          <li>Auto-save is enabled - changes save every 500ms</li>
          <li>Press <kbd>Escape</kbd> to close this modal</li>
          <li>Click outside this modal to close it</li>
        </ul>
      </section>
    </div>
  </div>
</div>

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-content {
    background-color: #1a202c;
    border-radius: 0.5rem;
    max-width: 800px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-header {
    padding: 1.5rem;
    border-bottom: 1px solid #2d3748;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.5rem;
    color: #e2e8f0;
  }

  .close-button {
    background: none;
    border: none;
    color: #718096;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    width: 2rem;
    height: 2rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 0.25rem;
    transition: all 0.15s ease;
  }

  .close-button:hover {
    background-color: #2d3748;
    color: #e2e8f0;
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
  }

  section {
    margin-bottom: 2rem;
  }

  section h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: #63b3ed;
    border-bottom: 1px solid #2d3748;
    padding-bottom: 0.5rem;
  }

  .shortcuts-table {
    width: 100%;
    border-collapse: collapse;
  }

  .shortcuts-table tr {
    border-bottom: 1px solid #2d3748;
  }

  .shortcuts-table tr:last-child {
    border-bottom: none;
  }

  .shortcuts-table td {
    padding: 0.75rem 0;
    vertical-align: top;
  }

  .shortcuts-table td:first-child {
    white-space: nowrap;
    padding-right: 1rem;
  }

  kbd {
    background-color: #2d3748;
    color: #e2e8f0;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    border: 1px solid #4a5568;
  }

  .tips {
    background-color: #2d3748;
    padding: 1rem;
    border-radius: 0.25rem;
  }

  .tips h3 {
    color: #48bb78;
  }

  .tips ul {
    margin: 0;
    padding-left: 1.5rem;
  }

  .tips li {
    margin-bottom: 0.5rem;
    color: #e2e8f0;
  }

  .tips li:last-child {
    margin-bottom: 0;
  }
</style>
