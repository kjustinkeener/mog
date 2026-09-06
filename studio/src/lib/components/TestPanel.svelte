<script lang="ts">
  import { runMogTest, runTransform, type TestOutcome } from '../api';
  import { serializeMog, studio } from '../store.svelte';
  import Icon from './Icon.svelte';

  let expected = $state('');
  let running = $state(false);
  let snapshotting = $state(false);
  let outcome = $state<TestOutcome | null>(null);
  let error = $state('');

  const stepCount = $derived(studio.mog.steps.length);

  // Fill the expected-output golden by running the recipe on the current input.
  async function snapshot(): Promise<void> {
    if (snapshotting) return;
    snapshotting = true;
    error = '';
    try {
      const res = await runTransform(serializeMog(), studio.input, false, null);
      expected = res.output;
      outcome = null;
    } catch (e) {
      error = String(e);
    } finally {
      snapshotting = false;
    }
  }

  async function runTest(): Promise<void> {
    if (running) return;
    running = true;
    error = '';
    outcome = null;
    try {
      outcome = await runMogTest(serializeMog(), studio.input, expected);
    } catch (e) {
      error = String(e);
    } finally {
      running = false;
    }
  }
</script>

<div class="test">
  <div class="pane">
    <div class="bar">
      <span class="lbl">Input (TestInput)</span>
    </div>
    <textarea class="editor" bind:value={studio.input} spellcheck="false"></textarea>
  </div>

  <div class="pane">
    <div class="bar">
      <span class="lbl">Expected output (golden)</span>
      <span class="grow"></span>
      <button class="ghost" disabled={snapshotting || stepCount === 0} onclick={snapshot}>
        <Icon name="camera" />{snapshotting ? 'Running…' : 'Snapshot from mog'}
      </button>
    </div>
    <textarea class="editor" bind:value={expected} spellcheck="false"
      placeholder="Paste or snapshot the output this mog should produce."></textarea>
  </div>

  <div class="actions">
    <button disabled={running || stepCount === 0} onclick={runTest}>
      <Icon name="play" />{running ? 'Testing…' : 'Run test'}
    </button>
    <span class="grow"></span>
    {#if outcome}
      <span class="verdict" class:ok={outcome.pass} class:bad={!outcome.pass}>
        {outcome.pass ? '✓ PASS' : '✗ FAIL'}
      </span>
    {/if}
  </div>

  <div class="pane result">
    <div class="bar"><span class="lbl">Result</span></div>
    {#if error}
      <pre class="body err">{error}</pre>
    {:else if outcome?.pass}
      <p class="hint ok">Output matches the expected golden. Edit the mog and re-run to catch regressions.</p>
    {:else if outcome}
      <p class="hint bad">{outcome.message ?? 'Output does not match the expected golden.'}</p>
      {#if outcome.diff}
        <pre class="body diff">{outcome.diff}</pre>
      {/if}
    {:else}
      <p class="hint">Snapshot (or paste) the expected output, then run the test. The clock and
        UUID are pinned during a test, so <span class="mono">{'{{@now}}'}</span> / <span class="mono">{'{{@uuid}}'}</span>
        mogs stay deterministic.</p>
    {/if}
  </div>
</div>

<style>
  .test {
    display: grid;
    grid-template-rows: 1fr 1fr auto 1fr;
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
  .body {
    flex: 1;
    margin: 0;
    padding: 8px;
    overflow: auto;
    font-family: var(--mono);
    font-size: 12.5px;
    white-space: pre;
    tab-size: 4;
  }
  .body.err {
    color: var(--danger);
    white-space: pre-wrap;
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
