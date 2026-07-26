<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getState, onStateChanged } from "$lib/api";
  import { formatElapsed } from "$lib/time";
  import type { StackView } from "$lib/types";

  let view = $state<StackView>({ active: null, stack: [], closed: [] });
  let unlisten: (() => void) | undefined;
  let timerInterval: ReturnType<typeof setInterval> | undefined;

  // Ticks once a second purely to re-derive `elapsed` below.
  let now = $state(Date.now());
  let elapsed = $derived(view.active ? formatElapsed(now - new Date(view.active.start).getTime()) : null);

  onMount(async () => {
    try {
      view = await getState();
    } catch {
      // Nothing to display yet — the dashboard will surface the actual error.
    }
    unlisten = await onStateChanged((updated) => {
      view = updated;
    });

    timerInterval = setInterval(() => {
      now = Date.now();
    }, 1000);
  });

  onDestroy(() => {
    unlisten?.();
    clearInterval(timerInterval);
  });
</script>

<!--
  Display only — no controls. Per the feature doc: "not meant for rapid
  interaction... opened deliberately." Fast interaction is the hotkeys.
  `data-tauri-drag-region` is what makes the window moveable at all: with
  `decorations: false` (tauri.conf.json) there's no title bar for the OS to
  drag by, so without this attribute the window is stuck wherever it opens.
-->
<main data-tauri-drag-region>
  {#if view.active}
    <p class="name">{view.active.name}</p>
    <p class="timer">{elapsed}</p>
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
  .timer {
    margin: 0.15rem 0 0;
    font-size: 0.9rem;
    font-variant-numeric: tabular-nums;
    opacity: 0.9;
  }
  .depth {
    margin: 0.2rem 0 0;
    font-size: 0.8rem;
    opacity: 0.75;
  }
</style>
