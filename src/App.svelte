<script lang="ts">
  import { onMount } from 'svelte';
  import { loadProject, activeTopicId, activeSectionId } from './stores/projectStore';
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

  onMount(async () => {
    await loadProject();
    
    // Set up global keyboard listeners
    window.addEventListener('keydown', handleGlobalKeydown);
  });

  function handleGlobalKeydown(e: KeyboardEvent) {
    // F2 to rename active topic/section
    if (e.key === 'F2') {
      e.preventDefault();
      sidebarComponent?.startEditingActive();
      return;
    }

    // Check for Ctrl+b (leader key)
    if (e.ctrlKey && e.key === 'b') {
      e.preventDefault();
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
      
      switch (e.key.toLowerCase()) {
        case 'q':
          showHelpModal = !showHelpModal;
          break;
        case 's':
          document.getElementById('sidebar')?.focus();
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
        case 'j':
        case 'k':
          // Focus sidebar for navigation
          document.getElementById('sidebar')?.focus();
          break;
      }
      
      // Reset leader after command
      isLeaderActive = false;
      if (leaderTimeout !== null) {
        clearTimeout(leaderTimeout);
        leaderTimeout = null;
      }
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

  function openRefineModal() {
    showRefineModal = true;
    refineTarget = 'topic';
  }
</script>

<div class="app-container">
  <div class="app-header">
    <Toolbar />
  </div>
  
  <div class="app-main">
    <Sidebar id="sidebar" bind:this={sidebarComponent} />
    <MainEditor 
      id="main-editor"
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
