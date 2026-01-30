<script lang="ts">
  import { mergedOutput, projectStore, saveProjectRefinement } from '../stores/projectStore';
  import type { Refinement } from '../stores/projectStore';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy } from 'svelte';

  let outputContent = '';
  let refinedContent = '';
  let activeTab: 'raw' | 'refine' = 'raw';
  let refineTab: 'generate' | 'history' = 'generate';
  let isRefining = false;
  let refineError = '';
  let focused = false;
  
  // Track listeners to clean up
  let unlistenFunctions: (() => void)[] = [];

  $: outputContent = $mergedOutput;

  onDestroy(() => {
    cleanupListeners();
  });

  function cleanupListeners() {
    unlistenFunctions.forEach(unlisten => unlisten());
    unlistenFunctions = [];
  }

  function handleFocus() {
    focused = true;
  }

  function handleBlur() {
    focused = false;
  }
  
  function switchTab(tab: 'raw' | 'refine') {
    activeTab = tab;
  }

  function switchRefineTab(tab: 'generate' | 'history') {
    refineTab = tab;
  }

  async function handleRefine() {
    if (!outputContent) return;
    
    // Reset state
    isRefining = true;
    refineError = '';
    refinedContent = '';
    cleanupListeners();
    
    try {
      // Setup listeners BEFORE calling invoke
      const unlistenChunk = await listen<string>('refine:chunk', (event) => {
        refinedContent += event.payload;
      });
      
      const unlistenDone = await listen('refine:done', () => {
        isRefining = false;
        cleanupListeners();
      });
      
      const unlistenError = await listen<string>('refine:error', (event) => {
        console.error('Refine stream error:', event.payload);
        refineError = event.payload;
        isRefining = false;
        cleanupListeners();
      });
      
      unlistenFunctions.push(unlistenChunk, unlistenDone, unlistenError);
      
      // Start the stream
      await invoke('refine_with_llm_stream', { content: outputContent });
    } catch (err) {
      console.error('Refine failed to start:', err);
      refineError = String(err);
      isRefining = false;
      cleanupListeners();
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).then(() => {
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

  function formatDate(isoString: string) {
    return new Date(isoString).toLocaleString();
  }

  function handleRestore(content: string) {
    // For merged output, "restore" mostly means copy to clipboard since we can't edit the source directly from here
    copyToClipboard(content);
  }

  async function handleSaveToHistory() {
    if (!refinedContent) return;
    
    try {
      const refinement: Refinement = {
          id: crypto.randomUUID(),
          original_content: outputContent, // Approximate original
          refined_content: refinedContent,
          timestamp: new Date().toISOString()
      };
      
      await saveProjectRefinement(refinement);
      
      // Feedback
      const indicator = document.createElement('div');
      indicator.className = 'copy-indicator';
      indicator.textContent = 'Saved to history!';
      document.body.appendChild(indicator);
      setTimeout(() => indicator.remove(), 1000);
    } catch (err) {
      console.error('Failed to save history:', err);
    }
  }
</script>

<div class="merged-output-container">
  <div class="output-header">
    <div class="tabs">
      <button 
        class="tab-btn {activeTab === 'raw' ? 'active' : ''}" 
        on:click={() => switchTab('raw')}
      >
        Merged Output
      </button>
      <button 
        class="tab-btn {activeTab === 'refine' ? 'active' : ''}" 
        on:click={() => switchTab('refine')}
      >
        Refine ✨
      </button>
    </div>
    
    <div class="header-actions">
      {#if activeTab === 'raw'}
        <span class="keyboard-hint">Ctrl+c: Copy</span>
        <button class="copy-button" on:click={() => copyToClipboard(outputContent)}>
          📋 Copy
        </button>
      {:else if refinedContent}
        <button class="copy-button" on:click={() => copyToClipboard(refinedContent)}>
          📋 Copy Refined
        </button>
      {/if}
    </div>
  </div>

  <div class="content-area">
    {#if activeTab === 'raw'}
      <pre 
        id="merged-output"
        class="output-content"
        tabindex="0"
        on:focus={handleFocus}
        on:blur={handleBlur}
      >{outputContent || 'Your merged prompt will appear here...'}</pre>
    {:else if activeTab === 'refine'}
      <div class="refine-wrapper">
        <div class="refine-tabs">
          <button 
            class="refine-tab-btn {refineTab === 'generate' ? 'active' : ''}" 
            on:click={() => switchRefineTab('generate')}
          >
            Generate
          </button>
          <button 
            class="refine-tab-btn {refineTab === 'history' ? 'active' : ''}" 
            on:click={() => switchRefineTab('history')}
          >
            History
            {#if $projectStore && $projectStore.history && $projectStore.history.length > 0}
              <span class="count">{$projectStore.history.length}</span>
            {/if}
          </button>
        </div>

        {#if refineTab === 'generate'}
          <div class="refine-container">
            {#if isRefining}
              <div class="loading-state">
                <div class="spinner"></div>
                <p>Refining prompt with AI...</p>
              </div>
            {:else if refineError}
               <div class="error-state">
                 <p>Refinement failed:</p>
                 <pre class="error-msg">{refineError}</pre>
                 <button class="action-btn" on:click={handleRefine}>Retry</button>
               </div>
            {:else if refinedContent}
               <pre class="output-content refined">{refinedContent}</pre>
               <div class="refine-actions">
                 <button class="action-btn secondary" on:click={handleRefine}>Re-generate</button>
                 <button class="action-btn primary" on:click={handleSaveToHistory}>Save to History</button>
               </div>
            {:else}
               <div class="empty-state">
                 <p>Use AI to improve your prompt's clarity and structure.</p>
                 <button 
                   class="action-btn primary" 
                   on:click={handleRefine}
                   disabled={!outputContent}
                  >
                   Refine with AI
                 </button>
                 {#if !outputContent}
                   <p class="hint">Add some content to your sections first.</p>
                 {/if}
               </div>
            {/if}
          </div>
        {:else if refineTab === 'history'}
          <div class="history-view">
            {#if $projectStore && $projectStore.history && $projectStore.history.length > 0}
              <div class="history-list">
                {#each [...$projectStore.history].reverse() as item}
                  <div class="history-item">
                    <div class="history-meta">
                      <span class="history-time">{formatDate(item.timestamp)}</span>
                      <button class="restore-btn" on:click={() => handleRestore(item.refined_content)}>
                        📋 Copy to Clipboard
                      </button>
                    </div>
                    <div class="history-content-preview">
                      <pre>{item.refined_content}</pre>
                    </div>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="empty-state">
                <span class="empty-icon">⏳</span>
                <h3>No History Yet</h3>
                <p>Refine the merged output and save it to see history here.</p>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}
  </div>
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
    background-color: #1a202c;
    border-bottom: 1px solid #2d3748;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1rem;
    height: 48px;
  }

  .tabs {
    display: flex;
    height: 100%;
  }

  .tab-btn {
    background: none;
    border: none;
    color: #a0aec0;
    padding: 0 1rem;
    height: 100%;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    color: #e2e8f0;
    background-color: #2d3748;
  }

  .tab-btn.active {
    color: #63b3ed;
    border-bottom-color: #63b3ed;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .content-area {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    position: relative;
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

  .refine-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }

  .refine-actions {
    padding: 1rem;
    border-top: 1px solid #2d3748;
    background-color: #1a202c;
    display: flex;
    justify-content: flex-end;
  }

  .empty-state, .loading-state, .error-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
    color: #a0aec0;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 3px solid #2d3748;
    border-top-color: #63b3ed;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .action-btn {
    padding: 0.625rem 1.25rem;
    border-radius: 0.375rem;
    font-weight: 500;
    font-size: 0.875rem;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
    margin-top: 1rem;
  }

  .action-btn.primary {
    background-color: #3182ce;
    color: white;
  }

  .action-btn.primary:hover:not(:disabled) {
    background-color: #2c5282;
  }
  
  .action-btn.secondary {
    background-color: #4a5568;
    color: white;
  }
  
  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-msg {
    color: #f56565;
    background-color: rgba(245, 101, 101, 0.1);
    padding: 1rem;
    border-radius: 0.5rem;
    margin: 1rem 0;
    text-align: left;
    white-space: pre-wrap;
    font-family: monospace;
    font-size: 0.8rem;
    max-width: 100%;
    overflow-x: auto;
  }

  .copy-button {
    background-color: #2d3748;
    color: #e2e8f0;
    border: 1px solid #4a5568;
    padding: 0.25rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    cursor: pointer;
  }

  .copy-button:hover {
    background-color: #4a5568;
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

  .count {
    background-color: #2d3748;
    color: #e2e8f0;
    font-size: 0.75rem;
    padding: 0.125rem 0.375rem;
    border-radius: 9999px;
    margin-left: 0.5rem;
  }

  .history-view {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    padding: 0;
  }

  .history-list {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .history-item {
    background-color: #1a202c;
    border: 1px solid #2d3748;
    border-radius: 0.5rem;
    overflow: hidden;
  }

  .history-meta {
    padding: 0.75rem 1rem;
    background-color: #2d3748;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #4a5568;
  }

  .history-time {
    color: #a0aec0;
    font-size: 0.875rem;
  }

  .restore-btn {
    background: none;
    border: 1px solid #4a5568;
    color: #e2e8f0;
    padding: 0.25rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .restore-btn:hover {
    background-color: #4a5568;
  }

  .history-content-preview {
    padding: 1rem;
    max-height: 200px;
    overflow: hidden;
    position: relative;
    background-color: #0d1117;
  }

  .history-content-preview::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 40px;
    background: linear-gradient(transparent, #0d1117);
  }

  .history-content-preview pre {
    margin: 0;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    color: #e2e8f0;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .refine-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .refine-tabs {
    display: flex;
    border-bottom: 1px solid #2d3748;
    background-color: #1a202c;
    justify-content: center;
    gap: 1rem;
    padding: 0.5rem;
  }

  .refine-tab-btn {
    background: none;
    border: none;
    color: #a0aec0;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    border-radius: 0.25rem;
    transition: all 0.2s;
    display: flex;
    align-items: center;
  }

  .refine-tab-btn:hover {
    color: #e2e8f0;
    background-color: #2d3748;
  }

  .refine-tab-btn.active {
    color: #63b3ed;
    background-color: rgba(99, 179, 237, 0.1);
  }

  .empty-icon {
    font-size: 2rem;
    margin-bottom: 1rem;
  }
</style>
