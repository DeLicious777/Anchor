<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as R from "ramda";
  import { listen } from "@tauri-apps/api/event";
  import { switchTask, interruptTask, returnPrevious, returnOriginal, completeTask, getState, onStateChanged } from "$lib/api";
  import type { StackView, TimeBlock } from "$lib/types";

  let name = $state("");
  let project = $state("");
  let client = $state("");
  let error = $state<string | null>(null);
  let nameInput: HTMLInputElement;

  let view = $state<StackView>({ active: null, stack: [], closed: [] });

  let unlistenState: (() => void) | undefined;
  let unlistenFocus: (() => void) | undefined;

  onMount(async () => {
    await refresh(getState());
    // Subsequent updates come from the event, not re-polling — this is what
    // keeps the dashboard and mini widget from ever disagreeing.
    unlistenState = await onStateChanged((updated) => {
      view = updated;
    });
    // Fired by the Switch/Interrupt global hotkeys (see lib.rs) — this window
    // is brought forward and focused there; here we just focus the input.
    unlistenFocus = await listen("focus-name-input", () => {
      nameInput?.focus();
    });
  });

  onDestroy(() => {
    unlistenState?.();
    unlistenFocus?.();
  });

  async function refresh(promise: Promise<StackView>) {
    error = null;
    try {
      view = await promise;
    } catch (e) {
      error = String(e);
    }
  }

  function nullable(s: string): string | null {
    const trimmed = s.trim();
    return trimmed.length === 0 ? null : trimmed;
  }

  function doSwitch() {
    if (!name.trim()) return;
    refresh(switchTask(name.trim(), nullable(project), nullable(client)));
  }

  function doInterrupt() {
    if (!name.trim()) return;
    refresh(interruptTask(name.trim(), nullable(project), nullable(client)));
  }

  function durationLabel(block: TimeBlock): string {
    if (!block.end) return "(active)";
    const ms = new Date(block.end).getTime() - new Date(block.start).getTime();
    const minutes = Math.round(ms / 60000);
    return `${minutes} min`;
  }

  // Most-recently-closed first — real use of Ramda, not just an installed-and-unused dependency.
  let closedMostRecentFirst = $derived(R.reverse(R.sortBy((b: TimeBlock) => b.start, view.closed)));

  // Per-task total minutes across all (possibly fragmented) closed Time Blocks
  // with the same name — a preview of what Export will later do properly.
  let totalsByName = $derived(
    R.toPairs(
      R.mapObjIndexed(
        (blocks: TimeBlock[] | undefined) =>
          R.sum(
            (blocks ?? [])
              .filter((b) => b.end)
              .map((b) => (new Date(b.end!).getTime() - new Date(b.start).getTime()) / 60000),
          ),
        R.groupBy((b: TimeBlock) => b.name, view.closed),
      ),
    ),
  );
</script>

<main class="container">
  <h1>Anchor — Interruption Stack (debug)</h1>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  <section>
    <h2>New task</h2>
    <div class="row">
      <input placeholder="Name" bind:value={name} bind:this={nameInput} />
      <input placeholder="Project (optional)" bind:value={project} />
      <input placeholder="Client (optional)" bind:value={client} />
    </div>
    <div class="row">
      <button onclick={doSwitch}>Switch</button>
      <button onclick={doInterrupt}>Interrupt</button>
    </div>
  </section>

  <section>
    <h2>Active</h2>
    {#if view.active}
      <p><strong>{view.active.name}</strong>{#if view.active.project} · {view.active.project}{/if}{#if view.active.client} · {view.active.client}{/if}</p>
      <p>started {new Date(view.active.start).toLocaleTimeString()}</p>
    {:else}
      <p>No active task.</p>
    {/if}
    <div class="row">
      <button onclick={() => refresh(returnPrevious())} disabled={view.stack.length === 0}>Return to Previous</button>
      <button onclick={() => refresh(returnOriginal())} disabled={view.stack.length === 0}>Return to Original</button>
      <button onclick={() => refresh(completeTask())} disabled={!view.active || view.stack.length > 0}>Complete</button>
    </div>
  </section>

  <section>
    <h2>Interruption stack (depth {view.stack.length})</h2>
    {#if view.stack.length === 0}
      <p>Empty.</p>
    {:else}
      <ol>
        {#each [...view.stack].reverse() as frame}
          <li>{frame.name}{#if frame.project} · {frame.project}{/if}</li>
        {/each}
      </ol>
    {/if}
  </section>

  <section>
    <h2>Completed / closed Time Blocks</h2>
    {#if closedMostRecentFirst.length === 0}
      <p>None yet.</p>
    {:else}
      <table>
        <thead>
          <tr><th>Name</th><th>Project</th><th>Client</th><th>Duration</th><th>Completion reason</th></tr>
        </thead>
        <tbody>
          {#each closedMostRecentFirst as block}
            <tr>
              <td>{block.name}</td>
              <td>{block.project ?? ""}</td>
              <td>{block.client ?? ""}</td>
              <td>{durationLabel(block)}</td>
              <td>{block.completion_reason ?? "(pending)"}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </section>

  <section>
    <h2>Totals by name (preview)</h2>
    {#if totalsByName.length === 0}
      <p>None yet.</p>
    {:else}
      <ul>
        {#each totalsByName as [taskName, minutes]}
          <li>{taskName}: {Math.round(minutes)} min</li>
        {/each}
      </ul>
    {/if}
  </section>
</main>

<style>
  .container {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  }
  section {
    margin-bottom: 2rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #ddd;
  }
  .row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  input {
    flex: 1;
    padding: 0.4em 0.6em;
  }
  button {
    padding: 0.4em 1em;
    cursor: pointer;
  }
  button:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  th, td {
    text-align: left;
    padding: 0.3em 0.6em;
    border-bottom: 1px solid #eee;
  }
  .error {
    color: #b00020;
    font-weight: 600;
  }
</style>
