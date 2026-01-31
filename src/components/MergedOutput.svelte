<script lang="ts">
  import { mergedOutput, projectStore, saveProjectRefinement, saveProjectErDiagram, saveProjectUmlDiagram } from '../stores/projectStore';
  import type { Refinement } from '../stores/projectStore';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onDestroy, onMount } from 'svelte';
  import mermaid from 'mermaid';

  let outputContent = '';
  let refinedContent = '';
  let activeTab: 'raw' | 'refine' | 'er' | 'uml' = 'raw';
  let refineTab: 'generate' | 'history' = 'generate';
  let erTab: 'editor' | 'render' = 'render';
  let umlTab: 'editor' | 'render' = 'render';
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
  
  // Track listeners to clean up
  let unlistenFunctions: (() => void)[] = [];

  $: outputContent = $mergedOutput;

  onDestroy(() => {
    cleanupListeners();
  });

  // Initialize Mermaid
  onMount(() => {
    mermaid.initialize({ 
      startOnLoad: false,
      theme: 'dark',
      securityLevel: 'loose',
    });
    
    // Load existing diagram if available
    if ($projectStore) {
      if ($projectStore.er_diagram) {
        erCode = $projectStore.er_diagram;
      }
      if ($projectStore.uml_diagram) {
        umlCode = $projectStore.uml_diagram;
      }
    }
  });

  // Watch for project updates
  $: if ($projectStore) {
    if ($projectStore.er_diagram && !erCode) {
      erCode = $projectStore.er_diagram;
    }
    if ($projectStore.uml_diagram && !umlCode) {
      umlCode = $projectStore.uml_diagram;
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
  
  function switchTab(tab: 'raw' | 'refine' | 'er' | 'uml') {
    activeTab = tab;
    if (tab === 'er' && erTab === 'render') {
      setTimeout(renderDiagram, 100);
    } else if (tab === 'uml' && umlTab === 'render') {
      setTimeout(renderUmlDiagram, 100);
    }
  }

  function switchRefineTab(tab: 'generate' | 'history') {
    refineTab = tab;
  }

  function switchErTab(tab: 'editor' | 'render') {
    erTab = tab;
    if (tab === 'render') {
      setTimeout(renderDiagram, 100);
    }
  }

  function switchUmlTab(tab: 'editor' | 'render') {
    umlTab = tab;
    if (tab === 'render') {
      setTimeout(renderUmlDiagram, 100);
    }
  }
  
  // Zoom Handler
  function handleWheel(e: WheelEvent) {
    if (erTab !== 'render') return;
    e.preventDefault();
    
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.1, Math.min(5, zoomScale * delta));
    zoomScale = newScale;
  }
  
  // Pan Handlers
  function handleMouseDown(e: MouseEvent) {
    if (erTab !== 'render') return;
    isPanning = true;
    startX = e.clientX - panX;
    startY = e.clientY - panY;
  }
  
  function handleMouseMove(e: MouseEvent) {
    if (!isPanning) return;
    e.preventDefault();
    panX = e.clientX - startX;
    panY = e.clientY - startY;
  }
  
  function handleMouseUp() {
    isPanning = false;
  }
  
  function handleResetView() {
    zoomScale = 1;
    panX = 0;
    panY = 0;
  }

  function handleZoomIn() {
    zoomScale = Math.min(5, zoomScale * 1.2);
  }

  function handleZoomOut() {
    zoomScale = Math.max(0.1, zoomScale * 0.8);
  }

  async function renderDiagram() {
    if (!erCode || !mermaidContainer) return;
    
    // Reset view when re-rendering
    handleResetView();
    
    try {
      mermaidContainer.innerHTML = '';
      const { svg } = await mermaid.render('mermaid-svg-' + Date.now(), erCode);
      mermaidContainer.innerHTML = svg;
    } catch (error) {
      console.error('Mermaid render error:', error);
      mermaidContainer.innerHTML = `<div class="error-msg">Failed to render diagram: ${error}</div>`;
    }
  }

  async function handleGenerateEr() {
    if (!outputContent) return;
    
    // Reset state
    isGeneratingEr = true;
    erError = '';
    cleanupListeners();
    
    // Switch to editor view to see it streaming in
    erTab = 'editor';
    erCode = ''; 
    
    try {
      const unlistenChunk = await listen<string>('er:chunk', (event) => {
        erCode += event.payload;
      });
      
      const unlistenDone = await listen('er:done', () => {
        isGeneratingEr = false;
        cleanupListeners();
        saveProjectErDiagram(erCode);
      });
      
      const unlistenError = await listen<string>('er:error', (event) => {
        console.error('ER stream error:', event.payload);
        erError = event.payload;
        isGeneratingEr = false;
        cleanupListeners();
      });
      
      unlistenFunctions.push(unlistenChunk, unlistenDone, unlistenError);
      
      await invoke('refine_er_diagram_with_llm_stream', { content: refinedContent || outputContent });
    } catch (err) {
      console.error('ER generation failed:', err);
      erError = String(err);
      isGeneratingEr = false;
      cleanupListeners();
    }
  }

  async function handleSaveEr() {
    if (!erCode) return;
    try {
      await saveProjectErDiagram(erCode);
      
      // Feedback
      const indicator = document.createElement('div');
      indicator.className = 'copy-indicator';
      indicator.textContent = 'ER Diagram Saved!';
      document.body.appendChild(indicator);
      setTimeout(() => indicator.remove(), 1000);
    } catch (err) {
      console.error('Failed to save ER diagram:', err);
    }
  }

  // UML Handlers

  function handleUmlWheel(e: WheelEvent) {
    if (umlTab !== 'render') return;
    e.preventDefault();
    
    const delta = e.deltaY > 0 ? 0.9 : 1.1;
    const newScale = Math.max(0.1, Math.min(5, umlZoomScale * delta));
    umlZoomScale = newScale;
  }
  
  function handleUmlMouseDown(e: MouseEvent) {
    if (umlTab !== 'render') return;
    isPanningUml = true;
    umlStartX = e.clientX - umlPanX;
    umlStartY = e.clientY - umlPanY;
  }
  
  function handleUmlMouseMove(e: MouseEvent) {
    if (!isPanningUml) return;
    e.preventDefault();
    umlPanX = e.clientX - umlStartX;
    umlPanY = e.clientY - umlStartY;
  }
  
  function handleUmlMouseUp() {
    isPanningUml = false;
  }
  
  function handleUmlResetView() {
    umlZoomScale = 1;
    umlPanX = 0;
    umlPanY = 0;
  }

  function handleUmlZoomIn() {
    umlZoomScale = Math.min(5, umlZoomScale * 1.2);
  }

  function handleUmlZoomOut() {
    umlZoomScale = Math.max(0.1, umlZoomScale * 0.8);
  }

  async function renderUmlDiagram() {
    if (!umlCode || !umlContainer) return;
    
    // Reset view when re-rendering
    handleUmlResetView();
    
    try {
      umlContainer.innerHTML = '';
      const { svg } = await mermaid.render('mermaid-uml-svg-' + Date.now(), umlCode);
      umlContainer.innerHTML = svg;
    } catch (error) {
      console.error('Mermaid render error:', error);
      umlContainer.innerHTML = `<div class="error-msg">Failed to render diagram: ${error}</div>`;
    }
  }

  async function handleGenerateUml() {
    if (!outputContent) return;
    
    // Reset state
    isGeneratingUml = true;
    umlError = '';
    cleanupListeners();
    
    // Switch to editor view to see it streaming in
    umlTab = 'editor';
    umlCode = ''; 
    
    try {
      const unlistenChunk = await listen<string>('uml:chunk', (event) => {
        umlCode += event.payload;
      });
      
      const unlistenDone = await listen('uml:done', () => {
        isGeneratingUml = false;
        cleanupListeners();
        saveProjectUmlDiagram(umlCode);
      });
      
      const unlistenError = await listen<string>('uml:error', (event) => {
        console.error('UML stream error:', event.payload);
        umlError = event.payload;
        isGeneratingUml = false;
        cleanupListeners();
      });
      
      unlistenFunctions.push(unlistenChunk, unlistenDone, unlistenError);
      
      await invoke('refine_uml_diagram_with_llm_stream', { content: refinedContent || outputContent });
    } catch (err) {
      console.error('UML generation failed:', err);
      umlError = String(err);
      isGeneratingUml = false;
      cleanupListeners();
    }
  }

  async function handleSaveUml() {
    if (!umlCode) return;
    try {
      await saveProjectUmlDiagram(umlCode);
      
      // Feedback
      const indicator = document.createElement('div');
      indicator.className = 'copy-indicator';
      indicator.textContent = 'UML Diagram Saved!';
      document.body.appendChild(indicator);
      setTimeout(() => indicator.remove(), 1000);
    } catch (err) {
      console.error('Failed to save UML diagram:', err);
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
    {:else if activeTab === 'er'}
      <div class="refine-wrapper">
        <div class="refine-tabs">
          <button 
            class="refine-tab-btn {erTab === 'editor' ? 'active' : ''}" 
            on:click={() => switchErTab('editor')}
          >
            Editor
          </button>
          <button 
            class="refine-tab-btn {erTab === 'render' ? 'active' : ''}" 
            on:click={() => switchErTab('render')}
          >
            Render
          </button>
        </div>

        <div class="er-container">
           {#if erTab === 'editor'}
              <div class="er-editor-container">
                 <textarea 
                   class="er-editor" 
                   bind:value={erCode}
                   placeholder="Mermaid ER diagram code will appear here..."
                 ></textarea>
                 
                 <div class="refine-actions">
                   {#if isGeneratingEr}
                     <div class="generating-indicator">
                        <div class="spinner-small"></div> Generating...
                     </div>
                   {:else}
                    <button class="action-btn secondary" on:click={handleGenerateEr}>
                      {erCode ? 'Re-generate from Refined' : 'Generate from Refined'}
                    </button>
                    <button class="action-btn primary" on:click={handleSaveEr} disabled={!erCode}>
                      Save
                    </button>
                   {/if}
                 </div>
                 {#if erError}
                    <div class="error-msg">{erError}</div>
                 {/if}
              </div>
           {:else}
              <div 
                class="er-render-container" 
                role="region"
                aria-label="ER Diagram Render View"
                on:wheel={handleWheel}
                on:mousedown={handleMouseDown}
                on:mousemove={handleMouseMove}
                on:mouseup={handleMouseUp}
                on:mouseleave={handleMouseUp}
                style="cursor: {isPanning ? 'grabbing' : 'grab'};"
              >
                <div 
                  class="zoom-container"
                  style="transform: translate({panX}px, {panY}px) scale({zoomScale});"
                  bind:this={mermaidContainer}
                >
                  <!-- Mermaid diagram rendered here -->
                  {#if !erCode}
                    <div class="empty-state-small">
                       <p>Generate a diagram first.</p>
                    </div>
                  {/if}
                </div>
                
                <div class="zoom-controls">
                  <button class="zoom-btn" on:click={handleZoomIn} title="Zoom In">+</button>
                  <button class="zoom-btn" on:click={handleZoomOut} title="Zoom Out">-</button>
                  <button class="zoom-btn" on:click={handleResetView} title="Reset View">Reset</button>
                </div>
              </div>
           {/if}
        </div>
      </div>
    {:else if activeTab === 'uml'}
      <div class="refine-wrapper">
        <div class="refine-tabs">
          <button 
            class="refine-tab-btn {umlTab === 'editor' ? 'active' : ''}" 
            on:click={() => switchUmlTab('editor')}
          >
            Editor
          </button>
          <button 
            class="refine-tab-btn {umlTab === 'render' ? 'active' : ''}" 
            on:click={() => switchUmlTab('render')}
          >
            Render
          </button>
        </div>

        <div class="er-container">
           {#if umlTab === 'editor'}
              <div class="er-editor-container">
                 <textarea 
                   class="er-editor" 
                   bind:value={umlCode}
                   placeholder="Mermaid Class diagram code will appear here..."
                 ></textarea>
                 
                 <div class="refine-actions">
                   {#if isGeneratingUml}
                     <div class="generating-indicator">
                        <div class="spinner-small"></div> Generating...
                     </div>
                   {:else}
                    <button class="action-btn secondary" on:click={handleGenerateUml}>
                      {umlCode ? 'Re-generate from Refined' : 'Generate from Refined'}
                    </button>
                    <button class="action-btn primary" on:click={handleSaveUml} disabled={!umlCode}>
                      Save
                    </button>
                   {/if}
                 </div>
                 {#if umlError}
                    <div class="error-msg">{umlError}</div>
                 {/if}
              </div>
           {:else}
              <div 
                class="er-render-container" 
                role="region"
                aria-label="UML Diagram Render View"
                on:wheel={handleUmlWheel}
                on:mousedown={handleUmlMouseDown}
                on:mousemove={handleUmlMouseMove}
                on:mouseup={handleUmlMouseUp}
                on:mouseleave={handleUmlMouseUp}
                style="cursor: {isPanningUml ? 'grabbing' : 'grab'};"
              >
                <div 
                  class="zoom-container"
                  style="transform: translate({umlPanX}px, {umlPanY}px) scale({umlZoomScale});"
                  bind:this={umlContainer}
                >
                  <!-- Mermaid diagram rendered here -->
                  {#if !umlCode}
                    <div class="empty-state-small">
                       <p>Generate a diagram first.</p>
                    </div>
                  {/if}
                </div>
                
                <div class="zoom-controls">
                  <button class="zoom-btn" on:click={handleUmlZoomIn} title="Zoom In">+</button>
                  <button class="zoom-btn" on:click={handleUmlZoomOut} title="Zoom Out">-</button>
                  <button class="zoom-btn" on:click={handleUmlResetView} title="Reset View">Reset</button>
                </div>
              </div>
           {/if}
        </div>
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
