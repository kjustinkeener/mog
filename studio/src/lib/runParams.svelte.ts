// Run-tab parameters (Impact + Batch subtabs), lifted out of the panel components so
// they survive subtab switches (each panel unmounts when its subtab is hidden) and
// app restarts. Persisted to localStorage, mirroring theme.ts. Each subtab keeps its
// own inputs/excludes/jobs; batch also carries its output-mode params. Transient run
// results (spinner, output, errors) stay component-local, not here.

import { studio } from './store.svelte';

export type BatchMode = 'stdout' | 'inplace' | 'outdir';

export interface ImpactParams {
  inputs: string;
  excludes: string;
  excludesEnabled: boolean;
  jobs: string;
}

export interface BatchParams {
  inputs: string;
  excludes: string;
  excludesEnabled: boolean;
  jobs: string;
  mode: BatchMode;
  outDir: string;
  backup: string;
  overwrite: boolean;
  dryRun: boolean;
  outputEncoding: string;
}

interface RunParams {
  impact: ImpactParams;
  batch: BatchParams;
}

const STORAGE_KEY = 'mog.runParams';

function defaults(): RunParams {
  return {
    impact: { inputs: '', excludes: '', excludesEnabled: false, jobs: '' },
    batch: {
      inputs: '',
      excludes: '',
      excludesEnabled: false,
      jobs: '',
      mode: 'stdout',
      outDir: '',
      backup: '',
      overwrite: false,
      dryRun: true,
      outputEncoding: 'preserve',
    },
  };
}

// Merge saved values over defaults so a new field added later still gets a default
// (rather than undefined) for anyone with older persisted state.
function load(): RunParams {
  const d = defaults();
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return d;
    const saved = JSON.parse(raw) as Partial<RunParams>;
    return {
      impact: { ...d.impact, ...(saved.impact ?? {}) },
      batch: { ...d.batch, ...(saved.batch ?? {}) },
    };
  } catch {
    return d;
  }
}

export const runParams: RunParams = $state(load());

// Persist on any change. $effect.root gives a module-level (non-component) effect
// scope; the inner $effect re-runs whenever any field read by JSON.stringify changes.
$effect.root(() => {
  $effect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(runParams));
    } catch {
      /* ignore (private mode, quota) */
    }
  });
});

/** Switch the Run subtab. As a convenience, when the destination tab has BOTH its
 *  inputs and excludes empty and the tab being left has either populated, copy those
 *  two fields across, so you don't retype the same file set when moving Impact -> Batch
 *  (or back). Only fires when both destination fields are empty. */
export function setRunSubtab(next: 'impact' | 'batch'): void {
  const prev = studio.mode;
  if ((prev === 'impact' || prev === 'batch') && prev !== next) {
    const from = runParams[prev];
    const to = runParams[next];
    const destEmpty = to.inputs.trim() === '' && to.excludes.trim() === '';
    const srcHas = from.inputs.trim() !== '' || from.excludes.trim() !== '';
    if (destEmpty && srcHas) {
      to.inputs = from.inputs;
      to.excludes = from.excludes;
      to.excludesEnabled = from.excludesEnabled;
    }
  }
  studio.mode = next;
}
