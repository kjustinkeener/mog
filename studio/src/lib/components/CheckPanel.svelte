<script lang="ts">
  import { flagCheck, type FlagCheck } from '../api';
  import { serializeMog, studio } from '../store.svelte';
  import Icon from './Icon.svelte';

  let running = $state(false);
  let report = $state<FlagCheck | null>(null);
  let error = $state('');

  const stepCount = $derived(studio.mog.steps.length);

  async function run(): Promise<void> {
    if (running) return;
    running = true;
    error = '';
    report = null;
    try {
      report = await flagCheck(serializeMog(), studio.input);
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="check">
  <div class="pane">
    <div class="bar"><span class="lbl">Input</span></div>
    <textarea class="editor" bind:value={studio.input} spellcheck="false"></textarea>
  </div>

  <div class="actions">
    <button disabled={running || stepCount === 0} onclick={run}>
      <Icon name="play" />{running ? 'Checking…' : 'Run --check'}
    </button>
    <span class="grow"></span>
    {#if report}
      <span class="verdict" class:ok={report.clean} class:bad={!report.clean}>
        {report.clean ? '✓ clean' : '✗ would fail --check'}
      </span>
    {/if}
  </div>

  <div class="pane result">
    <div class="bar">
      <span class="lbl">Check report</span>
      {#if report}
        <span class="grow"></span>
        {#if report.would_change}<span class="chip changed">would change</span>{/if}
        {#each report.counts as [tag, n] (tag)}
          <span class="chip" data-tag={tag}>{tag} {n}</span>
        {/each}
      {/if}
    </div>

    {#if error}
      <pre class="err">{error}</pre>
    {:else if report}
      {#if report.error}
        <p class="hint bad">Processing error: {report.error}</p>
      {/if}
      {#if report.flags.length > 0}
        <ul class="list">
          {#each report.flags as f (f.line + f.tag + f.message)}
            <li class="row">
              <span class="ln">{f.line}</span>
              <span class="tag" data-tag={f.tag}>{f.tag}</span>
              <span class="msg">{f.message}</span>
            </li>
          {/each}
        </ul>
      {:else if report.clean}
        <p class="hint ok">No flags, and the input would not change. This pipeline passes <span class="mono">--check</span>.</p>
      {:else}
        <p class="hint">No flags raised, but the pipeline would change the input, so <span class="mono">--check</span> would exit non-zero (the CI gate).</p>
      {/if}
    {:else}
      <p class="hint">Run the pipeline over the current input under <span class="mono">--check</span> to see the
        flags it raises (<span class="mono">detect_secrets</span> / <span class="mono">detect_pii</span> /
        <span class="mono">flag_matching</span>) and whether it would pass the CI gate.</p>
    {/if}
  </div>
</div>

<style>
  .check {
    display: grid;
    grid-template-rows: 1fr auto 1.4fr;
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
  .verdict {
    font-weight: 800;
    font-size: 13px;
  }
  .ok {
    color: var(--ok);
  }
  .bad {
    color: var(--danger);
  }
  .chip {
    font-size: 11px;
    font-weight: 700;
    padding: 1px 6px;
    border-radius: 10px;
    background: var(--panel);
    border: 1px solid var(--border);
    color: var(--muted);
  }
  .chip.changed {
    color: var(--accent);
    border-color: var(--accent-weak);
  }
  .chip[data-tag='FIXME'],
  .tag[data-tag='FIXME'] {
    color: var(--danger);
  }
  .chip[data-tag='WARN'],
  .tag[data-tag='WARN'],
  .chip[data-tag='XXX'],
  .tag[data-tag='XXX'] {
    color: var(--accent);
  }
  .list {
    margin: 0;
    padding: 0;
    list-style: none;
    overflow: auto;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 8px;
    border-bottom: 1px solid var(--border);
    padding: 3px 8px;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
  }
  .ln {
    min-width: 3ch;
    text-align: right;
    color: var(--muted);
  }
  .tag {
    font-weight: 700;
    min-width: 5ch;
  }
  .msg {
    color: var(--text);
  }
  .err {
    margin: 0;
    padding: 8px;
    color: var(--danger);
    white-space: pre-wrap;
    font-family: var(--mono);
    font-size: 12px;
  }
  .hint {
    padding: 8px;
    margin: 0;
    color: var(--muted);
    font-family: var(--sans);
    font-size: 12.5px;
  }
  .hint.ok {
    color: var(--ok);
  }
  .hint.bad {
    color: var(--danger);
  }
</style>
