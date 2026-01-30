<script lang="ts">
  import { onMount } from 'svelte';
  import { loadProject, activeTopicId, activeSectionId, workspaceStore, isLeaderKeyActive } from './stores/projectStore';
  import { createProject, switchProject, createSection } from './stores/projectStore';
  import Toolbar from './components/Toolbar.svelte';
  import Sidebar from './components/Sidebar.svelte';
  import MainEditor from './components/MainEditor.svelte';
  import HelpModal from './components/HelpModal.svelte';
  import RefineModal from './components/RefineModal.svelte';

  let showHelpModal = false;
  let showRefineModal = false;
  let refineTarget: 'topic' | 'merged' | null = null;
  let isLeaderActive = false;
  let leaderTimeout: number | null = null;
  let sidebarComponent: Sidebar;
  let toolbarComponent: Toolbar;

  // UI State
  let showSidebar = true;
  let showEditor = true;

  function toggleSidebar() {
    showSidebar = !showSidebar;
  }

  function toggleEditor() {
    showEditor = !showEditor;
  }

  onMount(async () => {
    await loadProject();
    
    // Set up global keyboard listeners
    window.addEventListener('keydown', handleGlobalKeydown);
  });

  function handleGlobalKeydown(e: KeyboardEvent) {
    // Focus sidebar and jump to active item with backslash
    if (e.key === '\\') {
      e.preventDefault();
      const sidebar = document.getElementById('sidebar') as HTMLElement;
      if (sidebar) {
        sidebar.focus();
      }
      return;
    }

    // F2 to rename active topic/section
    if (e.key === 'F2') {
      e.preventDefault();
      sidebarComponent?.startEditingActive();
      return;
    }

    // Ctrl+Shift+P to cycle projects forward
    if (e.ctrlKey && e.shiftKey && e.key === 'P') {
      e.preventDefault();
      cycleNextProject();
      return;
    }

    // Ctrl+Shift+S to create new section
    if (e.ctrlKey && e.shiftKey && e.key === 'S') {
      e.preventDefault();
      createNewSection();
      return;
    }

    // Ctrl+Alt+N to create new project
    if (e.ctrlKey && e.altKey && e.key === 'n') {
      e.preventDefault();
      toolbarComponent?.projectSwitcherComponent?.openNewProjectModal();
      return;
    }

    // Check for Ctrl+b (leader key)
    if (e.ctrlKey && e.key === 'b') {
      e.preventDefault();
      e.stopPropagation();
      isLeaderActive = true;
      
      // Clear any existing timeout
      if (leaderTimeout !== null) {
        clearTimeout(leaderTimeout);
      }
      
      // Reset leader after 1 second if no command is pressed
      leaderTimeout = window.setTimeout(() => {
        isLeaderActive = false;
        leaderTimeout = null;
      }, 1000);
      
      return;
    }

    // Handle leader key combinations
    if (isLeaderActive) {
      e.preventDefault();
      e.stopPropagation();
      
      switch (e.key.toLowerCase()) {
        case 'q':
          showHelpModal = !showHelpModal;
          break;
        case 's':
          // Create new section
          createNewSection();
          break;
        case 'e':
          document.getElementById('topic-editor')?.focus();
          break;
        case 'o':
          document.getElementById('merged-output')?.focus();
          break;
        case 'r':
          showRefineModal = true;
          refineTarget = 'merged';
          break;
        case 'p':
          // Toggle project switcher dropdown
          toolbarComponent?.projectSwitcherComponent?.toggleDropdown();
          break;
        case 'n':
          // Create new project
          toolbarComponent?.projectSwitcherComponent?.openNewProjectModal();
          break;
        case 'j':
        case 'k':
          // Focus sidebar for navigation
          document.getElementById('sidebar')?.focus();
          break;
        case ']':
          // Cycle to next project
          cycleNextProject();
          break;
        case '[':
          // Cycle to previous project
          cyclePrevProject();
          break;
      }
      
      // Reset leader after command
      isLeaderActive = false;
      if (leaderTimeout !== null) {
        clearTimeout(leaderTimeout);
        leaderTimeout = null;
      }
      
      return;
    }

    // Handle ESC key to close modals
    if (e.key === 'Escape') {
      if (showHelpModal) {
        showHelpModal = false;
      }
      if (showRefineModal) {
        showRefineModal = false;
      }
    }
  }

  async function createNewSection() {
    try {
      await createSection('New Section');
      // Focus sidebar after creation
      document.getElementById('sidebar')?.focus();
    } catch (error) {
      console.error('Failed to create section:', error);
    }
  }

  function cycleNextProject() {
    if (!$workspaceStore) return;
    
    const projects = $workspaceStore.projects;
    if (projects.length <= 1) return;
    
    const currentIndex = projects.findIndex(p => p.id === $workspaceStore.active_project_id);
    const nextIndex = (currentIndex + 1) % projects.length;
    switchProject(projects[nextIndex].id);
  }

  function cyclePrevProject() {
    if (!$workspaceStore) return;
    
    const projects = $workspaceStore.projects;
    if (projects.length <= 1) return;
    
    const currentIndex = projects.findIndex(p => p.id === $workspaceStore.active_project_id);
    const prevIndex = (currentIndex - 1 + projects.length) % projects.length;
    switchProject(projects[prevIndex].id);
  }

  function openRefineModal() {
    showRefineModal = true;
    refineTarget = 'topic';
  }
</script>

<div class="app-container">
  <div class="app-header">
    <Toolbar 
      bind:this={toolbarComponent} 
      {showSidebar}
      {showEditor}
      on:toggleSidebar={toggleSidebar}
      on:toggleEditor={toggleEditor}
    />
  </div>
  
  <div class="app-main">
    {#if showSidebar}
      <Sidebar id="sidebar" bind:this={sidebarComponent} />
    {/if}
    <MainEditor 
      id="main-editor"
      {showEditor}
      on:refineTopic={openRefineModal}
    />
  </div>
</div>

{#if showHelpModal}
  <HelpModal onClose={() => showHelpModal = false} />
{/if}

{#if showRefineModal}
  <RefineModal 
    target={refineTarget}
    onClose={() => showRefineModal = false}
  />
{/if}

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .app-header {
    flex-shrink: 0;
  }

  .app-main {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  #sidebar {
    outline: none;
  }

  #topic-editor,
  #merged-output {
    outline: none;
  }
</style>
