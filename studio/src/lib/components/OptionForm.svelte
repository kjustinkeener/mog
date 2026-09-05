<script lang="ts">
  import type { ActionDescriptor, JsonValue, ParamSpec, Step } from '../types';
  import { markDirty, isRegexField, openRegexPopover } from '../store.svelte';
  import MogSearchBox from './MogSearchBox.svelte';

  let { step, desc }: { step: Step; desc: ActionDescriptor } = $props();

  // The run_mog `file` field gets the recipe-library typeahead (free text still
  // works for paths); everything else uses the generic schema-driven inputs.
  function isRecipePicker(p: ParamSpec): boolean {
    return desc.name === 'run_mog' && p.key === 'file';
  }

  function current(p: ParamSpec): JsonValue | undefined {
    const v = step.options[p.key];
    return v !== undefined ? v : p.default ?? undefined;
  }

  function set(p: ParamSpec, v: JsonValue): void {
    step.options[p.key] = v;
    markDirty();
  }

  function clear(p: ParamSpec): void {
    delete step.options[p.key];
    markDirty();
  }

  function strVal(p: ParamSpec): string {
    const v = current(p);
    return typeof v === 'string' ? v : v === undefined || v === null ? '' : String(v);
  }

  function boolVal(p: ParamSpec): boolean {
    const v = current(p);
    return v === true;
  }

  function intStr(p: ParamSpec): string {
    const v = step.options[p.key];
    if (typeof v === 'number') return String(v);
    if (typeof p.default === 'number' && v === undefined) return String(p.default);
    return '';
  }

  function onInt(p: ParamSpec, raw: string): void {
    if (raw.trim() === '') {
      // empty: for an optional int (e.g. seed) that means "absent".
      clear(p);
      return;
    }
    const n = Number(raw);
    if (Number.isFinite(n)) set(p, Math.trunc(n));
  }

  function isMissing(p: ParamSpec): boolean {
    return p.required && strVal(p).length === 0;
  }
</script>

{#if desc.params.length === 0}
  <p class="none">No options.</p>
{:else}
  <div class="form">
    {#each desc.params as p (p.key)}
      <div class="field">
        <label for={`${step._uid}-${p.key}`}>
          {p.label}
          {#if p.required}<span class="req">*</span>{/if}
        </label>

        {#if isRecipePicker(p)}
          <MogSearchBox
            value={strVal(p)}
            oninput={(v) => set(p, v)}
            onpick={(h) => set(p, h.name)}
            placeholder="library mog or path…"
          />
        {:else if p.kind === 'bool'}
          <input
            id={`${step._uid}-${p.key}`}
            type="checkbox"
            class="chk"
            checked={boolVal(p)}
            onchange={(e) => set(p, e.currentTarget.checked)}
          />
        {:else if p.kind === 'enum'}
          <select
            id={`${step._uid}-${p.key}`}
            value={strVal(p)}
            onchange={(e) => set(p, e.currentTarget.value)}
          >
            {#each p.values ?? [] as opt (opt)}
              <option value={opt}>{opt}</option>
            {/each}
          </select>
        {:else if p.kind === 'int'}
          <input
            id={`${step._uid}-${p.key}`}
            type="number"
            value={intStr(p)}
            oninput={(e) => onInt(p, e.currentTarget.value)}
          />
        {:else if p.kind === 'multiline_text'}
          <textarea
            id={`${step._uid}-${p.key}`}
            rows="3"
            class:missing={isMissing(p)}
            value={strVal(p)}
            oninput={(e) => set(p, e.currentTarget.value)}
          ></textarea>
        {:else if isRegexField(desc.name, p.key)}
          <input
            id={`${step._uid}-${p.key}`}
            type="text"
            class="mono"
            class:missing={isMissing(p)}
            data-regex-field=""
            value={strVal(p)}
            oninput={(e) => set(p, e.currentTarget.value)}
            onfocus={(e) => openRegexPopover(e.currentTarget, strVal(p), (v) => set(p, v))}
          />
        {:else}
          <input
            id={`${step._uid}-${p.key}`}
            type="text"
            class:missing={isMissing(p)}
            value={strVal(p)}
            oninput={(e) => set(p, e.currentTarget.value)}
          />
        {/if}

        <p class="help">
          {p.help}{#if isMissing(p)}<span class="err"> required</span>{/if}
        </p>
      </div>
    {/each}
  </div>
{/if}

<style>
  .form {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .chk {
    width: auto;
    align-self: flex-start;
  }
  .help {
    margin: 0;
    font-size: 11px;
    color: var(--muted);
    line-height: 1.35;
  }
  .req {
    color: var(--danger);
  }
  .err {
    color: var(--danger);
    font-weight: 600;
  }
  .missing {
    border-color: var(--danger);
  }
  .none {
    margin: 0;
    color: var(--muted);
    font-style: italic;
  }
</style>
