<script lang="ts">
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';
  import { listActions, listCategories, logPath, userMogPath, type MogDetail } from './lib/api';
  import { addStep, loadMog, newPipeline, serializeMog, setCategories, setDescriptors, setTab, studio } from './lib/store.svelte';
  import { setRunSubtab } from './lib/runParams.svelte';
  import { markMogLoaded } from './lib/recentMogs.svelte';
  import { prewarmCatalog } from './lib/marketCatalog';
  import { initFileLogging, isFileLogging, setFileLogging } from './lib/log';
  import { getTheme, initTheme, setTheme, type Theme } from './lib/theme';
  import Palette from './lib/components/Palette.svelte';
  import StepCard from './lib/components/StepCard.svelte';
  import PreviewPane from './lib/components/PreviewPane.svelte';
  import BatchPane from './lib/components/BatchPane.svelte';
  import TestPanel from './lib/components/TestPanel.svelte';
  import CheckPanel from './lib/components/CheckPanel.svelte';
  import ImpactPanel from './lib/components/ImpactPanel.svelte';
  import MogBrowser from './lib/components/MogBrowser.svelte';
  import RegexPopover from './lib/components/RegexPopover.svelte';
  import Icon from './lib/components/Icon.svelte';

  let loadError = $state('');

  // Settings popover (gear in the header).
  let settingsOpen = $state(false);
  let fileLogging = $state(false);
  let logFilePath = $state('');
  let theme = $state<Theme>('auto');
  function toggleFileLogging(on: boolean): void {
    fileLogging = on;
    setFileLogging(on);
  }
  function chooseTheme(t: Theme): void {
    theme = t;
    setTheme(t);
  }

  // Load a recipe's steps into the editor, and its fixture TestInput into the
  // preview so it's ready to Transform. Fresh, untitled pipeline.
  function applyMog(detail: MogDetail): void {
    try {
      const doc = JSON.parse(detail.content);
      loadMog(doc, null);
      studio.sourceName = detail.resolvable ?? detail.name;
      if (detail.fixture_input !== null) studio.input = detail.fixture_input;
      markMogLoaded(detail.name);
      loadError = '';
    } catch (e) {
      loadError = `Could not open mog "${detail.name}": ${e}`;
    }
  }

  // Open a recipe chosen in the Browse tab (its detail is already loaded), then
  // jump to Edit to work on it.
  function openMogDetail(detail: MogDetail): void {
    if (!confirmDiscard()) return;
    applyMog(detail);
    setTab('edit');
  }

  // Load a recipe and jump straight to the Run tab to apply it across files.
  function runMogDetail(detail: MogDetail): void {
    if (!confirmDiscard()) return;
    applyMog(detail);
    setTab('run');
  }

  // Add a run_mog step calling the given recipe by name to the current pipeline.
  function insertRunMog(name: string): void {
    const desc = studio.byName['run_mog'];
    if (!desc) {
      loadError = 'The run_mog action is not available in this engine build.';
      return;
    }
    addStep(desc);
    const s = studio.mog.steps.find((x) => x._uid === studio.selectedUid);
    if (s) s.options.file = name;
    setTab('edit');
  }

  // Resizable columns: palette and editor have explicit widths; the right pane
  // fills the rest. Two dividers drag the boundaries. Widths persist.
  let paletteW = $state(230);
  let editorW = $state(420);
  let dragWhich: 'palette' | 'editor' | null = $state(null);
  let dragStartX = 0;
  let dragStartW = 0;

  onMount(async () => {
    initTheme();
    theme = getTheme();
    initFileLogging();
    fileLogging = isFileLogging();
    try {
      const saved = JSON.parse(localStorage.getItem('mog.cols') ?? 'null');
      if (typeof saved?.paletteW === 'number') paletteW = saved.paletteW;
      if (typeof saved?.editorW === 'number') editorW = saved.editorW;
    } catch {
      /* ignore malformed saved widths */
    }
    try {
      logFilePath = await logPath();
    } catch {
      /* older build without the log_path command */
    }
    try {
      setDescriptors(await listActions());
    } catch (e) {
      loadError = `Could not load actions from the mog engine: ${e}`;
    }
    // Category order/labels are a nicety; older engines lack --list-categories, so
    // failure is silent and the palette falls back to descriptor order.
    try {
      setCategories(await listCategories());
    } catch {
      /* engine without --list-categories: keep the descriptor-order fallback */
    }
    // Prewarm the marketplace catalog in the background so the first Browse open is
    // instant instead of paying the `mog market list` subprocess on the click.
    prewarmCatalog();
  });

  $effect(() => {
    localStorage.setItem('mog.cols', JSON.stringify({ paletteW, editorW }));
  });

  function startDrag(which: 'palette' | 'editor', e: PointerEvent): void {
    dragWhich = which;
    dragStartX = e.clientX;
    dragStartW = which === 'palette' ? paletteW : editorW;
    e.preventDefault();
  }
  function onDragMove(e: PointerEvent): void {
    if (!dragWhich) return;
    const w = Math.max(160, Math.min(760, dragStartW + (e.clientX - dragStartX)));
    if (dragWhich === 'palette') paletteW = w;
    else editorW = w;
  }
  function endDrag(): void {
    dragWhich = null;
  }

  function confirmDiscard(): boolean {
    if (!studio.dirty) return true;
    return window.confirm('Discard unsaved changes?');
  }

  // Basename of the saved file, or null when the pipeline has never been saved.
  function savedBasename(): string | null {
    if (!studio.currentPath) return null;
    const parts = studio.currentPath.split(/[\\/]/);
    return parts[parts.length - 1];
  }

  // Display filename shown by the centered title: the local file's basename, or the
  // marketplace recipe's basename (e.g. "factory/repo-tidy.mog" -> "repo-tidy.mog").
  function docFilename(): string | null {
    const id = studio.currentPath ?? studio.sourceName;
    if (!id) return null;
    const parts = id.split(/[\\/]/);
    return parts[parts.length - 1];
  }

  function doNew(): void {
    if (confirmDiscard()) newPipeline();
  }

  function setAllCollapsed(collapsed: boolean): void {
    for (const s of studio.mog.steps) s._collapsed = collapsed;
  }

  async function doOpen(): Promise<void> {
    if (!confirmDiscard()) return;
    const path = await open({
      multiple: false,
      title: 'Open .mog',
      filters: [{ name: 'mog', extensions: ['mog', 'json'] }],
    });
    if (typeof path !== 'string') return;
    try {
      const text = await readTextFile(path);
      const doc = JSON.parse(text);
      loadMog(doc, path);
      loadError = '';
    } catch (e) {
      loadError = `Could not open: ${e}`;
    }
  }

  async function doSave(): Promise<void> {
    if (!studio.currentPath) {
      await doSaveAs();
      return;
    }
    await writeTextFile(studio.currentPath, serializeMog());
    studio.dirty = false;
  }

  // kebab-case slug of a pipeline name, matching the mog recipe naming convention
  // (e.g. "Strip Trailing Whitespace" -> "strip-trailing-whitespace").
  function slugify(s: string): string {
    return s
      .trim()
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '');
  }

  async function doSaveAs(): Promise<void> {
    // Default into the user's mog library with a slugified filename. When a file is
    // already open, keep defaulting to its own path. Falls back to the bare name on
    // builds without the user_recipe_path command (older engine binary / no library).
    let defaultPath = studio.currentPath ?? '';
    if (!defaultPath) {
      const file = `${slugify(studio.mog.name ?? '') || 'untitled'}.mog`;
      try {
        defaultPath = (await userMogPath(file)) ?? file;
      } catch {
        defaultPath = file;
      }
    }
    const path = await save({
      title: 'Save .mog',
      defaultPath,
      filters: [{ name: 'mog', extensions: ['mog'] }],
    });
    if (typeof path !== 'string') return;
    await writeTextFile(path, serializeMog());
    studio.currentPath = path;
    studio.dirty = false;
  }
</script>

<div class="app">
  <header class="top">
    <nav class="tabs">
      <button class:active={studio.tab === 'browse'} onclick={() => setTab('browse')}><Icon name="search" />Browse</button>
      <button class:active={studio.tab === 'run'} onclick={() => setTab('run')}><Icon name="play" />Run</button>
      <button class:active={studio.tab === 'edit'} onclick={() => setTab('edit')}><Icon name="edit" />Edit</button>
    </nav>
    <span class="grow"></span>
    {#if studio.tab === 'edit' || studio.tab === 'run'}
      <div class="doc">
        <input
          class="title-in"
          type="text"
          placeholder="Untitled pipeline"
          aria-label="Pipeline name"
          size={Math.max((studio.mog.name ?? '').length, 'Untitled pipeline'.length)}
          value={studio.mog.name ?? ''}
          oninput={(e) => {
            studio.mog.name = e.currentTarget.value;
            studio.dirty = true;
          }}
        />
        {#if docFilename()}
          <span class="doc-file" title={studio.currentPath ?? studio.sourceName}>({docFilename()})</span>
        {/if}
      </div>
    {/if}
    <button
      class="ghost gear"
      title="Settings"
      aria-label="Settings"
      class:active={settingsOpen}
      onclick={() => (settingsOpen = !settingsOpen)}
    ><Icon name="settings" /></button>
  </header>

  {#if settingsOpen}
    <div class="settings-backdrop" onpointerdown={() => (settingsOpen = false)} role="presentation"></div>
    <div class="settings-panel" role="dialog" aria-label="Settings">
      <div class="sp-title">Settings</div>
      <div class="sp-field">
        <span class="sp-label">Theme</span>
        <div class="subtabs seg">
          <button class:active={theme === 'auto'} onclick={() => chooseTheme('auto')}>Auto</button>
          <button class:active={theme === 'light'} onclick={() => chooseTheme('light')}>Light</button>
          <button class:active={theme === 'dark'} onclick={() => chooseTheme('dark')}>Dark</button>
        </div>
        <span class="sp-help">Auto follows your OS setting.</span>
      </div>
      <label class="sp-row">
        <input type="checkbox" checked={fileLogging} onchange={(e) => toggleFileLogging(e.currentTarget.checked)} />
        <span>
          Write logs to a file
          <span class="sp-help">Appends console/perf logs (e.g. Marketplace timings) to a file on disk.</span>
        </span>
      </label>
      {#if logFilePath}
        <div class="sp-path" title={logFilePath}>{logFilePath}</div>
      {:else}
        <div class="sp-help">Log path unavailable, rebuild the app to enable file logging.</div>
      {/if}
    </div>
  {/if}

  {#if loadError}
    <div class="banner">{loadError}</div>
  {/if}

  {#if studio.tab === 'browse'}
    <div class="browse-wrap">
      <MogBrowser
        inline
        initialQuery=""
        onclose={() => setTab('edit')}
        onopen={openMogDetail}
        onrun={runMogDetail}
        oninsert={insertRunMog}
      />
    </div>
  {:else if studio.tab === 'run'}
    <section class="run-col">
      <div class="run-head">
        <span class="lbl">Run</span>
        <span class="count">{studio.mog.steps.length} step{studio.mog.steps.length === 1 ? '' : 's'}</span>
        <span class="grow"></span>
        <div class="subtabs">
          <button
            class:active={studio.mode === 'impact'}
            title="Dry run across your files: preview what the pipeline would change (per-file +/- and diffs). Nothing is written."
            onclick={() => setRunSubtab('impact')}><Icon name="eye" />Impact</button>
          <button
            class:active={studio.mode === 'batch'}
            title="Run the pipeline across your files for real, writing results in place, to an output directory, or as a preview."
            onclick={() => setRunSubtab('batch')}><Icon name="layers" />Batch</button>
        </div>
      </div>
      <div class="tab-body">
        {#if studio.mode === 'batch'}
          <BatchPane />
        {:else}
          <ImpactPanel />
        {/if}
      </div>
    </section>
  {:else}
    <main
      class="body"
      class:dragging={dragWhich}
      style="grid-template-columns: {paletteW}px 5px {editorW}px 5px 1fr"
    >
      <aside class="palette-col"><Palette /></aside>

      <div
        class="divider"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize palette"
        onpointerdown={(e) => startDrag('palette', e)}
      ></div>

      <section class="editor-col">
        <div class="editor-head">
          <span class="lbl">Pipeline</span>
          <div class="fileops">
            <button class="ghost fbtn" title="New" aria-label="New" onclick={doNew}><Icon name="file-plus" /></button>
            <button class="ghost fbtn" title="Open" aria-label="Open" onclick={doOpen}><Icon name="folder" /></button>
            <button class="ghost fbtn" title="Save" aria-label="Save" disabled={!studio.dirty && !!studio.currentPath} onclick={doSave}><Icon name="save" /></button>
            <button class="ghost fbtn" title="Save As" aria-label="Save As" onclick={doSaveAs}><Icon name="save-plus" /></button>
          </div>
          <span class="savestate" title={studio.currentPath ?? 'Not saved to a file yet'}>
            {#if studio.dirty}
              <span class="dot" aria-hidden="true"></span>Unsaved
            {:else if savedBasename()}
              <Icon name="check" size={13} />{savedBasename()}
            {:else}
              Not saved
            {/if}
          </span>
          <span class="grow"></span>
          {#if studio.mog.steps.length > 0}
            <button class="ghost eh" title="Collapse all steps" onclick={() => setAllCollapsed(true)}><Icon name="chevrons-up" /></button>
            <button class="ghost eh" title="Expand all steps" onclick={() => setAllCollapsed(false)}><Icon name="chevrons-down" /></button>
          {/if}
          <span class="count">{studio.mog.steps.length} step{studio.mog.steps.length === 1 ? '' : 's'}</span>
        </div>
        <div class="steps">
          {#if studio.mog.steps.length === 0}
            <p class="empty">Add actions from the palette to build a pipeline.</p>
          {:else}
            {#each studio.mog.steps as step, i (step._uid)}
              {#if step.section}
                <div class="section-hdr">{step.section}</div>
              {/if}
              {#if studio.dragUid && studio.dropUid === step._uid && studio.dropPos === 'before'}
                <div class="drop-line"></div>
              {/if}
              <StepCard {step} index={i} />
              {#if studio.dragUid && studio.dropUid === step._uid && studio.dropPos === 'after'}
                <div class="drop-line"></div>
              {/if}
            {/each}
          {/if}
        </div>
      </section>

      <div
        class="divider"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize editor"
        onpointerdown={(e) => startDrag('editor', e)}
      ></div>

      <section class="right-col">
        <div class="subtabs verify-head">
          <button class:active={studio.mode === 'preview'} onclick={() => (studio.mode = 'preview')}><Icon name="eye" />Preview</button>
          <button class:active={studio.mode === 'test'} onclick={() => (studio.mode = 'test')}><Icon name="flask" />Test</button>
          <button class:active={studio.mode === 'check'} onclick={() => (studio.mode = 'check')}><Icon name="shield" />Check</button>
        </div>
        <div class="tab-body">
          {#if studio.mode === 'test'}
            <TestPanel />
          {:else if studio.mode === 'check'}
            <CheckPanel />
          {:else}
            <PreviewPane />
          {/if}
        </div>
      </section>
    </main>
  {/if}
</div>

{#if studio.regexPopover.open}
  <RegexPopover />
{/if}

<svelte:window onpointermove={onDragMove} onpointerup={endDrag} />

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .top {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 10px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .fileops {
    display: flex;
    gap: 2px;
    margin-left: 4px;
  }
  .fbtn {
    padding: 2px 4px;
    color: var(--muted);
  }
  .fbtn:hover:not(:disabled) {
    color: var(--text);
  }
  .doc {
    /* Centered in the header regardless of the tab/gear widths on either side. */
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .title-in {
    width: auto;
    max-width: 42ch;
    font-size: 14px;
    font-weight: 600;
    color: var(--text);
    background: transparent;
    border: 1px solid transparent;
    border-radius: 6px;
    padding: 3px 7px;
    text-align: center;
  }
  .title-in::placeholder {
    color: var(--muted);
    font-weight: 400;
  }
  .doc-file {
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
    opacity: 0.8;
  }
  .title-in:hover {
    border-color: var(--border);
  }
  .title-in:focus {
    border-color: var(--accent);
    background: var(--panel);
  }
  .savestate {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--muted);
    font-size: 12px;
    white-space: nowrap;
  }
  .savestate .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--warn);
  }
  .grow {
    flex: 1;
  }
  .gear {
    margin-left: 4px;
    color: var(--muted);
    padding: 4px 6px;
  }
  .gear:hover,
  .gear.active {
    color: var(--text);
  }
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 150;
  }
  .settings-panel {
    position: fixed;
    top: 44px;
    right: 10px;
    z-index: 151;
    width: 320px;
    max-width: 92vw;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 28px color-mix(in srgb, #000 30%, transparent);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .sp-title {
    font-weight: 700;
    font-size: 13px;
  }
  .sp-field {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .sp-label {
    font-size: 13px;
    color: var(--text);
  }
  .seg {
    align-self: flex-start;
  }
  .sp-row {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
  }
  .sp-row input {
    width: auto;
    margin-top: 2px;
  }
  .sp-help {
    display: block;
    color: var(--muted);
    font-size: 11px;
    line-height: 1.4;
    margin-top: 2px;
  }
  .sp-path {
    font-family: var(--mono);
    font-size: 11px;
    color: var(--muted);
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 5px 7px;
    overflow-wrap: anywhere;
  }
  .tabs {
    display: flex;
    gap: 2px;
  }
  .tabs button {
    border-radius: 0;
    font-weight: 700;
    padding: 4px 14px;
    background: var(--tab-off);
  }
  .tabs button:hover {
    background: var(--tab-off);
  }
  .tabs button:first-child {
    border-radius: 6px 0 0 6px;
  }
  .tabs button:not(:first-child) {
    border-left: none;
  }
  .tabs button:last-child {
    border-radius: 0 6px 6px 0;
  }
  .tabs button.active {
    background: linear-gradient(rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.2)), var(--brand-grad);
    background-origin: border-box;
    background-clip: border-box;
    color: #fff;
    border-color: transparent;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9), 0 0 2px rgba(0, 0, 0, 0.8);
  }
  .tabs button.active :global(svg) {
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.9)) drop-shadow(0 0 1px rgba(0, 0, 0, 0.8));
  }
  .subtabs {
    display: flex;
    gap: 2px;
  }
  .subtabs button {
    border-radius: 0;
    background: var(--tab-off);
  }
  .subtabs button:hover {
    background: var(--tab-off);
  }
  .subtabs button:first-child {
    border-radius: 6px 0 0 6px;
  }
  .subtabs button:not(:first-child) {
    border-left: none;
  }
  .subtabs button:last-child {
    border-radius: 0 6px 6px 0;
  }
  .subtabs button.active {
    background: linear-gradient(rgba(0, 0, 0, 0.2), rgba(0, 0, 0, 0.2)), var(--brand-grad);
    background-origin: border-box;
    background-clip: border-box;
    color: #fff;
    border-color: transparent;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9), 0 0 2px rgba(0, 0, 0, 0.8);
  }
  .subtabs button.active :global(svg) {
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.9)) drop-shadow(0 0 1px rgba(0, 0, 0, 0.8));
  }
  .browse-wrap {
    flex: 1;
    min-height: 0;
    display: flex;
  }
  .run-col {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }
  .run-head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
    background: var(--panel);
  }
  .verify-head {
    padding: 6px 8px;
    border-bottom: 1px solid var(--border);
  }
  .tab-body {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .tab-body > :global(*) {
    flex: 1;
    min-height: 0;
  }
  .banner {
    background: var(--danger);
    color: #fff;
    padding: 6px 12px;
    font-size: 12px;
  }
  .body {
    display: grid;
    /* grid-template-columns is set inline from the resizable widths */
    flex: 1;
    min-height: 0;
  }
  .body.dragging {
    cursor: col-resize;
    user-select: none;
  }
  .divider {
    background: var(--border);
    cursor: col-resize;
    transition: background 0.1s;
  }
  .divider:hover,
  .body.dragging .divider {
    background: var(--accent);
  }
  .palette-col {
    min-height: 0;
  }
  .editor-col {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border-right: 1px solid var(--border);
    background: var(--bg);
  }
  .editor-head {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--border);
  }
  .editor-head .eh {
    font-size: 12px;
    color: var(--muted);
    padding: 2px 6px;
  }
  .lbl {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
  }
  .count {
    color: var(--muted);
    font-size: 12px;
  }
  .steps {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  /* Keep step cards at their natural height so the list scrolls instead of
     compressing the cards to fit. */
  .steps > :global(*) {
    flex-shrink: 0;
  }
  .section-hdr {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    font-weight: 700;
    color: var(--muted);
  }
  .section-hdr::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--border);
  }
  .steps > .section-hdr:first-child {
    margin-top: 0;
  }
  .empty {
    color: var(--muted);
    text-align: center;
    padding: 24px 12px;
  }
  .drop-line {
    height: 2px;
    margin: -5px 0;
    border-radius: 2px;
    background: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .right-col {
    min-height: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
</style>
