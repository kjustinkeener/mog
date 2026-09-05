<script lang="ts">
  import { onMount } from 'svelte';
  import { studio, closeRegexPopover } from '../store.svelte';
  import {
    analyzeRegex,
    initRegexEngine,
    substituteRegex,
    type RegexAnalysis,
    type RegexSubstitution,
  } from '../regexEngine';
  import { explainRegex } from '../regexExplain';
  import RegexBuilder from './RegexBuilder.svelte';
  import Icon from './Icon.svelte';

  // Vetted starter patterns (all linear-engine safe: no backrefs/lookaround).
  const PATTERN_LIBRARY: { label: string; pattern: string }[] = [
    { label: 'Email', pattern: '[\\w.+-]+@[\\w-]+\\.[\\w.-]+' },
    { label: 'URL', pattern: 'https?://[^\\s]+' },
    { label: 'IPv4', pattern: '\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b' },
    {
      label: 'UUID',
      pattern:
        '[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}',
    },
    { label: 'ISO date', pattern: '\\d{4}-\\d{2}-\\d{2}' },
    { label: 'Phone', pattern: '\\+?\\d[\\d\\s().-]{7,}\\d' },
  ];

  // Seeded from the field the popover is anchored to; edited freely and only
  // written back to the field on Apply. Test text starts from the preview input.
  let pattern = $state(studio.regexPopover.seed);
  let replacement = $state('');
  let ignoreCase = $state(false);
  let multiline = $state(false);
  let testText = $state(studio.input);
  let engineReady = $state(false);
  let engineError = $state('');

  let ta: HTMLTextAreaElement;
  let backdrop: HTMLDivElement;
  let root: HTMLDivElement;

  onMount(async () => {
    try {
      await initRegexEngine();
      engineReady = true;
    } catch (e) {
      engineError = `Could not load the regex engine: ${e}`;
    }
  });

  const anchor = $derived(studio.regexPopover.anchor);
  // Anchor below the field, clamped into the viewport.
  const posStyle = $derived.by<string>(() => {
    const a = anchor;
    if (!a) return 'top: 60px; left: 60px;';
    const width = Math.max(a.w, 460);
    const left = Math.min(Math.max(8, a.x), window.innerWidth - width - 8);
    const top = a.y + 4;
    return `top: ${top}px; left: ${left}px; width: ${width}px;`;
  });

  function apply(): void {
    studio.regexPopover.apply?.(pattern);
    closeRegexPopover();
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      e.stopPropagation();
      closeRegexPopover();
    }
  }

  // Close when the user points outside the popover and outside any regex field
  // (clicking another regex field reopens it for that field).
  function onPointerDown(e: PointerEvent): void {
    const t = e.target as HTMLElement | null;
    if (!t) return;
    if (root && root.contains(t)) return;
    if (t.closest('[data-regex-field]')) return;
    closeRegexPopover();
  }

  const analysis = $derived.by<RegexAnalysis | null>(() => {
    if (!engineReady) return null;
    return analyzeRegex(pattern, testText, ignoreCase, multiline);
  });

  function escapeHtml(s: string): string {
    return s.replace(/[&<>]/g, (c) => (c === '&' ? '&amp;' : c === '<' ? '&lt;' : '&gt;'));
  }

  const highlightHtml = $derived.by<string>(() => {
    const a = analysis;
    if (!a || a.matches.length === 0) return escapeHtml(testText) + '\n';
    let out = '';
    let pos = 0;
    a.matches.forEach((m, i) => {
      if (m.start > pos) out += escapeHtml(testText.slice(pos, m.start));
      if (m.start === m.end) {
        out += '<mark class="zero"></mark>';
      } else {
        out += `<mark class="${i % 2 === 0 ? 'even' : 'odd'}">${escapeHtml(
          testText.slice(m.start, m.end),
        )}</mark>`;
      }
      pos = Math.max(pos, m.end);
    });
    if (pos < testText.length) out += escapeHtml(testText.slice(pos));
    return out + '\n';
  });

  function syncScroll(): void {
    if (backdrop && ta) {
      backdrop.scrollTop = ta.scrollTop;
      backdrop.scrollLeft = ta.scrollLeft;
    }
  }

  const matchCount = $derived(analysis?.matches.length ?? 0);
  const hasGroups = $derived((analysis?.matches[0]?.groups.length ?? 0) > 1);

  function groupText(start: number | null, end: number | null): string {
    if (start === null || end === null) return '';
    return testText.slice(start, end);
  }

  const substitution = $derived.by<RegexSubstitution | null>(() => {
    if (!engineReady || replacement === '') return null;
    return substituteRegex(pattern, replacement, testText, ignoreCase, multiline);
  });

  function fancyConstructs(p: string): string[] {
    const out: string[] = [];
    if (/\(\?<[=!]/.test(p)) out.push('lookbehind');
    if (/\(\?[=!]/.test(p)) out.push('lookahead');
    if (/\\\d/.test(p) || /\\k</.test(p)) out.push('backreference');
    return out;
  }
  const constructs = $derived(
    analysis?.engine === 'backtracking' ? fancyConstructs(pattern) : [],
  );

  const explanation = $derived(pattern ? explainRegex(pattern) : []);

  const replBackref = $derived(/\\(\d|k<|g<)/.test(replacement));
  function fixBackrefs(s: string): string {
    return s
      .replace(/\\k<([A-Za-z_]\w*)>/g, '${$1}')
      .replace(/\\g<([A-Za-z_]\w*)>/g, '${$1}')
      .replace(/\\g<(\d+)>/g, '$$$1')
      .replace(/\\(\d+)/g, '$$$1');
  }

  function pickPattern(e: Event): void {
    const sel = e.currentTarget as HTMLSelectElement;
    const i = sel.selectedIndex - 1;
    if (i >= 0 && i < PATTERN_LIBRARY.length) pattern = PATTERN_LIBRARY[i].pattern;
    sel.selectedIndex = 0;
  }
</script>

<svelte:window onkeydown={onKey} onpointerdown={onPointerDown} />

<div class="popover" bind:this={root} style={posStyle} role="dialog" aria-label="Regex helper">
  <div class="head">
    <span class="lbl">Regex helper</span>
    <span class="sub">exact mog flavor</span>
    <span class="grow"></span>
    <button class="apply" onclick={apply} title="Write this pattern into the field"><Icon name="check" />Apply to field</button>
    <button class="x" onclick={closeRegexPopover} title="Close (Esc)" aria-label="Close"><Icon name="x" /></button>
  </div>

  {#if engineError}
    <div class="engine-err">{engineError}</div>
  {/if}

  <div class="pattern-row">
    <span class="slash">/</span>
    <input class="pattern mono" type="text" spellcheck="false" placeholder="pattern" bind:value={pattern} />
    <span class="slash">/</span>
    <label class="flag" title="Ignore case (?i)"><input type="checkbox" bind:checked={ignoreCase} /> i</label>
    <label class="flag" title="Multiline: ^ and $ at line boundaries (?m)"><input type="checkbox" bind:checked={multiline} /> m</label>
    <select class="lib" title="Insert a vetted pattern" onchange={pickPattern}>
      <option value="">library…</option>
      {#each PATTERN_LIBRARY as p}
        <option value={p.pattern}>{p.label}</option>
      {/each}
    </select>
  </div>

  <div class="status">
    {#if !engineReady && !engineError}
      <span class="badge neutral">loading engine…</span>
    {:else if !analysis || analysis.engine === 'empty'}
      <span class="badge neutral">type a pattern</span>
    {:else if analysis.engine === 'invalid'}
      <span class="badge bad">invalid</span>
      <span class="err mono">{analysis.error}</span>
    {:else if analysis.engine === 'backtracking'}
      <span class="badge warn" title="mog runs this on the backtracking engine, O(n²) on large input.">backtracking</span>
      {#if constructs.length > 0}<span class="constructs">uses {constructs.join(', ')}</span>{/if}
      <span class="count">{matchCount} match{matchCount === 1 ? '' : 'es'}</span>
      {#if analysis.truncated}<span class="trunc">first 5000</span>{/if}
    {:else}
      <span class="badge ok">linear</span>
      <span class="count">{matchCount} match{matchCount === 1 ? '' : 'es'}</span>
      {#if analysis.truncated}<span class="trunc">first 5000</span>{/if}
    {/if}
  </div>

  <div class="scroll">
    <details class="section">
      <summary>Explain pattern</summary>
      {#if explanation.length === 0}
        <p class="muted">Type a pattern to see a plain-English breakdown.</p>
      {:else}
        <div class="explain">
          {#each explanation as part, idx (idx)}
            <div class="epart" style="padding-left: {part.depth * 14}px">
              <code class="etok mono">{part.token}</code>
              <span class="edesc">{part.desc}</span>
            </div>
          {/each}
        </div>
      {/if}
    </details>

    <details class="section">
      <summary>Build without regex</summary>
      <RegexBuilder onApply={(p) => (pattern = p)} />
    </details>

    <div class="repl-row">
      <span class="arrow" title="Replacement (preview only)">→</span>
      <input
        class="repl mono"
        type="text"
        spellcheck="false"
        placeholder="replacement preview (use $1, {'${name}'}); optional"
        bind:value={replacement}
      />
    </div>
    {#if replBackref}
      <div class="backref-hint">
        <span class="msg">Looks like a <code>\1</code> backreference. mog uses <code>$1</code> or <code>${'{name}'}</code>.</span>
        <button class="ghost fix" onclick={() => (replacement = fixBackrefs(replacement))}><Icon name="wrench" />Fix</button>
      </div>
    {/if}

    <div class="text-head">
      <span class="lbl">Test text</span>
      <button class="ghost" onclick={() => (testText = studio.input)}><Icon name="download" />Use preview input</button>
    </div>
    <div class="editor">
      <div class="backdrop" bind:this={backdrop} aria-hidden="true">{@html highlightHtml}</div>
      <textarea
        class="input mono"
        bind:this={ta}
        bind:value={testText}
        onscroll={syncScroll}
        spellcheck="false"
        placeholder="Text to test the pattern against…"
      ></textarea>
    </div>

    {#if substitution && replacement !== ''}
      <div class="result">
        <div class="lbl">
          Result
          {#if substitution.engine === 'invalid'}<span class="cap err">invalid pattern</span>
          {:else if !substitution.changed}<span class="cap">(no change)</span>{/if}
        </div>
        <pre class="mono out">{substitution.output}</pre>
      </div>
    {/if}

    {#if hasGroups && analysis}
      <div class="groups">
        <div class="lbl">Capture groups (first match)</div>
        <table>
          <tbody>
            {#each analysis.matches[0].groups as g, gi}
              {#if gi > 0}
                <tr>
                  <td class="gi">{gi}{g.name ? ` (${g.name})` : ''}</td>
                  <td class="gv mono">{g.start === null ? '(absent)' : groupText(g.start, g.end)}</td>
                </tr>
              {/if}
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>

<style>
  .popover {
    position: fixed;
    z-index: 200;
    max-width: 92vw;
    max-height: 78vh;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px 10px;
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    box-shadow: 0 8px 28px color-mix(in srgb, #000 30%, transparent);
  }
  .scroll {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-height: 0;
  }
  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .grow {
    flex: 1;
  }
  .apply {
    flex: none;
    color: var(--accent);
    font-weight: 700;
  }
  .x {
    flex: none;
    color: var(--muted);
    padding: 2px 6px;
  }
  .lbl {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
  }
  .sub {
    color: var(--muted);
    font-size: 11px;
  }
  .engine-err {
    background: var(--danger);
    color: #fff;
    padding: 6px 8px;
    border-radius: 6px;
    font-size: 12px;
  }
  .pattern-row,
  .repl-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .slash,
  .arrow {
    color: var(--muted);
    font-family: var(--mono);
  }
  .pattern,
  .repl {
    flex: 1;
  }
  .flag {
    display: flex;
    align-items: center;
    gap: 3px;
    font-family: var(--mono);
    cursor: pointer;
    user-select: none;
  }
  .flag input {
    width: auto;
  }
  .lib {
    width: auto;
    flex: none;
    color: var(--muted);
    padding: 3px 4px;
  }
  .backref-hint {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--warn-weak);
    color: var(--warn);
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 12px;
  }
  .backref-hint .msg {
    flex: 1;
    line-height: 1.5;
  }
  .backref-hint .fix {
    flex: none;
    color: var(--warn);
    font-weight: 700;
  }
  .constructs {
    color: var(--warn);
    font-size: 12px;
  }
  .section {
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
  }
  .section > summary {
    cursor: pointer;
    padding: 5px 8px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
    user-select: none;
  }
  .section[open] > summary {
    border-bottom: 1px solid var(--border);
  }
  .section > :global(*:not(summary)) {
    padding: 8px;
  }
  .muted {
    margin: 0;
    color: var(--muted);
    font-size: 12px;
  }
  .explain {
    max-height: 180px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .epart {
    display: flex;
    gap: 8px;
    align-items: baseline;
    font-size: 12px;
  }
  .etok {
    color: var(--accent);
    background: var(--panel-2);
    padding: 0 4px;
    border-radius: 3px;
    white-space: pre;
    flex: none;
  }
  .edesc {
    color: var(--text);
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 20px;
    flex-wrap: wrap;
  }
  .badge {
    font-size: 11px;
    font-weight: 700;
    padding: 1px 7px;
    border-radius: 10px;
  }
  .badge.ok {
    background: color-mix(in srgb, var(--ok) 18%, transparent);
    color: var(--ok);
  }
  .badge.warn {
    background: var(--warn-weak);
    color: var(--warn);
  }
  .badge.bad {
    background: color-mix(in srgb, var(--danger) 18%, transparent);
    color: var(--danger);
  }
  .badge.neutral {
    background: var(--panel-2);
    color: var(--muted);
  }
  .count {
    color: var(--muted);
    font-size: 12px;
  }
  .trunc {
    color: var(--warn);
    font-size: 11px;
  }
  .err {
    color: var(--danger);
    font-size: 12px;
    white-space: pre-wrap;
  }
  .text-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .editor {
    position: relative;
    height: 150px;
  }
  .backdrop,
  .input {
    position: absolute;
    inset: 0;
    margin: 0;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font-family: var(--mono);
    font-size: 13px;
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    overflow: auto;
    tab-size: 4;
  }
  .backdrop {
    color: var(--text);
    background: var(--panel);
    pointer-events: none;
    z-index: 0;
  }
  .input {
    color: transparent;
    background: transparent;
    caret-color: var(--text);
    resize: none;
    z-index: 1;
  }
  .backdrop :global(mark) {
    border-radius: 2px;
    color: inherit;
  }
  .backdrop :global(mark.even) {
    background: color-mix(in srgb, var(--accent) 32%, transparent);
  }
  .backdrop :global(mark.odd) {
    background: color-mix(in srgb, var(--accent) 18%, transparent);
  }
  .backdrop :global(mark.zero) {
    background: var(--warn);
    width: 2px;
    display: inline-block;
    height: 1em;
    vertical-align: text-bottom;
  }
  .result {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .result .out {
    margin: 0;
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    overflow: auto;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 13px;
    line-height: 1.5;
    max-height: 160px;
  }
  .cap {
    color: var(--muted);
    font-size: 11px;
  }
  .cap.err {
    color: var(--danger);
  }
  .groups {
    max-height: 160px;
    overflow: auto;
    border-top: 1px solid var(--border);
    padding-top: 6px;
  }
  .groups table {
    border-collapse: collapse;
    width: 100%;
    margin-top: 4px;
  }
  .groups td {
    padding: 2px 6px;
    vertical-align: top;
    border-bottom: 1px solid var(--border);
  }
  .gi {
    color: var(--muted);
    white-space: nowrap;
    width: 1%;
  }
  .gv {
    font-size: 12px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  code {
    font-family: var(--mono);
    background: var(--panel-2);
    padding: 0 3px;
    border-radius: 3px;
  }
</style>
