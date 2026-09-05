<script lang="ts">
  import { findFlags, flagCounts } from '../flags';

  let { text, onjump }: { text: string; onjump: (line: number) => void } = $props();

  const flags = $derived(findFlags(text));
  const counts = $derived(flagCounts(flags));
</script>

{#if flags.length > 0}
  <div class="flags">
    <div class="bar">
      <span class="lbl">Flags</span>
      <span class="total">{flags.length} need review</span>
      <span class="grow"></span>
      {#each counts as c (c.tag)}
        <span class="chip" data-tag={c.tag}>{c.tag} {c.count}</span>
      {/each}
    </div>
    <ul class="list">
      {#each flags as f (f.line + f.tag + f.message)}
        <li>
          <button class="row" onclick={() => onjump(f.line)} title="Jump to line {f.line}">
            <span class="ln">{f.line}</span>
            <span class="tag" data-tag={f.tag}>{f.tag}</span>
            <span class="msg">{f.message}</span>
          </button>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .flags {
    display: flex;
    flex-direction: column;
    min-height: 0;
    max-height: 200px;
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
  .total {
    font-size: 12px;
    color: var(--danger);
    font-weight: 600;
  }
  .grow {
    flex: 1;
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
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    border-bottom: 1px solid var(--border);
    padding: 3px 8px;
    cursor: pointer;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--text);
  }
  .row:hover {
    background: var(--panel-2);
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
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
