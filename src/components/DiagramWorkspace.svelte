<script lang="ts">
  import { onMount, tick, createEventDispatcher } from 'svelte';
  import mermaid from 'mermaid';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import type { Refinement } from '../stores/projectStore';
  import { saveProjectRefinement } from '../stores/projectStore';
  import { marked } from 'marked';

  export let type: 'er' | 'uml' | 'flowchart' | 'journey';
  export let code: string = '';
  export let history: Refinement[] = [];
  
  const dispatch = createEventDispatcher();

  let activeTab: 'editor' | 'render' | 'chat' = 'render';
  let chatMode: 'edit' | 'ask' = 'edit';

  let failedMessage: { content: string, mode: 'edit' | 'ask', error: string } | null = null;
  let chatInput = '';
  let isChatGenerating = false;
  let error = '';
  
  // DOM Elements
  let container: HTMLElement;
  let chatContainer: HTMLElement;
  let previewContainer: HTMLElement;
  
  // Render State
  let scale = 1;
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let startX = 0;
  let startY = 0;

  // Mermaid Init
  onMount(() => {
    mermaid.initialize({ 
      startOnLoad: false,
      theme: 'dark',
      securityLevel: 'loose'
    });
    // Initial render if code exists
    if (code) renderDiagram();
  });

  // Watch for external code changes (prop update)
  $: if (code && activeTab === 'render') {
      renderDiagram();
  }

  // --- Rendering Logic ---
  async function renderDiagram() {
    await tick();
    if (container && code) {
      try {
        error = '';
        container.innerHTML = '';
        const { svg } = await mermaid.render(`mermaid-${type}-${Date.now()}`, code);
        container.innerHTML = svg;
      } catch (e) {
        console.error('Mermaid render error:', e);
        error = String(e);
      }
    }
  }

  async function renderPreview() {
    await tick();
    if (previewContainer && code) {
        try {
           previewContainer.innerHTML = '';
           const { svg } = await mermaid.render(`mermaid-preview-${Date.now()}`, code);
           previewContainer.innerHTML = svg;
        } catch (e) {
            console.error('Mermaid preview error:', e);
            previewContainer.innerHTML = `<div class="error-msg small">${String(e)}</div>`;
        }
    }
  }

  // --- Zoom/Pan Logic ---
  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey) {
      e.preventDefault();
      const delta = e.deltaY > 0 ? 0.9 : 1.1;
      scale = Math.max(0.1, Math.min(5, scale * delta));
    }
  }
  
  function handleMouseDown(e: MouseEvent) {
    isPanning = true;
    startX = e.clientX - panX;
    startY = e.clientY - panY;
  }
  
  function handleMouseMove(e: MouseEvent) {
    if (!isPanning) return;
    panX = e.clientX - startX;
    panY = e.clientY - startY;
  }
  
  function handleMouseUp() {
    isPanning = false;
  }
  
  function handleZoomIn() { scale = Math.min(5, scale * 1.2); }
  function handleZoomOut() { scale = Math.max(0.1, scale / 1.2); }
  function handleResetView() { scale = 1; panX = 0; panY = 0; }

  // --- Chat Logic ---
  function scrollToBottom() {
      if (chatContainer) {
          setTimeout(() => {
              chatContainer.scrollTop = chatContainer.scrollHeight;
          }, 100);
      }
  }

  async function restoreHistoryItem(item: Refinement) {
    if (!item.mode || item.mode === 'edit') {
        code = item.refined_content;
        dispatch('update', code); // Notify parent
        activeTab = 'render';
        await renderDiagram();
    }
  }

  async function handleSendMessage() {
      if (!chatInput.trim() || isChatGenerating) return;
      
      const instruction = chatInput;
      const currentMode = chatMode; // Capture current mode
      chatInput = '';
      failedMessage = null; // Clear any previous failure
      isChatGenerating = true;
      let streamedResponse = '';
      
      try {
          let command = '';
          let eventPrefix = '';
          
          if (chatMode === 'edit') {
              switch (type) {
                case 'er': command = 'edit_er_diagram_with_llm_stream'; eventPrefix = 'er'; break;
                case 'uml': command = 'edit_uml_diagram_with_llm_stream'; eventPrefix = 'uml'; break;
                case 'flowchart': command = 'edit_flowchart_with_llm_stream'; eventPrefix = 'flowchart'; break;
                case 'journey': command = 'edit_user_journey_with_llm_stream'; eventPrefix = 'journey'; break;
              }
          } else {
              command = 'ask_llm_about_diagram_stream';
              eventPrefix = 'ask';
          }
          
          // Unique event prefix for this instance? 
          // Ideally we should generate unique events, but for now we rely on the global ones.
          // Note: If multiple workspaces are open, they might conflict. 
          // But usually only one is active at a time in the tabs.
          
          const unlistenChunk = await listen<string>(`${eventPrefix}:chunk`, (event) => {
             streamedResponse += event.payload;
          });
          
          const unlistenDone = await listen<void>(`${eventPrefix}:done`, async () => {
              unlistenChunk();
              unlistenDone();
              
              if (chatMode === 'edit') {
                  code = streamedResponse;
                  dispatch('update', code);
                  if (activeTab === 'render') {
                      await renderDiagram();
                  } else if (activeTab === 'chat') {
                      await renderPreview();
                  }
              }
              
              await addToHistory(instruction, streamedResponse, chatMode);
              isChatGenerating = false;
              scrollToBottom();
          });
          
           if (chatMode === 'edit') {
                await invoke(command, { currentDiagram: code, instruction });
           } else {
                await invoke(command, { currentDiagram: code, question: instruction, diagramType: type });
           }
           
      } catch (e: any) {
          console.error(e);
          isChatGenerating = false;
          
          let rawMsg = String(e);
          // Attempt to extract message from error object if possible
          if (e && e.message) rawMsg = e.message;
          
          if (rawMsg.includes('RESOURCE_EXHAUSTED') || rawMsg.includes('429')) {
              error = "⚠️ Quota Exceeded: Free tier limit reached. ";
              
              // Extract retry time if available
              const match = rawMsg.match(/retry in ([0-9.]+)s/);
              if (match) {
                  error += `Please try again in ${Math.round(parseFloat(match[1]))} seconds.`;
              } else {
                  error += "Please wait a moment before trying again.";
              }
              
              failedMessage = {
                  content: instruction,
                  mode: currentMode,
                  error: error
              };
              // Clear global error so it doesn't show the banner at top, we show it in bubble
              error = ''; 
          } else {
              error = "Error: " + rawMsg;
              failedMessage = {
                  content: instruction,
                  mode: currentMode,
                  error: rawMsg
              };
               error = '';
          }
      }
  }
  
  async function addToHistory(prompt: string, result: string, mode: 'edit' | 'ask') {
      const refinement: Refinement = {
          id: Math.random().toString(36).substring(2) + Date.now().toString(36),
          original_content: prompt,
          refined_content: result,
          timestamp: new Date().toISOString(),
          kind: type,
          mode: mode
      };
      
      await saveProjectRefinement(refinement);
      history = [...history, refinement];
      // Note: We are mutating history locally, but parent binding should handle it if 'bind:history' is used properly.
  }

</script>

<div class="workspace">
    <div class="toolbar">
        <div class="tabs">
            <button class:active={activeTab === 'render'} on:click={() => { activeTab = 'render'; error = ''; renderDiagram(); }}>Render</button>
            <button class:active={activeTab === 'editor'} on:click={() => { activeTab = 'editor'; error = ''; }}>Editor</button>
            <button class:active={activeTab === 'chat'} on:click={() => { activeTab = 'chat'; error = ''; renderPreview(); scrollToBottom(); }}>Chat & Refine</button>
        </div>
        {#if activeTab === 'render'}
            <div class="zoom-controls">
                <button on:click={handleZoomIn} title="Zoom In">+</button>
                <button on:click={handleZoomOut} title="Zoom Out">-</button>
                <button on:click={handleResetView} title="Reset View">Reset</button>
            </div>
        {/if}
    </div>

    <div class="content-area">
        {#if activeTab === 'editor'}
            <div class="editor-view">
                 <textarea bind:value={code} on:input={() => dispatch('update', code)} placeholder="Mermaid code..."></textarea>
            </div>
        
        {:else if activeTab === 'render'}
            <div class="render-view">
                {#if error}
                    <div class="error-msg">{error}</div>
                {:else}
                    <div 
                        class="diagram-container"
                        on:wheel={handleWheel}
                        on:mousedown={handleMouseDown}
                        on:mousemove={handleMouseMove}
                        on:mouseup={handleMouseUp}
                        on:mouseleave={handleMouseUp}
                    >
                        <div 
                            class="zoom-content"
                            style="transform: translate({panX}px, {panY}px) scale({scale});"
                            bind:this={container}
                        ></div>
                    </div>
                {/if}
            </div>

        {:else if activeTab === 'chat'}
            <div class="chat-view">
                 <!-- Split View for Chat: Preview on Left, Chat on Right -->
                 <div class="chat-preview">
                     {#if code}
                         <!-- Interactive Preview -->
                         <div 
                            class="diagram-container"
                            on:wheel={handleWheel}
                            on:mousedown={handleMouseDown}
                            on:mousemove={handleMouseMove}
                            on:mouseup={handleMouseUp}
                            on:mouseleave={handleMouseUp}
                         >
                             <div 
                                class="zoom-content" 
                                style="transform: translate({panX}px, {panY}px) scale({scale});"
                             >
                                 <div class="preview-container" bind:this={previewContainer}>
                                     <div class="placeholder">Generating preview...</div> 
                                 </div>
                             </div>
                         </div>
                     {:else}
                        <div class="placeholder">No diagram code</div>
                     {/if}
                     
                     <!-- Overlay Zoom Controls for Chat -->
                     <div class="zoom-controls overlay">
                        <button on:click={handleZoomIn} title="Zoom In">+</button>
                        <button on:click={handleZoomOut} title="Zoom Out">-</button>
                        <button on:click={handleResetView} title="Reset View">Reset</button>
                    </div>
                 </div>
                 
                 <div class="chat-interface">
                     <div class="chat-list" bind:this={chatContainer}>
                        {#if error}
                            <div class="chat-bubble error">
                                <div class="bubble-content">{error}</div>
                            </div>
                        {/if}
                        {#each history as item}
                             <div class="chat-bubble user">
                                <div class="bubble-content">{item.original_content}</div>
                            </div>
                            <div class="chat-bubble ai {item.mode === 'ask' ? 'ask-mode' : ''}" on:click={() => restoreHistoryItem(item)}>
                                {#if item.mode === 'ask'}
                                {@html marked.parse(item.refined_content)}
                                {:else}
                                    <div class="bubble-content">Updated Diagram</div>
                                {/if}
                                <small>
                                    {#if item.mode === 'ask'}AI (Answer){:else}AI (Edit){/if} • {new Date(item.timestamp).toLocaleTimeString()}
                                </small>
                            </div>
                        {/each}
                        
                        {#if failedMessage}
                             <div class="chat-bubble user failed">
                                <div class="bubble-content">{failedMessage.content}</div>
                            </div>
                            <div class="chat-bubble error-retry">
                                <div class="error-content">
                                    <span>{failedMessage.error}</span>
                                    <button class="retry-btn" on:click={() => {
                                        chatInput = failedMessage?.content || '';
                                        chatMode = failedMessage?.mode || 'edit';
                                        handleSendMessage();
                                    }}>
                                        ↻ Retry
                                    </button>
                                </div>
                            </div>
                        {/if}
                        {#if isChatGenerating}
                             <div class="chat-bubble ai generating">...</div>
                        {/if}
                     </div>
                     
                     <div class="chat-input-area">
                        <div class="mode-toggle">
                            <button class:active={chatMode === 'edit'} on:click={() => chatMode = 'edit'}>Refine</button>
                            <button class:active={chatMode === 'ask'} on:click={() => chatMode = 'ask'}>Ask</button>
                        </div>
                        <textarea 
                            bind:value={chatInput} 
                            placeholder={chatMode === 'edit' ? "Change diagram..." : "Ask question..."}
                            on:keydown={(e) => { if(e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSendMessage(); } }}
                        ></textarea>
                        <button class="send-btn" on:click={handleSendMessage} disabled={isChatGenerating}>Send</button>
                     </div>
                 </div>
            </div>
        {/if}
    </div>
</div>

<style>
    .workspace {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: #0d1117;
        color: #c9d1d9;
    }
    
    .toolbar {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 0.5rem 1rem;
        background: #161b22;
        border-bottom: 1px solid #30363d;
        flex-shrink: 0;
    }
    
    .tabs {
        display: flex;
        gap: 2px;
    }
    
    .tabs button {
        background: transparent;
        border: none;
        color: #8b949e;
        padding: 0.5rem 1rem;
        cursor: pointer;
        border-bottom: 2px solid transparent;
        font-weight: 500;
    }
    
    .tabs button:hover {
        color: #c9d1d9;
    }
    
    .tabs button.active {
        color: #58a6ff;
        border-bottom-color: #58a6ff;
    }
    
    .zoom-controls button {
        background: #21262d;
        color: #c9d1d9;
        border: 1px solid #30363d;
        padding: 0.25rem 0.5rem;
        border-radius: 4px;
        cursor: pointer;
        margin-left: 0.5rem;
    }
    
    .zoom-controls.overlay {
        position: absolute;
        bottom: 1rem;
        left: 1rem;
        background: rgba(22, 27, 34, 0.8);
        padding: 0.5rem;
        border-radius: 6px;
        border: 1px solid #30363d;
        z-index: 10;
        display: flex;
    }

    .content-area {
        flex: 1;
        overflow: hidden;
        position: relative;
    }
    
    .editor-view, .render-view, .chat-view {
        width: 100%;
        height: 100%;
        overflow: hidden;
    }
    
    .editor-view textarea {
        width: 100%;
        height: 100%;
        background: #0d1117;
        color: #c9d1d9;
        border: none;
        padding: 1rem;
        font-family: monospace;
        resize: none;
        outline: none;
    }
    
    .render-view {
        overflow: hidden;
        cursor: grab;
    }
    
    .render-view:active {
        cursor: grabbing;
    }
    
    .diagram-container {
        width: 100%;
        height: 100%;
        overflow: hidden;
        cursor: grab;
    }
    
    .diagram-container:active {
        cursor: grabbing;
    }
    
    .zoom-content {
        transform-origin: 0 0;
        width: 100%;
        height: 100%;
    }
    
    .error-msg {
        color: #f85149;
        padding: 1rem;
    }
    
    /* Chat View */
    .chat-view {
        display: flex;
    }
    
    .chat-preview {
        flex: 1; /* Left side preview */
        border-right: 1px solid #30363d;
        position: relative;
        overflow: hidden;
        background: #0d1117;
        /* Ensure cursor grab works here if clicking background */
    }
    
    .preview-container {
        /* Simple preview styling */
        /* Content handled by zoom-content wrapper */
    }
    
    .chat-interface {
        flex: 1; /* Right side chat */
        display: flex;
        flex-direction: column;
        max-width: 400px;
        background: #161b22;
        border-left: 1px solid #30363d;
    }
    
    .chat-list {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }
    
    .chat-bubble {
        padding: 0.8rem;
        border-radius: 6px;
        max-width: 90%;
        font-size: 0.9rem;
    }
    
    .chat-bubble.user {
        align-self: flex-end;
        background: #1f6feb;
        color: white;
    }
    
    .chat-bubble.ai {
        align-self: flex-start;
        background: #21262d;
        border: 1px solid #30363d;
        cursor: pointer;
    }
    
    .chat-bubble.error {
        align-self: center;
        background: #3d1616;
        border: 1px solid #f85149;
        color: #f85149;
        width: 90%;
        text-align: center;
    }
    
    .chat-bubble.user.failed {
        opacity: 0.7;
        border: 1px dashed #f85149;
    }

    .chat-bubble.error-retry {
        align-self: flex-start; /* Or center? */
        background: #3d1616;
        border: 1px solid #f85149;
        color: #f85149;
        max-width: 90%;
        padding: 0.5rem;
    }

    .error-content {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        align-items: center;
    }

    .retry-btn {
        background: #238636;
        color: white;
        border: none;
        padding: 0.3rem 0.8rem;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.85rem;
        display: flex;
        align-items: center;
        gap: 0.3rem;
    }
    
    .retry-btn:hover {
        background: #2ea043;
    }
    
    .chat-bubble.ai.ask-mode {
        border-left: 3px solid #3fb950;
    }
    
    .chat-bubble small {
        display: block;
        margin-top: 0.5rem;
        font-size: 0.75rem;
        color: #8b949e;
    }
    
    .chat-input-area {
        padding: 1rem;
        border-top: 1px solid #30363d;
    }
    
    .mode-toggle {
        display: flex;
        gap: 0.5rem;
        margin-bottom: 0.5rem;
    }
    
    .mode-toggle button {
        flex: 1;
        background: #21262d;
        border: 1px solid #30363d;
        color: #8b949e;
        padding: 0.25rem;
        border-radius: 4px;
        cursor: pointer;
        font-size: 0.8rem;
    }
    
    .mode-toggle button.active {
        background: #1f6feb;
        color: white;
        border-color: #1f6feb;
    }
    
    .mode-toggle button.active:nth-child(2) {
        background: #238636;
        border-color: #238636;
    }
    
    textarea {
        width: 100%;
        height: 60px;
        background: #0d1117;
        border: 1px solid #30363d;
        color: #c9d1d9;
        border-radius: 4px;
        padding: 0.5rem;
        resize: vertical;
        margin-bottom: 0.5rem;
    }
    
    .send-btn {
        width: 100%;
        background: #238636;
        color: white;
        border: none;
        padding: 0.5rem;
        border-radius: 4px;
        cursor: pointer;
        font-weight: 600;
    }
    
    .send-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
