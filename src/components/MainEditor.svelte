<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { activeTopic, activeSection } from '../stores/projectStore';
  import TopicEditor from './TopicEditor.svelte';
  import SectionEditor from './SectionEditor.svelte';
  import MergedOutput from './MergedOutput.svelte';

  const dispatch = createEventDispatcher();

  export let showEditor = true;
  export let id: string = '';

  function handleRefineTopic() {
    dispatch('refineTopic');
  }

  function handleRefineSection() {
    dispatch('refineSection');
  }
</script>

<div class="main-editor" {id}>
  {#if showEditor}
    {#if $activeTopic}
      <TopicEditor on:refineTopic={handleRefineTopic} />
    {:else if $activeSection}
      <SectionEditor on:refineSection={handleRefineSection} />
    {:else}
      <TopicEditor on:refineTopic={handleRefineTopic} />
    {/if}
  {/if}
  <MergedOutput />
</div>

<style>
  .main-editor {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
</style>
