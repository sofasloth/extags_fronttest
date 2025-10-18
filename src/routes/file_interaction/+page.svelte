<script lang="ts">
  import { fade } from 'svelte/transition';
  import type { Snippet } from 'svelte';
	import DirectoryManager from '../../Components/DirectoryManager.svelte';

  let {onclose, children} = $props<{
    onclose: () => void;
    children: Snippet;
  }>();
  function handleBackgroundClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onclose();
    }
  }
  // 드래그 이벤트 핸들러 추가
  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
  }
  
  function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
  }
</script>

<div
  class="modal-background"
  onclick={(e) => handleBackgroundClick(e)}
  ondragover={handleDragOver}
  ondrop={handleDrop}
  ondragenter={(e) => e.preventDefault()}
  transition:fade={{duration: 100}}
  >
  <DirectoryManager/>
</div>


<style>
  .modal-background {
    position: fixed;
    top: 0;
    left: 0;

    width: 100vw;
    height: 100vh;
    margin: 0;
    padding: 0;
    background-color: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(5px);
    z-index: 100;

    display: flex;
    justify-content: center;
    align-items: center;

    pointer-events: auto;
  }

  /* .modal-background > :global(*) {
    pointer-events: auto;
  } */
</style> 