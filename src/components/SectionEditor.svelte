<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { activeSection, activeSectionId } from '../stores/projectStore';
  import type { Refinement } from '../stores/projectStore';

  const dispatch = createEventDispatcher();

  let activeTab: 'view' | 'history' = 'view';
  let mergedContent = '';

  $: if ($activeSection) {
    const sortedTopics = [...$activeSection.topics].sort((a, b) => a.order_index - b.order_index);
    mergedContent = sortedTopics
      .map(topic => topic.content.trim())
      .filter(content => content.length > 0)
      .join('\n\n');
  }

  function formatDate(isoString: string) {
    return new Date(isoString).toLocaleString();
  }

  function handleCopy(content: string) {
    navigator.clipboard.writeText(content).catch(console.error);
    // Show toast?
  }
</script>

<div class="section-editor-container">
  {#if $activeSection}
    <div class="editor-header">
      <div class="header-left">
        <h3>{$activeSection.name}</h3>
        <div class="editor-tabs">
          <button 
            class="tab-btn {activeTab === 'view' ? 'active' : ''}" 
            on:click={() => activeTab = 'view'}
          >
            Merged View
          </button>
          <button 
            class="tab-btn {activeTab === 'history' ? 'active' : ''}" 
            on:click={() => activeTab = 'history'}
          >
            History
            {#if $activeSection.history && $activeSection.history.length > 0}
              <span class="count">{$activeSection.history.length}</span>
            {/if}
          </button>
        </div>
      </div>
      <div class="header-right">
        <button class="action-btn" on:click={() => dispatch('refineSection')}>
          ✨ Refine Section
        </button>
      </div>
    </div>
    
    <div class="editor-wrapper">
      {#if activeTab === 'view'}
        {#if mergedContent}
          <div class="content-view">
            <pre>{mergedContent}</pre>
          </div>
        {:else}
          <div class="empty-state">
            <div class="empty-icon">📂</div>
            <h3>Empty Section</h3>
            <p>Add topics to this section to see merged content.</p>
          </div>
        {/if}
      {:else}
        <div class="history-view">
          {#if $activeSection.history && $activeSection.history.length > 0}
            <div class="history-list">
              {#each [...$activeSection.history].reverse() as item}
                <div class="history-item">
                  <div class="history-meta">
                    <span class="history-time">{formatDate(item.timestamp)}</span>
                    <button class="restore-btn" on:click={() => handleCopy(item.refined_content)}>
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
              <p>Refine this section with AI to generate history versions.</p>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">📂</div>
      <h2>No Section Selected</h2>
    </div>
  {/if}
</div>

<style>
  .section-editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    background-color: #0d1117;
    border-right: 1px solid #2d3748;
    overflow: hidden;
  }

  .editor-wrapper {
    position: relative;
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .editor-header {
    padding: 1rem;
    border-bottom: 1px solid #2d3748;
    background-color: #1a202c;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 1rem;
    flex: 1;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .editor-header h3 {
    margin: 0;
    font-size: 1rem;
    color: #e2e8f0;
  }

  .editor-tabs {
    display: flex;
    background-color: #2d3748;
    padding: 0.25rem;
    border-radius: 0.375rem;
    margin-left: 1rem;
  }

  .tab-btn {
    padding: 0.25rem 0.75rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    color: #a0aec0;
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tab-btn:hover {
    color: #e2e8f0;
  }

  .tab-btn.active {
    background-color: #4a5568;
    color: white;
    font-weight: 600;
  }

  .count {
    background-color: #1a202c;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
    font-size: 0.65rem;
  }

  .action-btn {
    padding: 0.375rem 0.875rem;
    background-color: #3182ce;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .action-btn:hover {
    background-color: #2c5282;
  }

  .content-view {
    flex: 1;
    overflow: auto;
    padding: 1.5rem;
    background-color: #0d1117;
  }

  .content-view pre {
    margin: 0;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: #e2e8f0;
    white-space: pre-wrap;
  }

  .history-view {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem;
  }

  .history-list {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
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
  }

  .history-time {
    font-size: 0.75rem;
    color: #a0aec0;
  }

  .restore-btn {
    padding: 0.25rem 0.75rem;
    background-color: #4a5568;
    color: white;
    border: none;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .restore-btn:hover {
    background-color: #2c5282;
  }

  .history-content-preview {
    padding: 1rem;
    margin: 0;
    background-color: #0d1117;
    overflow-x: auto;
  }

  .history-content-preview pre {
    margin: 0;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    color: #e2e8f0;
    white-space: pre-wrap;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: #718096;
    text-align: center;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
    opacity: 0.5;
  }

  .empty-state h3 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    color: #e2e8f0;
  }

  .empty-state p {
    margin: 0;
    font-size: 1rem;
    max-width: 400px;
  }
</style>
