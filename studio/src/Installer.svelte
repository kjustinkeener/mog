<script lang="ts">
  import { onMount } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWindow } from '@tauri-apps/api/window'
  import type { SetupState } from './main'

  let { setup }: { setup: SetupState } = $props()

  const REPO = 'https://github.com/kjustinkeener/mog'
  const DOCS_URL = `${REPO}/tree/main/docs`
  const LICENSE_URL = `${REPO}/blob/main/LICENSE`
  const MCP_DOC_URL = `${REPO}/blob/main/docs/mcp-clients.md`

  let desktopShortcut = $state(true)
  let registerMcp = $state(true)
  let addToPath = $state(true)
  let phase = $state<'idle' | 'installing' | 'done' | 'error'>('idle')
  let errMsg = $state('')
  let shining = $state(false)

  const busy = $derived(phase === 'installing')

  // The installer window is transparent so the card floats with its own rounded
  // frame, but app.css paints a solid body behind it. Clear the backgrounds for
  // this window and restore them on teardown so nothing leaks into a later mount.
  onMount(() => {
    const els = [document.documentElement, document.body, document.getElementById('app')].filter(
      Boolean,
    ) as HTMLElement[]
    const prev = els.map((e) => e.style.background)
    for (const e of els) e.style.background = 'transparent'
    return () => els.forEach((e, i) => (e.style.background = prev[i]))
  })

  function close() {
    getCurrentWindow()
      .close()
      .catch(() => {})
  }

  function open(url: string) {
    invoke('open_url', { url }).catch(() => {})
  }

  // Easter egg: a single subtle shimmer sweep across the brand hero. Guarded so a
  // mid-sweep click does not restart it. No sparks/physics: this is a serious app.
  function shimmer() {
    if (shining) return
    shining = true
    setTimeout(() => (shining = false), 900)
  }

  async function install() {
    if (busy) return
    phase = 'installing'
    errMsg = ''
    try {
      const exe = await invoke<string>('perform_install', {
        desktopShortcut,
        registerMcp,
        addToPath,
      })
      phase = 'done'
      // A beat so "Installed" is readable, then hand off to the installed copy.
      setTimeout(() => invoke('launch_installed_and_exit', { exe }).catch(() => {}), 700)
    } catch (e) {
      errMsg = String(e)
      phase = 'error'
    }
  }
</script>

<!-- The whole card drags the window; Tauri auto-excludes buttons/inputs/checkboxes. -->
<div class="wrap" data-tauri-drag-region>
  <div class="card" class:busy data-tauri-drag-region>
    <button class="close" onclick={close} title="Close" aria-label="Close">×</button>

    <!-- pointer-events off so this block drags; the hero turns them back on. -->
    <div class="top">
      <button class="hero" onclick={shimmer} aria-label="Mog">
        <span class="ring"></span>
        <img class="logo" src="/favicon.svg" alt="" />
        <span class="shine" class:go={shining}></span>
      </button>
      <h1 class="word">Mog Studio</h1>
      <p class="tag">Install Mog Studio and the <code>mog</code> engine.</p>
    </div>

    <div class="dest">
      <span class="destlabel">Installs to</span>
      <code class="path" title={setup.install_dir}>{setup.install_dir}</code>
    </div>

    <ul class="does">
      <li>Seeds the mog library</li>
      <li>Creates a Start-Menu shortcut</li>
    </ul>

    <div class="opts">
      <label class="opt">
        <input type="checkbox" bind:checked={addToPath} disabled={busy} />
        <span>Add <code>mog</code> to your PATH (run it from any terminal)</span>
      </label>
      <label class="opt">
        <input type="checkbox" bind:checked={registerMcp} disabled={busy} />
        <span
          >Register the MCP server for
          <button
            type="button"
            class="inlink"
            onclick={(e) => {
              e.preventDefault()
              open(MCP_DOC_URL)
            }}>supported agents</button
          ></span
        >
      </label>
      <label class="opt">
        <input type="checkbox" bind:checked={desktopShortcut} disabled={busy} />
        <span>Add a Desktop shortcut</span>
      </label>
    </div>

    {#if !setup.has_engine}
      <p class="warn">This build has no embedded engine (dev build); install will fail.</p>
    {/if}

    {#if phase === 'done'}
      <div class="state ok"><span class="check">✓</span> Installed</div>
    {:else}
      {#if phase === 'error'}
        <p class="state err">{errMsg}</p>
      {/if}
      <button class="cta" onclick={install} disabled={busy || !setup.has_engine}>
        {busy ? 'Installing…' : phase === 'error' ? 'Try again' : 'Install'}
      </button>
    {/if}

    <div class="foot">
      <div class="links">
        <button class="flink" onclick={() => open(DOCS_URL)}>Docs</button>
        <span class="dot">·</span>
        <button class="flink" onclick={() => open(REPO)}>GitHub</button>
        <span class="dot">·</span>
        <button class="flink" onclick={() => open(LICENSE_URL)}>License</button>
      </div>
      <div class="fver">v{setup.version} · {setup.build_date}</div>
    </div>
  </div>
</div>

<style>
  /* Chrome, not a document: a frameless card that selects text on a drag reads as
     broken. Inputs/buttons stay interactive; nothing here needs copying. */
  .wrap {
    width: 100vw;
    height: 100vh;
    background: transparent;
    user-select: none;
    cursor: default;
  }
  .card {
    position: absolute;
    inset: 0;
    box-sizing: border-box;
    padding: 30px 30px 20px;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    color: var(--text);
    /* Faint brand glow at the top edge over the panel colour. */
    background:
      radial-gradient(120% 66% at 50% -8%, color-mix(in srgb, var(--accent) 24%, transparent), transparent 62%),
      var(--panel);
    /* WebView2 does not make the DOM corners transparent on Windows, so a rounded
       card would show a dark sliver. Fill the window square; the OS rounds the
       window frame. */
    border-radius: 0;
    border: 1px solid var(--border);
    overflow: hidden;
  }

  .close {
    position: absolute;
    top: 8px;
    right: 8px;
    z-index: 2;
    width: 26px;
    height: 26px;
    display: grid;
    place-items: center;
    padding: 0;
    border: none;
    border-radius: 7px;
    background: transparent;
    box-shadow: none;
    color: var(--muted);
    font-size: 18px;
    line-height: 1;
    cursor: pointer;
  }
  .close:hover {
    background: color-mix(in srgb, var(--text) 12%, transparent);
    color: var(--text);
    border: none;
  }

  /* Non-interactive header: clicks fall through to the drag region. */
  .top {
    pointer-events: none;
    width: 100%;
  }

  /* Circular gradient-ring hero around the mog logo; clickable (shimmer egg). */
  .hero {
    pointer-events: auto;
    position: relative;
    width: 92px;
    height: 92px;
    margin: 2px auto 12px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    box-shadow: none;
    cursor: pointer;
    display: grid;
    place-items: center;
    overflow: hidden;
  }
  .hero:hover {
    background: transparent;
    border: none;
  }
  .hero:active {
    transform: scale(0.97);
    transition: transform 0.1s ease;
  }
  .ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    padding: 3px;
    background: var(--brand-grad-strong);
    /* Ring: gradient border via a masked box so the centre shows the panel. */
    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
    filter: drop-shadow(0 2px 10px color-mix(in srgb, var(--accent) 40%, transparent));
  }
  .logo {
    width: 54px;
    height: 54px;
    object-fit: contain;
  }
  /* One-shot diagonal light band, clipped to the circle by the hero's radius. */
  .shine {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    pointer-events: none;
    background: linear-gradient(
      105deg,
      transparent 42%,
      rgba(255, 255, 255, 0.55) 50%,
      transparent 58%
    );
    transform: translateX(-160%);
  }
  .shine.go {
    animation: sweep 0.9s ease-out;
  }
  @keyframes sweep {
    from {
      transform: translateX(-160%);
    }
    to {
      transform: translateX(160%);
    }
  }

  .word {
    margin: 0;
    font-size: 24px;
    font-weight: 700;
    background: var(--brand-grad-strong);
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .tag {
    margin: 6px 0 16px;
    color: var(--muted);
    font-size: 13px;
  }

  .dest {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 3px;
    box-sizing: border-box;
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 10px;
    text-align: left;
  }
  .destlabel {
    color: var(--muted);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .path {
    font-size: 12.5px;
    word-break: break-all;
  }

  .does {
    align-self: stretch;
    margin: 12px 0 4px;
    padding-left: 18px;
    color: var(--muted);
    font-size: 12.5px;
    text-align: left;
  }
  .does li {
    margin: 2px 0;
  }

  .opts {
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 9px;
    margin: 6px 0 4px;
  }
  .opt {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    font-size: 13px;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }
  .opt input {
    margin: 2px 0 0;
    accent-color: var(--accent-strong);
    cursor: pointer;
  }
  .inlink {
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
    font: inherit;
    color: var(--accent-strong);
    text-decoration: underline;
    cursor: pointer;
  }
  .inlink:hover {
    background: none;
    border: none;
    filter: brightness(1.1);
  }

  code {
    background: var(--panel-2);
    padding: 0 4px;
    border-radius: 4px;
    font-family: var(--mono);
  }

  .warn {
    margin: 4px 0 0;
    color: var(--warn);
    font-size: 12.5px;
  }

  .cta {
    width: 100%;
    margin-top: 12px;
    padding: 12px 18px;
    border: none;
    border-radius: 11px;
    font-size: 15px;
    font-weight: 600;
    color: #fff;
    background: var(--brand-grad-strong);
    box-shadow: 0 8px 22px color-mix(in srgb, var(--accent) 40%, transparent);
    cursor: pointer;
    transition:
      transform 0.08s ease,
      filter 0.15s ease,
      box-shadow 0.15s ease;
  }
  .cta:hover:not(:disabled) {
    filter: brightness(1.06);
    border: none;
    box-shadow: 0 10px 26px color-mix(in srgb, var(--accent) 52%, transparent);
  }
  .cta:active:not(:disabled) {
    transform: translateY(1px);
  }
  .cta:disabled {
    opacity: 0.6;
    cursor: default;
    box-shadow: none;
  }

  .state {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    margin: 12px 0 0;
    font-size: 14px;
  }
  .state.ok {
    color: var(--ok);
    font-weight: 600;
  }
  .state.err {
    color: var(--danger);
    font-size: 12.5px;
    word-break: break-word;
    margin-bottom: 0;
  }
  .check {
    font-weight: 700;
  }

  .foot {
    margin-top: auto;
    padding-top: 14px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    color: var(--muted);
  }
  .links {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .flink {
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
    font: inherit;
    font-size: 11px;
    color: var(--accent-strong);
    cursor: pointer;
  }
  .flink:hover {
    background: none;
    border: none;
    text-decoration: underline;
  }
  .dot {
    opacity: 0.5;
  }
  .fver {
    font-size: 11px;
  }
</style>
