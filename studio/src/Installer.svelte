<script lang="ts">
  import { invoke } from '@tauri-apps/api/core'
  import type { SetupState } from './main'

  let { setup }: { setup: SetupState } = $props()

  let desktopShortcut = $state(true)
  let phase = $state<'idle' | 'installing' | 'error'>('idle')
  let errMsg = $state('')

  async function install() {
    phase = 'installing'
    errMsg = ''
    try {
      const exe = await invoke<string>('perform_install', {
        desktopShortcut,
      })
      // Hand off to the installed copy; this window closes.
      await invoke('launch_installed_and_exit', { exe })
    } catch (e) {
      errMsg = String(e)
      phase = 'error'
    }
  }
</script>

<div class="card">
  <div class="brand">
    <img src="/favicon.svg" alt="" width="44" height="44" />
    <div class="titles">
      <h1>Mog Studio</h1>
      <span class="ver">v{setup.version} · {setup.build_date}</span>
    </div>
  </div>

  <p class="lead">
    Install Mog Studio and the <code>mog</code> engine for your account.
  </p>

  <div class="dest">
    <span class="label">Installs to</span>
    <code class="path">{setup.install_dir}</code>
  </div>

  <ul class="does">
    <li>Adds <code>mog</code> to your PATH and registers the MCP server</li>
    <li>Seeds the recipe library</li>
    <li>Creates a Start-Menu shortcut</li>
  </ul>

  <label class="opt">
    <input type="checkbox" bind:checked={desktopShortcut} disabled={phase === 'installing'} />
    Add a Desktop shortcut
  </label>

  {#if !setup.has_engine}
    <p class="warn">This build has no embedded engine (dev build); install will fail.</p>
  {/if}

  {#if phase === 'error'}
    <p class="err">{errMsg}</p>
  {/if}

  <button class="go" onclick={install} disabled={phase === 'installing' || !setup.has_engine}>
    {phase === 'installing' ? 'Installing…' : 'Install'}
  </button>
</div>

<style>
  :global(body) { margin: 0; }
  .card {
    box-sizing: border-box;
    width: 100vw;
    min-height: 100vh;
    padding: 22px 26px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    background: var(--bg);
    color: var(--text);
    font: 14px/1.45 system-ui, sans-serif;
  }
  .brand { display: flex; align-items: center; gap: 12px; }
  .titles { display: flex; flex-direction: column; }
  h1 {
    margin: 0;
    font-size: 22px;
    background: var(--brand-grad-strong);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .ver { color: var(--muted); font-size: 12px; }
  .lead { margin: 0; color: var(--text); }
  .dest {
    display: flex;
    flex-direction: column;
    gap: 3px;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
  }
  .label { color: var(--muted); font-size: 11px; text-transform: uppercase; letter-spacing: .04em; }
  .path { font-size: 12.5px; word-break: break-all; }
  .does { margin: 0; padding-left: 18px; color: var(--muted); font-size: 13px; }
  .does li { margin: 2px 0; }
  code { background: var(--panel-2); padding: 0 4px; border-radius: 4px; }
  .opt { display: flex; align-items: center; gap: 8px; font-size: 13px; }
  .warn { margin: 0; color: var(--warn); font-size: 12.5px; }
  .err { margin: 0; color: var(--danger); font-size: 12.5px; word-break: break-word; }
  .go {
    margin-top: auto;
    padding: 11px;
    border: none;
    border-radius: 9px;
    font-size: 15px;
    font-weight: 600;
    color: #fff;
    background: var(--brand-grad-strong);
    cursor: pointer;
  }
  .go:disabled { opacity: .6; cursor: default; }
</style>
