<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TopicEditor from './TopicEditor.svelte';
  import MergedOutput from './MergedOutput.svelte';

  const dispatch = createEventDispatcher();

  export let showEditor = true;
  export let id: string = '';

  function handleRefineTopic() {
    dispatch('refineTopic');
  }
</script>

<div class="main-editor" {id}>
  {#if showEditor}
    <TopicEditor on:refineTopic={handleRefineTopic} />
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
