<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import {
    projectStore,
    activeSectionId,
    activeTopicId,
    createSection,
    createTopic,
    deleteSection,
    deleteTopic,
    updateSectionName,
    updateTopicName,
    reorderItem,
    type Section,
    type Topic
  } from '../stores/projectStore';

  let expandedSections = new Set<string>();
  let flatItems: Array<{ id: string; type: 'section' | 'topic'; name: string; sectionId?: string }> = [];
  let highlightedIndex = 0;
  let focused = false;
  let showDeleteModal = false;
  let itemToDelete: { id: string; type: 'section' | 'topic'; name: string } | null = null;
  
  // Editing state
  let editingItemId: string | null = null;
  let editingName = '';
  let inputElement: HTMLInputElement;

  $: sections = $projectStore?.sections.sort((a, b) => a.order_index - b.order_index) || [];

  $: {
    // Rebuild flat list whenever sections change
    flatItems = [];
    sections.forEach(section => {
      flatItems.push({
        id: section.id,
        type: 'section',
        name: section.name,
      });
      if (expandedSections.has(section.id)) {
        const topics = section.topics.sort((a, b) => a.order_index - b.order_index);
        topics.forEach(topic => {
          flatItems.push({
            id: topic.id,
            type: 'topic',
            name: topic.name,
            sectionId: section.id,
          });
        });
      }
    });
  }

  onMount(() => {
    // Attach keyboard listener to the sidebar element, not window
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
      sidebar.addEventListener('keydown', handleKeydown);
    }
  });

  onDestroy(() => {
    const sidebar = document.querySelector('.sidebar');
    if (sidebar) {
      sidebar.removeEventListener('keydown', handleKeydown);
    }
  });

  function handleFocus() {
    focused = true;
  }

  function handleBlur() {
    focused = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!focused) return;

    // Navigation
    if (e.key === 'j') {
      e.preventDefault();
      highlightedIndex = Math.min(highlightedIndex + 1, flatItems.length - 1);
    } else if (e.key === 'k') {
      e.preventDefault();
      highlightedIndex = Math.max(highlightedIndex - 1, 0);
    }
    // Expand/collapse
    else if (e.key === 'h') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      if (item.type === 'section') {
        expandedSections.delete(item.id);
        expandedSections = new Set(expandedSections);
      }
    } else if (e.key === 'l') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      if (item.type === 'section') {
        expandedSections.add(item.id);
        expandedSections = new Set(expandedSections);
      }
    }
    // Select
    else if (e.key === 'Enter') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      if (item.type === 'section') {
        activeSectionId.set(item.id);
        activeTopicId.set(null);
        // Toggle expand/collapse
        if (expandedSections.has(item.id)) {
          expandedSections.delete(item.id);
        } else {
          expandedSections.add(item.id);
        }
        expandedSections = new Set(expandedSections);
      } else if (item.type === 'topic') {
        activeSectionId.set(item.sectionId!);
        activeTopicId.set(item.id);
        // Focus the editor
        document.getElementById('topic-editor')?.focus();
      }
    }
    // Create new
    else if (e.key === 'n') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      if (item.type === 'section') {
        // When on a section, create a new topic in that section
        createTopic(item.id, 'New Topic').then(topic => {
          activeTopicId.set(topic.id);
          expandedSections.add(item.id);
          expandedSections = new Set(expandedSections);
          // Auto-start editing the new topic
          startEditing(topic.id, topic.name);
        });
      } else if (item.type === 'topic') {
        // When on a topic, create a new topic in the same section
        const sectionId = item.sectionId!;
        createTopic(sectionId, 'New Topic').then(topic => {
          activeTopicId.set(topic.id);
          if (!expandedSections.has(sectionId)) {
            expandedSections.add(sectionId);
            expandedSections = new Set(expandedSections);
          }
          // Auto-start editing the new topic
          startEditing(topic.id, topic.name);
        });
      }
    }
    // Delete
    else if (e.key === 'd') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      if (item.type === 'section') {
        promptDelete(item.id, 'section', item.name);
      } else if (item.type === 'topic') {
        promptDelete(item.id, 'topic', item.name);
      }
    }
    // Reorder with Ctrl+Shift
    else if (e.ctrlKey && e.shiftKey && e.key === 'J') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      reorderItem(item.type, item.id, highlightedIndex + 2);
    } else if (e.ctrlKey && e.shiftKey && e.key === 'K') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      reorderItem(item.type, item.id, highlightedIndex - 1);
    }
    // Rename with F2
    else if (e.key === 'F2') {
      e.preventDefault();
      const item = flatItems[highlightedIndex];
      startEditing(item.id, item.name);
    }
  }

  function startEditing(id: string, name: string) {
    editingItemId = id;
    editingName = name;
    tick().then(() => {
      if (inputElement) {
        inputElement.focus();
        inputElement.select();
      }
    });
  }

  async function saveEdit() {
    if (!editingItemId) return;
    
    // Find item type
    const item = flatItems.find(i => i.id === editingItemId);
    if (!item) {
      editingItemId = null;
      return;
    }
    
    try {
      if (item.type === 'section') {
        await updateSectionName(editingItemId, editingName);
      } else {
        await updateTopicName(editingItemId, editingName);
      }
    } catch (error) {
      console.error('Failed to update name:', error);
    }
    
    editingItemId = null;
    // Refocus sidebar after edit
    document.querySelector<HTMLElement>('.sidebar')?.focus();
  }

  function cancelEdit() {
    editingItemId = null;
    document.querySelector<HTMLElement>('.sidebar')?.focus();
  }

  function handleInputKeydown(e: KeyboardEvent) {
    e.stopPropagation(); // Prevent Sidebar navigation while typing
    
    if (e.key === 'Enter') {
      saveEdit();
    } else if (e.key === 'Escape') {
      cancelEdit();
    }
  }

  function selectItem(item: typeof flatItems[0]) {
    if (item.type === 'section') {
      activeSectionId.set(item.id);
      activeTopicId.set(null);
      if (expandedSections.has(item.id)) {
        expandedSections.delete(item.id);
      } else {
        expandedSections.add(item.id);
      }
      expandedSections = new Set(expandedSections);
    } else if (item.type === 'topic') {
      activeSectionId.set(item.sectionId!);
      activeTopicId.set(item.id);
      document.getElementById('topic-editor')?.focus();
    }
  }

  function getIcon(type: 'section' | 'topic') {
    return type === 'section' ? '📁' : '📄';
  }

  function promptDelete(id: string, type: 'section' | 'topic', name: string) {
    itemToDelete = { id, type, name };
    showDeleteModal = true;
  }

  function confirmDelete() {
    if (!itemToDelete) return;
    
    if (itemToDelete.type === 'section') {
      deleteSection(itemToDelete.id);
    } else {
      deleteTopic(itemToDelete.id);
    }
    
    showDeleteModal = false;
    itemToDelete = null;
  }

  function cancelDelete() {
    showDeleteModal = false;
    itemToDelete = null;
  }

  export let id: string = '';

  export function startEditingActive() {
    // Find active topic or section and start editing
    if ($activeTopicId) {
      const topic = flatItems.find(i => i.id === $activeTopicId && i.type === 'topic');
      if (topic) {
        startEditing(topic.id, topic.name);
      }
    } else if ($activeSectionId) {
      const section = flatItems.find(i => i.id === $activeSectionId && i.type === 'section');
      if (section) {
        startEditing(section.id, section.name);
      }
    }
  }
</script>

<div 
  class="sidebar"
  {id}
  tabindex="0"
  on:focus={handleFocus}
  on:blur={handleBlur}
>
  <div class="sidebar-header">
    <h2>PromptMux</h2>
    <span class="keyboard-hint">Press Ctrl+b q for help</span>
  </div>

  <div class="sections-list">
    {#each sections as section (section.id)}
      {@const isExpanded = expandedSections.has(section.id)}
      {@const isActive = $activeSectionId === section.id}
      
      <div class="section-item">
        <div class="item-row-wrapper">
          <div
            class="item-row"
            class:highlighted={flatItems[highlightedIndex]?.id === section.id}
            class:active={isActive}
            on:click={() => selectItem({ id: section.id, type: 'section', name: section.name })}
          >
            <span class="icon">{isExpanded ? '📂' : '📁'}</span>
            {#if editingItemId === section.id}
              <input
                bind:this={inputElement}
                type="text"
                class="edit-input"
                bind:value={editingName}
                on:keydown={handleInputKeydown}
                on:blur={saveEdit}
                on:click|stopPropagation
              />
            {:else}
              <span class="name" on:dblclick|stopPropagation={() => startEditing(section.id, section.name)}>{section.name}</span>
            {/if}
            <span class="count">{section.topics.length}</span>
          </div>
          <button 
            class="delete-btn"
            on:click|stopPropagation={() => promptDelete(section.id, 'section', section.name)}
            title="Delete section"
          >
            🗑️
          </button>
        </div>

        {#if isExpanded}
          <div class="topics-list">
            {#each section.topics.sort((a, b) => a.order_index - b.order_index) as topic (topic.id)}
              {@const isTopicActive = $activeTopicId === topic.id}
              
              <div class="item-row-wrapper">
                <div
                  class="item-row topic"
                  class:highlighted={flatItems[highlightedIndex]?.id === topic.id}
                  class:active={isTopicActive}
                  on:click={() => selectItem({
                    id: topic.id,
                    type: 'topic',
                    name: topic.name,
                    sectionId: section.id
                  })}
                >
                  <span class="icon indent">{getIcon('topic')}</span>
                  {#if editingItemId === topic.id}
                    <input
                      bind:this={inputElement}
                      type="text"
                      class="edit-input"
                      bind:value={editingName}
                      on:keydown={handleInputKeydown}
                      on:blur={saveEdit}
                      on:click|stopPropagation
                    />
                  {:else}
                    <span class="name" on:dblclick|stopPropagation={() => startEditing(topic.id, topic.name)}>{topic.name}</span>
                  {/if}
                </div>
                <button 
                  class="delete-btn"
                  on:click|stopPropagation={() => promptDelete(topic.id, 'topic', topic.name)}
                  title="Delete topic"
                >
                  🗑️
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</div>

{#if showDeleteModal}
  <div class="modal-backdrop" on:click={cancelDelete}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Confirm Delete</h3>
      <p class="delete-message">
        Are you sure you want to delete the {itemToDelete?.type} <strong>"{itemToDelete?.name}"</strong>?
      </p>
      {#if itemToDelete?.type === 'section'}
        <p class="delete-warning">⚠️ This will also delete all topics within this section.</p>
      {/if}
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={cancelDelete}>Cancel</button>
        <button class="btn btn-danger" on:click={confirmDelete}>Delete</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .sidebar {
    width: 300px;
    min-width: 250px;
    background-color: #1a202c;
    border-right: 1px solid #2d3748;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 1rem;
    border-bottom: 1px solid #2d3748;
  }

  .sidebar-header h2 {
    margin: 0;
    font-size: 1.25rem;
    color: #e2e8f0;
  }

  .keyboard-hint {
    display: block;
    font-size: 0.75rem;
    color: #718096;
    margin-top: 0.5rem;
  }

  .sections-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .section-item {
    margin-bottom: 0.25rem;
  }

  .item-row {
    display: flex;
    align-items: center;
    padding: 0.5rem 1rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
    user-select: none;
  }

  .item-row:hover {
    background-color: #2d3748;
  }

  .item-row.highlighted {
    background-color: #3182ce;
  }

  .item-row.active {
    background-color: #2c5282;
  }

  .item-row.topic .indent {
    margin-left: 1rem;
  }

  .icon {
    margin-right: 0.5rem;
    font-size: 1rem;
  }

  .name {
    flex: 1;
    font-size: 0.875rem;
    color: #e2e8f0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .count {
    font-size: 0.75rem;
    color: #718096;
    background-color: #2d3748;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
  }

  .topics-list {
    margin-left: 0.5rem;
  }

  .topic {
    font-size: 0.875rem;
  }

  .item-row-wrapper {
    display: flex;
    align-items: center;
    position: relative;
  }

  .delete-btn {
    position: absolute;
    right: 0.5rem;
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s ease;
    font-size: 1rem;
    line-height: 1;
  }

  .item-row-wrapper:hover .delete-btn {
    opacity: 1;
  }

  .delete-btn:hover {
    transform: scale(1.1);
    filter: brightness(1.2);
  }

  .delete-btn:active {
    transform: scale(0.95);
  }

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
    padding: 1.5rem;
    min-width: 400px;
    max-width: 500px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-content h3 {
    margin: 0 0 1rem 0;
    font-size: 1.25rem;
    color: #e2e8f0;
  }

  .delete-message {
    margin: 0 0 0.5rem 0;
    font-size: 0.95rem;
    color: #cbd5e0;
    line-height: 1.5;
  }

  .delete-message strong {
    color: #fc8181;
  }

  .delete-warning {
    margin: 0.5rem 0 1.5rem 0;
    font-size: 0.875rem;
    color: #f6ad55;
    background-color: rgba(246, 173, 85, 0.1);
    padding: 0.75rem;
    border-radius: 0.375rem;
    border-left: 3px solid #f6ad55;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .btn {
    padding: 0.625rem 1.25rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: background-color 0.15s ease;
  }

  .btn-secondary {
    background-color: #4a5568;
    color: #e2e8f0;
  }

  .btn-secondary:hover {
    background-color: #718096;
  }

  .btn-danger {
    background-color: #e53e3e;
    color: white;
  }

  .btn-danger:hover {
    background-color: #c53030;
  }

  .edit-input {
    flex: 1;
    background-color: #2d3748;
    border: 1px solid #4299e1;
    color: #e2e8f0;
    font-size: 0.875rem;
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    outline: none;
    min-width: 0;
  }
</style>
