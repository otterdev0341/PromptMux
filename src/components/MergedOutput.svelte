<script lang="ts">
  import { mergedOutput, projectStore, saveProjectRefinement, deleteProjectRefinement, saveProjectErDiagram, saveProjectUmlDiagram, saveProjectFlowchart, saveProjectUserJourney, saveProjectUserStories } from '../stores/projectStore';
  import type { Refinement } from '../stores/projectStore';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy, onMount, tick } from 'svelte';
  import mermaid from 'mermaid';
  import { UndoHistory } from '../utils/UndoHistory';
  import { debounce } from '../utils/debounce';
  import DiagramWorkspace from './DiagramWorkspace.svelte';

  let outputContent = '';
  let refinedContent = '';
  let activeTab: 'raw' | 'refine' | 'er' | 'uml' | 'flowchart' | 'journey' = 'raw';
  let refineTab: 'generate' | 'history' = 'generate';
  let erTab: 'editor' | 'render' = 'render';
  let umlTab: 'editor' | 'render' = 'render';
  let flowchartTab: 'editor' | 'render' = 'render';
  let journeyTab: 'editor' | 'render' | 'showcase' = 'render';
  let isRefining = false;
  let refineError = '';
  let focused = false;
  
  // ER State
  let erCode = '';
  let isGeneratingEr = false;
  let erError = '';
  let mermaidContainer: HTMLElement;
  
  // UML State
  let umlCode = '';
  let isGeneratingUml = false;
  let umlError = '';
  let umlContainer: HTMLElement;
  
  // Zoom/Pan State (Shared or Duplicated? Duplicating for simplicity/independence for now)
  let zoomScale = 1;
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let startX = 0;
  let startY = 0;

  let umlZoomScale = 1;
  let umlPanX = 0;
  let umlPanY = 0;
  let isPanningUml = false;
  let umlStartX = 0;
  let umlStartY = 0;
  
  // Flowchart State
  let flowchartCode = '';
  let isGeneratingFlowchart = false;
  let flowchartError = '';
  let flowchartContainer: HTMLElement;
  
  let flowchartZoomScale = 1;
  let flowchartPanX = 0;
  let flowchartPanY = 0;
  let isPanningFlowchart = false;
  let flowchartStartX = 0;
  let flowchartStartY = 0;
  
  // Journey State
  let journeyCode = '';
  let userStoriesContent = '';
  let isGeneratingJourney = false;
  let isGeneratingStories = false;
  let journeyError = '';
  let storiesError = '';
  let journeyContainer: HTMLElement;
  
  let journeyZoomScale = 1;
  let journeyPanX = 0;
  let journeyPanY = 0;
  let isPanningJourney = false;
  let journeyStartX = 0;
  let journeyStartY = 0;
  
  // Track listeners to clean up
  let unlistenFunctions: (() => void)[] = [];

  // Undo History
  // Refinement History (Chat)
  $: erChatHistory = $projectStore?.history?.filter(h => h.kind === 'er') || [];
  $: umlChatHistory = $projectStore?.history?.filter(h => h.kind === 'uml') || [];
  $: flowchartChatHistory = $projectStore?.history?.filter(h => h.kind === 'flowchart') || [];
  $: journeyChatHistory = $projectStore?.history?.filter(h => h.kind === 'journey') || [];

  // Undo History
  const erUndoHistory = new UndoHistory<string>();
  const umlUndoHistory = new UndoHistory<string>();
  const flowchartUndoHistory = new UndoHistory<string>();
  const journeyUndoHistory = new UndoHistory<string>();

  const debouncedErPush = debounce((code: string) => erUndoHistory.push(code), 1000);
  const debouncedUmlPush = debounce((code: string) => umlUndoHistory.push(code), 1000);
  const debouncedFlowchartPush = debounce((code: string) => flowchartUndoHistory.push(code), 1000);
  const debouncedJourneyPush = debounce((code: string) => journeyUndoHistory.push(code), 1000);

  $: outputContent = $mergedOutput;

  onDestroy(() => {
    cleanupListeners();
  });

  // Initialize - mermaid init moved to DiagramWorkspace, check if needed here for other things?
  // We still use mermaid for Showcase or something? Maybe. But mostly DiagramWorkspace handles it.
  
  let lastProjectId: string | null = null;

  // Watch for project updates - only sync when switching projects
  $: if ($projectStore) {
    if ($projectStore.id !== lastProjectId) {
      lastProjectId = $projectStore.id;
      erCode = $projectStore.er_diagram || '';
      umlCode = $projectStore.uml_diagram || '';
      flowchartCode = $projectStore.flowchart || '';
      
      // Initialize Undo History
      erUndoHistory.clear();
      erUndoHistory.push(erCode);
      
      umlUndoHistory.clear();
      umlUndoHistory.push(umlCode);
      
      flowchartUndoHistory.clear();
      flowchartUndoHistory.push(flowchartCode);

      journeyCode = $projectStore.user_journey || '';
      userStoriesContent = $projectStore.user_stories || '';
      journeyUndoHistory.clear();
      journeyUndoHistory.push(journeyCode);
    }
  }

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
  
  function switchTab(tab: 'raw' | 'refine' | 'er' | 'uml' | 'flowchart' | 'journey') {
    activeTab = tab;
  }

  function switchRefineTab(tab: 'generate' | 'history') {
    refineTab = tab;
  }

  // Dead Code: switchErTab, switchUmlTab, switchFlowchartTab, switchJourneyTab
  // These are handled within DiagramWorkspace or not needed as we don't have sub-tabs in MergedOutput anymore (except Journey Showcase)
  


  function generateUserStoriesIfNeeded() {
      if (!userStoriesContent) {
          generateUserStories();
      }
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

  async function handleDeleteHistoryItem(id: string) {
    if (!confirm('Are you sure you want to delete this refinement?')) return;
    try {
      await deleteProjectRefinement(id);
    } catch (error) {
      console.error('Failed to delete refinement:', error);
    }
  }

  function handleLoadHistoryItem(content: string) {
    refinedContent = content;
    refineTab = 'generate';
  }

  function handleRestore(content: string) {
    // For merged output, "restore" mostly means copy to clipboard since we can't edit the source directly from here
    copyToClipboard(content);
  }

  async function generateUserStories() {
    if (!outputContent) return;
    
    // Reset state
    isGeneratingStories = true;
    storiesError = '';
    cleanupListeners();
    
    // Switch to showcase view to see it
    journeyTab = 'showcase';
    userStoriesContent = ''; 
    
    try {
      const unlistenChunk = await listen<string>('stories:chunk', (event) => {
        userStoriesContent += event.payload;
      });
      
      const unlistenDone = await listen('stories:done', () => {
        isGeneratingStories = false;
        cleanupListeners();
        saveProjectUserStories(userStoriesContent);
      });
      
      const unlistenError = await listen<string>('stories:error', (event) => {
        console.error('Stories stream error:', event.payload);
        storiesError = event.payload;
        isGeneratingStories = false;
        cleanupListeners();
      });
      
      unlistenFunctions.push(unlistenChunk, unlistenDone, unlistenError);
      
      await invoke('refine_user_stories_with_llm_stream', { content: refinedContent || outputContent });
    } catch (err) {
      console.error('Stories generation failed:', err);
      storiesError = String(err);
      isGeneratingStories = false;
      cleanupListeners();
    }
  }
  
  async function handleSaveStories() {
    if (!userStoriesContent) return;
    try {
      await saveProjectUserStories(userStoriesContent);
      
      const indicator = document.createElement('div');
      indicator.className = 'copy-indicator';
      indicator.textContent = 'Stories Saved!';
      document.body.appendChild(indicator);
      setTimeout(() => indicator.remove(), 1000);
    } catch (err) {
      console.error('Failed to save Stories:', err);
    }
  }

  // Dead code removed: handleMergedEditorKeydown, handleEditorBlur, handleEditorScroll, line counts


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
      <button 
        class="tab-btn {activeTab === 'er' ? 'active' : ''}" 
        on:click={() => switchTab('er')}
      >
        ER Diagram
      </button>
      <button 
        class="tab-btn {activeTab === 'uml' ? 'active' : ''}" 
        on:click={() => switchTab('uml')}
      >
        UML Diagram
      </button>
      <button 
        class="tab-btn {activeTab === 'flowchart' ? 'active' : ''}" 
        on:click={() => switchTab('flowchart')}
      >
        Flowchart
      </button>
      <button 
        class="tab-btn {activeTab === 'journey' ? 'active' : ''}" 
        on:click={() => switchTab('journey')}
      >
        Journey
      </button>
    </div>
    
    <div class="header-actions">
      {#if activeTab === 'raw'}
        <span class="keyboard-hint">Ctrl+c: Copy</span>
        <button class="copy-button" on:click={() => copyToClipboard(outputContent)}>
          📋 Copy
        </button>
      {:else if refinedContent && activeTab === 'refine'}
        <button class="copy-button" on:click={() => copyToClipboard(refinedContent)}>
          📋 Copy Refined
        </button>
      {:else if erCode && activeTab === 'er'}
        <button class="copy-button" on:click={() => copyToClipboard(erCode)}>
          📋 Copy Code
        </button>
      {:else if umlCode && activeTab === 'uml'}
        <button class="copy-button" on:click={() => copyToClipboard(umlCode)}>
          📋 Copy Code
        </button>
      {:else if flowchartCode && activeTab === 'flowchart'}
        <button class="copy-button" on:click={() => copyToClipboard(flowchartCode)}>
          📋 Copy Code
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
               <textarea class="output-content refined" bind:value={refinedContent}></textarea>
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
                  <div 
                    class="history-item clickable"
                    role="button"
                    tabindex="0"
                    on:click={() => handleLoadHistoryItem(item.refined_content)}
                    on:keydown={(e) => e.key === 'Enter' && handleLoadHistoryItem(item.refined_content)}
                  >
                    <div class="history-meta">
                      <span class="history-time">{formatDate(item.timestamp)}</span>
                      <div class="history-actions">
                        <button 
                          class="restore-btn" 
                          on:click|stopPropagation={() => handleRestore(item.refined_content)}
                          title="Copy to Clipboard"
                        >
                          📋
                        </button>
                        <button 
                          class="restore-btn delete" 
                          on:click|stopPropagation={() => handleDeleteHistoryItem(item.id)}
                          title="Delete"
                        >
                          🗑️
                        </button>
                      </div>
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
    {:else if activeTab === 'er'}
        <div class="workspace-wrapper">
             <DiagramWorkspace 
                type="er" 
                bind:code={erCode}
                history={erChatHistory}
                on:update={() => saveProjectErDiagram(erCode)}
            />
        </div>

    {:else if activeTab === 'uml'}
        <div class="workspace-wrapper">
             <DiagramWorkspace 
                type="uml" 
                bind:code={umlCode} 
                history={umlChatHistory}
                on:update={() => saveProjectUmlDiagram(umlCode)}
            />
        </div>
        
    {:else if activeTab === 'flowchart'}
        <div class="workspace-wrapper">
             <DiagramWorkspace 
                type="flowchart" 
                bind:code={flowchartCode} 
                history={flowchartChatHistory}
                on:update={() => saveProjectFlowchart(flowchartCode)}
            />
        </div>
        
    {:else if activeTab === 'journey'}
           <!-- Journey has a special 'Showcase' tab -->
           <div class="workspace-wrapper journey-wrapper">
                <div class="journey-controls">
                     <!-- External toggle for Showcase vs Diagram modes -->
                     <button 
                        class="toggle-btn {journeyTab === 'showcase' ? '' : 'active'}" 
                        on:click={() => journeyTab = 'render'}
                     >
                        Diagram
                     </button>
                     <button 
                        class="toggle-btn {journeyTab === 'showcase' ? 'active' : ''}" 
                        on:click={() => { journeyTab = 'showcase'; generateUserStoriesIfNeeded(); }}
                     >
                        Showcase
                     </button>
                </div>
                
                {#if journeyTab === 'showcase'}
                   <div class="showcase-view">
                       <!-- Keeps existing showcase logic -->
                       <div class="showcase-container">
                           {#if isGeneratingStories}
                               <div class="loading-overlay">
                                   <div class="spinner"></div>
                                   <p>Generating User Stories...</p>
                               </div>
                           {/if}
                           
                           {#if storiesError}
                               <div class="error-msg">{storiesError}</div>
                           {/if}
                           
                           <div class="stories-content">
                               {#if !userStoriesContent && !isGeneratingStories}
                                    <div class="empty-state">
                                        <p>No user stories generated yet.</p>
                                        <button class="action-btn" on:click={generateUserStories}>Generate Stories</button>
                                    </div>
                               {:else}
                                    <pre>{userStoriesContent}</pre>
                               {/if}
                           </div>
                       </div>
                   </div>
                {:else}
                    <DiagramWorkspace 
                        type="journey" 
                        bind:code={journeyCode} 
                        history={journeyChatHistory}
                        on:update={() => saveProjectUserJourney(journeyCode)}
                    />
                {/if}
           </div>
      {/if}
    </div>
  </div>


<style>
/* ... existing styles ... */
.workspace-wrapper {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
}

.journey-wrapper {
    position: relative;
}

.journey-controls {
    position: absolute;
    top: 0.5rem;
    right: 15rem; /* Adjust based on toolbar layout */
    z-index: 10;
    display: flex;
    gap: 0.5rem;
    background: #161b22;
    padding: 0.25rem;
    border-radius: 4px;
    border: 1px solid #30363d;
}

.toggle-btn {
    background: transparent;
    border: none;
    color: #8b949e;
    padding: 0.25rem 0.75rem;
    font-size: 0.8rem;
    cursor: pointer;
    border-radius: 4px;
}

.toggle-btn.active {
    background: #1f6feb;
    color: white;
}

.showcase-view {
    flex: 1;
    overflow: auto;
    padding: 1rem;
}
 

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
    resize: none;
    border: none;
  }

  .refine-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
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
    transition: all 0.2s;
  }
  
  .history-item.clickable {
    cursor: pointer;
  }

  .history-item.clickable:hover {
    border-color: #4a5568;
    background-color: #262c3a;
  }

  .history-meta {
    padding: 0.75rem 1rem;
    background-color: #2d3748;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #4a5568;
  }

  .history-actions {
    display: flex;
    gap: 0.5rem;
  }

  
  .history-time {
    color: #a0aec0;
    font-size: 0.875rem;
  }

  .restore-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 1rem;
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: background-color 0.2s;
  }

  .restore-btn:hover {
    background-color: #4a5568;
  }
  
  .restore-btn.delete:hover {
    background-color: #c53030;
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

  .er-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    background-color: #0d1117;
  }

  .er-editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .er-editor {
    flex: 1;
    background-color: #0d1117;
    color: #e2e8f0;
    border: none;
    padding: 1rem;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    resize: none;
    outline: none;
  }

  .er-render-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden; /* Changed from auto to hidden for custom pan/zoom */
    background-color: #0d1117;
    position: relative; /* Context for absolute controls */
  }
  
  .zoom-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: transform 0.1s ease-out;
    transform-origin: center;
  }
  
  .zoom-controls {
    position: absolute;
    bottom: 1rem;
    right: 1rem;
    display: flex;
    gap: 0.5rem;
    background: rgba(13, 17, 23, 0.8);
    padding: 0.5rem;
    border-radius: 0.5rem;
    border: 1px solid #30363d;
  }
  
  .zoom-btn {
    background: #21262d;
    border: 1px solid #30363d;
    color: #c9d1d9;
    width: 32px;
    height: 32px;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.2rem;
  }
  
  .zoom-btn:hover {
    background: #30363d;
    color: #58a6ff;
  }

  /* Mermaid styles adjustment */
  :global(.er-render-container svg) {
    max-width: none; /* Allow scaling */
    height: auto;
  }

  .generating-indicator {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #a0aec0;
    font-size: 0.875rem;
  }

  .spinner-small {
    width: 16px;
    height: 16px;
    border: 2px solid #2d3748;
    border-top-color: #63b3ed;
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }
  
  /* Line Numbers Styles */
  .editor-wrapper {
    flex: 1;
    display: flex;
    position: relative;
    overflow: hidden;
    background-color: #0d1117;
    height: 100%;
  }
  
  .line-numbers {
    padding: 1rem 0.5rem 1rem 0;
    text-align: right;
    background-color: #0d1117;
    color: #4a5568;
    border-right: 1px solid #2d3748;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    user-select: none;
    min-width: 2.5rem;
    overflow: hidden;
  }
  
  .line-number {
    display: block;
    padding-right: 0.5rem;
    color: #4a5568;
  }

  .empty-state-small {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #a0aec0;
    font-size: 0.875rem;
    padding: 2rem;
    border: 1px dashed #2d3748;
    border-radius: 0.5rem;
  }
</style>
