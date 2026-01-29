<script lang="ts">
  import { mergedOutput } from '../stores/projectStore';
  import { get } from 'svelte/store';

  let outputContent = '';
  let focused = false;

  $: outputContent = $mergedOutput;

  function handleFocus() {
    focused = true;
  }

  function handleBlur() {
    focused = false;
  }

  function copyToClipboard() {
    navigator.clipboard.writeText(outputContent).then(() => {
      // Show copy feedback
      const copyIndicator = document.createElement('div');
      copyIndicator.className = 'copy-indicator';
      copyIndicator.textContent = 'Copied to clipboard!';
      document.body.appendChild(copyIndicator);
      setTimeout(() => copyIndicator.remove(), 1000);
    }).catch(error => {
      console.error('Failed to copy:', error);
    });
  }
</script>

<div class="merged-output-container">
  <div class="output-header">
    <h3>Merged Output</h3>
    <div class="header-actions">
      <span class="keyboard-hint">Ctrl+c: Copy</span>
      <button class="copy-button" on:click={copyToClipboard}>
        📋 Copy
      </button>
    </div>
  </div>
  <pre 
    id="merged-output"
    class="output-content"
    tabindex="0"
    on:focus={handleFocus}
    on:blur={handleBlur}
  >{outputContent || 'Your merged prompt will appear here...'}</pre>
</div>

<style>
  .merged-output-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #0d1117;
    overflow: hidden;
  }

  .output-header {
    padding: 1rem;
    border-bottom: 1px solid #2d3748;
    background-color: #1a202c;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .output-header h3 {
    margin: 0;
    font-size: 1rem;
    color: #e2e8f0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .keyboard-hint {
    font-size: 0.75rem;
    color: #718096;
  }

  .copy-button {
    background-color: #3182ce;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .copy-button:hover {
    background-color: #2c5282;
  }

  .output-content {
    flex: 1;
    padding: 1.5rem;
    background-color: #0d1117;
    color: #e2e8f0;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    overflow: auto;
    white-space: pre-wrap;
    word-wrap: break-word;
    margin: 0;
    outline: none;
  }

  :global(.copy-indicator) {
    position: fixed;
    top: 1rem;
    right: 1rem;
    background-color: #48bb78;
    color: white;
    padding: 0.5rem 1rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    font-weight: 500;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    z-index: 1000;
  }
</style>
