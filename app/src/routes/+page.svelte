<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as R from "ramda";
  import {
    startTask,
    switchTask,
    interruptTask,
    returnPrevious,
    returnOriginal,
    completeTask,
    renameActive,
    editIdentity,
    resizeBlock,
    deleteBlock,
    getState,
    onStateChanged,
    listTemplates,
    onTemplatesChanged,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    getExportSettings,
    updateExportSettings,
    chooseSaveLocation,
    exportXlsx,
    exportJson,
    getHotkeyBindings,
    updateHotkeyBindings,
  } from "$lib/api";
  import type { StackView, TimeBlock, ClosedBlock, TaskTemplate, ExportSettings, HotkeyBindings } from "$lib/types";
  import { formatElapsed, toLocalInputValue, fromLocalInputValue } from "$lib/time";

  let name = $state("");
  let project = $state("");
  let client = $state("");
  let error = $state<string | null>(null);

  let view = $state<StackView>({ active: null, stack: [], closed: [] });

  // --- History View row actions (timeline-reconstruction.md) ----------------
  // Edit Identity, Resize and Delete are row-level, so they live here rather
  // than on the Timeline Editor (#14), which owns the same corrections as
  // direct manipulation — dragging a boundary instead of typing a time.
  //
  // Resize is typed here because risk R9 must not wait on that surface: until
  // some route can correct a `system-inferred` end, a wrong inferred end is
  // uncorrectable, and R4 depends on this one being reachable. Add and Move
  // remain the Timeline Editor's, having no sensible tabular form.
  //
  // All three are thin: this collects input and calls the command. Every rule —
  // the block must exist, must not be the active one, must not have an
  // unresolved interruption pointing at it, must not overlap, must not end in
  // the future — is enforced by the domain, and its error is surfaced verbatim.
  // Duplicating those checks here would let the UI and replay drift apart.
  let editingId = $state<string | null>(null);
  // The block as it stood when editing opened. Kept whole so a save can send an
  // untouched boundary back verbatim rather than round-tripped through the
  // input, which drops sub-second precision and would record a span change the
  // user never made.
  let editOriginal = $state<ClosedBlock | null>(null);
  let editName = $state("");
  let editProject = $state("");
  let editClient = $state("");
  let editStart = $state("");
  let editEnd = $state("");
  // Deleting a Time Block destroys a billing record and MVP has no undo, so it
  // is confirmed — the persona rule reserves confirmations for exactly this.
  let pendingDeleteId = $state<string | null>(null);

  function beginEdit(block: ClosedBlock) {
    editingId = block.id;
    editOriginal = block;
    editName = block.name;
    editProject = block.project ?? "";
    editClient = block.client ?? "";
    editStart = inputValue(block.start);
    editEnd = inputValue(block.end);
    error = null;
  }

  /** A closed block always has an end; the shared `TimeBlock` type allows null. */
  function inputValue(instant: string | null): string {
    return instant ? toLocalInputValue(instant) : "";
  }

  /**
   * Whether this block's span is fixed — `timeline-reconstruction.md`'s
   * identity-only tier. Read straight off the backend's canonical projection,
   * never recomputed from the stack, so the affordance cannot drift from the
   * rule the domain actually enforces. The active block is not a case here: it
   * never appears among the closed blocks this table renders.
   */
  function spanLocked(block: ClosedBlock): boolean {
    return block.derived_interruption_status === "pending";
  }

  function cancelEdit() {
    editingId = null;
    editOriginal = null;
  }

  /**
   * Saves whatever the user actually changed, as one or two transitions.
   *
   * **Times go first.** `resize_block` is the call the domain can refuse — an
   * overlap, an end in the future, a block an unresolved interruption still
   * points at — so attempting it before the identity edit means a rejected
   * correction leaves the block completely untouched instead of half-saved
   * under a new name. The reverse order has no such guarantee, and the
   * asymmetry is real rather than a coin flip: `EditIdentity`'s preconditions
   * (block exists, is not active) are a strict subset of `Resize`'s, so an
   * identity edit cannot fail once a resize on the same block has succeeded.
   *
   * **Each half is skipped when nothing in it changed.** Both are recorded
   * facts, and both mark the block's capture origin adjusted, so issuing either
   * for an untouched field would write a correction into the log that never
   * happened.
   */
  async function saveEdit() {
    if (!editingId || !editOriginal) return;
    const target = editingId;
    const original = editOriginal;

    // Compared in input format, not as instants: the field carries seconds
    // while the stored value carries more, so comparing the parsed instants
    // would report every untouched span as edited.
    const startEdited = editStart !== inputValue(original.start);
    const endEdited = editEnd !== inputValue(original.end);
    const identityEdited =
      editName !== original.name ||
      nullable(editProject) !== original.project ||
      nullable(editClient) !== original.client;

    try {
      if (startEdited || endEdited) {
        // Only the edited boundary is re-derived from its input; the other is
        // the stored instant itself, to the precision it was recorded at.
        const start = startEdited ? fromLocalInputValue(editStart) : original.start;
        const end = endEdited ? fromLocalInputValue(editEnd) : original.end;
        if (!start || !end) {
          error = "A Time Block needs both a start and an end time.";
          return;
        }
        view = await resizeBlock(target, start, end);
      }
      if (identityEdited) {
        view = await editIdentity(target, editName, nullable(editProject), nullable(editClient));
      }
      cancelEdit();
      error = null;
    } catch (e) {
      error = String(e);
    }
  }

  async function confirmDelete() {
    if (!pendingDeleteId) return;
    try {
      view = await deleteBlock(pendingDeleteId);
      pendingDeleteId = null;
      error = null;
    } catch (e) {
      error = String(e);
      pendingDeleteId = null;
    }
  }

  let templates = $state<TaskTemplate[]>([]);
  let templateFormName = $state("");
  let templateFormProject = $state("");
  let templateFormClient = $state("");
  let editingTemplateId = $state<string | null>(null);
  let showSuggestions = $state(false);

  let exportSettings = $state<ExportSettings>({ rounding_enabled: true, rounding_interval_minutes: 15 });
  let rangePreset = $state<"today" | "this-week" | "custom">("today");
  let customStart = $state("");
  let customEnd = $state("");
  let exportMessage = $state<string | null>(null);

  let activeTab = $state<"dashboard" | "settings">("dashboard");
  let hotkeyForm = $state<HotkeyBindings>({
    switch: "",
    interrupt: "",
    return_previous: "",
    return_original: "",
    complete: "",
  });
  let hotkeyError = $state<string | null>(null);
  let hotkeySaved = $state(false);

  // Renames the currently active task in place (e.g. giving an auto-named
  // "Anchor N" task a real name, or retargeting it to an existing template
  // or past task). Synced from `view.active` whenever the active task's
  // identity changes (see the $effect below), so opening this always starts
  // from the task's current values rather than stale leftovers.
  let renameName = $state("");
  let renameProject = $state("");
  let renameClient = $state("");
  let renameError = $state<string | null>(null);
  let showRenameSuggestions = $state(false);
  let syncedActiveId = $state<string | null>(null);

  // Ticks once a second purely to re-derive the active-task timer below — never
  // sent anywhere, never persisted, just drives the live elapsed-time display.
  let now = $state(Date.now());

  let unlistenState: (() => void) | undefined;
  let unlistenTemplates: (() => void) | undefined;
  let timerInterval: ReturnType<typeof setInterval> | undefined;

  onMount(async () => {
    await refresh(getState());
    // Subsequent updates come from the event, not re-polling — this is what
    // keeps the dashboard and mini widget from ever disagreeing.
    unlistenState = await onStateChanged((updated) => {
      view = updated;
    });

    templates = await listTemplates();
    unlistenTemplates = await onTemplatesChanged((updated) => {
      templates = updated;
    });

    exportSettings = await getExportSettings();
    hotkeyForm = await getHotkeyBindings();

    timerInterval = setInterval(() => {
      now = Date.now();
    }, 1000);
  });

  onDestroy(() => {
    unlistenState?.();
    unlistenTemplates?.();
    clearInterval(timerInterval);
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

  // A blank name is allowed here — the backend auto-assigns "Anchor N" when
  // given one (see `commands::name_or_default`), same as the hotkeys.
  //
  // Start and Switch are separate commands with separate preconditions (ADR
  // 0005); `switch` no longer silently means Start when nothing is active. The
  // choice is made here, in the presentation layer, from live state — the same
  // `view.active` that labels the button.
  function doStartOrSwitch() {
    const args = [name.trim(), nullable(project), nullable(client)] as const;
    refresh(view.active ? switchTask(...args) : startTask(...args));
  }

  function doInterrupt() {
    refresh(interruptTask(name.trim(), nullable(project), nullable(client)));
  }

  // Re-syncs the rename form whenever the active task actually changes
  // identity (a new Time Block started) — not on every field edit the user
  // makes to it, and not while nothing is active.
  $effect(() => {
    if (view.active && view.active.id !== syncedActiveId) {
      renameName = view.active.name;
      renameProject = view.active.project ?? "";
      renameClient = view.active.client ?? "";
      renameError = null;
      syncedActiveId = view.active.id;
    } else if (!view.active) {
      syncedActiveId = null;
    }
  });

  async function saveRename() {
    if (!view.active || !renameName.trim()) return;
    renameError = null;
    try {
      view = await renameActive(renameName.trim(), nullable(renameProject), nullable(renameClient));
    } catch (e) {
      renameError = String(e);
    }
  }

  function selectRenameSuggestion(s: { name: string; project: string | null; client: string | null }) {
    renameName = s.name;
    renameProject = s.project ?? "";
    renameClient = s.client ?? "";
    showRenameSuggestions = false;
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

  // Local-time range boundaries, converted to UTC ISO strings at the backend
  // call boundary — the backend only ever deals in two concrete timestamps,
  // never in the notion of "today" or "this week" itself.
  function startOfLocalDay(d: Date): Date {
    return new Date(d.getFullYear(), d.getMonth(), d.getDate());
  }

  function addDays(d: Date, days: number): Date {
    const r = new Date(d);
    r.setDate(r.getDate() + days);
    return r;
  }

  // "This Week" starts Monday (ISO week) — Date.getDay() is 0=Sunday..6=Saturday.
  function mostRecentMonday(d: Date): Date {
    const daysSinceMonday = (d.getDay() + 6) % 7;
    return addDays(startOfLocalDay(d), -daysSinceMonday);
  }

  function resolveRange(): { start: Date; end: Date } | null {
    if (rangePreset === "today") {
      const start = startOfLocalDay(new Date());
      return { start, end: addDays(start, 1) };
    }
    if (rangePreset === "this-week") {
      const start = mostRecentMonday(new Date());
      return { start, end: addDays(start, 7) };
    }
    if (!customStart || !customEnd) return null;
    const start = new Date(`${customStart}T00:00:00`);
    const end = addDays(new Date(`${customEnd}T00:00:00`), 1); // inclusive end day
    return { start, end };
  }

  async function saveHotkeys() {
    hotkeyError = null;
    hotkeySaved = false;
    try {
      hotkeyForm = await updateHotkeyBindings(hotkeyForm);
      hotkeySaved = true;
    } catch (e) {
      hotkeyError = String(e);
    }
  }

  async function onRoundingChange() {
    error = null;
    try {
      exportSettings = await updateExportSettings(exportSettings.rounding_enabled, exportSettings.rounding_interval_minutes);
    } catch (e) {
      error = String(e);
    }
  }

  async function doExport(kind: "xlsx" | "json") {
    error = null;
    exportMessage = null;
    const range = resolveRange();
    if (!range) {
      error = "Select a valid custom date range.";
      return;
    }
    try {
      const suggested = `anchor-export-${new Date().toISOString().slice(0, 10)}.${kind}`;
      const path = await chooseSaveLocation(suggested, kind);
      if (!path) return; // user cancelled the save dialog

      const rangeStart = range.start.toISOString();
      const rangeEnd = range.end.toISOString();
      if (kind === "xlsx") {
        await exportXlsx(path, rangeStart, rangeEnd, exportSettings.rounding_enabled, exportSettings.rounding_interval_minutes);
      } else {
        await exportJson(path, rangeStart, rangeEnd, exportSettings.rounding_enabled, exportSettings.rounding_interval_minutes);
      }
      exportMessage = `Exported to ${path}`;
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

  // Wall-clock time in the user's locale. The stored value is UTC; the History
  // View is read by a person recalling their own day, so it shows local time.
  const timeOfDay = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });

  function clockLabel(instant: string | null): string {
    return instant ? timeOfDay.format(new Date(instant)) : "—";
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

  // Re-derives every second as `now` ticks — the only reason `now` exists.
  let activeElapsed = $derived(view.active ? formatElapsed(now - new Date(view.active.start).getTime()) : null);

  // Distinct (name, project, client) combos actually used in the timeline —
  // the "past task" half of the rename picker's "Both" sources, alongside
  // Task Templates. Ramda's uniqBy, not a hand-rolled Set/Map dedup.
  //
  // The key is a JSON array rather than delimiter-joined text. A delimiter
  // has to be a character no field can contain, and the previous choice —
  // NUL — was correct on that count and made git classify this whole file
  // as binary: no textual diffs and no line-level merges, on the largest
  // file in the project. `JSON.stringify` needs no such character, so the
  // ambiguity it was guarding against cannot arise at all.
  //
  // `?? ""` is kept deliberately: it preserves the existing equivalence
  // classes, where an absent project and an empty one are the same task.
  let historyEntries = $derived(
    R.uniqBy((b: TimeBlock) => JSON.stringify([b.name, b.project ?? "", b.client ?? ""]), view.closed),
  );

  // Combined rename suggestions: saved templates and raw task history,
  // visually tagged by source since they can otherwise look identical (a
  // template and a past task can share the same name/project/client).
  let renameSuggestions = $derived(
    renameName.trim().length === 0
      ? []
      : R.take(
          8,
          [
            ...templates.map((t) => ({ name: t.name, project: t.project, client: t.client, source: "template" as const })),
            ...historyEntries.map((b) => ({ name: b.name, project: b.project, client: b.client, source: "history" as const })),
          ].filter((s) => s.name.toLowerCase().includes(renameName.trim().toLowerCase())),
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

  <nav class="tabs">
    <button type="button" class:active={activeTab === "dashboard"} onclick={() => (activeTab = "dashboard")}>
      Dashboard
    </button>
    <button type="button" class:active={activeTab === "settings"} onclick={() => (activeTab = "settings")}>
      Settings
    </button>
  </nav>

  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if activeTab === "settings"}
    <section>
      <h2>Hotkeys</h2>
      {#if hotkeyError}
        <p class="error">{hotkeyError}</p>
      {/if}
      {#if hotkeySaved}
        <p class="success">Saved — new bindings are live.</p>
      {/if}
      <div class="hotkey-row">
        <label for="hotkey-switch">Switch</label>
        <input id="hotkey-switch" bind:value={hotkeyForm.switch} oninput={() => (hotkeySaved = false)} />
      </div>
      <div class="hotkey-row">
        <label for="hotkey-interrupt">Interrupt</label>
        <input id="hotkey-interrupt" bind:value={hotkeyForm.interrupt} oninput={() => (hotkeySaved = false)} />
      </div>
      <div class="hotkey-row">
        <label for="hotkey-return-previous">Return to Previous</label>
        <input id="hotkey-return-previous" bind:value={hotkeyForm.return_previous} oninput={() => (hotkeySaved = false)} />
      </div>
      <div class="hotkey-row">
        <label for="hotkey-return-original">Return to Original</label>
        <input id="hotkey-return-original" bind:value={hotkeyForm.return_original} oninput={() => (hotkeySaved = false)} />
      </div>
      <div class="hotkey-row">
        <label for="hotkey-complete">Complete</label>
        <input id="hotkey-complete" bind:value={hotkeyForm.complete} oninput={() => (hotkeySaved = false)} />
      </div>
      <p class="hint">
        Accelerator format, e.g. <code>Ctrl+Alt+S</code>. All five are applied together — if any one of them can't be
        registered (invalid format, or already bound to another app), nothing changes and the previous bindings stay
        active.
      </p>
      <div class="row">
        <button onclick={saveHotkeys}>Save</button>
      </div>
    </section>
  {/if}

  {#if activeTab === "dashboard"}
  <section>
    <h2>New task</h2>
    <div class="row autocomplete">
      <input
        placeholder="Name"
        bind:value={name}
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
      <!-- Only ever offer transitions whose precondition currently holds.
           `Start` requires nothing active, `Switch` and `Interrupt` require
           something active — the domain rejects the others with NoActiveTask /
           AlreadyActive. Showing a button that can only fail teaches the user
           that Anchor's buttons are unreliable, when in fact the state model is
           doing its job.

           This is presentation deciding from live state, not the domain
           branching — the same split ADR 0005 made when `switch` stopped
           silently meaning `Start`. The buttons narrow what is *offered*; they
           do not enforce anything, and the commands stay independently
           callable and independently validated. -->
      <button onclick={doStartOrSwitch}>{view.active ? "Switch" : "Start"}</button>
      {#if view.active}
        <button onclick={doInterrupt}>Interrupt</button>
      {/if}
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
    <h2>Export</h2>
    {#if exportMessage}
      <p class="success">{exportMessage}</p>
    {/if}
    <div class="row">
      <label><input type="radio" name="range-preset" value="today" bind:group={rangePreset} /> Today</label>
      <label><input type="radio" name="range-preset" value="this-week" bind:group={rangePreset} /> This Week</label>
      <label><input type="radio" name="range-preset" value="custom" bind:group={rangePreset} /> Custom</label>
    </div>
    {#if rangePreset === "custom"}
      <div class="row">
        <input type="date" bind:value={customStart} />
        <input type="date" bind:value={customEnd} />
      </div>
    {/if}
    <div class="row">
      <label>
        <input type="checkbox" bind:checked={exportSettings.rounding_enabled} onchange={onRoundingChange} />
        Round durations
      </label>
      <input
        type="number"
        min="1"
        step="1"
        style="max-width: 6rem"
        bind:value={exportSettings.rounding_interval_minutes}
        onchange={onRoundingChange}
        disabled={!exportSettings.rounding_enabled}
      />
      <span>minutes</span>
    </div>
    <div class="row">
      <button onclick={() => doExport("xlsx")}>Export XLSX</button>
      <button onclick={() => doExport("json")}>Export JSON</button>
    </div>
  </section>

  <section>
    <h2>Active</h2>
    {#if view.active}
      <p><strong>{view.active.name}</strong>{#if view.active.project} · {view.active.project}{/if}{#if view.active.client} · {view.active.client}{/if}</p>
      <p>started {new Date(view.active.start).toLocaleTimeString()}</p>
      <p class="timer">{activeElapsed}</p>

      {#if renameError}
        <p class="error">{renameError}</p>
      {/if}
      <div class="row autocomplete">
        <input
          placeholder="Name"
          bind:value={renameName}
          onfocus={() => (showRenameSuggestions = true)}
          onblur={() => (showRenameSuggestions = false)}
        />
        <input placeholder="Project (optional)" bind:value={renameProject} />
        <input placeholder="Client (optional)" bind:value={renameClient} />
        {#if showRenameSuggestions && renameSuggestions.length > 0}
          <ul class="suggestions">
            {#each renameSuggestions as s}
              <li>
                <button type="button" onmousedown={() => selectRenameSuggestion(s)}>
                  {s.name}{#if s.project} · {s.project}{/if}{#if s.client} · {s.client}{/if}
                  <span class="suggestion-source">{s.source === "template" ? "template" : "history"}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
      <div class="row">
        <button onclick={saveRename} disabled={!renameName.trim()}>Rename</button>
      </div>
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
      <div class="table-scroll">
      <table>
        <thead>
          <!-- "End" is the time; "End source" is how that time was established.
               These were previously one column headed "End time" that actually
               rendered the determination — so the times themselves, which
               `interruption-stack.md` requires the History View to show, were
               absent while the header claimed otherwise. -->
          <tr><th>Name</th><th>Project</th><th>Client</th><th>Start</th><th>End</th><th>Duration</th><th>End source</th><th>Capture</th><th>Interruption</th><th></th></tr>
        </thead>
        <tbody>
          {#each closedMostRecentFirst as block}
            <tr>
              <td>{block.name}</td>
              <td>{block.project ?? ""}</td>
              <td>{block.client ?? ""}</td>
              <td class="clock">{clockLabel(block.start)}</td>
              <td class="clock">{clockLabel(block.end)}</td>
              <td>{durationLabel(block)}</td>
              <!-- Inferred ends must stay visually distinct from user-determined
                   ones — never silently folded together (interruption-stack.md). -->
              <td class:inferred={block.end_determination === "system-inferred"}>
                {block.end_determination === "system-inferred" ? "inferred" : "exact"}
              </td>
              <td>{block.capture_origin}</td>
              <!-- The backend's canonical projection, never `interruption_outcome`
                   directly: absent is ambiguous, and no view may reinterpret it. -->
              <td>{block.derived_interruption_status}</td>
              <td class="actions">
                <button type="button" onclick={() => beginEdit(block)}>Edit</button>
                <button type="button" onclick={() => (pendingDeleteId = block.id)}>Delete</button>
              </td>
            </tr>
            {#if pendingDeleteId === block.id}
              <!-- Its own full-width row, not inline in the actions cell: inline
                   it more than doubled that column's width and could be pushed
                   off-screen at the dashboard's default 800x600, so the user
                   could click Delete without ever reading the question. -->
              <tr class="confirm-row">
                <td colspan="10">
                  <strong>Delete “{block.name}” permanently?</strong>
                  This removes it from the timeline and from every future export.
                  <em>There is no undo</em> — recovering it means adding it again by hand.
                  <button type="button" class="danger" onclick={confirmDelete}>Delete permanently</button>
                  <button type="button" onclick={() => (pendingDeleteId = null)}>Cancel</button>
                </td>
              </tr>
            {/if}
            {#if editingId === block.id}
              <tr class="edit-row">
                <td colspan="10">
                  <!-- The three Rename fields on a block that has already
                       finished, plus its two boundaries. No client-side
                       validation: the domain rejects an edit to the active
                       block, an unknown id, an overlapping span, an end in the
                       future, and a reshape of a block an unresolved
                       interruption still points at — each error surfaced above
                       rather than pre-empted here.

                       `step="1"` is load-bearing, not decoration: a
                       datetime-local input defaults to minute granularity and
                       would silently round a correction the user typed to the
                       second. -->
                  <label>Name <input bind:value={editName} /></label>
                  <label>Project <input bind:value={editProject} /></label>
                  <label>Client <input bind:value={editClient} /></label>
                  <!-- The tier rule, read from the projection and never
                       recomputed (`timeline-editor.md` decision 7): a block an
                       unresolved interruption still points at is identity-only,
                       so its boundaries are shown but not editable. Offering
                       them would invite effort into a commit the domain is
                       certain to refuse with `BlockReferencedByOpenFrame`.

                       `disabled`, not `readonly`: the readonly attribute does
                       not apply to `datetime-local` and would be silently
                       ignored. -->
                  <label>Start <input type="datetime-local" step="1" bind:value={editStart} disabled={spanLocked(block)} /></label>
                  <label>End <input type="datetime-local" step="1" bind:value={editEnd} disabled={spanLocked(block)} /></label>
                  {#if spanLocked(block)}
                    <p class="hint">
                      An unresolved interruption still points at this block, so its times
                      are fixed until that interruption is resumed or dismissed. Its name,
                      project and client can still be corrected.
                    </p>
                  {/if}
                  {#if block.end_determination === "system-inferred" && !spanLocked(block)}
                    <!-- The R9/R4 path stated where it is acted on: this end was
                         never observed, and the user is the only one who can say
                         what it was. -->
                    <p class="hint">
                      This end was inferred after a gap, not observed — accurate only to
                      about a minute. Correcting it records it as exact.
                    </p>
                  {/if}
                  <button type="button" onclick={saveEdit}>Save</button>
                  <button type="button" onclick={cancelEdit}>Cancel</button>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      </div>
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
  {/if}
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
  .tabs {
    display: flex;
    gap: 0.25rem;
    margin-bottom: 1.5rem;
    border-bottom: 1px solid #ddd;
  }
  .tabs button {
    padding: 0.5em 1em;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    cursor: pointer;
    font-size: 1rem;
  }
  .tabs button.active {
    border-bottom-color: #396cd8;
    font-weight: 600;
  }
  .hotkey-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }
  .hotkey-row label {
    width: 12rem;
    flex-shrink: 0;
  }
  .hotkey-row input {
    flex: 1;
    padding: 0.4em 0.6em;
  }
  .hint {
    color: #666;
    font-size: 0.9rem;
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
  /* An inferred end time is accurate only to roughly the heartbeat interval,
     and `interruption-stack.md` requires it be surfaced distinctly rather than
     folded in with user-determined ends. Deliberately not colour-only — the
     italic carries the same signal without depending on colour perception. */
  td.inferred {
    font-style: italic;
    color: #8a6d00;
  }
  /* Scanning a column of times is the History View's main reading task, so the
     digits must line up (visual-redesign.md's monospace-timestamps rule). */
  td.actions {
    white-space: nowrap;
  }
  .table-scroll {
    overflow-x: auto;
  }
  tr.confirm-row td {
    background: #fff6f6;
    padding: 0.6rem 0.5rem;
  }
  tr.confirm-row button {
    margin-left: 0.5rem;
  }
  button.danger {
    color: #b00020;
    font-weight: 600;
  }
  tr.edit-row label {
    margin-right: 0.75rem;
  }
  td.clock {
    font-variant-numeric: tabular-nums;
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  .error {
    color: #b00020;
    font-weight: 600;
  }
  .timer {
    font-variant-numeric: tabular-nums;
    font-size: 1.5rem;
    font-weight: 700;
    margin: 0.25rem 0;
  }
  .success {
    color: #0a7a2f;
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
  .suggestion-source {
    float: right;
    color: #999;
    font-size: 0.8rem;
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
