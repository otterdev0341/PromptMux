<script lang="ts">
  import { activeTopic, activeSection, mergedOutput, updateTopicContent, saveTopicRefinement, saveSectionRefinement } from '../stores/projectStore';
  import { refineWithLlm } from '../stores/projectStore';
  import type { Refinement } from '../stores/projectStore';

  export let target: 'topic' | 'section' | 'merged' | null = null;
  export let onClose: () => void;

  let loading = false;
  let error: string | null = null;
  let refinedContent = '';
  let originalContent = '';

  $: if (target === 'topic' && $activeTopic) {
    originalContent = $activeTopic.content;
  } else if (target === 'section' && $activeSection) {
     const sortedTopics = [...$activeSection.topics].sort((a, b) => a.order_index - b.order_index);
     originalContent = sortedTopics
        .map(topic => topic.content.trim())
        .filter(content => content.length > 0)
        .join('\n\n');
  } else if (target === 'merged') {
    originalContent = $mergedOutput;
  }

  async function handleRefine() {
    if (!originalContent.trim()) {
      error = 'No content to refine';
      return;
    }

    loading = true;
    error = null;

    try {
      refinedContent = await refineWithLlm(originalContent);
      
      const refinement: Refinement = {
          id: crypto.randomUUID(),
          original_content: originalContent,
          refined_content: refinedContent,
          timestamp: new Date().toISOString()
      };

      // Save to history based on target
      if (target === 'topic' && $activeTopic) {
        await saveTopicRefinement($activeTopic.id, refinement);
      } else if (target === 'section' && $activeSection) {
        await saveSectionRefinement($activeSection.id, refinement);
      }
    } catch (err) {
      error = err as string;
      console.error('Failed to refine:', err);
    } finally {
      loading = false;
    }
  }

  async function handleAccept() {
    if (target === 'topic' && $activeTopic) {
      try {
        await updateTopicContent($activeTopic.id, refinedContent);
        onClose();
      } catch (err) {
        error = err as string;
      }
    } else if (target === 'merged') {
      // For merged output, we can't directly update it
      // The user would need to manually copy the refined content
      error = 'For merged output, please copy the refined content manually';
    }
  }

  function handleCancel() {
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget && !loading) {
      onClose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !loading) {
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="modal-backdrop" on:click={handleBackdropClick}>
  <div class="modal-content">
    <div class="modal-header">
      <h2>
        {target === 'topic' ? '✨ Refine Topic' : '✨ Refine Merged Output'}
      </h2>
      <button 
        class="close-button" 
        on:click={handleCancel}
        disabled={loading}
      >✕</button>
    </div>

    <div class="modal-body">
      {#if loading}
        <div class="loading-state">
          <div class="spinner"></div>
          <p>Refining your prompt with AI...</p>
          <span class="subtext">This may take a few moments</span>
        </div>
      {:else if refinedContent}
        <div class="result-state">
          <div class="content-comparison">
            <div class="content-panel">
              <h3>Original</h3>
              <pre class="content-preview">{originalContent}</pre>
            </div>

            <div class="content-panel">
              <h3>Refined</h3>
              <pre class="content-preview refined">{refinedContent}</pre>
            </div>
          </div>

          {#if error}
            <div class="error-message">{error}</div>
          {/if}

          <div class="modal-actions">
            <button 
              class="btn btn-secondary" 
              on:click={handleCancel}
            >
              Cancel
            </button>
            <button 
              class="btn btn-primary" 
              on:click={handleAccept}
              disabled={!!error}
            >
              Accept Refined Version
            </button>
          </div>
        </div>
      {:else}
        <div class="initial-state">
          <div class="info-box">
            <p>
              This will use the LLM API configured in your settings to refine and improve your prompt.
            </p>
            <div class="content-preview-box">
              <h4>Content to refine:</h4>
              <pre>{originalContent || '(empty)'}</pre>
            </div>
          </div>

          {#if error}
            <div class="error-message">{error}</div>
          {/if}

          <div class="modal-actions">
            <button 
              class="btn btn-secondary" 
              on:click={handleCancel}
              disabled={loading}
            >
              Cancel
            </button>
            <button 
              class="btn btn-primary" 
              on:click={handleRefine}
              disabled={loading}
            >
              {loading ? 'Refining...' : 'Refine with AI'}
            </button>
          </div>
        </div>
      {/if}
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
    max-width: 900px;
    max-height: 85vh;
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

  .close-button:hover:not(:disabled) {
    background-color: #2d3748;
    color: #e2e8f0;
  }

  .close-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .modal-body {
    padding: 1.5rem;
    overflow-y: auto;
  }

  .loading-state {
    text-align: center;
    padding: 3rem 1rem;
  }

  .spinner {
    width: 3rem;
    height: 3rem;
    border: 4px solid #2d3748;
    border-top-color: #3182ce;
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin: 0 auto 1.5rem;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .loading-state p {
    margin: 0;
    font-size: 1.125rem;
    color: #e2e8f0;
  }

  .subtext {
    display: block;
    font-size: 0.875rem;
    color: #718096;
    margin-top: 0.5rem;
  }

  .initial-state,
  .result-state {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .info-box p {
    color: #e2e8f0;
    line-height: 1.6;
    margin: 0;
  }

  .content-preview-box {
    margin-top: 1rem;
  }

  .content-preview-box h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.875rem;
    color: #718096;
  }

  .content-preview-box pre {
    background-color: #0d1117;
    color: #e2e8f0;
    padding: 1rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    line-height: 1.6;
    overflow: auto;
    max-height: 200px;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    white-space: pre-wrap;
  }

  .content-comparison {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
  }

  .content-panel h3 {
    margin: 0 0 0.5rem 0;
    font-size: 0.875rem;
    color: #718096;
  }

  .content-panel .content-preview {
    background-color: #0d1117;
    padding: 1rem;
    border-radius: 0.25rem;
    overflow: auto;
    max-height: 300px;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
  }

  .content-panel .content-preview.refined {
    background-color: #0d3d1f;
    border: 1px solid #276749;
  }

  .error-message {
    background-color: #742a2a;
    color: #feb2b2;
    padding: 1rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    padding-top: 1rem;
    border-top: 1px solid #2d3748;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 0.25rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    border: none;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-secondary {
    background-color: #2d3748;
    color: #e2e8f0;
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: #4a5568;
  }

  .btn-primary {
    background-color: #3182ce;
    color: white;
  }

  .btn-primary:hover:not(:disabled) {
    background-color: #2c5282;
  }
</style>
