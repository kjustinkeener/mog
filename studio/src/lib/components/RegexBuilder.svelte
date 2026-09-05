<script lang="ts">
  // No-regex builder: assemble a pattern from plain-language blocks, for people
  // who don't want to write regex by hand. Each block emits a regex fragment;
  // the fragments concatenate into a pattern you can send to the tester.

  import Icon from './Icon.svelte';

  let { onApply }: { onApply: (pattern: string) => void } = $props();

  type Quant = 'one' | 'plus' | 'star' | 'opt' | 'exact';
  type Kind = 'text' | 'digit' | 'word' | 'space' | 'any' | 'set' | 'start' | 'end' | 'boundary';

  interface Block {
    id: number;
    kind: Kind;
    text: string;
    negate: boolean;
    quant: Quant;
    count: number;
    capture: boolean;
  }

  const QUANTIFIABLE: Kind[] = ['text', 'digit', 'word', 'space', 'any', 'set'];
  const KIND_LABEL: Record<Kind, string> = {
    text: 'Literal text',
    digit: 'Digit',
    word: 'Word character',
    space: 'Whitespace',
    any: 'Any character',
    set: 'One of these',
    start: 'Start of line',
    end: 'End of line',
    boundary: 'Word boundary',
  };
  const PALETTE: Kind[] = ['text', 'digit', 'word', 'space', 'any', 'set', 'start', 'end', 'boundary'];

  let uid = 0;
  let blocks = $state<Block[]>([]);

  function add(kind: Kind): void {
    blocks.push({
      id: (uid += 1),
      kind,
      text: '',
      negate: false,
      quant: 'one',
      count: 2,
      capture: false,
    });
  }
  function remove(id: number): void {
    blocks = blocks.filter((b) => b.id !== id);
  }
  function move(i: number, delta: number): void {
    const j = i + delta;
    if (j < 0 || j >= blocks.length) return;
    const next = blocks.slice();
    [next[i], next[j]] = [next[j], next[i]];
    blocks = next;
  }

  function escLiteral(s: string): string {
    return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }
  function escClass(s: string): string {
    return s.replace(/[\]\\^-]/g, '\\$&');
  }
  function quantSuffix(b: Block): string {
    switch (b.quant) {
      case 'plus':
        return '+';
      case 'star':
        return '*';
      case 'opt':
        return '?';
      case 'exact':
        return `{${Math.max(0, Math.trunc(b.count))}}`;
      default:
        return '';
    }
  }

  function atom(b: Block): string {
    switch (b.kind) {
      case 'digit':
        return '\\d';
      case 'word':
        return '\\w';
      case 'space':
        return '\\s';
      case 'any':
        return '.';
      case 'set':
        return `[${b.negate ? '^' : ''}${escClass(b.text)}]`;
      case 'start':
        return '^';
      case 'end':
        return '$';
      case 'boundary':
        return '\\b';
      case 'text': {
        const e = escLiteral(b.text);
        // A multi-char literal must be grouped before a quantifier binds to it.
        return b.text.length > 1 && b.quant !== 'one' ? `(?:${e})` : e;
      }
    }
  }

  function fragment(b: Block): string {
    if (!QUANTIFIABLE.includes(b.kind)) return atom(b);
    const body = atom(b) + quantSuffix(b);
    return b.capture ? `(${body})` : body;
  }

  const pattern = $derived(blocks.map(fragment).join(''));
</script>

<div class="builder">
  <div class="palette">
    {#each PALETTE as k (k)}
      <button class="add" onclick={() => add(k)}><Icon name="plus" />{KIND_LABEL[k]}</button>
    {/each}
  </div>

  {#if blocks.length === 0}
    <p class="empty">Add blocks to compose a pattern without writing regex.</p>
  {:else}
    <div class="blocks">
      {#each blocks as b, i (b.id)}
        <div class="block">
          <span class="bkind">{KIND_LABEL[b.kind]}</span>

          {#if b.kind === 'text'}
            <input class="btext mono" type="text" placeholder="text" bind:value={b.text} />
          {:else if b.kind === 'set'}
            <input class="btext mono" type="text" placeholder="e.g. aeiou" bind:value={b.text} />
            <label class="mini"><input type="checkbox" bind:checked={b.negate} /> not</label>
          {/if}

          {#if QUANTIFIABLE.includes(b.kind)}
            <select class="quant" bind:value={b.quant}>
              <option value="one">once</option>
              <option value="plus">1+</option>
              <option value="star">0+</option>
              <option value="opt">optional</option>
              <option value="exact">exactly…</option>
            </select>
            {#if b.quant === 'exact'}
              <input class="count" type="number" min="0" bind:value={b.count} />
            {/if}
            <label class="mini" title="Capture this piece as a group"><input type="checkbox" bind:checked={b.capture} /> capture</label>
          {/if}

          <span class="grow"></span>
          <button class="ic" title="Move up" onclick={() => move(i, -1)} disabled={i === 0}><Icon name="arrow-up" /></button>
          <button class="ic" title="Move down" onclick={() => move(i, 1)} disabled={i === blocks.length - 1}><Icon name="arrow-down" /></button>
          <button class="ic" title="Remove" onclick={() => remove(b.id)}><Icon name="x" /></button>
        </div>
      {/each}
    </div>

    <div class="out-row">
      <code class="built mono">{pattern || '(empty)'}</code>
      <button class="primary" onclick={() => onApply(pattern)} disabled={pattern === ''}><Icon name="check" />Use this pattern</button>
    </div>
  {/if}
</div>

<style>
  .builder {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .palette {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .add {
    font-size: 11px;
    padding: 2px 7px;
  }
  .empty {
    margin: 0;
    color: var(--muted);
    font-style: italic;
    font-size: 12px;
  }
  .blocks {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .block {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 6px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .bkind {
    font-size: 11px;
    font-weight: 700;
    color: var(--text);
    white-space: nowrap;
  }
  .btext {
    flex: 1;
    min-width: 60px;
  }
  .quant {
    width: auto;
    padding: 2px 4px;
    font-size: 12px;
  }
  .count {
    width: 56px;
  }
  .mini {
    display: flex;
    align-items: center;
    gap: 3px;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
  }
  .mini input {
    width: auto;
  }
  .grow {
    flex: 1;
  }
  .ic {
    padding: 1px 6px;
    font-size: 12px;
  }
  .out-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .built {
    flex: 1;
    padding: 5px 8px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow-x: auto;
    white-space: nowrap;
  }
</style>
