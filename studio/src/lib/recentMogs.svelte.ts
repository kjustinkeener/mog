// Tracks mogs that have been loaded into the Run or Edit tabs, with the last-loaded
// time. Used to float recently-used recipes to the top of the Marketplace search
// results (most recently loaded first). Persisted to localStorage, mirroring
// runParams.svelte.ts / theme.ts.

export interface RecentMog {
  name: string;
  at: number; // epoch ms of the most recent load
}

const STORAGE_KEY = 'mog.recentMogs';
const CAP = 200; // keep the list bounded; oldest fall off

function load(): RecentMog[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const saved = JSON.parse(raw) as RecentMog[];
    if (!Array.isArray(saved)) return [];
    return saved
      .filter((r) => r && typeof r.name === 'string' && typeof r.at === 'number')
      .sort((a, b) => b.at - a.at)
      .slice(0, CAP);
  } catch {
    return [];
  }
}

// Sorted most-recent-first. Never mutate in place; reassign so readers re-run.
export const recentMogs: { list: RecentMog[] } = $state({ list: load() });

/** Record that a recipe was just loaded into Run/Edit. Bumps it to the front. */
export function markMogLoaded(name: string): void {
  const next = recentMogs.list.filter((r) => r.name !== name);
  next.unshift({ name, at: Date.now() });
  recentMogs.list = next.slice(0, CAP);
}

/** Load order rank: 0 = most recently loaded, higher = older, Infinity = never. */
export function loadRank(name: string): number {
  const i = recentMogs.list.findIndex((r) => r.name === name);
  return i === -1 ? Infinity : i;
}

$effect.root(() => {
  $effect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(recentMogs.list));
    } catch {
      /* ignore (private mode, quota) */
    }
  });
});
