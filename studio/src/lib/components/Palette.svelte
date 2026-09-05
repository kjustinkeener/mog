<script lang="ts">
  import type { ActionDescriptor } from '../types';
  import { addStep, catLabel, orderedCategories, studio } from '../store.svelte';

  let query = $state('');

  const filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    // Order and labels come from the engine's category registry when loaded, else
    // descriptor order + title-cased names. Either way every category the engine
    // ships appears; nothing is gated by a hardcoded list.
    const groups: { cat: string; items: ActionDescriptor[] }[] = [];
    for (const cat of orderedCategories()) {
      const items = studio.descriptors.filter((d) => {
        if (d.category !== cat) return false;
        if (!q) return true;
        return (
          d.name.includes(q) ||
          d.label.toLowerCase().includes(q) ||
          d.summary.toLowerCase().includes(q) ||
          d.aliases.some((a) => a.includes(q))
        );
      });
      if (items.length) groups.push({ cat, items });
    }
    return groups;
  });
</script>

<div class="palette">
  <div class="search">
    <input type="text" placeholder="Search actions…" bind:value={query} />
  </div>
  <div class="list">
    {#each filtered as group (group.cat)}
      <div class="group">
        <div class="cat" style={`color: var(--cat-${group.cat}, var(--cat-fallback))`}>{catLabel(group.cat)}</div>
        {#each group.items as d (d.name)}
          <button class="item" title={d.summary} onclick={() => addStep(d)}>
            <span class="dot" style={`background: var(--cat-${d.category}, var(--cat-fallback))`}></span>
            <span class="name">{d.label}</span>
            <span class="plus">+</span>
          </button>
        {/each}
      </div>
    {/each}
    {#if filtered.length === 0}
      <p class="empty">No actions match.</p>
    {/if}
  </div>
</div>

<style>
  .palette {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--panel);
    border-right: 1px solid var(--border);
  }
  .search {
    padding: 8px;
    border-bottom: 1px solid var(--border);
  }
  .list {
    overflow-y: auto;
    padding: 6px;
    flex: 1;
  }
  .group {
    margin-bottom: 8px;
  }
  .cat {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 700;
    padding: 4px 4px 2px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 7px;
    width: 100%;
    text-align: left;
    border: none;
    background: transparent;
    padding: 4px 6px;
    border-radius: 5px;
  }
  .item:hover {
    background: var(--accent-weak);
  }
  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
  }
  .name {
    flex: 1;
  }
  .plus {
    color: var(--muted);
  }
  .empty {
    color: var(--muted);
    padding: 8px;
  }
</style>
