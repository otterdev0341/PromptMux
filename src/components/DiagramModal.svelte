<script lang="ts">
  import { onMount, tick, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import mermaid from 'mermaid';
  import { projectStore, activeTopicId, activeSectionId, loadProject } from '../stores/projectStore';
  import { saveProjectRefinement, type Refinement } from '../stores/projectStore';
  import { get } from 'svelte/store';

  export let onClose: () => void;
  
  let step: 'options' | 'generating' | 'result' | 'chat' = 'options';
  let targetType: 'er' | 'uml' | 'flowchart' | 'journey' = 'er';
  let error = '';
  let diagramCode = '';
  
  let container: HTMLDivElement;
  let scale = 1;
  let isPanning = false;
  let startX = 0;
  let startY = 0;
  let panX = 0;
  let panY = 0;
  
  let contextName = '';
  let contentToRefine = '';
  
  // Chat / Result Tab State
  let resultTab: 'editor' | 'render' = 'render';
  let chatHistory: Refinement[] = [];
  let chatInput = '';
  let isChatGenerating = false;
  let chatContainer: HTMLDivElement;
  let chatMode: 'edit' | 'ask' = 'edit';

  onMount(() => {
    // Determine context
    const project = $projectStore;
    if (!project) return;
    
    if ($activeTopicId) {
      // Find topic
      for (const section of project.sections) {
        const topic = section.topics.find(t => t.id === $activeTopicId);
        if (topic) {
          contextName = `Topic: ${topic.name}`;
          contentToRefine = topic.content;
          break;
        }
      }
    } else if ($activeSectionId) {
      // Find section
      const section = project.sections.find(s => s.id === $activeSectionId);
      if (section) {
        contextName = `Section: ${section.name}`;
        // Combine all topics
        contentToRefine = section.topics.map(t => `Topic: ${t.name}\n${t.content}`).join('\n\n');
      }
    }
    
    // Initialize mermaid
    mermaid.initialize({ 
      startOnLoad: false,
      theme: 'dark',
      securityLevel: 'loose'
    });
    
    // Add keydown listener for shortcuts
    window.addEventListener('keydown', handleKeydown);
    
    return () => {
        window.removeEventListener('keydown', handleKeydown);
    };
  });
  
  function handleKeydown(e: KeyboardEvent) {
    if (step === 'options') {
        switch(e.key.toLowerCase()) {
            case 'e': generateDiagram('er'); break;
            case 'u': generateDiagram('uml'); break;
            case 'f': generateDiagram('flowchart'); break;
            case 'j': generateDiagram('journey'); break;
            case 'escape': close(); break;
        }
        return;
    }
    
    if (e.key === 'Escape') {
        close();
    }
  }
  
  function loadHistory(type: string) {
      const project = $projectStore;
      if (project && project.history) {
          chatHistory = project.history.filter(h => h.kind === type);
          /* 
             We only want to load the diagram from history IF the last item was an 'edit'.
             However, 'ask' items don't change the diagram.
             So we should search backwards for the last 'edit' item to set the diagram code?
             Or just rely on what we have if we are continuing a session?
             If we are just opening the modal, we need to find the latest state.
          */
          if (chatHistory.length > 0) {
              // Find last edit to restore diagram code
              const lastEdit = [...chatHistory].reverse().find(h => !h.mode || h.mode === 'edit');
              if (lastEdit) {
                  diagramCode = lastEdit.refined_content;
              } else {
                  // If all history items are 'ask' (unlikely since we start with gen), 
                  // or if legacy items, default to last.
                  const last = chatHistory[chatHistory.length - 1];
                  diagramCode = last.refined_content;
              }
              
              step = 'chat';
              renderDiagram();
              scrollToBottom();
          }
      } else {
          chatHistory = [];
      }
  }
  
  function scrollToBottom() {
      if (chatContainer) {
          setTimeout(() => {
              chatContainer.scrollTop = chatContainer.scrollHeight;
          }, 100);
      }
  }

  async function generateDiagram(type: 'er' | 'uml' | 'flowchart' | 'journey') {
    targetType = type;
    
    loadHistory(type);
    if (chatHistory.length > 0) {
        step = 'chat';
        return;
    }

    step = 'generating';
    error = '';
    diagramCode = '';
    resultTab = 'render'; 
    
    if (!contentToRefine) {
        error = 'No content found to generate diagram.';
        step = 'chat'; 
        return;
    }

    try {
      let command = '';
      let eventPrefix = '';
      
      switch (type) {
        case 'er': 
            command = 'refine_er_diagram_with_llm_stream'; 
            eventPrefix = 'er';
            break;
        case 'uml': 
            command = 'refine_uml_diagram_with_llm_stream'; 
            eventPrefix = 'uml';
            break;
        case 'flowchart': 
            command = 'refine_flowchart_with_llm_stream'; 
            eventPrefix = 'flowchart';
            break;
        case 'journey': 
            command = 'refine_user_journey_with_llm_stream'; 
            eventPrefix = 'journey';
            break;
      }
      
      const unlistenChunk = await listen<string>(`${eventPrefix}:chunk`, (event) => {
        diagramCode += event.payload;
      });
      
      const unlistenError = await listen<string>(`${eventPrefix}:error`, (event) => {
        error = event.payload;
        cleanup();
        step = 'chat';
      });

      const unlistenDone = await listen<void>(`${eventPrefix}:done`, async () => {
        cleanup();
        
        await addToHistory("Initial Diagram Generation", diagramCode, 'edit');
        
        step = 'chat';
        await renderDiagram();
        scrollToBottom();
      });
      
      const cleanup = () => {
          unlistenChunk();
          unlistenError();
          unlistenDone();
      };
      
      await invoke(command, { content: contentToRefine });
      
    } catch (e) {
      error = String(e);
      step = 'chat'; 
    }
  }
  
  async function handleSendMessage() {
      if (!chatInput.trim() || isChatGenerating) return;
      
      const instruction = chatInput;
      chatInput = '';
      isChatGenerating = true;
      let streamedResponse = '';
      
      try {
          let command = '';
          let eventPrefix = '';
          
          if (chatMode === 'edit') {
              // EDIT MODE
              switch (targetType) {
                case 'er': 
                    command = 'edit_er_diagram_with_llm_stream'; 
                    eventPrefix = 'er';
                    break;
                case 'uml': 
                    command = 'edit_uml_diagram_with_llm_stream'; 
                    eventPrefix = 'uml';
                    break;
                case 'flowchart': 
                    command = 'edit_flowchart_with_llm_stream'; 
                    eventPrefix = 'flowchart';
                    break;
                case 'journey': 
                    command = 'edit_user_journey_with_llm_stream'; 
                    eventPrefix = 'journey';
                    break;
              }
          } else {
              // ASK MODE
              command = 'ask_llm_about_diagram_stream';
              eventPrefix = 'ask';
          }
          
          const unlistenChunk = await listen<string>(`${eventPrefix}:chunk`, (event) => {
             streamedResponse += event.payload;
             // If manual sync needed during stream?
          });
          
          const unlistenDone = await listen<void>(`${eventPrefix}:done`, async () => {
              unlistenChunk();
              unlistenDone();
              
              if (chatMode === 'edit') {
                  diagramCode = streamedResponse;
                  await renderDiagram();
              }
              
              await addToHistory(instruction, streamedResponse, chatMode);
              isChatGenerating = false;
              scrollToBottom();
          });
          
           if (chatMode === 'edit') {
                await invoke(command, { currentDiagram: diagramCode, instruction });
           } else {
                await invoke(command, { currentDiagram: diagramCode, question: instruction, diagramType: targetType });
           }
           
      } catch (e) {
          console.error(e);
          isChatGenerating = false;
      }
  }
  
  async function addToHistory(prompt: string, result: string, mode: 'edit' | 'ask') {
      const refinement: Refinement = {
          id: Math.random().toString(36).substring(2) + Date.now().toString(36),
          original_content: prompt,
          refined_content: result,
          timestamp: new Date().toISOString(),
          kind: targetType,
          mode: mode
      };
      
      await saveProjectRefinement(refinement);
      chatHistory = [...chatHistory, refinement];
  }
  
  async function restoreHistoryItem(item: Refinement) {
      if (!item.mode || item.mode === 'edit') {
          diagramCode = item.refined_content;
          await renderDiagram();
      }
  }
  
  async function renderDiagram() {
    await tick();
    if (container && diagramCode && !error) {
      try {
        container.innerHTML = '';
        const { svg } = await mermaid.render('mermaid-diagram-' + Date.now(), diagramCode);
        container.innerHTML = svg;
        panX = 0; panY = 0; scale = 1; 
      } catch (e) {
        console.error('Mermaid render error:', e);
      }
    }
  }
  
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
  
  function close() {
      onClose();
  }
</script>

<div class="modal-backdrop" on:click={close}>
  <div class="modal-window" on:click|stopPropagation>
    <div class="modal-header">
      <h2>Generate Diagram <small>{contextName}</small></h2>
      <button class="close-btn" on:click={close}>×</button>
    </div>
    
    <div class="modal-body">
      {#if step === 'options'}
        <div class="options-grid">
          <button class="option-card" on:click={() => generateDiagram('er')}>
            <span class="shortcut-badge">E</span>
            <span class="icon">🗂️</span>
            <h3>ER Diagram</h3>
            <p>Entity Relationship Diagram for data modeling</p>
          </button>
          <button class="option-card" on:click={() => generateDiagram('uml')}>
            <span class="shortcut-badge">U</span>
            <span class="icon">🏗️</span>
            <h3>UML Diagram</h3>
            <p>Class diagrams for system architecture</p>
          </button>
          <button class="option-card" on:click={() => generateDiagram('flowchart')}>
            <span class="shortcut-badge">F</span>
            <span class="icon">🔄</span>
            <h3>Flowchart</h3>
            <p>Process flow and logic visualization</p>
          </button>
          <button class="option-card" on:click={() => generateDiagram('journey')}>
            <span class="shortcut-badge">J</span>
            <span class="icon">🗺️</span>
            <h3>User Journey</h3>
            <p>User interactions and flows</p>
          </button>
        </div>
        
      {:else if step === 'generating'}
        <div class="loading-view">
          <div class="spinner"></div>
          <p>Generating {targetType} diagram...</p>
          <pre class="stream-preview">{diagramCode}</pre>
        </div>
        
      {:else if step === 'chat'}
        <div class="split-view">
            <div class="left-panel">
                 <div class="result-tabs">
                    <button 
                        class="tab-btn {resultTab === 'editor' ? 'active' : ''}" 
                        on:click={() => resultTab = 'editor'}
                    >
                        Editor
                    </button>
                    <button 
                        class="tab-btn {resultTab === 'render' ? 'active' : ''}" 
                        on:click={() => { resultTab = 'render'; renderDiagram(); }}
                    >
                        Render
                    </button>
                </div>
                
                <div class="result-content">
                    {#if resultTab === 'editor'}
                        <div class="editor-container">
                            <textarea 
                                bind:value={diagramCode} 
                                placeholder="Mermaid code..."
                            ></textarea>
                            <div class="editor-actions">
                                <button class="action-btn" on:click={() => { resultTab = 'render'; renderDiagram(); }}>Render</button>
                            </div>
                        </div>
                    {:else}
                        {#if error}
                            <div class="error-msg">
                                <p>Error: {error}</p>
                            </div>
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
                    {/if}
                </div>
            </div>
            
            <div class="right-panel">
                <div class="chat-list" bind:this={chatContainer}>
                    {#each chatHistory as item}
                        <div class="chat-bubble user" on:click={() => restoreHistoryItem(item)}>
                            <div class="bubble-content">{item.original_content}</div>
                        </div>
                        <div class="chat-bubble ai {item.mode === 'ask' ? 'ask-mode' : ''}" on:click={() => restoreHistoryItem(item)}>
                            {#if item.mode === 'ask'}
                                <div class="bubble-content markdown-body">{item.refined_content}</div>
                            {:else}
                                <div class="bubble-content">Updated Diagram</div>
                            {/if}
                            <small>
                                {#if item.mode === 'ask'}Antigravity (Answer){:else}Antigravity (Edit){/if} • {new Date(item.timestamp).toLocaleTimeString()}
                            </small>
                        </div>
                    {/each}
                    
                    {#if isChatGenerating}
                         <div class="chat-bubble ai generating">
                            <div class="typing-indicator"><span>.</span><span>.</span><span>.</span></div>
                        </div>
                    {/if}
                </div>
                
                <div class="chat-input-area">
                    <div class="mode-toggle">
                        <button class:active={chatMode === 'edit'} on:click={() => chatMode = 'edit'}>Refine (Edit)</button>
                        <button class:active={chatMode === 'ask'} on:click={() => chatMode = 'ask'}>Ask (Chat)</button>
                    </div>
                    
                    <textarea 
                        bind:value={chatInput} 
                        placeholder={chatMode === 'edit' ? "Describe changes (e.g., 'Add a user table')..." : "Ask about the diagram (e.g., 'Explain the relationship')..."}
                        on:keydown={(e) => { if(e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); handleSendMessage(); } }}
                    ></textarea>
                    <button class="send-btn" on:click={handleSendMessage} disabled={isChatGenerating}>
                        Send
                    </button>
                </div>
                
                <div class="chat-footer">
                     <button class="back-link" on:click={() => step = 'options'}>Back (Clear)</button>
                </div>
            </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  /* Keep existing styles... */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .modal-window {
    background: #1a202c;
    width: 95%;
    height: 95%;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    border: 1px solid #4a5568;
    max-width: 1600px;
  }
  
  .modal-header {
    padding: 1rem;
    border-bottom: 1px solid #2d3748;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .modal-header h2 {
    margin: 0;
    font-size: 1.2rem;
    color: #e2e8f0;
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .modal-header small {
    font-size: 0.9rem;
    color: #a0aec0;
    font-weight: normal;
    background: #2d3748;
    padding: 0.2rem 0.6rem;
    border-radius: 4px;
  }
  
  .close-btn {
    background: none;
    border: none;
    color: #a0aec0;
    font-size: 1.5rem;
    cursor: pointer;
  }
  
  .modal-body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  
  .options-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1.5rem;
    padding: 2rem;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
    align-self: center;
  }
  
  .option-card {
    background: #2d3748;
    border: 1px solid #4a5568;
    padding: 2rem;
    border-radius: 8px;
    cursor: pointer;
    text-align: center;
    transition: all 0.2s;
    position: relative;
  }
  
  .option-card:hover {
    background: #4a5568;
    transform: translateY(-2px);
  }

  .shortcut-badge {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: #4a5568;
    color: #edf2f7;
    width: 24px;
    height: 24px;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.8rem;
    font-weight: bold;
    border: 1px solid #718096;
  }
  
  .option-card .icon {
    font-size: 3rem;
    display: block;
    margin-bottom: 1rem;
  }
  
  .loading-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
  }
  
  .spinner {
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-radius: 50%;
    border-top: 3px solid #4299e1;
    width: 40px;
    height: 40px;
    animation: spin 1s linear infinite;
    margin-bottom: 1rem;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .stream-preview {
    margin-top: 1rem;
    background: #0d1117;
    padding: 1rem;
    border-radius: 4px;
    max-width: 80%;
    max-height: 200px;
    overflow: auto;
    font-size: 0.8rem;
    color: #a0aec0;
  }
  
  /* Split View */
  .split-view {
      display: flex;
      flex: 1;
      overflow: hidden;
  }
  
  .left-panel {
      flex: 2; /* Takes more space */
      display: flex;
      flex-direction: column;
      border-right: 1px solid #2d3748;
      min-width: 0; /* Flexbox trick */
  }
  
  .right-panel {
      flex: 1;
      display: flex;
      flex-direction: column;
      background: #171923;
      min-width: 300px;
      max-width: 400px;
  }
  
  /* Tabs in Left Panel */
  .result-tabs {
    display: flex;
    background: #1a202c;
    border-bottom: 1px solid #2d3748;
  }

  .tab-btn {
    padding: 0.75rem 1.5rem;
    background: none;
    border: none;
    color: #a0aec0;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tab-btn.active {
    color: #4299e1;
    border-bottom-color: #4299e1;
    background: rgba(66, 153, 225, 0.1);
  }

  .result-content {
    flex: 1;
    overflow: hidden;
    position: relative;
    display: flex;
    flex-direction: column;
  }

  .editor-container {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .editor-container textarea {
    flex: 1;
    background: #0d1117;
    color: #e2e8f0;
    border: none;
    padding: 1rem;
    font-family: monospace;
    font-size: 0.9rem;
    resize: none;
    outline: none;
  }
  
  .editor-actions {
      padding: 0.5rem;
      border-top: 1px solid #2d3748;
      display: flex;
      justify-content: flex-end;
  }
  
  .diagram-container {
    flex: 1;
    background: #0d1117;
    overflow: hidden;
    position: relative;
    cursor: grab;
  }
  
  .diagram-container:active {
    cursor: grabbing;
  }
  
  .zoom-content {
    transform-origin: 0 0;
  }
  
  /* Chat Styles */
  .chat-list {
      flex: 1;
      overflow-y: auto;
      padding: 1rem;
      display: flex;
      flex-direction: column;
      gap: 1rem;
  }
  
  .chat-bubble {
      padding: 0.8rem 1rem;
      border-radius: 8px;
      max-width: 90%;
      cursor: pointer;
      border: 1px solid transparent;
  }
  
  .chat-bubble:hover {
      border-color: #4a5568;
  }
  
  .chat-bubble.user {
      align-self: flex-end;
      background: #3182ce;
      color: white;
  }
  
  .chat-bubble.ai {
      align-self: flex-start;
      background: #2d3748;
      color: #e2e8f0;
  }

  .chat-bubble.ai.ask-mode {
      background: #2c3e50; /* Slightly different color for ask mode */
      border-left: 3px solid #48bb78;
  }
  
  .chat-bubble.ai small {
      display: block;
      margin-top: 0.5rem;
      font-size: 0.7rem;
      opacity: 0.6;
  }
  
  .bubble-content.markdown-body {
      white-space: pre-wrap;
  }
  
  .typing-indicator span {
      animation: blink 1.4s infinite both;
      margin: 0 2px;
      font-size: 1.5rem;
      line-height: 10px;
  }
  
  .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
  .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }
  
  @keyframes blink {
      0% { opacity: 0.2; }
      20% { opacity: 1; }
      100% { opacity: 0.2; }
  }
  
  .chat-input-area {
      padding: 1rem;
      border-top: 1px solid #2d3748;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
  }
  
  .mode-toggle {
      display: flex;
      gap: 0.5rem;
      margin-bottom: 0.5rem;
  }
  
  .mode-toggle button {
      background: #2d3748;
      border: 1px solid #4a5568;
      color: #a0aec0;
      padding: 0.3rem 0.8rem;
      border-radius: 4px;
      cursor: pointer;
      font-size: 0.8rem;
      flex: 1;
  }
  
  .mode-toggle button.active {
      background: #4299e1;
      color: white;
      border-color: #4299e1;
  }

  .mode-toggle button.active:nth-child(2) {
       background: #48bb78; /* Green for Ask */
       border-color: #48bb78;
  }
  
  .chat-input-area textarea {
      background: #2d3748;
      border: 1px solid #4a5568;
      color: white;
      border-radius: 4px;
      padding: 0.5rem;
      min-height: 60px;
      resize: vertical;
  }
  
  .send-btn {
      align-self: flex-end;
      background: #38a169;
      color: white;
      border: none;
      padding: 0.5rem 1rem;
      border-radius: 4px;
      cursor: pointer;
  }
  
  .send-btn:disabled {
      opacity: 0.5;
      cursor: not-allowed;
  }
  
  .chat-footer {
      padding: 0.5rem;
      text-align: center;
      border-top: 1px solid #2d3748;
  }
  
  .back-link {
      background: none;
      border: none;
      color: #718096;
      font-size: 0.8rem;
      cursor: pointer;
  }
  
  .back-link:hover {
      color: #a0aec0;
  }
  
  .action-btn {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    background: #4299e1;
    color: white;
    border: none;
    cursor: pointer;
  }
</style>
