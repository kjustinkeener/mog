<script lang="ts">
  // Self-contained "update from the registry" control. Fully wired to the
  // `mog_update` Tauri command; drop it anywhere (toolbar, Settings, browser
  // header) and restyle as needed. Does a --check first so the user sees what
  // would change, then applies on confirm.
  import { mogUpdate, type UpdateResult } from '../api';

  let busy = $state(false);
  let plan = $state<UpdateResult | null>(null);
  let error = $state<string | null>(null);

  function recipeCount(r: UpdateResult | null): number {
    if (!r?.recipes) return 0;
    const p = r.recipes;
    return p.added.length + p.changed.length + p.revoked.length;
  }

  async function check() {
    busy = true;
    error = null;
    try {
      plan = await mogUpdate(true, false, false);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function apply() {
    busy = true;
    error = null;
    try {
      plan = await mogUpdate(false, false, false);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<div class="update">
  <button onclick={check} disabled={busy}>
    {busy ? 'Checking…' : 'Check for updates'}
  </button>

  {#if error}
    <span class="err">{error}</span>
  {:else if plan}
    {@const rc = recipeCount(plan)}
    {@const eng = plan.engine?.update_available ?? false}
    {#if plan.check}
      {#if rc === 0 && !eng}
        <span class="ok">Up to date.</span>
      {:else}
        <span class="avail">
          {#if rc > 0}{rc} recipe{rc === 1 ? '' : 's'}{/if}
          {#if rc > 0 && eng}, {/if}
          {#if eng}engine {plan.engine?.version}{/if}
          available.
        </span>
        <button onclick={apply} disabled={busy}>Update</button>
      {/if}
    {:else}
      <span class="ok">
        {#if rc > 0}Updated {rc} recipe{rc === 1 ? '' : 's'}.{/if}
        {#if plan.engine?.applied}
          Engine updated to {plan.engine.version} (restart to use it).
        {/if}
        {#if rc === 0 && !plan.engine?.applied}Already up to date.{/if}
      </span>
    {/if}
  {/if}
</div>

<style>
  .update {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
  }
  .ok {
    color: var(--fg-muted, #6b7280);
  }
  .avail {
    color: var(--accent, #2563eb);
  }
  .err {
    color: var(--danger, #dc2626);
  }
</style>
