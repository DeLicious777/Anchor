<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getState, onStateChanged } from "$lib/api";
  import type { StackView } from "$lib/types";

  let view = $state<StackView>({ active: null, stack: [], closed: [] });
  let unlisten: (() => void) | undefined;

  onMount(async () => {
    try {
      view = await getState();
    } catch {
      // Nothing to display yet — the dashboard will surface the actual error.
    }
    unlisten = await onStateChanged((updated) => {
      view = updated;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<!--
  Display only — no controls. Per the feature doc: "not meant for rapid
  interaction... opened deliberately." Fast interaction is the hotkeys.
-->
<main>
  {#if view.active}
    <p class="name">{view.active.name}</p>
    <p class="depth">{view.stack.length > 0 ? `${view.stack.length} deep` : "no interruption"}</p>
  {:else}
    <p class="name empty">No active task</p>
  {/if}
</main>

<style>
  :global(body) {
    margin: 0;
    background: rgba(20, 20, 20, 0.92);
    color: #f0f0f0;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    -webkit-user-select: none;
    user-select: none;
  }
  main {
    padding: 0.75rem 1rem;
    display: flex;
    flex-direction: column;
    justify-content: center;
    height: 100vh;
    box-sizing: border-box;
  }
  .name {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .name.empty {
    opacity: 0.6;
    font-weight: 400;
  }
  .depth {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
    opacity: 0.75;
  }
</style>
