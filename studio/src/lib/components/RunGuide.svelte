<script lang="ts">
  // In-app onboarding for the Run subtabs, shown in the (otherwise empty) results
  // area before a run. Layered: what it does -> when to use it -> a numbered how-to
  // of the actual controls -> tips & gotchas. One content set per subtab. Step/tip
  // bodies carry light authored markup (control names, globs), rendered via {@html};
  // the strings are static in this file, never user input.
  import Icon from './Icon.svelte';

  let { variant, ondismiss }: { variant: 'impact' | 'batch'; ondismiss?: () => void } =
    $props();

  interface Step {
    title: string;
    body: string;
  }
  interface Tip {
    kind: 'ok' | 'warn' | 'accent';
    icon: string;
    body: string;
  }
  interface Guide {
    icon: string;
    title: string;
    subtitle: string;
    when: string;
    steps: Step[];
    tips: Tip[];
  }

  const ui = (s: string): string => `<b class="ui">${s}</b>`;
  const code = (s: string): string => `<code class="k">${s}</code>`;

  const impact: Guide = {
    icon: 'eye',
    title: 'Preview impact',
    subtitle:
      'A safe dry run across many files: see exactly what your mog would change, before a single byte is written.',
    when: `Reach for Impact right before a big edit, when you want to know the blast radius. It runs your whole pipeline over every matching file and reports what would change, but never writes anything. Nothing on disk is touched.`,
    steps: [
      {
        title: 'Point it at your files',
        body: `In ${ui('Input Files')}, add one path or glob per line, e.g. ${code('src/**/*.sql')}. Use the file and folder buttons to browse, or type globs by hand. A folder is added as ${code('folder/*')} (the files directly inside it).`,
      },
      {
        title: 'Narrow it down (optional)',
        body: `Add anything you want skipped to ${ui('Excluded Files')}, one per line. Handy for vendored code, generated files, or a stray dump your glob swept in.`,
      },
      {
        title: 'Preview the impact',
        body: `Hit ${ui('Preview impact')}. Your pipeline runs over every matching file in parallel (tune ${ui('Parallel files')} if you like) and, again, writes nothing.`,
      },
      {
        title: 'Read the results',
        body: `You get a summary (scanned / would change / errors) and a per-file list with ${code('+added')} / ${code('-removed')} line counts. Click any changed file to expand its unified diff.`,
      },
    ],
    tips: [
      {
        kind: 'ok',
        icon: 'shield',
        body: `Impact never writes. It's the seatbelt you check before running ${ui('Batch')} for real.`,
      },
      {
        kind: 'warn',
        icon: 'target',
        body: `"0 would change" usually means your mog matched nothing, not that everything is already fine. Recheck your globs and the pipeline.`,
      },
      {
        kind: 'accent',
        icon: 'filter',
        body: `Expecting 6 files but see 35? Your glob is too wide. Expand the extras to see why, then tighten ${ui('Input Files')} or add an exclude.`,
      },
      {
        kind: 'accent',
        icon: 'layers',
        body: `Once it looks right, switch to ${ui('Batch')} to apply it, your file set comes with you.`,
      },
    ],
  };

  const batch: Guide = {
    icon: 'layers',
    title: 'Run batch',
    subtitle:
      'Apply your mog across many files for real: in place, into a new folder, or as a printed preview.',
    when: `Batch is the real run: it writes. Use it once Impact looks right. You choose how the output is delivered, and a dry run plus backups keep you safe until you commit.`,
    steps: [
      {
        title: 'Point it at your files',
        body: `Same ${ui('Input Files')} and ${ui('Excluded Files')} as Impact, one path or glob per line. If you came from Impact with an empty Batch, your file set was carried over for you.`,
      },
      {
        title: 'Choose an output mode',
        body: `${ui('Preview')} prints the transformed text to stdout and writes nothing. ${ui('In place')} overwrites each file (set a backup suffix like ${code('.bak')} to keep originals). ${ui('Output directory')} writes transformed copies elsewhere, leaving the originals untouched.`,
      },
      {
        title: 'Set your safety options',
        body: `For the writing modes, ${ui('Dry run')} reports what would change without writing. Pick an ${ui('Encoding')} (${ui('Preserve')} keeps each file's own) and tune ${ui('Parallel files')}.`,
      },
      {
        title: 'Run it',
        body: `Hit ${ui('Run batch')}. You get an ${code('OK')} / ${code('Errors')} header and a line-by-line report of exactly what happened.`,
      },
    ],
    tips: [
      {
        kind: 'warn',
        icon: 'save',
        body: `${ui('In place')} overwrites your originals. Set a backup suffix (e.g. ${code('.bak')}) unless you're sure, or run under version control.`,
      },
      {
        kind: 'ok',
        icon: 'check',
        body: `Keep ${ui('Dry run')} on for the first pass, confirm the report, then turn it off to actually write.`,
      },
      {
        kind: 'accent',
        icon: 'eye',
        body: `Not sure what it'll do? Run it in ${ui('Impact')} first for a per-file diff, your inputs carry over.`,
      },
      {
        kind: 'accent',
        icon: 'play',
        body: `${ui('Preview')} mode never writes, it just prints the transformed output, great for eyeballing a single file.`,
      },
    ],
  };

  const g = $derived(variant === 'batch' ? batch : impact);
</script>

<div class="guide">
  <header class="ghead">
    <span class="gicon"><Icon name={g.icon} size={22} /></span>
    <div class="gtitle">
      <h2>{g.title}</h2>
      <p>{g.subtitle}</p>
    </div>
    {#if ondismiss}
      <button class="back" onclick={ondismiss} title="Back to results"><Icon name="x" />Results</button>
    {/if}
  </header>

  <div class="callout">
    <span class="cl-ico"><Icon name="target" size={18} /></span>
    <div>
      <strong>When to use it</strong>
      <p>{g.when}</p>
    </div>
  </div>

  <ol class="steps">
    {#each g.steps as s, i (i)}
      <li class="step">
        <span class="num">{i + 1}</span>
        <div class="stext">
          <strong>{s.title}</strong>
          <p>{@html s.body}</p>
        </div>
      </li>
    {/each}
  </ol>

  <div class="tips">
    <div class="tips-head"><Icon name="wrench" size={15} />Tips &amp; gotchas</div>
    <ul>
      {#each g.tips as t (t.body)}
        <li class="tip {t.kind}">
          <span class="tip-ico"><Icon name={t.icon} size={15} /></span>
          <span>{@html t.body}</span>
        </li>
      {/each}
    </ul>
  </div>
</div>

<style>
  .guide {
    max-width: 760px;
    margin: 0 auto;
    padding: 6px 6px 18px;
    font-family: var(--sans);
    color: var(--text);
  }

  /* Header */
  .ghead {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 6px 2px 14px;
  }
  .gicon {
    flex: none;
    display: grid;
    place-items: center;
    width: 42px;
    height: 42px;
    border-radius: 11px;
    background: var(--accent-weak);
    color: var(--accent-strong);
    border: 1px solid var(--border);
  }
  .gtitle {
    flex: 1;
    min-width: 0;
  }
  .gtitle h2 {
    margin: 2px 0 3px;
    font-size: 19px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .gtitle p {
    margin: 0;
    color: var(--muted);
    font-size: 13px;
    line-height: 1.45;
  }
  .back {
    flex: none;
    align-self: center;
  }

  /* When-to-use callout */
  .callout {
    display: flex;
    gap: 10px;
    padding: 11px 13px;
    background: var(--accent-weak);
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent-strong);
    border-radius: 9px;
    margin-bottom: 16px;
  }
  .cl-ico {
    flex: none;
    color: var(--accent-strong);
    margin-top: 1px;
  }
  .callout strong {
    display: block;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--accent-strong);
    margin-bottom: 3px;
  }
  .callout p {
    margin: 0;
    font-size: 13px;
    line-height: 1.5;
    color: var(--text);
  }

  /* Numbered how-to */
  .steps {
    list-style: none;
    margin: 0 0 16px;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .step {
    display: flex;
    gap: 12px;
    padding: 10px 10px;
    border-radius: 9px;
  }
  .step:hover {
    background: var(--panel-2);
  }
  .num {
    flex: none;
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--tab-on);
    color: #fff;
    font-size: 13px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
  }
  .stext {
    flex: 1;
    min-width: 0;
  }
  .stext strong {
    display: block;
    font-size: 13.5px;
    font-weight: 650;
    margin-bottom: 2px;
  }
  .stext p {
    margin: 0;
    font-size: 13px;
    line-height: 1.55;
    color: var(--muted);
  }

  /* Tips & gotchas */
  .tips {
    background: var(--panel-2);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px 14px;
  }
  .tips-head {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
    margin-bottom: 9px;
  }
  .tips ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tip {
    display: flex;
    gap: 9px;
    font-size: 13px;
    line-height: 1.5;
    padding-left: 9px;
    border-left: 2px solid var(--border-strong);
  }
  .tip-ico {
    flex: none;
    margin-top: 2px;
  }
  .tip.ok {
    border-left-color: var(--ok);
  }
  .tip.ok .tip-ico {
    color: var(--ok);
  }
  .tip.warn {
    border-left-color: var(--warn);
  }
  .tip.warn .tip-ico {
    color: var(--warn);
  }
  .tip.accent {
    border-left-color: var(--accent-strong);
  }
  .tip.accent .tip-ico {
    color: var(--accent-strong);
  }

  /* Inline markup used inside authored step/tip bodies */
  .guide :global(b.ui) {
    font-weight: 600;
    color: var(--text);
    background: var(--panel);
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    padding: 0 5px;
    font-size: 0.92em;
    white-space: nowrap;
  }
  .guide :global(code.k) {
    font-family: var(--mono);
    font-size: 0.88em;
    color: var(--accent-strong);
    background: var(--accent-weak);
    border-radius: 4px;
    padding: 1px 5px;
  }
</style>
