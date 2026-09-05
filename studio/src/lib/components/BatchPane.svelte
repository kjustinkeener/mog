<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { runBatch, type BatchResult } from '../api';
  import { serializeMog, studio } from '../store.svelte';
  import { runParams } from '../runParams.svelte';
  import Icon from './Icon.svelte';
  import RunGuide from './RunGuide.svelte';

  // Persisted params (survive subtab switches + restarts); see runParams.svelte.ts.
  const p = runParams.batch;

  let running = $state(false);
  let result = $state<BatchResult | null>(null);
  let error = $state('');
  // Reopen the built-in guide after a run has produced results.
  let guideOpen = $state(false);

  function lines(s: string): string[] {
    return s.split('\n').map((x) => x.trim()).filter((x) => x.length > 0);
  }

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

  async function pickOutDir(): Promise<void> {
    const dir = await open({ directory: true, title: 'Output directory' });
    if (typeof dir === 'string') p.outDir = dir;
  }

  async function run(): Promise<void> {
    if (running) return;
    running = true;
    error = '';
    result = null;
    guideOpen = false;
    try {
      const args = {
        pipeline: serializeMog(),
        inputs: lines(p.inputs),
        in_place: p.mode === 'inplace',
        out_dir: p.mode === 'outdir' && p.outDir ? p.outDir : null,
        // Dry run only applies to the writing modes; stdout preview prints the
        // transformed output itself (not a change report).
        dry_run: p.mode === 'stdout' ? false : p.dryRun,
        output_encoding: p.outputEncoding,
        backup: p.mode === 'inplace' && p.backup ? p.backup : null,
        overwrite: p.overwrite,
        excludes: p.excludesEnabled ? lines(p.excludes) : [],
        jobs: p.jobs.trim() ? Number(p.jobs) : null,
      };
      if (args.inputs.length === 0) {
        error = 'Add at least one input file or glob.';
        return;
      }
      result = await runBatch(args);
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="batch">
  <div class="cols">
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

    <div class="col outcol">
      <span class="cap">Output mode</span>
      <div class="modes">
        <label class="radio" title="Print the transformed output to stdout instead of writing files (single-file preview)."><input type="radio" value="stdout" bind:group={p.mode} /> Preview (print to stdout)</label>
        <label class="radio" title="Overwrite each input file with its transformed result. Set a backup suffix to keep the originals."><input type="radio" value="inplace" bind:group={p.mode} /> In place</label>
        <label class="radio" title="Write transformed copies into a chosen directory, leaving the original files untouched."><input type="radio" value="outdir" bind:group={p.mode} /> Output directory</label>
      </div>

      {#if p.mode === 'inplace'}
        <span class="cap">Backup suffix (optional, e.g. <span class="mono">.bak</span>)</span>
        <input type="text" bind:value={p.backup} placeholder=".bak" />
      {/if}
      {#if p.mode === 'outdir'}
        <span class="cap">Directory</span>
        <div class="row">
          <input type="text" bind:value={p.outDir} placeholder="output dir" />
          <button class="ghost" onclick={pickOutDir}><Icon name="folder" />Browse…</button>
        </div>
        <label class="radio"><input type="checkbox" bind:checked={p.overwrite} /> Overwrite existing</label>
      {/if}

    </div>

    <div class="col optcol">
      <span class="cap">Options</span>
      <div class="opts">
        {#if p.mode === 'inplace' || p.mode === 'outdir'}
          <label class="radio" title="Report what would change without writing anything."><input type="checkbox" bind:checked={p.dryRun} /> Dry run</label>
        {/if}
        <label class="enc" title="Encoding written to files. Preserve keeps each input file's detected encoding.">Encoding
          <select bind:value={p.outputEncoding}>
            <option value="preserve">Preserve</option>
            <option value="utf-8">UTF-8</option>
            <option value="utf-8-bom">UTF-8 + BOM</option>
            <option value="utf-16le">UTF-16 LE</option>
            <option value="utf-16be">UTF-16 BE</option>
            <option value="ansi">ANSI (Windows-1252)</option>
          </select>
        </label>
        <label class="jobs" title="Number of files processed in parallel. Blank = auto (one per CPU core); 1 = sequential.">Parallel files <input type="number" bind:value={p.jobs} placeholder="auto" /></label>
      </div>

      <button class="run" disabled={running || studio.mog.steps.length === 0} onclick={run}>
        <Icon name="play" />{running ? 'Running…' : 'Run batch'}
      </button>
    </div>
  </div>

  <div class="results">
    {#if guideOpen || (!result && !error)}
      <RunGuide variant="batch" ondismiss={result || error ? () => (guideOpen = false) : undefined} />
    {:else if error}
      <pre class="err">{error}</pre>
    {:else if result}
      <div class="rhead">
        <span class:ok={result.ok} class:bad={!result.ok}>{result.ok ? 'OK' : 'Errors'}</span>
        <span class="grow"></span>
        <button class="guide-btn" title="Show the guide" onclick={() => (guideOpen = true)}><Icon name="target" />Guide</button>
      </div>
      {#each result.lines as line (line)}
        <div class="line">{line}</div>
      {/each}
      {#each result.errors as line (line)}
        <div class="line rerr">{line}</div>
      {/each}
    {/if}
  </div>
</div>

<style>
  .batch {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 10px;
    gap: 10px;
    overflow: hidden;
  }
  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    gap: 10px;
  }
  .col {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .outcol {
    padding: 4px 9px;
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
  .modes,
  .opts {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .opts {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    margin-top: 2px;
  }
  .optcol {
    padding: 4px 9px;
  }
  .outcol input[type='text'] {
    width: 190px;
  }
  .enc {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }
  .enc select {
    width: auto;
  }
  .radio {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }
  .radio input {
    width: auto;
  }
  .row {
    display: flex;
    gap: 6px;
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
    margin-top: 8px;
    align-self: flex-start;
  }
  .results {
    flex: 1;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    padding: 8px;
    font-family: var(--mono);
    font-size: 12px;
  }
  .rhead {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
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
    font-family: var(--sans);
  }
  .ok {
    color: var(--ok);
    font-weight: 700;
  }
  .bad {
    color: var(--danger);
    font-weight: 700;
  }
  .line {
    white-space: pre-wrap;
    padding: 1px 0;
  }
  .rerr,
  .err {
    color: var(--danger);
  }
  .inputs-head {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .inputs-head .cap {
    flex: 1;
  }
</style>
