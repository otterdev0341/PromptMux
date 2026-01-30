<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { createEventDispatcher } from 'svelte';
  import { activeTopic, updateTopicContent, activeTopicId, projectStore, workspaceStore } from '../stores/projectStore';
  import { debounce } from '../utils/debounce';

  const dispatch = createEventDispatcher();

  let editorContent = '';
  let focused = false;
  let textareaElement: HTMLTextAreaElement;
  let showFocusHint = false;
  let isViewMode = true; // Start in view mode
  let isSaving = false;
  let hasUnsavedChanges = false;
  let isProgrammaticFocus = false; // Track programmatic focus to prevent click toggle

  let lastTopicId: string | null = null;

  $: if ($activeTopic) {
    showFocusHint = true;
    
    // Only reset to view mode if the topic ID has changed
    // This prevents typing/autosave from kicking you out of edit mode
    if ($activeTopic.id !== lastTopicId) {
      editorContent = $activeTopic.content;
      isViewMode = true;
      hasUnsavedChanges = false;
      lastTopicId = $activeTopic.id;
      
      // Auto-focus when topic changes, but keep view mode
      tick().then(() => {
        if (textareaElement) {
          textareaElement.focus();
          console.log('Auto-focused textarea for topic:', $activeTopic.name);
          // Hide hint after 2 seconds
          setTimeout(() => {
            showFocusHint = false;
          }, 2000);
        }
      });
    } else if (!focused && $activeTopic.content !== editorContent) {
      // If we are NOT focused, allow external updates (e.g. from refine or other sources)
      // to update the editor content
      editorContent = $activeTopic.content;
    }
  }

  function toggleEditMode() {
    isViewMode = !isViewMode;
    
    if (!isViewMode) {
      // Switching to edit mode
      isProgrammaticFocus = true;
      
      // Wait for DOM update then force focus
      tick().then(() => {
        if (textareaElement) {
          textareaElement.focus();
          // Ensure cursor is at the end
          const len = textareaElement.value.length;
          textareaElement.setSelectionRange(len, len);
        }
        
        // Clear flag shortly after
        setTimeout(() => {
          isProgrammaticFocus = false;
        }, 100);
      });
    }
  }

  const debouncedSave = debounce(async (topicId: string, content: string) => {
    try {
      await updateTopicContent(topicId, content);
    } catch (error) {
      console.error('Failed to save topic:', error);
    }
  }, 500);

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    editorContent = target.value;
    
    if ($activeTopicId) {
      // Optimistically update the workspace store for real-time merged view preview
      // Note: projectStore is now derived (read-only), so we update workspaceStore instead
      workspaceStore.update(workspace => {
        if (!workspace) return null;
        
        const projectIndex = workspace.projects.findIndex(
          p => p.id === workspace.active_project_id
        );
        
        if (projectIndex === -1) return workspace;
        
        const newWorkspace = { ...workspace };
        newWorkspace.projects = [...workspace.projects];
        newWorkspace.projects[projectIndex] = { ...workspace.projects[projectIndex] };
        newWorkspace.projects[projectIndex].sections = workspace.projects[projectIndex].sections.map(section => ({
          ...section,
          topics: section.topics.map(topic => 
            topic.id === $activeTopicId 
              ? { ...topic, content: editorContent }
              : topic
          )
        }));
        
        return newWorkspace;
      });
      
      debouncedSave($activeTopicId, editorContent);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+e - toggle edit mode
    if (e.ctrlKey && e.key === 'e') {
      e.preventDefault();
      toggleEditMode();
    }
    // In view mode, most keys are disabled
    else if (isViewMode) {
      e.preventDefault();
      return;
    }
    // Ctrl+s - manual save (only in edit mode)
    else if (e.ctrlKey && e.key === 's') {
      e.preventDefault();
      if ($activeTopicId) {
        isSaving = true;
        updateTopicContent($activeTopicId, editorContent)
          .then(() => {
            hasUnsavedChanges = false;
            isSaving = false;
            // Show brief save indicator
            const saveIndicator = document.createElement('div');
            saveIndicator.className = 'save-indicator';
            saveIndicator.textContent = 'Saved!';
            document.body.appendChild(saveIndicator);
            setTimeout(() => saveIndicator.remove(), 1000);
          })
          .catch(error => {
            isSaving = false;
            console.error('Failed to save:', error);
          });
      }
    }
    // Ctrl+r - refine current topic (only in edit mode)
    else if (e.ctrlKey && e.key === 'r') {
      e.preventDefault();
      dispatch('refineTopic');
    }
  }

  function handleFocus() {
    focused = true;
  }

  function handleBlur() {
    focused = false;
    // Save on blur
    if ($activeTopicId) {
      updateTopicContent($activeTopicId, editorContent)
        .catch(error => console.error('Failed to save on blur:', error));
    }
  }
</script>

<div class="topic-editor-container">
  {#if $activeTopic}
    <div class="editor-header">
      <div class="header-left">
        <h3>{$activeTopic.name}</h3>
        <div class="status-indicator">
          {#if isSaving}
            <span class="status-saving" title="Saving...">
              💾 <span class="status-text">Saving...</span>
            </span>
          {:else if hasUnsavedChanges}
            <span class="status-unsaved" title="Unsaved changes">
              ⚠️ <span class="status-text">Unsaved</span>
            </span>
          {:else if isViewMode}
            <span class="status-view" title="View mode - Press Ctrl+e to edit">
              👁️ <span class="status-text">View</span>
            </span>
          {:else}
            <span class="status-edit" title="Edit mode">
              ✏️ <span class="status-text">Edit</span>
            </span>
          {/if}
        </div>
      </div>
      <div class="header-right">
        <span class="section-name">{$activeTopic.sectionName}</span>
        <button class="mode-toggle-btn" on:click={toggleEditMode} title="Press Ctrl+e to toggle">
          {#if isViewMode}
            ✏️ Edit
          {:else}
            👁️ View
          {/if}
        </button>
      </div>
    </div>
    {#if showFocusHint && !isViewMode && !focused}
      <div class="focus-hint">
        ✨ Editor ready! Start typing or click here to focus.
      </div>
    {/if}
    
    <div class="editor-wrapper">
      <!-- Always render the textarea, just change its state -->
      <textarea
        bind:this={textareaElement}
        id="topic-editor"
        class="topic-editor"
        class:readonly-mode={isViewMode}
        bind:value={editorContent}
        on:input={handleInput}
        on:keydown={handleKeydown}
        on:focus={handleFocus}
        on:blur={handleBlur}
        on:click={() => {
          console.log('Textarea clicked!');
          // Only auto-switch if we're in view mode AND not handling programmatic focus
          if (isViewMode && !isProgrammaticFocus) {
            console.log('In view mode, auto-switching to edit mode');
            toggleEditMode();
          }
        }}
        placeholder="Start writing your prompt here..."
        spellcheck="false"
        tabindex="0"
        autocomplete="off"
        autocapitalize="off"
        autocorrect="off"
        readonly={isViewMode}
      ></textarea>
      
      {#if isViewMode}
        <div class="view-mode-overlay">
          <p>Press <kbd>Ctrl+e</kbd> or click the Edit button to edit this topic</p>
        </div>
      {/if}
    </div>
  {:else}
    <div class="empty-state">
      <div class="empty-icon">📝</div>
      <h2>No Topic Selected</h2>
      <p>Select a topic from the sidebar to start editing, or press Ctrl+b s then n to create a new one.</p>
    </div>
  {/if}
</div>

<style>
  .topic-editor-container {
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

  .status-indicator {
    display: flex;
    align-items: center;
    padding: 0.25rem 0.75rem;
    border-radius: 0.375rem;
    font-size: 0.75rem;
    font-weight: 500;
  }

  .status-saving {
    background-color: rgba(66, 153, 225, 0.2);
    color: #63b3ed;
    border: 1px solid #3182ce;
  }

  .status-unsaved {
    background-color: rgba(246, 173, 85, 0.2);
    color: #f6ad55;
    border: 1px solid #ed8936;
  }

  .status-view {
    background-color: rgba(160, 174, 192, 0.2);
    color: #a0aec0;
    border: 1px solid #718096;
  }

  .status-edit {
    background-color: rgba(72, 187, 120, 0.2);
    color: #48bb78;
    border: 1px solid #38a169;
  }

  .status-text {
    margin-left: 0.35rem;
  }

  .mode-toggle-btn {
    padding: 0.375rem 0.875rem;
    background-color: #3182ce;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .mode-toggle-btn:hover {
    background-color: #2c5282;
    transform: translateY(-1px);
  }

  .mode-toggle-btn:active {
    transform: translateY(0);
  }

  .view-mode-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }

  .view-mode-content {
    flex: 1;
    width: 100%;
    height: 100%;
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
  }

  .view-mode-overlay {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    display: flex;
    align-items: flex-end; /* Move to bottom */
    justify-content: center;
    background-color: transparent; /* Transparent background */
    padding-bottom: 2rem;
    pointer-events: none; /* Let clicks pass through to textarea */
    animation: fadeIn 0.3s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .view-mode-overlay p {
    color: #e2e8f0;
    font-size: 0.875rem;
    text-align: center;
    background-color: rgba(45, 55, 72, 0.9);
    padding: 0.5rem 1rem;
    border-radius: 9999px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid #4a5568;
    margin: 0;
  }

  .view-mode-overlay kbd {
    background-color: #2d3748;
    border: 1px solid #4a5568;
    border-radius: 0.25rem;
    padding: 0.125rem 0.5rem;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    color: #e2e8f0;
  }

  .section-name {
    font-size: 0.75rem;
    color: #718096;
    background-color: #2d3748;
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
  }

  .keyboard-hints {
    display: flex;
    gap: 1rem;
    align-items: center;
    font-size: 0.75rem;
    color: #718096;
  }

  .focus-btn {
    padding: 0.25rem 0.75rem;
    background-color: #3182ce;
    color: white;
    border: none;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .focus-btn:hover {
    background-color: #2c5282;
  }

  .focus-hint {
    padding: 0.75rem 1.5rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    font-size: 0.875rem;
    text-align: center;
    animation: slideIn 0.3s ease-out;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateY(-10px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .topic-editor {
    flex: 1;
    width: 100%;
    padding: 1.5rem;
    background-color: #0d1117;
    color: #e2e8f0;
    border: none;
    resize: none;
    font-family: 'Fira Code', 'Monaco', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.6;
    outline: none;
  }

  .topic-editor.readonly-mode {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .topic-editor::placeholder {
    color: #4a5568;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #718096;
    text-align: center;
    padding: 2rem;
  }

  .empty-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
  }

  .empty-state h2 {
    margin: 0 0 0.5rem 0;
    font-size: 1.5rem;
    color: #e2e8f0;
  }

  .empty-state p {
    margin: 0;
    font-size: 1rem;
    max-width: 400px;
  }

  :global(.save-indicator) {
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
