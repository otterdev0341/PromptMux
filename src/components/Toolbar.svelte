<script lang="ts">
  import { createSection, projectStore, platform, getPlatform } from '../stores/projectStore';
  import { onMount, createEventDispatcher } from 'svelte';
  import ProjectSwitcher from './ProjectSwitcher.svelte';
  import SettingsModal from './SettingsModal.svelte';

  const dispatch = createEventDispatcher();

  export let showSidebar = true;
  export let showEditor = true;

  let showNewSectionDialog = false;
  let showSettingsDialog = false;
  let newSectionName = '';
  let projectSwitcherComponent: ProjectSwitcher;

  export { projectSwitcherComponent };

  onMount(async () => {
    try {
      const os = await getPlatform();
      platform.set(os);
    } catch (error) {
      console.error('Failed to detect software platform:', error);
      platform.set('Unknown');
    }
  });

  function handleToggleSidebar() {
    dispatch('toggleSidebar');
  }

  function handleToggleEditor() {
    dispatch('toggleEditor');
  }

  function handleNewSection() {
    newSectionName = 'New Section';
    showNewSectionDialog = true;
  }
  
  function handleOpenSettings() {
    showSettingsDialog = true;
  }
  
  function handleCloseSettings() {
    showSettingsDialog = false;
  }

  async function confirmNewSection() {
    if (newSectionName.trim()) {
      await createSection(newSectionName);
      showNewSectionDialog = false;
      newSectionName = '';
    }
  }

  function cancelNewSection() {
    showNewSectionDialog = false;
    newSectionName = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmNewSection();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelNewSection();
    }
  }
</script>

<div class="toolbar">
  <div class="toolbar-left">
    <button class="icon-btn" on:click={handleToggleSidebar} title={showSidebar ? "Hide Sidebar" : "Show Sidebar"}>
      {showSidebar ? '◀' : '▶'}
    </button>
    <h1 class="app-title">PromptMux</h1>
    <ProjectSwitcher bind:this={projectSwitcherComponent} />
    <span class="platform-badge">
      <span class="platform-icon">💻</span>
      {$platform}
    </span>
  </div>

  <div class="toolbar-right">
    <button class="icon-btn" on:click={handleToggleEditor} title={showEditor ? "Hide Editor" : "Show Editor"}>
      {showEditor ? '📝' : '📝'}
    </button>
    <button class="toolbar-btn" on:click={handleOpenSettings} title="LLM Settings">
      <span class="btn-icon">⚙️</span>
      <span class="btn-text">Settings</span>
    </button>
    
    <button class="toolbar-btn" on:click={handleNewSection} title="New Section (Ctrl+b then n)">
      <span class="btn-icon">➕</span>
      <span class="btn-text">New Section</span>
    </button>
  </div>
</div>

{#if showSettingsDialog}
  <SettingsModal onClose={handleCloseSettings} />
{/if}

{#if showNewSectionDialog}
  <div class="modal-backdrop" on:click={cancelNewSection}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Create New Section</h3>
      <input
        type="text"
        class="input"
        bind:value={newSectionName}
        placeholder="Section name..."
        on:keydown={handleKeydown}
        autofocus
      />
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={cancelNewSection}>Cancel</button>
        <button class="btn btn-primary" on:click={confirmNewSection}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .toolbar {
    height: 60px;
    min-height: 60px;
    background-color: #1a202c;
    border-bottom: 1px solid #2d3748;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .app-title {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: #63b3ed;
  }

  .project-name {
    font-size: 0.875rem;
    color: #718096;
    padding: 0.25rem 0.75rem;
    background-color: #2d3748;
    border-radius: 0.25rem;
  }

  .platform-badge {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    color: #a0aec0;
    padding: 0.25rem 0.6rem;
    background-color: #1e293b;
    border: 1px solid #334155;
    border-radius: 0.25rem;
  }

  .platform-icon {
    font-size: 0.85rem;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .icon-btn {
    background: none;
    border: none;
    color: #a0aec0;
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 0.375rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .icon-btn:hover {
    color: #e2e8f0;
    background-color: #2d3748;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #3182ce;
    color: white;
    border: none;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }

  .toolbar-btn:hover {
    background-color: #2c5282;
  }

  .btn-icon {
    font-size: 1rem;
  }

  .btn-text {
    font-size: 0.875rem;
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
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
  }

  .modal-content h3 {
    margin: 0 0 1rem 0;
    font-size: 1.125rem;
    color: #e2e8f0;
  }

  .input {
    width: 100%;
    padding: 0.75rem;
    background-color: #0d1117;
    border: 1px solid #2d3748;
    border-radius: 0.375rem;
    color: #e2e8f0;
    font-size: 0.875rem;
    outline: none;
    box-sizing: border-box;
  }

  .input:focus {
    border-color: #3182ce;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1rem;
  }

  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: background-color 0.15s ease;
  }

  .btn-secondary {
    background-color: #2d3748;
    color: #e2e8f0;
  }

  .btn-secondary:hover {
    background-color: #4a5568;
  }

  .btn-primary {
    background-color: #3182ce;
    color: white;
  }

  .btn-primary:hover {
    background-color: #2c5282;
  }
</style>
