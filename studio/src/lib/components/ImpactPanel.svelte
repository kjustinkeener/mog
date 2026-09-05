<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { diffFile, runBatchReport, type BatchImpact, type FileImpact } from '../api';
  import { serializeMog, studio } from '../store.svelte';
  import { runParams } from '../runParams.svelte';
  import Icon from './Icon.svelte';
  import RunGuide from './RunGuide.svelte';

  // Persisted params (survive subtab switches + restarts); see runParams.svelte.ts.
  const p = runParams.impact;

  let running = $state(false);
  let report = $state<BatchImpact | null>(null);
  let error = $state('');
  let expanded = $state<Record<string, boolean>>({});
  // Per-file diffs are fetched lazily on expand (the --json report carries counts,
  // not diffs). '' = not fetched, string = diff text, 'ERROR: …' surfaced inline.
  let diffs = $state<Record<string, string>>({});
  let onlyChanged = $state(true);
  // Reopen the built-in guide after a run has produced results.
  let guideOpen = $state(false);

  const stepCount = $derived(studio.mog.steps.length);

  // Split a path into its directory prefix (with trailing separator) and the filename,
  // so the row can bold the filename and left-truncate only the directory part.
  function splitPath(p: string): { dir: string; sep: string; file: string } {
    const i = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
    if (i === -1) return { dir: '', sep: '', file: p };
    return { dir: p.slice(0, i), sep: p[i], file: p.slice(i + 1) };
  }

  function lines(s: string): string[] {
    return s
      .split('\n')
      .map((x) => x.trim())
      .filter((x) => x.length > 0);
  }

  const shown = $derived(
    report ? report.files.filter((f) => !onlyChanged || f.changed || f.error) : [],
  );

  type Field = 'inputs' | 'excludes';
  function append(field: Field, add: string): void {
    if (!add) return;
    if (field === 'inputs') p.inputs = p.inputs ? `${p.inputs}\n${add}` : add;
    else p.excludes = p.excludes ? `${p.excludes}\n${add}` : add;
  }

  async function addFiles(field: Field): Promise<void> {
    const picked = await open({ multiple: true, title: 'Add files' });
    const paths = Array.isArray(picked) ? picked : typeof picked === 'string' ? [picked] : [];
    append(field, paths.join('\n'));
  }

  // Pick folders; each is added as a `<folder>/*` glob (files directly in it).
  async function addFolders(field: Field): Promise<void> {
    const picked = await open({ directory: true, multiple: true, title: 'Add a folder (matched as folder/*)' });
    const dirs = Array.isArray(picked) ? picked : typeof picked === 'string' ? [picked] : [];
    append(field, dirs.map((d) => `${d.replace(/[\\/]+$/, '')}${d.includes('\\') ? '\\' : '/'}*`).join('\n'));
  }

  async function run(): Promise<void> {
    if (running) return;
    running = true;
    error = '';
    report = null;
    expanded = {};
    diffs = {};
    guideOpen = false;
    try {
      const inputs = lines(p.inputs);
      if (inputs.length === 0) {
        error = 'Add at least one input file or glob.';
        return;
      }
      report = await runBatchReport(
        serializeMog(),
        inputs,
        p.excludesEnabled ? lines(p.excludes) : [],
        p.jobs.trim() ? Number(p.jobs) : null,
      );
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }

  async function toggle(f: FileImpact): Promise<void> {
    if (!f.changed || f.error) return;
    const nowOpen = !expanded[f.path];
    expanded = { ...expanded, [f.path]: nowOpen };
    // Fetch the diff on first expand.
    if (nowOpen && diffs[f.path] === undefined) {
      diffs = { ...diffs, [f.path]: '' }; // mark loading
      try {
        const d = await diffFile(serializeMog(), f.path);
        diffs = { ...diffs, [f.path]: d || '(no textual diff)' };
      } catch (e) {
        diffs = { ...diffs, [f.path]: `ERROR: ${String(e)}` };
      }
    }
  }
</script>

<div class="impact">
  <div class="controls">
    <div class="col">
      <div class="inputs-head">
        <span class="cap">Input Files (one path or glob per line, e.g. <span class="mono">src/**/*.sql</span>)</span>
        <button class="ghost" title="Add files…" aria-label="Add input files" onclick={() => addFiles('inputs')}><Icon name="file-plus" /></button>
        <button class="ghost" title="Add folder…" aria-label="Add input folder" onclick={() => addFolders('inputs')}><Icon name="folder" /></button>
      </div>
      <textarea rows="3" bind:value={p.inputs} spellcheck="false"></textarea>

      <label class="excl-toggle"><input type="checkbox" bind:checked={p.excludesEnabled} /> Exclude some files</label>
      {#if p.excludesEnabled}
        <div class="inputs-head">
          <span class="cap">Excluded Files (one per line)</span>
          <button class="ghost" title="Exclude files…" aria-label="Exclude files" onclick={() => addFiles('excludes')}><Icon name="file-plus" /></button>
          <button class="ghost" title="Exclude folder…" aria-label="Exclude folder" onclick={() => addFolders('excludes')}><Icon name="folder" /></button>
        </div>
        <textarea rows="2" bind:value={p.excludes} spellcheck="false"></textarea>
      {/if}
    </div>
    <div class="col optcol">
      <span class="cap">Options</span>
      <label class="jobs" title="Number of files processed in parallel. Blank = auto (one per CPU core); 1 = sequential.">Parallel files <input type="number" bind:value={p.jobs} placeholder="auto" /></label>
      <button class="run" disabled={running || stepCount === 0} onclick={run}>
        <Icon name="play" />{running ? 'Scanning…' : 'Preview impact'}
      </button>
    </div>
  </div>

  <div class="results">
    {#if guideOpen || (!report && !error)}
      <RunGuide variant="impact" ondismiss={report || error ? () => (guideOpen = false) : undefined} />
    {:else if error}
      <pre class="err">{error}</pre>
    {:else if report}
      <div class="summary">
        <span class="stat"><b>{report.total}</b> scanned</span>
        <span class="stat changed"><b>{report.changed}</b> would change</span>
        {#if report.errors > 0}<span class="stat bad"><b>{report.errors}</b> errors</span>{/if}
        <span class="grow"></span>
        <button class="guide-btn" title="Show the guide" onclick={() => (guideOpen = true)}><Icon name="target" />Guide</button>
        <label class="toggle"><input type="checkbox" bind:checked={onlyChanged} /> Changed only</label>
      </div>
      {#if shown.length === 0}
        <p class="hint">No files would change. (Dry run: nothing was written.)</p>
      {/if}
      <ul class="files">
        {#each shown as f (f.path)}
          {@const sp = splitPath(f.path)}
          <li class="file">
            <button class="frow" onclick={() => toggle(f)} class:clickable={f.changed && !f.error}>
              <span class="badge" class:on={f.changed} class:bad={!!f.error}>
                {f.error ? 'error' : f.changed ? 'changed' : 'same'}
              </span>
              <span class="path" title={f.path}><span class="dir">{sp.dir}</span><span class="sep">{sp.sep}</span><span class="file">{sp.file}</span></span>
              {#if f.changed}<span class="delta"><span class="add">+{f.lines_added}</span> <span class="rem">-{f.lines_removed}</span></span>{/if}
              {#if f.changed && !f.error}<span class="caret">{expanded[f.path] ? '▾' : '▸'}</span>{/if}
            </button>
            {#if f.error}
              <pre class="ferr">{f.error}</pre>
            {:else if expanded[f.path]}
              <pre class="fdiff">{diffs[f.path] === '' ? 'Loading diff…' : diffs[f.path]}</pre>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .impact {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 10px;
    gap: 10px;
    overflow: hidden;
  }
  .controls {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .optcol {
    padding: 4px 9px;
  }
  .inputs-head {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .inputs-head .cap {
    flex: 1;
  }
  .excl-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
    font-size: 12.5px;
    margin-top: 2px;
  }
  .excl-toggle input {
    width: auto;
  }
  .jobs {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }
  .jobs input {
    width: 70px;
  }
  .run {
    align-self: flex-start;
    margin-top: 4px;
  }
  .results {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    padding: 8px;
  }
  .summary {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 2px 4px 8px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 6px;
  }
  .stat {
    color: var(--muted);
    font-size: 12.5px;
  }
  .stat b {
    color: var(--text);
  }
  .stat.changed b {
    color: var(--accent);
  }
  .stat.bad b {
    color: var(--danger);
  }
  .grow {
    flex: 1;
  }
  .guide-btn {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    padding: 3px 8px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
    font-size: 12.5px;
  }
  .toggle input {
    width: auto;
  }
  .files {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .frow {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    padding: 4px 4px;
    color: var(--text);
    font-family: var(--mono);
    font-size: 12px;
  }
  .frow.clickable {
    cursor: pointer;
  }
  .frow.clickable:hover {
    background: var(--panel-2);
  }
  .badge {
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.3px;
    padding: 1px 6px;
    border-radius: 10px;
    border: 1px solid var(--border);
    color: var(--muted);
    min-width: 6ch;
    text-align: center;
  }
  .badge.on {
    color: var(--accent);
    border-color: var(--accent-weak);
  }
  .badge.bad {
    color: var(--danger);
  }
  .path {
    /* Take the free space and truncate from the LEFT so the filename stays visible;
       the delta/caret stay pinned to the right instead of being pushed off. */
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    overflow: hidden;
  }
  .path .dir {
    min-width: 0;
    flex: 0 1 auto;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl; /* clip + ellipsis on the left edge */
    text-align: left;
    color: var(--muted);
  }
  .path .sep {
    flex: none;
    color: var(--muted);
  }
  .path .file {
    flex: none;
    white-space: nowrap;
    font-weight: 700;
    color: var(--text);
  }
  .delta .add {
    color: var(--ok);
  }
  .delta .rem {
    color: var(--danger);
  }
  .caret {
    color: var(--muted);
    min-width: 1.5ch;
  }
  .fdiff,
  .ferr {
    margin: 0;
    padding: 6px 8px;
    background: var(--panel-2);
    border-bottom: 1px solid var(--border);
    overflow: auto;
    font-family: var(--mono);
    font-size: 12px;
    white-space: pre;
  }
  .ferr {
    color: var(--danger);
    white-space: pre-wrap;
  }
  .err {
    margin: 0;
    color: var(--danger);
    white-space: pre-wrap;
    font-family: var(--mono);
    font-size: 12px;
  }
  .hint {
    color: var(--muted);
    font-family: var(--sans);
    font-size: 12.5px;
  }
</style>
