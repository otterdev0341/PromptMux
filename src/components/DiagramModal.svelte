<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import mermaid from 'mermaid';
  import { projectStore, activeSectionId, activeTopicId } from '../stores/projectStore';
  
  export let onClose: () => void;
  
  // State
  let step: 'options' | 'generating' | 'result' = 'options';
  let targetType: 'er' | 'uml' | 'flowchart' | 'journey' | null = null;
  let diagramCode = '';
  let error = '';
  let container: HTMLElement;
  
  // Zoom/Pan State
  let scale = 1;
  let panX = 0;
  let panY = 0;
  let isPanning = false;
  let startX = 0;
  let startY = 0;
  
  // Context
  let contextName = '';
  let contentToRefine = '';
  
  // Result Tab State
  let resultTab: 'editor' | 'render' = 'render';

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
    if (step !== 'options') return;
    
    switch(e.key.toLowerCase()) {
        case 'e': generateDiagram('er'); break;
        case 'u': generateDiagram('uml'); break;
        case 'f': generateDiagram('flowchart'); break;
        case 'j': generateDiagram('journey'); break;
        case 'escape': close(); break;
    }
  }
  
  async function generateDiagram(type: 'er' | 'uml' | 'flowchart' | 'journey') {
    targetType = type;
    step = 'generating';
    error = '';
    diagramCode = '';
    resultTab = 'render'; // Reset tab
    
    if (!contentToRefine) {
        error = 'No content found to generate diagram.';
        step = 'result';
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
        step = 'result';
      });

      const unlistenDone = await listen<void>(`${eventPrefix}:done`, async () => {
        cleanup();
        step = 'result';
        await renderDiagram();
      });
      
      const cleanup = () => {
          unlistenChunk();
          unlistenError();
          unlistenDone();
      };
      
      await invoke(command, { content: contentToRefine });
      
    } catch (e) {
      error = String(e);
      step = 'result'; 
    }
  }
  
  async function renderDiagram() {
    await tick();
    if (container && diagramCode && !error) {
      try {
        container.innerHTML = '';
        const { svg } = await mermaid.render('mermaid-diagram-' + Date.now(), diagramCode);
        container.innerHTML = svg;
      } catch (e) {
        console.error('Mermaid render error:', e);
         // Often Mermaid throws if code is incomplete, which might happen if LLM fails partially
         // But we still show the code
      }
    }
  }
  
  // Zoom/Pan logic
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
        
      {:else if step === 'result'}
        <div class="result-view">
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
                    </div>
                {:else}
                    {#if error}
                        <div class="error-msg">
                            <p>Error: {error}</p>
                            <pre>{diagramCode}</pre>
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
            
            <!-- Controls -->
             <div class="controls">
                <button class="action-btn secondary" on:click={() => step = 'options'}>Back to Options</button>
                {#if resultTab === 'editor'}
                     <button class="action-btn" on:click={() => { resultTab = 'render'; renderDiagram(); }}>Render View</button>
                {/if}
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
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .modal-window {
    background: #1a202c;
    width: 90%;
    height: 90%;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
    border: 1px solid #4a5568;
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
  
  .result-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

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

  .controls {
    padding: 1rem;
    border-top: 1px solid #2d3748;
    display: flex;
    justify-content: flex-end;
    gap: 1rem;
  }
  
  .action-btn {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    background: #4299e1;
    color: white;
    border: none;
    cursor: pointer;
  }

  .action-btn.secondary {
    background: #4a5568;
  }
  
  .error-msg {
    padding: 2rem;
    color: #fc8181;
    text-align: center;
  }
</style>
