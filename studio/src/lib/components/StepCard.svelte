<script lang="ts">
  import type { Step } from '../types';
  import { catLabel, clearDrag, markDirty, openRegexPopover, removeStep, reorderToTarget, studio } from '../store.svelte';
  import { analyzeRegex, initRegexEngine } from '../regexEngine';
  import OptionForm from './OptionForm.svelte';
  import Icon from './Icon.svelte';

  let { step, index }: { step: Step; index: number } = $props();

  const desc = $derived(step.action ? studio.byName[step.action] : undefined);
  const scoped = $derived(!!(step.only_lines_matching || step.except_lines_matching));
  // One-line summary shown when the card is collapsed: the step's own comment if
  // set, else the action's summary.
  const collapsedDesc = $derived((step.description?.trim() || desc?.summary || '').trim());
  const isStop = $derived(studio.stopAfter === index);

  const REGEX_REPLACE = new Set(['replace_regex', 'replace_regex_multiline']);

  // Load the regex engine only for regex-replace steps, so the pill can reflect the
  // step's actual pattern rather than the action's static complexity.
  let regexReady = $state(false);
  $effect(() => {
    if (!regexReady && desc && REGEX_REPLACE.has(desc.name)) {
      initRegexEngine()
        .then(() => (regexReady = true))
        .catch(() => {});
    }
  });

  const WHOLE_FILE = {
    cls: 'whole',
    text: 'whole-file',
    title:
      'Buffers the whole input at once (sorting, dedupe, columnar/compose); memory-bound, cannot stream.',
  };
  const BACKTRACKING = {
    cls: 'warn',
    text: 'O(n²)',
    title:
      'This pattern uses backreferences or lookaround, so it runs on the backtracking engine (O(n²) on large input).',
  };

  // Perf pill. For regex-replace steps it is PATTERN-accurate: it warns only when
  // this step's actual find pattern needs the backtracking engine. Other non-linear
  // actions fall back to their descriptor complexity.
  const perf = $derived.by(() => {
    if (desc?.complexity === 'whole_file') return WHOLE_FILE;
    if (desc?.complexity !== 'regex_backtracking') return null;

    if (desc && REGEX_REPLACE.has(desc.name)) {
      const find = typeof step.options.find === 'string' ? step.options.find : '';
      if (find === '') return null; // no pattern yet
      if (!regexReady) return BACKTRACKING; // conservative until the engine loads
      const ic = step.options.ignore_case === true;
      const ml = desc.name === 'replace_regex_multiline';
      return analyzeRegex(find, '', ic, ml).engine === 'backtracking' ? BACKTRACKING : null;
    }
    // A non-replace action classed backtracking: keep the descriptor-level warning.
    return BACKTRACKING;
  });

  function toggleStop(): void {
    studio.stopAfter = isStop ? null : index;
  }

  function onDragStart(e: DragEvent): void {
    studio.dragUid = step._uid;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', step._uid); // required to start a drag in WebView2
    }
  }

  function onDragOver(e: DragEvent): void {
    if (!studio.dragUid) return;
    e.preventDefault(); // accept the drop (otherwise the cursor shows "blocked")
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
    // Insertion line goes above or below this card based on the cursor position.
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    studio.dropUid = step._uid;
    studio.dropPos = e.clientY < rect.top + rect.height / 2 ? 'before' : 'after';
  }

  function onDrop(e: DragEvent): void {
    e.preventDefault();
    if (studio.dragUid) {
      reorderToTarget(studio.dragUid, step._uid, studio.dropPos === 'after');
    }
    clearDrag();
  }
</script>

<div
  class="card"
  class:disabled={step.disabled}
  class:sel={studio.selectedUid === step._uid}
  class:dragging={studio.dragUid === step._uid}
  ondragenter={onDragOver}
  ondragover={onDragOver}
  ondrop={onDrop}
  role="presentation"
>
  <div
    class="head"
    draggable="true"
    ondragstart={onDragStart}
    ondragend={clearDrag}
    onclick={() => (studio.selectedUid = step._uid)}
    role="presentation"
  >
    <button class="ghost caret" title={step._collapsed ? 'Expand' : 'Collapse'} onclick={(e) => { e.stopPropagation(); step._collapsed = !step._collapsed; }}>
      <Icon name={step._collapsed ? 'chevron-right' : 'chevron-down'} />
    </button>
    <span class="num">{index + 1}</span>
    {#if desc}
      <span class="chip" style={`background: var(--cat-${desc.category}, var(--cat-fallback))`}>{catLabel(desc.category)}</span>
      <span class="title">{desc.label}</span>
    {:else}
      <span class="chip unknown">?</span>
      <span class="title">{step.action ?? '(no action)'}</span>
    {/if}
    {#if scoped}<span class="badge" title="Runs only on matching lines">scoped</span>{/if}
    {#if perf}<span class="perf {perf.cls}" title={perf.title}>{perf.text}</span>{/if}

    <span class="spacer"></span>

    <button class="ghost" title="Preview up to this step" class:on={isStop} onclick={(e) => { e.stopPropagation(); toggleStop(); }}><Icon name="target" /></button>
    <button class="ghost" title={step.disabled ? 'Enable' : 'Disable'} onclick={(e) => { e.stopPropagation(); step.disabled = !step.disabled; markDirty(); }}>
      <Icon name={step.disabled ? 'square' : 'check-square'} />
    </button>
    <button class="ghost danger" title="Delete" onclick={(e) => { e.stopPropagation(); removeStep(step._uid); }}><Icon name="trash" /></button>
  </div>

  {#if step._collapsed && collapsedDesc}
    <div class="subline" title={collapsedDesc}>{collapsedDesc}</div>
  {/if}

  {#if !step._collapsed}
    <div class="body">
      <label class="fieldlabel">
        <span class="lbl">Section header</span>
        <input
          class="section-in"
          type="text"
          placeholder="optional; starts a new section here"
          value={step.section ?? ''}
          oninput={(e) => { step.section = e.currentTarget.value || undefined; markDirty(); }}
        />
      </label>
      <label class="fieldlabel">
        <span class="lbl">Step comment</span>
        <input
          class="descr"
          type="text"
          placeholder="optional; shown when collapsed"
          value={step.description ?? ''}
          oninput={(e) => { step.description = e.currentTarget.value; markDirty(); }}
        />
      </label>

      {#if desc}
        <OptionForm {step} {desc} />
      {:else}
        <p class="warn">Unknown action "{step.action}". This step will fail.</p>
      {/if}

      <div class="scope">
        <button class="ghost scopetoggle" onclick={() => (step._scopeOpen = !step._scopeOpen)}>
          <Icon name={step._scopeOpen ? 'chevron-down' : 'chevron-right'} /> Apply to lines… {#if scoped}<span class="badge">active</span>{/if}
        </button>
        {#if step._scopeOpen}
          <div class="scopebody">
            <span class="cap">Only lines matching</span>
            <input type="text" class="mono" placeholder="regex (leave blank for all)"
              data-regex-field=""
              value={step.only_lines_matching ?? ''}
              onfocus={(e) => openRegexPopover(e.currentTarget, step.only_lines_matching ?? '', (v) => { step.only_lines_matching = v || undefined; markDirty(); })}
              oninput={(e) => { step.only_lines_matching = e.currentTarget.value || undefined; markDirty(); }} />
            <span class="cap">Except lines matching {#if step.only_lines_matching}<span class="dim">(ignored: "only" wins)</span>{/if}</span>
            <input type="text" class="mono" placeholder="regex"
              data-regex-field=""
              value={step.except_lines_matching ?? ''}
              onfocus={(e) => openRegexPopover(e.currentTarget, step.except_lines_matching ?? '', (v) => { step.except_lines_matching = v || undefined; markDirty(); })}
              oninput={(e) => { step.except_lines_matching = e.currentTarget.value || undefined; markDirty(); }} />
            <label class="inline">
              <input type="checkbox" checked={step.match_ignore_case}
                onchange={(e) => { step.match_ignore_case = e.currentTarget.checked; markDirty(); }} />
              Ignore case when matching
            </label>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    overflow: hidden;
  }
  .card.sel {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }
  .card.disabled {
    opacity: 0.55;
  }
  .card.dragging {
    opacity: 0.4;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 2px 6px;
    background: var(--panel-2);
    cursor: grab;
  }
  .head:active {
    cursor: grabbing;
  }
  .caret {
    font-size: 11px;
    line-height: 1;
    color: var(--muted);
    padding: 2px 0;
    margin-right: -2px;
  }
  .num {
    color: var(--muted);
    font-variant-numeric: tabular-nums;
    min-width: 12px;
    text-align: right;
  }
  .chip {
    color: #fff;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    padding: 1px 6px;
    border-radius: 4px;
  }
  .chip.unknown {
    background: var(--danger);
  }
  .title {
    font-weight: 600;
  }
  .badge {
    font-size: 10px;
    color: var(--accent);
    background: var(--accent-weak);
    border-radius: 4px;
    padding: 0 5px;
  }
  .perf {
    font-size: 10px;
    border-radius: 4px;
    padding: 0 5px;
    cursor: help;
  }
  .perf.warn {
    color: var(--warn);
    background: var(--warn-weak);
  }
  .perf.whole {
    color: var(--muted);
    background: var(--panel-2);
  }
  .spacer {
    flex: 1;
  }
  .head .ghost {
    font-size: 12px;
    line-height: 1;
    padding: 2px 3px;
  }
  .head .ghost.on {
    color: var(--accent);
  }
  .head .ghost.danger:hover {
    color: var(--danger);
  }
  .subline {
    padding: 3px 8px 5px 30px;
    font-size: 11px;
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .body {
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .fieldlabel {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .fieldlabel .lbl {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
  }
  .descr {
    font-size: 12px;
  }
  .section-in {
    font-size: 11px;
    color: var(--muted);
    background: var(--panel-2);
  }
  .warn {
    margin: 0;
    color: var(--danger);
  }
  .scope {
    border-top: 1px dashed var(--border);
    padding-top: 6px;
  }
  .scopetoggle {
    color: var(--muted);
  }
  .scopebody {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 4px 2px 2px;
  }
  .inline {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
  }
  .inline input {
    width: auto;
  }
  .dim {
    color: var(--muted);
  }
</style>
