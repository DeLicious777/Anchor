<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as R from "ramda";
  import { listen } from "@tauri-apps/api/event";
  import {
    switchTask,
    interruptTask,
    returnPrevious,
    returnOriginal,
    completeTask,
    getState,
    onStateChanged,
    listTemplates,
    onTemplatesChanged,
    createTemplate,
    updateTemplate,
    deleteTemplate,
  } from "$lib/api";
  import type { StackView, TimeBlock, TaskTemplate } from "$lib/types";

  let name = $state("");
  let project = $state("");
  let client = $state("");
  let error = $state<string | null>(null);
  let nameInput: HTMLInputElement;

  let view = $state<StackView>({ active: null, stack: [], closed: [] });

  let templates = $state<TaskTemplate[]>([]);
  let templateFormName = $state("");
  let templateFormProject = $state("");
  let templateFormClient = $state("");
  let editingTemplateId = $state<string | null>(null);
  let showSuggestions = $state(false);

  let unlistenState: (() => void) | undefined;
  let unlistenFocus: (() => void) | undefined;
  let unlistenTemplates: (() => void) | undefined;

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

    templates = await listTemplates();
    unlistenTemplates = await onTemplatesChanged((updated) => {
      templates = updated;
    });
  });

  onDestroy(() => {
    unlistenState?.();
    unlistenFocus?.();
    unlistenTemplates?.();
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

  function selectTemplate(t: TaskTemplate) {
    name = t.name;
    project = t.project ?? "";
    client = t.client ?? "";
    showSuggestions = false;
  }

  function resetTemplateForm() {
    editingTemplateId = null;
    templateFormName = "";
    templateFormProject = "";
    templateFormClient = "";
  }

  function editTemplate(t: TaskTemplate) {
    editingTemplateId = t.id;
    templateFormName = t.name;
    templateFormProject = t.project ?? "";
    templateFormClient = t.client ?? "";
  }

  async function saveTemplate() {
    if (!templateFormName.trim()) return;
    error = null;
    try {
      if (editingTemplateId) {
        await updateTemplate(editingTemplateId, templateFormName.trim(), nullable(templateFormProject), nullable(templateFormClient));
      } else {
        await createTemplate(templateFormName.trim(), nullable(templateFormProject), nullable(templateFormClient));
      }
      resetTemplateForm();
    } catch (e) {
      error = String(e);
    }
  }

  async function removeTemplate(id: string) {
    error = null;
    try {
      await deleteTemplate(id);
      if (editingTemplateId === id) resetTemplateForm();
    } catch (e) {
      error = String(e);
    }
  }

  function durationLabel(block: TimeBlock): string {
    if (!block.end) return "(active)";
    const ms = new Date(block.end).getTime() - new Date(block.start).getTime();
    const minutes = Math.round(ms / 60000);
    return `${minutes} min`;
  }

  // Case-insensitive substring match on the name being typed, capped so the
  // dropdown never grows unbounded — a display concern, not a perf workaround.
  let templateSuggestions = $derived(
    name.trim().length === 0
      ? []
      : R.take(
          8,
          templates.filter((t) => t.name.toLowerCase().includes(name.trim().toLowerCase())),
        ),
  );

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
    <div class="row autocomplete">
      <input
        placeholder="Name"
        bind:value={name}
        bind:this={nameInput}
        onfocus={() => (showSuggestions = true)}
        onblur={() => (showSuggestions = false)}
      />
      <input placeholder="Project (optional)" bind:value={project} />
      <input placeholder="Client (optional)" bind:value={client} />
      {#if showSuggestions && templateSuggestions.length > 0}
        <ul class="suggestions">
          {#each templateSuggestions as t}
            <li>
              <button type="button" onmousedown={() => selectTemplate(t)}>
                {t.name}{#if t.project} · {t.project}{/if}{#if t.client} · {t.client}{/if}
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
    <div class="row">
      <button onclick={doSwitch}>Switch</button>
      <button onclick={doInterrupt}>Interrupt</button>
    </div>
  </section>

  <section>
    <h2>Task templates</h2>
    {#if templates.length === 0}
      <p>No templates yet.</p>
    {:else}
      <ul class="template-list">
        {#each templates as t}
          <li>
            <span>{t.name}{#if t.project} · {t.project}{/if}{#if t.client} · {t.client}{/if}</span>
            <span class="template-actions">
              <button type="button" onclick={() => editTemplate(t)}>Edit</button>
              <button type="button" onclick={() => removeTemplate(t.id)}>Delete</button>
            </span>
          </li>
        {/each}
      </ul>
    {/if}
    <div class="row">
      <input placeholder="Name" bind:value={templateFormName} />
      <input placeholder="Project (optional)" bind:value={templateFormProject} />
      <input placeholder="Client (optional)" bind:value={templateFormClient} />
    </div>
    <div class="row">
      <button onclick={saveTemplate}>{editingTemplateId ? "Save" : "Create"}</button>
      {#if editingTemplateId}
        <button type="button" onclick={resetTemplateForm}>Cancel</button>
      {/if}
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
  .autocomplete {
    position: relative;
  }
  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 10;
    width: 100%;
    margin: 0;
    padding: 0.25rem;
    list-style: none;
    background: white;
    border: 1px solid #ccc;
    border-radius: 4px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  .suggestions li button {
    width: 100%;
    text-align: left;
    padding: 0.4em 0.6em;
    background: none;
    border: none;
  }
  .suggestions li button:hover {
    background: #f0f0f0;
  }
  .template-list {
    list-style: none;
    padding: 0;
    margin: 0 0 1rem 0;
  }
  .template-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.3em 0;
    border-bottom: 1px solid #eee;
  }
  .template-actions {
    display: flex;
    gap: 0.5rem;
  }
</style>
