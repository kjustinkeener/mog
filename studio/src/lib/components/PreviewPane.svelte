<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { readTextFile } from '@tauri-apps/plugin-fs';
  import { runTransform } from '../api';
  import { serializeMog, studio } from '../store.svelte';
  import FlagsPanel from './FlagsPanel.svelte';
  import Icon from './Icon.svelte';

  const stepCount = $derived(studio.mog.steps.length);

  let outputEl = $state<HTMLPreElement | undefined>(undefined);

  // Scroll the output so the flagged line is roughly centered.
  function jumpToLine(line: number): void {
    if (!outputEl) return;
    const lh = parseFloat(getComputedStyle(outputEl).lineHeight);
    const lineHeight = Number.isFinite(lh) ? lh : 18;
    outputEl.scrollTop = Math.max(0, (line - 1) * lineHeight - outputEl.clientHeight / 2);
  }

  async function transform(): Promise<void> {
    if (studio.running) return;
    studio.running = true;
    studio.error = '';
    try {
      const pipeline = serializeMog();
      const res = await runTransform(pipeline, studio.input, studio.showDiff, studio.stopAfter);
      studio.output = res.output;
      studio.outputIsDiff = res.is_diff;
      const inB = new TextEncoder().encode(studio.input).length;
      const outB = new TextEncoder().encode(res.output).length;
      const scope = studio.stopAfter !== null ? ` · after step ${studio.stopAfter + 1}/${stepCount}` : '';
      studio.status = res.is_diff
        ? `${res.changed ? 'diff shown' : 'no change'}${scope}`
        : `${res.changed ? 'changed' : 'unchanged'} · ${inB}→${outB} bytes${scope}`;
    } catch (e) {
      studio.output = '';
      studio.error = String(e);
      studio.status = '';
    } finally {
      studio.running = false;
    }
  }

  async function loadSample(): Promise<void> {
    const path = await open({ multiple: false, title: 'Load sample text' });
    if (typeof path === 'string') {
      studio.input = await readTextFile(path);
    }
  }
</script>

<div class="preview">
  <div class="pane">
    <div class="bar">
      <span class="lbl">Input</span>
      <span class="grow"></span>
      <button class="ghost" onclick={loadSample}><Icon name="download" />Load sample…</button>
    </div>
    <textarea class="editor" bind:value={studio.input} spellcheck="false"></textarea>
  </div>

  <div class="actions">
    <button disabled={studio.running || stepCount === 0} onclick={transform}>
      <Icon name="play" />{studio.running ? 'Running…' : 'Transform'}
    </button>
    <label class="toggle">
      <input type="checkbox" bind:checked={studio.showDiff} />
      Diff
    </label>
    {#if studio.stopAfter !== null}
      <button class="ghost clearstop" onclick={() => (studio.stopAfter = null)}><Icon name="x" />clear step limit</button>
    {/if}
    <span class="grow"></span>
    <span class="status">{studio.status}</span>
  </div>

  <div class="pane">
    <div class="bar">
      <span class="lbl">{studio.outputIsDiff ? 'Diff' : 'Output'}</span>
    </div>
    {#if studio.error}
      <pre class="output err">{studio.error}</pre>
    {:else}
      <pre class="output" class:diff={studio.outputIsDiff} bind:this={outputEl}>{studio.output}</pre>
    {/if}
  </div>

  {#if !studio.outputIsDiff && !studio.error}
    <FlagsPanel text={studio.output} onjump={jumpToLine} />
  {/if}
</div>

<style>
  .preview {
    display: grid;
    grid-template-rows: 1fr auto 1fr auto;
    height: 100%;
    gap: 6px;
    padding: 8px;
  }
  .pane {
    display: flex;
    flex-direction: column;
    min-height: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--panel);
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    background: var(--panel-2);
    border-bottom: 1px solid var(--border);
  }
  .lbl {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
  }
  .grow {
    flex: 1;
  }
  .editor {
    flex: 1;
    border: none;
    border-radius: 0;
    resize: none;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .toggle {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--text);
  }
  .toggle input {
    width: auto;
  }
  .clearstop {
    color: var(--accent);
  }
  .status {
    color: var(--muted);
    font-size: 12px;
  }
  .output {
    flex: 1;
    margin: 0;
    padding: 8px;
    overflow: auto;
    font-family: var(--mono);
    font-size: 12.5px;
    white-space: pre;
    tab-size: 4;
  }
  .output.err {
    color: var(--danger);
    white-space: pre-wrap;
  }
</style>
