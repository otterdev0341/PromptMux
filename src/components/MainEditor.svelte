<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import TopicEditor from './TopicEditor.svelte';
  import MergedOutput from './MergedOutput.svelte';

  const dispatch = createEventDispatcher();

  function handleRefineTopic() {
    dispatch('refineTopic');
  }

  export let id: string = '';
</script>

<div class="main-editor" {id}>
  <TopicEditor on:refineTopic={handleRefineTopic} />
  <MergedOutput />
</div>

<style>
  .main-editor {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
</style>
