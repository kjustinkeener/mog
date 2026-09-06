<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { cachedCatalog, loadCatalog } from '../marketCatalog';
  import {
    marketSearch,
    marketShow,
    listActions,
    type MogHit,
    type MogDetail,
  } from '../api';
  import { renderMarkdown } from '../markdown';
  import { loadRank, recentMogs } from '../recentMogs.svelte';
  import { logLine } from '../log';
  import Icon from './Icon.svelte';

  let {
    onclose,
    onopen,
    onrun,
    oninsert,
    initialQuery = '',
    inline = false,
  }: {
    onclose: () => void;
    onopen: (detail: MogDetail) => void;
    onrun: (detail: MogDetail) => void;
    oninsert: (name: string) => void;
    initialQuery?: string;
    /** Render as a filled panel (inside the Browse tab) rather than a fixed overlay. */
    inline?: boolean;
  } = $props();

  // Seed the search box from the launcher's query once (a deliberate snapshot).
  // Curated grouping for the tag facet rail. Each catalog tag is placed in the
  // first category that lists it; anything unlisted falls into "Other", so a new
  // tag is never dropped, just uncategorized until curated here.
  // Ordered most-likely-used first (matching the seeded recipe popularity):
  // everyday whitespace/list hygiene up top, niche language/format conversion last.
  const TAG_CATEGORIES: { label: string; tags: string[] }[] = [
    {
      label: 'Whitespace & EOL',
      tags: ['whitespace', 'trailing whitespace', 'trim', 'eol', 'crlf', 'lf', 'line endings', 'newline', 'final newline', 'tabs', 'spaces', 'indent', 'untabify', 'expand', 'hygiene', 'dos2unix', 'unix', 'mac', 'lint', 'blank', 'invisible', 'zero-width'],
    },
    {
      label: 'Lists & text',
      tags: ['list', 'sort', 'dedupe', 'unique', 'duplicate', 'order', 'reorder', 'reverse', 'shift', 'swap', 'merge', 'select', 'filter', 'cut', 'fill', 'chunk', 'unwrap', 'reflow', 'align', 'compact', 'preserve', 'requirements', 'wordlist', 'text', 'bullet', 'bulletize', 'numbered', 'ordered-list', 'checkbox', 'checklist', 'prefix', 'comment', 'comment out', 'block', 'disable', 'join', 'insert', 'paste', 'extract', 'set', 'line', 'line-numbers', 'column', 'columns', 'records', 'reshape', 'string', 'fragment'],
    },
    {
      label: 'Data & formats',
      tags: ['csv', 'tsv', 'json', 'jsonl', 'ndjson', 'json5', 'jsonc', 'yaml', 'toml', 'ini', 'xml', 'html', 'edi', 'x12', 'hl7', 'frontmatter', 'fixed-width', 'data', 'dataset', 'table', 'spreadsheet', 'merged-cells', 'header', 'headers', 'keys', 'values', 'array', 'pivot', 'unpivot', 'melt', 'transpose', 'wide', 'long', 'cast', 'lookup', 'vlookup', 'reconcile', 'intersect', 'enrich', 'interchange', 'format', 'pretty', 'quote', 'trailing-comma', 'delimiter', 'dotenv', 'env', 'properties', 'config', 'logfmt', 'minify', 'plaintext', 'unescape', 'analytics', 'finance', 'outputs'],
    },
    {
      label: 'Databases & SQL',
      tags: ['sql', 'mssql', 'sqlserver', 'tsql', 'mysql', 'mysqldump', 'postgresql', 'postgres', 'oracle', 'sqlite', 'duckdb', 'snowflake', 'bigquery', 'redshift', 'databricks', 'spark', 'presto', 'trino', 'database', 'dbt', 'warehouse', 'migration', 'schema', 'in-clause'],
    },
    {
      label: 'Code',
      tags: ['code', 'codemod', 'codegen', 'javascript', 'java', 'python', 'typescript', 'shell', 'bash', 'identifier', 'case', 'camelcase', 'snake_case', 'kebab-case', 'slug', 'commonjs', 'esm', 'modules', 'import', 'imports', 'jupyter', 'ipynb', 'notebook', 'repl', 'console', 'debug', 'filename', 'directory', 'idempotent', 'template', 'gitignore', 'dev'],
    },
    {
      label: 'DevOps & tooling',
      tags: ['docker', 'compose', 'configmap', 'k8s', 'kubernetes', 'ci', 'git', 'changelog', 'changes', 'release', 'semver', 'version', 'apiversion', 'patch', 'ops', 'testing', 'airflow'],
    },
    {
      label: 'Docs & references',
      tags: ['docs', 'markdown', 'mrkdwn', 'slack', 'prose', 'writing', 'headings', 'typography', 'paragraph', 'sentences', 'bold', 'italic', 'emphasis', 'blockquote', 'quotes', 'dashes', 'toc', 'example', 'bibliography', 'bibtex', 'citation', 'ris', 'license', 'spdx', 'copyright', 'links', 'review', 'fence', 'tasklist'],
    },
    {
      label: 'Web & network',
      tags: ['url', 'web', 'network', 'dns', 'hosts', 'domains', 'ip', 'querystring', 'percent-encoding', 'blocklist', 'allowlist', 'adblock', 'canonical'],
    },
    {
      label: 'Dates & calendar',
      tags: ['date', 'time', 'year', 'epoch', 'timestamp', 'calendar', 'ical', 'icalendar', 'srt', 'vtt', 'webvtt', 'subtitles'],
    },
    {
      label: 'Encode & escape',
      tags: ['encode', 'decode', 'base64', 'b64', 'escape', 'entities', 'escape codes', 'ascii', 'unicode', 'encoding', 'mojibake', 'accents', 'locale'],
    },
    {
      label: 'Security & privacy',
      tags: ['redact', 'redaction', 'secrets', 'pii', 'mask', 'sanitize', 'anonymize', 'pseudonymize', 'password', 'token', 'jwt', 'auth', 'ldap', 'ldif', 'email', 'phone', 'credit-card', 'pci', 'healthcare', 'audit', 'safety', 'contacts', 'vcard', 'share', 'privacy', 'security'],
    },
    {
      label: 'AI & datasets',
      tags: ['ai', 'ml', 'llm', 'model', 'embedding', 'rag', 'agent', 'chat', 'claude', 'openai', 'fine-tuning', 'sharegpt', 'transcript'],
    },
    {
      label: 'Numbers & IDs',
      tags: ['number', 'decimal', 'count', 'tally', 'hash', 'uuid', 'level'],
    },
    {
      label: 'Terminal & logs',
      tags: ['ansi', 'color', 'decolorize', 'vt100', 'terminal', 'console', 'log', 'clf', 'access-log', 'stacktrace', 'observability', 'diff', 'compare', 'baseline', 'validate'],
    },
    {
      label: 'Transform',
      tags: ['convert', 'normalize', 'cleanup', 'clean', 'strip', 'enrich', 'repair', 'inspect', 'export', 'update', 'reset', 'single-action'],
    },
  ];

  let query = $state(untrack(() => initialQuery));
  let catalog = $state<MogHit[]>([]); // full list, for the tag facet options
  let results = $state<MogHit[]>([]); // ranked results for the current query
  let selectedTags = $state<Set<string>>(new Set());
  let listError = $state('');

  let selectedName = $state<string | null>(null);
  let detail = $state<MogDetail | null>(null);
  let detailLoading = $state(false);
  let detailError = $state('');

  let timer: ReturnType<typeof setTimeout> | null = null;

  // --- Temporary: Marketplace initial-load profiling. Flip PERF off (or strip
  // this block and its call sites) once the load path is tuned. ---
  const PERF = true;
  let perfT0 = 0;
  function plog(label: string): void {
    if (PERF) logLine(`[mkt] +${(performance.now() - perfT0).toFixed(1)}ms  ${label}`);
  }
  async function timed<T>(label: string, fn: () => Promise<T>): Promise<T> {
    const t = performance.now();
    const r = await fn();
    if (PERF) logLine(`[mkt]   ${label}: ${(performance.now() - t).toFixed(1)}ms`);
    return r;
  }

  onMount(async () => {
    perfT0 = performance.now();
    plog('onMount start');
    let catalogOk = true;
    const cached = cachedCatalog();
    if (cached) {
      catalog = cached;
      plog(`catalog from cache (${catalog.length} recipes)`);
    } else {
      try {
        catalog = await timed(`loadCatalog() [catalog]`, () => loadCatalog());
        plog(`catalog set (${catalog.length} recipes)`);
      } catch {
        catalogOk = false; // facet options are best-effort
      }
    }
    // An empty initial query returns the same list as the catalog we just fetched,
    // so reuse it instead of calling marketSearch('') a second time. Only run a
    // separate search when there's an actual query (or the catalog fetch failed).
    if (catalogOk && query.trim() === '') {
      results = catalog;
      plog('results reuse catalog (deduped empty-query call)');
    } else {
      await timed('runSearch()', runSearch);
    }
    plog(`results set (${results.length}); tagGroups=${tagGroups.length}, shown=${shownCount}`);
    // Best-effort: the action count feeds a landing stat, not core browsing.
    listActions().then((a) => (actionCount = a.length)).catch(() => {});
    await tick();
    plog('after tick (DOM updated)');
    requestAnimationFrame(() => plog('after paint (rAF)'));
  });

  async function runSearch(): Promise<void> {
    try {
      results = await marketSearch(query);
      listError = '';
    } catch (e) {
      results = [];
      listError = String(e);
    }
  }
  function scheduleSearch(): void {
    if (timer) clearTimeout(timer);
    timer = setTimeout(runSearch, 150);
  }

  const allTags = $derived.by<Set<string>>(() => {
    const t = performance.now();
    const s = new Set<string>();
    for (const r of catalog) for (const t of r.tags) s.add(t);
    if (PERF) logLine(`[mkt]   allTags derive: ${(performance.now() - t).toFixed(1)}ms (${s.size} tags / ${catalog.length} recipes)`);
    return s;
  });

  // Tag -> usage count across the catalog, so the uncategorized "Other" tail (and
  // any long group) can be ordered by popularity instead of alphabetically.
  const tagCounts = $derived.by<Map<string, number>>(() => {
    const m = new Map<string, number>();
    for (const r of catalog) for (const t of r.tags) m.set(t, (m.get(t) ?? 0) + 1);
    return m;
  });

  // Bucket the catalog's actual tags into the curated categories (+ an "Other"
  // group for anything uncategorized), keeping only non-empty groups.
  const tagGroups = $derived.by<{ label: string; tags: string[] }[]>(() => {
    const t0 = performance.now();
    const present = allTags;
    const claimed = new Set<string>();
    const groups: { label: string; tags: string[] }[] = [];
    for (const cat of TAG_CATEGORIES) {
      const tags = cat.tags.filter((t) => present.has(t));
      for (const t of tags) claimed.add(t);
      if (tags.length > 0) groups.push({ label: cat.label, tags });
    }
    const other = [...present]
      .filter((t) => !claimed.has(t))
      .sort((a, b) => (tagCounts.get(b) ?? 0) - (tagCounts.get(a) ?? 0) || a.localeCompare(b));
    if (other.length > 0) groups.push({ label: 'Other', tags: other });
    if (PERF) logLine(`[mkt]   tagGroups derive: ${(performance.now() - t0).toFixed(1)}ms (${groups.length} groups)`);
    return groups;
  });

  // Popular-first: seeded download_count descending. Applied when browsing (no
  // query); a text search keeps the engine's relevance ranking.
  const byPopularity = (a: MogHit, b: MogHit): number => b.download_count - a.download_count;

  // The engine's README repeats the recipe's title + description, which the detail
  // header already shows. Drop everything before the first "## " section so only the
  // value-add (Run / Example / Pipeline ...) renders; empty if there's nothing past
  // the intro.
  function readmeBody(md: string): string {
    const m = md.match(/^##\s.*/m);
    return m && m.index !== undefined ? md.slice(m.index) : '';
  }

  // Float recently-loaded mogs to the top, most-recently-loaded first, preserving the
  // existing relative order of everything else (relevance for a query, popularity for
  // a browse). Reads recentMogs.list so the order updates live after a load.
  function recentsFirst(list: MogHit[]): MogHit[] {
    void recentMogs.list; // reactive dependency
    const recent: MogHit[] = [];
    const rest: MogHit[] = [];
    for (const r of list) (loadRank(r.name) === Infinity ? rest : recent).push(r);
    recent.sort((a, b) => loadRank(a.name) - loadRank(b.name));
    return [...recent, ...rest];
  }

  // Strict intersection: ALL selected tags present. Popular-first when browsing.
  const andMatches = $derived.by<MogHit[]>(() => {
    const list = results.filter((r) => {
      for (const t of selectedTags) if (!r.tags.includes(t)) return false;
      return true;
    });
    const ordered = query.trim() === '' ? [...list].sort(byPopularity) : list;
    return recentsFirst(ordered);
  });

  // Near-misses: match ANY selected tag but not ALL. Only meaningful with 2+ tags;
  // shown below a separator so the exact matches stay on top.
  const orExtras = $derived.by<MogHit[]>(() => {
    if (selectedTags.size < 2) return [];
    const inAnd = new Set(andMatches.map((r) => r.name));
    const list = results.filter((r) => {
      if (inAnd.has(r.name)) return false;
      for (const t of selectedTags) if (r.tags.includes(t)) return true;
      return false;
    });
    return query.trim() === '' ? [...list].sort(byPopularity) : list;
  });

  const shownCount = $derived(andMatches.length + orExtras.length);

  // Landing page (shown until a mog is selected): headline stats + a few
  // popular picks so the empty detail pane invites a click instead of sitting blank.
  const totalTags = $derived(allTags.size);
  const featured = $derived.by<MogHit[]>(() => [...catalog].sort(byPopularity).slice(0, 6));
  let actionCount = $state(0); // engine transform primitives, for the landing stat

  function toggleTag(t: string): void {
    const next = new Set(selectedTags);
    if (next.has(t)) next.delete(t);
    else next.add(t);
    selectedTags = next;
  }

  async function select(name: string): Promise<void> {
    selectedName = name;
    detail = null;
    detailError = '';
    detailLoading = true;
    try {
      detail = await timed(`marketShow('${name}')`, () => marketShow(name));
      if (PERF) logLine(`[mkt]   detail readme=${detail.readme?.length ?? 0} chars, .mog=${detail.content.length} chars`);
    } catch (e) {
      detailError = String(e);
    } finally {
      detailLoading = false;
    }
  }

  // Double-click a result: select it and jump straight to Run. The detail may not
  // be loaded yet (single-click fetches it lazily), so fetch here if needed.
  async function openRun(name: string): Promise<void> {
    let d = detail && selectedName === name ? detail : null;
    if (!d) {
      await select(name);
      d = detail;
    }
    if (d) onrun(d);
  }

  function onKey(e: KeyboardEvent): void {
    if (e.key === 'Escape' && !inline) onclose();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="overlay" class:inline>
  {#if !inline}
    <header class="bar">
      <span class="grow"></span>
      <button class="close" onclick={onclose} title="Close (Esc)"><Icon name="x" /></button>
    </header>
  {/if}

  <div class="cols">
    <aside class="rail">
      <input
        class="search"
        type="text"
        spellcheck="false"
        placeholder="Search mogs…"
        bind:value={query}
        oninput={scheduleSearch}
      />
      <div class="facet-head">
        <span class="facet-label">Tags</span>
        {#if selectedTags.size > 0}
          <button class="clear-link" title="Clear tags" aria-label="Clear tags" onclick={() => (selectedTags = new Set())}><Icon name="x" size={13} /></button>
        {/if}
      </div>
      {#each tagGroups as g (g.label)}
        <div class="tag-group">
          <div class="group-label">{g.label}</div>
          <div class="tags-facet">
            {#each g.tags as t (t)}
              <button
                class="tag-chip"
                class:on={selectedTags.has(t)}
                onclick={() => toggleTag(t)}
              >{t}</button>
            {/each}
          </div>
        </div>
      {/each}
    </aside>

    {#snippet mogRow(h: MogHit)}
      <button
        class="row"
        class:sel={selectedName === h.name}
        onclick={() => select(h.name)}
        ondblclick={() => openRun(h.name)}
        title="Double-click to run"
      >
        <div class="row1">
          <span class="name mono">{h.name}</span>
          <span class="grow"></span>
        </div>
        <div class="desc">{h.description}</div>
      </button>
    {/snippet}

    <div class="list">
      <div class="list-strip">
        <strong class="mk-title">Mog Marketplace</strong>
        <span class="grow"></span>
        <span class="hint">{shownCount} of {results.length}{results.length === catalog.length ? '' : ` (${catalog.length} total)`}</span>
      </div>
      {#if listError}
        <div class="msg err">{listError}</div>
      {:else if shownCount === 0}
        <div class="msg">No mogs match.</div>
      {:else}
        {#each andMatches as h (h.name)}{@render mogRow(h)}{/each}
        {#if orExtras.length > 0}
          <div class="or-sep">matching any selected tag</div>
          {#each orExtras as h (h.name)}{@render mogRow(h)}{/each}
        {/if}
      {/if}
    </div>

    <div class="detail">
      {#if detailLoading}
        <div class="msg">Loading…</div>
      {:else if detailError}
        <div class="msg err">{detailError}</div>
      {:else if !detail}
        <div class="landing">
          <div class="hero">
            <div class="glyph" aria-hidden="true">
              <svg viewBox="-10 -10 120 120">
                <defs>
                  <linearGradient id="mogGrad" x1="0" y1="0" x2="1" y2="1">
                    <stop offset="0.3" stop-color="#00a2a9" />
                    <stop offset="1" stop-color="#ff8b00" />
                  </linearGradient>
                  <filter id="mogRound" filterUnits="userSpaceOnUse" x="-15" y="-15" width="130" height="130">
                    <feGaussianBlur in="SourceGraphic" stdDeviation="1.0" result="b" />
                    <feColorMatrix in="b" type="matrix"
                      values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 22 -9" />
                  </filter>
                  <mask id="mogMask" maskUnits="userSpaceOnUse" x="-10" y="-10" width="120" height="120">
                    <g fill="#fff" stroke="#fff" filter="url(#mogRound)">
                      {#each [0, 40, 80, 120, 160, 200, 240, 280, 320] as a (a)}
                        <polygon points="39,17 61,17 55,2 45,2" transform="rotate({a} 50 50)" />
                      {/each}
                      <circle cx="50" cy="50" r="36" fill="none" stroke-width="6" />
                      <path d="M40 34 L68 50 L40 66 Z" stroke="none" />
                    </g>
                  </mask>
                </defs>
                <rect x="-10" y="-10" width="120" height="120" fill="url(#mogGrad)" mask="url(#mogMask)" />
              </svg>
            </div>
            <h1 class="htitle">Mog Marketplace</h1>
            <p class="tagline">
              Hundreds of pre-built transformation scripts for code, data,
              configuration, and other text files.
            </p>
            <div class="stats">
              <div class="stat"><span class="num">{catalog.length}</span><span class="lbl">mogs</span></div>
              {#if actionCount > 0}<div class="stat"><span class="num">{actionCount}</span><span class="lbl">actions</span></div>{/if}
              <div class="stat"><span class="num">{tagGroups.length}</span><span class="lbl">categories</span></div>
              <div class="stat"><span class="num">{totalTags}</span><span class="lbl">tags</span></div>
            </div>
          </div>

          {#if featured.length > 0}
            <div class="section-label">Popular picks</div>
            <div class="cards">
              {#each featured as h (h.name)}
                <button class="card" onclick={() => select(h.name)} ondblclick={() => openRun(h.name)} title="Double-click to run">
                  <span class="card-name mono">{h.name}</span>
                  <span class="card-desc">{h.description}</span>
                  {#if h.tags.length > 0}
                    <span class="card-tags">
                      {#each h.tags.slice(0, 3) as t (t)}<span class="ct">{t}</span>{/each}
                    </span>
                  {/if}
                </button>
              {/each}
            </div>
          {/if}

          <div class="section-label">How it works</div>
          <ol class="steps">
            <li><span class="step-n">1</span><span><strong>Find a mog</strong> that does what you need. Search by keyword or filter by tag, then open it to read what it changes and how.</span></li>
            <li><span class="step-n">2</span><span><strong>Point it at your files.</strong> Run a single file, a folder, or a whole glob (*.*). It targets exactly the lines that match, and skips the rest.</span></li>
            <li><span class="step-n">3</span><span><strong>Preview the impact.</strong> See every edit as a line-by-line diff before touching anything, so there are no surprises.</span></li>
            <li><span class="step-n">4</span><span><strong>Apply with confidence.</strong> The same mechanical edit lands identically across every match.</span></li>
            <li><span class="step-n">5</span><span><strong>Reuse and automate.</strong> Insert a mog as a repeatable step, or open it in the editor to fork your own.</span></li>
            <li><span class="step-n">6</span><span><strong>Take it to the command line.</strong> Every mog runs the same from a terminal, so drop it into scripts, git hooks, CI, or data pipelines:<pre class="step-code mono">mog -m recipe.mog src/**/*.ts</pre></span></li>
          </ol>
        </div>
      {:else}
        <div class="detail-head">
          <div class="dh-row">
            <strong class="dname">{detail.name}</strong>
            {#if detail.fixture}
              <span class="fix" class:pass={detail.fixture.pass} class:fail={!detail.fixture.pass}>
                {detail.fixture.pass ? 'fixture ✓' : 'fixture ✗'}
              </span>
            {/if}
          </div>
          <p class="ddesc">{detail.description}</p>
          {#if detail.tags.length > 0}
            <div class="dtags">
              {#each detail.tags as t (t)}<span class="tag">{t}</span>{/each}
            </div>
          {/if}
          <div class="actions">
            <button onclick={() => detail && onrun(detail)}><Icon name="play" />Run</button>
            <button onclick={() => detail && onopen(detail)}><Icon name="edit" />Edit</button>
            <button onclick={() => detail && oninsert(detail.name)}><Icon name="plus" />Insert run_mog step</button>
          </div>
        </div>
        {@const readme = readmeBody(detail.readme ?? '')}
        {#if readme}
          <div class="content-label">Readme</div>
          <div class="readme">{@html renderMarkdown(readme)}</div>
        {/if}
        <div class="content-label">.mog</div>
        <pre class="content mono">{detail.content}</pre>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: flex;
    flex-direction: column;
    background: var(--bg);
  }
  /* As the Browse tab: fill the content area in flow instead of covering the app. */
  .overlay.inline {
    position: relative;
    inset: auto;
    z-index: auto;
    flex: 1;
    min-height: 0;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .hint {
    color: var(--muted);
    font-size: 12px;
  }
  .list-strip {
    position: sticky;
    top: 0;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
  }
  .mk-title {
    color: var(--accent);
    font-size: 12px;
  }
  .grow {
    flex: 1;
  }
  .cols {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 220px 320px 1fr;
  }
  .rail {
    border-right: 1px solid var(--border);
    padding: 10px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .facet-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-top: 4px;
  }
  .facet-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
  }
  .clear-link {
    display: inline-flex;
    align-items: center;
    background: none;
    border: none;
    padding: 0;
    line-height: 1;
    color: var(--muted);
    cursor: pointer;
  }
  .clear-link:hover {
    color: var(--text);
  }
  .tag-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .group-label {
    font-size: 10px;
    color: var(--muted);
    font-weight: 600;
  }
  .tags-facet {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag-chip {
    font-size: 11px;
    padding: 1px 7px;
    border-radius: 10px;
    color: var(--muted);
  }
  .tag-chip.on {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
  }
  .list {
    border-right: 1px solid var(--border);
    overflow-y: auto;
    background: var(--bg);
  }
  .row {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border);
    border-radius: 0;
    padding: 8px 10px;
    cursor: pointer;
  }
  .row:hover {
    background: var(--panel-2);
  }
  .row.sel {
    background: var(--accent-weak);
  }
  .or-sep {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 10px;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    background: var(--panel-2);
    border-bottom: 1px solid var(--border);
  }
  .or-sep::before,
  .or-sep::after {
    content: '';
    flex: 1;
    height: 1px;
    background: var(--border);
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
  .desc {
    color: var(--muted);
    font-size: 11px;
    line-height: 1.35;
    margin-top: 2px;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .detail {
    overflow-y: auto;
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .detail-head {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .dh-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .dname {
    font-size: 16px;
  }
  .src {
    font-size: 11px;
    color: var(--muted);
    background: var(--panel-2);
    border-radius: 3px;
    padding: 1px 6px;
  }
  .fix {
    font-size: 11px;
    border-radius: 10px;
    padding: 1px 7px;
    font-weight: 700;
  }
  .fix.pass {
    color: var(--ok);
    background: color-mix(in srgb, var(--ok) 16%, transparent);
  }
  .fix.fail {
    color: var(--danger);
    background: color-mix(in srgb, var(--danger) 16%, transparent);
  }
  .ddesc {
    margin: 0;
    line-height: 1.5;
    color: var(--text);
  }
  .dtags {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .tag {
    font-size: 10px;
    color: var(--muted);
    background: var(--panel-2);
    border-radius: 3px;
    padding: 0 5px;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin: 4px 0 6px;
  }
  .content-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--muted);
    font-weight: 700;
    margin-bottom: 4px;
  }
  .content {
    margin: 0;
    padding: 10px 12px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: auto;
    white-space: pre;
    font-size: 12px;
    line-height: 1.45;
    /* Don't let the flex column shrink the code block to a sliver once the README
       is stacked below it; take natural height and let .detail scroll. */
    flex: none;
    max-height: 45vh;
  }
  .msg {
    padding: 14px;
    color: var(--muted);
  }
  .msg.err {
    color: var(--danger);
    white-space: pre-wrap;
  }
  .readme + .content-label {
    margin-top: 14px;
  }
  .readme {
    font-size: 13px;
    line-height: 1.6;
    padding-bottom: 12px;
    overflow-wrap: anywhere;
    flex: none;
  }
  .readme :global(h1) {
    font-size: 16px;
    margin: 8px 0 6px;
  }
  .readme :global(h2) {
    font-size: 14px;
    margin: 14px 0 6px;
  }
  .readme :global(h3) {
    font-size: 13px;
    margin: 10px 0 4px;
  }
  .readme :global(p) {
    margin: 6px 0;
  }
  .readme :global(ul),
  .readme :global(ol) {
    margin: 6px 0;
    padding-left: 20px;
  }
  .readme :global(li) {
    margin: 2px 0;
  }
  .readme :global(a) {
    color: var(--accent);
  }
  .readme :global(strong) {
    font-weight: 700;
  }
  .readme :global(hr) {
    border: 0;
    border-top: 1px solid var(--border);
    margin: 12px 0;
  }
  .readme :global(code) {
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 12px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 4px;
  }
  .readme :global(pre.md-code) {
    margin: 8px 0;
    padding: 10px 12px;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: auto;
  }
  .readme :global(pre.md-code code) {
    border: 0;
    background: none;
    padding: 0;
    font-size: 12px;
    line-height: 1.45;
    white-space: pre;
  }

  /* --- Landing (empty detail pane) --- */
  .landing {
    max-width: 640px;
    margin: 0 auto;
    padding: 8px 4px 24px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 13px 16px 22px;
    position: relative;
    overflow: hidden;
    border-radius: 14px;
    border: 1px solid var(--border);
    background:
      radial-gradient(120% 90% at 50% -10%, color-mix(in srgb, var(--accent) 12%, transparent), transparent 60%),
      var(--panel);
    margin-bottom: 12px;
  }
  .glyph {
    width: 92px;
    height: 92px;
    margin-bottom: -4px;
    filter: drop-shadow(0 4px 14px color-mix(in srgb, var(--accent) 40%, transparent));
  }
  .glyph svg {
    width: 100%;
    height: 100%;
    display: block;
  }
  .htitle {
    margin: 0;
    font-size: 24px;
    font-weight: 800;
    letter-spacing: -0.02em;
    background: linear-gradient(100deg, #00a2a9, #ff8b00);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .tagline {
    margin: 8px 0 0;
    max-width: 440px;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.55;
  }
  .stats {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    margin-top: 18px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 74px;
    padding: 8px 12px;
    border-radius: 10px;
    background: var(--panel-2);
    border: 1px solid var(--border);
  }
  .stat .num {
    font-size: 18px;
    font-weight: 800;
    color: var(--text);
    line-height: 1.1;
  }
  .stat .lbl {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--muted);
    margin-top: 2px;
  }
  .section-label {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.6px;
    color: var(--muted);
    font-weight: 700;
    margin: 14px 0 8px;
  }
  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 8px;
  }
  .card {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 5px;
    min-width: 0;
    text-align: left;
    padding: 11px 12px;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--panel);
    cursor: pointer;
  }
  .card > * {
    min-width: 0;
    max-width: 100%;
  }
  .card:hover {
    background: var(--panel);
    border-color: var(--border);
    box-shadow: none;
  }
  .card-name {
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
    overflow-wrap: anywhere;
  }
  .card-desc {
    font-size: 11px;
    line-height: 1.4;
    color: var(--muted);
    overflow-wrap: anywhere;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  .card-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 3px;
    margin-top: auto;
  }
  .card-tags .ct {
    font-size: 9px;
    color: var(--muted);
    background: var(--panel-2);
    border-radius: 3px;
    padding: 0 5px;
  }
  .steps {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .steps li {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 13px;
    color: var(--text);
    line-height: 1.4;
  }
  .step-code {
    margin: 8px 0 0;
    padding: 8px 10px;
    border-radius: 6px;
    background: var(--code-bg, rgba(127, 127, 127, 0.12));
    border: 1px solid var(--border, rgba(127, 127, 127, 0.2));
    font-size: 12px;
    color: var(--text);
    overflow-x: auto;
    white-space: pre;
  }
  .step-n {
    flex: none;
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    font-size: 11px;
    font-weight: 800;
    color: #fff;
    background: linear-gradient(135deg, #00a2a9, #ff8b00);
    margin-top: -1px;
    text-shadow:
      1px 1px 2px rgba(0, 0, 0, 0.5),
      -1px 1px 2px rgba(0, 0, 0, 0.5),
      1px -1px 2px rgba(0, 0, 0, 0.5),
      -1px -1px 2px rgba(0, 0, 0, 0.5);
    box-shadow: 0 0 3px rgba(0, 0, 0, 0.55);
  }
  .steps strong {
    color: var(--text);
    font-weight: 700;
  }
</style>
