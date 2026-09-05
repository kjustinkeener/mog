<script lang="ts">
  import { marketSearch, type RecipeHit } from '../api';
  import Icon from './Icon.svelte';

  // Controlled typeahead over the recipe library (mog market search/list).
  // The parent owns `value`: on input it gets the raw text (so free-text paths
  // still work for run_mog); on pick it gets the chosen recipe to act on.
  let {
    value,
    oninput,
    onpick,
    onbrowse,
    placeholder = 'Search mogs…',
  }: {
    value: string;
    oninput: (v: string) => void;
    onpick: (hit: RecipeHit) => void;
    onbrowse?: () => void;
    placeholder?: string;
  } = $props();

  let results = $state<RecipeHit[]>([]);
  let open = $state(false);
  let active = $state(0);
  let loading = $state(false);
  let err = $state('');
  let timer: ReturnType<typeof setTimeout> | null = null;
  let boxEl: HTMLDivElement;

  async function run(): Promise<void> {
    loading = true;
    err = '';
    try {
      results = await marketSearch(value);
      active = 0;
      open = true;
    } catch (e) {
      err = String(e);
      results = [];
      open = true;
    } finally {
      loading = false;
    }
  }

  function schedule(): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(run, 150);
  }

  function onType(e: Event): void {
    oninput((e.currentTarget as HTMLInputElement).value);
    schedule();
  }

  function onFocus(): void {
    if (results.length === 0) run();
    else open = true;
  }

  function choose(h: RecipeHit): void {
    open = false;
    results = [];
    onpick(h);
  }

  function onKey(e: KeyboardEvent): void {
    if (!open || results.length === 0) return;
    if (e.key === 'ArrowDown') {
      active = Math.min(active + 1, results.length - 1);
      e.preventDefault();
    } else if (e.key === 'ArrowUp') {
      active = Math.max(active - 1, 0);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      if (results[active]) {
        choose(results[active]);
        e.preventDefault();
      }
    } else if (e.key === 'Escape') {
      open = false;
    }
  }

  function onWindowPointer(e: PointerEvent): void {
    if (boxEl && !boxEl.contains(e.target as Node)) open = false;
  }
</script>

<div class="mog-search" bind:this={boxEl}>
  <input
    type="text"
    spellcheck="false"
    {placeholder}
    {value}
    oninput={onType}
    onfocus={onFocus}
    onkeydown={onKey}
  />
  {#if open}
    <div class="dropdown">
      {#if loading}
        <div class="msg">searching…</div>
      {:else if err}
        <div class="msg err">{err}</div>
      {:else if results.length === 0}
        <div class="msg">no mogs</div>
      {:else}
        {#each results as h, i (h.name)}
          <button
            type="button"
            class="hit"
            class:active={i === active}
            onclick={() => choose(h)}
            onmouseenter={() => (active = i)}
          >
            <div class="row1">
              <span class="name mono">{h.name}</span>
              {#if h.featured}<span class="star" title="Featured">★</span>{/if}
            </div>
            <div class="desc">{h.description}</div>
            {#if h.tags.length > 0}
              <div class="tags">
                {#each h.tags.slice(0, 6) as t (t)}<span class="tag">{t}</span>{/each}
              </div>
            {/if}
          </button>
        {/each}
      {/if}
      {#if onbrowse}
        <button
          type="button"
          class="browse-all"
          onclick={() => {
            open = false;
            onbrowse?.();
          }}
        ><Icon name="search" />Browse all mogs…</button>
      {/if}
    </div>
  {/if}
</div>
<svelte:window onpointerdown={onWindowPointer} />

<style>
  .mog-search {
    position: relative;
  }
  .dropdown {
    position: absolute;
    top: calc(100% + 3px);
    left: 0;
    right: 0;
    z-index: 50;
    max-height: 340px;
    overflow-y: auto;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    box-shadow: 0 6px 20px rgba(0, 0, 0, 0.18);
    min-width: 280px;
  }
  .msg {
    padding: 8px 10px;
    color: var(--muted);
    font-size: 12px;
  }
  .msg.err {
    color: var(--danger);
    white-space: pre-wrap;
  }
  .hit {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    padding: 6px 10px;
    cursor: pointer;
  }
  .hit:last-child {
    border-bottom: none;
  }
  .hit.active,
  .hit:hover {
    background: var(--accent-weak);
    border-color: var(--border);
  }
  .row1 {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .name {
    font-weight: 700;
    color: var(--text);
    font-size: 12px;
  }
  .star {
    color: var(--warn);
    font-size: 11px;
  }
  .desc {
    color: var(--muted);
    font-size: 11px;
    line-height: 1.35;
    margin-top: 1px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    margin-top: 3px;
  }
  .tag {
    font-size: 10px;
    color: var(--muted);
    background: var(--panel-2);
    border-radius: 3px;
    padding: 0 4px;
  }
  .browse-all {
    display: block;
    width: 100%;
    text-align: center;
    border: none;
    border-radius: 0;
    background: var(--panel-2);
    color: var(--accent);
    font-size: 12px;
    font-weight: 700;
    padding: 6px;
    cursor: pointer;
    position: sticky;
    bottom: 0;
  }
  .browse-all:hover {
    background: var(--accent-weak);
  }
</style>
