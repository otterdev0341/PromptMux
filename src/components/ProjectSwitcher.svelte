<script lang="ts">
  import { workspaceStore, switchProject, createProject, deleteProject, renameProject } from '../stores/projectStore';
  
  let showDropdown = false;
  let showNewProjectModal = false;
  let showDeleteModal = false;
  let showRenameModal = false;
  
  let newProjectName = '';
  let renameProjectId = '';
  let renameProjectName = '';
  let deleteProjectId = '';
  let deleteProjectName = '';
  
  $: projects = $workspaceStore?.projects || [];
  $: activeProjectId = $workspaceStore?.active_project_id || '';
  $: activeProject = projects.find(p => p.id === activeProjectId);
  
  export function toggleDropdown() {
    showDropdown = !showDropdown;
  }
  
  function closeDropdown() {
    showDropdown = false;
  }
  
  async function handleSwitchProject(projectId: string) {
    await switchProject(projectId);
    closeDropdown();
  }
  
  export function openNewProjectModal() {
    newProjectName = '';
    showNewProjectModal = true;
    closeDropdown();
  }
  
  async function handleCreateProject() {
    if (!newProjectName.trim()) return;
    
    await createProject(newProjectName);
    showNewProjectModal = false;
  }
  
  function openRenameModal(projectId: string, currentName: string) {
    renameProjectId = projectId;
    renameProjectName = currentName;
    showRenameModal = true;
    closeDropdown();
  }
  
  async function handleRenameProject() {
    if (!renameProjectName.trim()) return;
    
    await renameProject(renameProjectId, renameProjectName);
    showRenameModal = false;
  }
  
  function openDeleteModal(projectId: string, projectName: string) {
    deleteProjectId = projectId;
    deleteProjectName = projectName;
    showDeleteModal = true;
    closeDropdown();
  }
  
  async function handleDeleteProject() {
    await deleteProject(deleteProjectId);
    showDeleteModal = false;
  }
  
  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.project-switcher')) {
      closeDropdown();
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="project-switcher">
  <button class="project-button" on:click={toggleDropdown}>
    <span class="project-icon">📁</span>
    <span class="project-name">{activeProject?.name || 'No Project'}</span>
    <span class="dropdown-arrow">{showDropdown ? '▲' : '▼'}</span>
  </button>
  
  {#if showDropdown}
    <div class="dropdown-menu">
      <div class="dropdown-header">
        <span class="dropdown-title">Projects</span>
        <button class="icon-btn" on:click={openNewProjectModal} title="Create new project">
          ➕
        </button>
      </div>
      
      <div class="projects-list">
        {#each projects as project (project.id)}
          {@const isActive = project.id === activeProjectId}
          <div
            class="project-item"
            class:active={isActive}
            on:click={() => handleSwitchProject(project.id)}
            on:dblclick={() => openRenameModal(project.id, project.name)}
          >
            <span class="project-item-icon">{isActive ? '✓' : '📄'}</span>
            <span class="project-item-name">{project.name}</span>
            <span class="project-item-count">{project.sections.length} sections</span>
            
            {#if !isActive}
              <div class="project-item-actions">
                <button
                  class="icon-btn-small"
                  on:click|stopPropagation={() => openRenameModal(project.id, project.name)}
                  title="Rename project"
                >
                  ✏️
                </button>
                <button
                  class="icon-btn-small icon-btn-danger"
                  on:click|stopPropagation={() => openDeleteModal(project.id, project.name)}
                  title="Delete project"
                >
                  🗑️
                </button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
      
      <div class="dropdown-footer">
        <button class="btn-create" on:click={openNewProjectModal}>
          ➕ New Project
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- New Project Modal -->
{#if showNewProjectModal}
  <div class="modal-backdrop" on:click={() => showNewProjectModal = false}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Create New Project</h3>
      <input
        type="text"
        class="modal-input"
        placeholder="Project name"
        bind:value={newProjectName}
        on:keydown={(e) => e.key === 'Enter' && handleCreateProject()}
        autofocus
      />
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={() => showNewProjectModal = false}>
          Cancel
        </button>
        <button class="btn btn-primary" on:click={handleCreateProject}>
          Create
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Rename Project Modal -->
{#if showRenameModal}
  <div class="modal-backdrop" on:click={() => showRenameModal = false}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Rename Project</h3>
      <input
        type="text"
        class="modal-input"
        placeholder="Project name"
        bind:value={renameProjectName}
        on:keydown={(e) => e.key === 'Enter' && handleRenameProject()}
        autofocus
      />
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={() => showRenameModal = false}>
          Cancel
        </button>
        <button class="btn btn-primary" on:click={handleRenameProject}>
          Rename
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Delete Project Modal -->
{#if showDeleteModal}
  <div class="modal-backdrop" on:click={() => showDeleteModal = false}>
    <div class="modal-content" on:click|stopPropagation>
      <h3>Confirm Delete</h3>
      <p class="delete-message">
        Are you sure you want to delete the project <strong>"{deleteProjectName}"</strong>?
      </p>
      <p class="delete-warning">⚠️ This will permanently delete all sections and topics in this project.</p>
      <div class="modal-actions">
        <button class="btn btn-secondary" on:click={() => showDeleteModal = false}>
          Cancel
        </button>
        <button class="btn btn-danger" on:click={handleDeleteProject}>
          Delete
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .project-switcher {
    position: relative;
    display: inline-block;
  }
  
  .project-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background-color: #2d3748;
    border: 1px solid #4a5568;
    border-radius: 0.375rem;
    color: #e2e8f0;
    cursor: pointer;
    transition: background-color 0.15s ease;
    font-size: 0.875rem;
  }
  
  .project-button:hover {
    background-color: #4a5568;
  }
  
  .project-icon {
    font-size: 1rem;
  }
  
  .project-name {
    flex: 1;
    font-weight: 500;
  }
  
  .dropdown-arrow {
    font-size: 0.75rem;
    color: #a0aec0;
  }
  
  .dropdown-menu {
    position: absolute;
    top: calc(100% + 0.5rem);
    left: 0;
    right: 0;
    background-color: #1a202c;
    border: 1px solid #4a5568;
    border-radius: 0.375rem;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    min-width: 300px;
    max-width: 400px;
  }
  
  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #2d3748;
  }
  
  .dropdown-title {
    font-size: 0.875rem;
    font-weight: 600;
    color: #e2e8f0;
  }
  
  .icon-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    font-size: 1rem;
    color: #a0aec0;
    transition: color 0.15s ease;
  }
  
  .icon-btn:hover {
    color: #4299e1;
  }
  
  .projects-list {
    max-height: 300px;
    overflow-y: auto;
    padding: 0.5rem 0;
  }
  
  .project-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.625rem 1rem;
    cursor: pointer;
    transition: background-color 0.15s ease;
    position: relative;
  }
  
  .project-item:hover {
    background-color: #2d3748;
  }
  
  .project-item.active {
    background-color: #2c5282;
  }
  
  .project-item-icon {
    font-size: 0.875rem;
    width: 1rem;
    text-align: center;
  }
  
  .project-item-name {
    flex: 1;
    font-size: 0.875rem;
    color: #e2e8f0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .project-item-count {
    font-size: 0.75rem;
    color: #718096;
    background-color: #2d3748;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
  }
  
  .project-item-actions {
    display: flex;
    gap: 0.25rem;
    opacity: 0;
    transition: opacity 0.15s ease;
  }
  
  .project-item:hover .project-item-actions {
    opacity: 1;
  }
  
  .icon-btn-small {
    background: transparent;
    border: none;
    cursor: pointer;
    padding: 0.25rem;
    font-size: 0.875rem;
    transition: transform 0.15s ease;
  }
  
  .icon-btn-small:hover {
    transform: scale(1.1);
  }
  
  .icon-btn-danger:hover {
    color: #fc8181;
  }
  
  .dropdown-footer {
    padding: 0.75rem 1rem;
    border-top: 1px solid #2d3748;
  }
  
  .btn-create {
    width: 100%;
    padding: 0.5rem;
    background-color: #4299e1;
    border: none;
    border-radius: 0.25rem;
    color: white;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.15s ease;
  }
  
  .btn-create:hover {
    background-color: #3182ce;
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
    z-index: 2000;
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
  
  .modal-input {
    width: 100%;
    padding: 0.625rem;
    background-color: #2d3748;
    border: 1px solid #4a5568;
    border-radius: 0.375rem;
    color: #e2e8f0;
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }
  
  .modal-input:focus {
    outline: none;
    border-color: #4299e1;
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
  
  .btn-primary {
    background-color: #4299e1;
    color: white;
  }
  
  .btn-primary:hover {
    background-color: #3182ce;
  }
  
  .btn-danger {
    background-color: #e53e3e;
    color: white;
  }
  
  .btn-danger:hover {
    background-color: #c53030;
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
</style>
