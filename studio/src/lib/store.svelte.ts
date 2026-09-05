// Central reactive studio state (Svelte 5 runes) plus pipeline helpers:
// building steps from descriptors, (de)serializing .mog, and pruning to a clean
// canonical JSON on save.

import { categoryLabel, type ActionDescriptor, type Category, type JsonValue, type Mog, type Step } from './types';

let uidCounter = 0;
export function newUid(): string {
  uidCounter += 1;
  return `s${uidCounter}`;
}

/** Anchored regex helper popover: opened when a regex field is focused, seeded
 *  from that field, and non-destructive, it writes back only when `apply` is
 *  called (the "Apply" button). `anchor` is the focused field's viewport rect. */
export interface RegexPopoverState {
  open: boolean;
  seed: string;
  anchor: { x: number; y: number; w: number; h: number } | null;
  apply: ((value: string) => void) | null;
}

interface StudioState {
  descriptors: ActionDescriptor[];
  byName: Record<string, ActionDescriptor>;
  regexPopover: RegexPopoverState;
  /** Engine category registry (order + labels); empty until loaded, and stays
   *  empty on engines without `--list-categories` (palette then falls back). */
  categories: Category[];
  mog: Mog;
  selectedUid: string | null;
  input: string;
  output: string;
  outputIsDiff: boolean;
  showDiff: boolean;
  stopAfter: number | null;
  running: boolean;
  error: string;
  status: string;
  currentPath: string | null;
  /** Identity of a mog loaded from the marketplace (its resolvable, e.g.
   *  "factory/repo-tidy.mog") when it has no local file path. Display-only. */
  sourceName: string | null;
  dirty: boolean;
  /** Top-level workspace: Browse (marketplace) / Edit (author + verify) / Run
   *  (apply across files). */
  tab: 'browse' | 'edit' | 'run';
  /** Sub-view within a tab: preview/test/check under Edit, impact/batch under Run. */
  mode: 'preview' | 'test' | 'check' | 'impact' | 'batch';
  ready: boolean;
  dragUid: string | null;
  dropUid: string | null;
  dropPos: 'before' | 'after' | null;
}

export const studio: StudioState = $state({
  descriptors: [],
  byName: {},
  regexPopover: { open: false, seed: '', anchor: null, apply: null },
  categories: [],
  mog: { name: '', description: '', steps: [] },
  selectedUid: null,
  input: 'Hello World\nhello world\nHELLO WORLD\n',
  output: '',
  outputIsDiff: false,
  showDiff: false,
  stopAfter: null,
  running: false,
  error: '',
  status: '',
  currentPath: null,
  sourceName: null,
  dirty: false,
  tab: 'browse',
  mode: 'preview',
  ready: false,
  dragUid: null,
  dropUid: null,
  dropPos: null,
});

export function clearDrag(): void {
  studio.dragUid = null;
  studio.dropUid = null;
  studio.dropPos = null;
}

export function setDescriptors(list: ActionDescriptor[]): void {
  studio.descriptors = list;
  const map: Record<string, ActionDescriptor> = {};
  for (const d of list) {
    map[d.name] = d;
    for (const a of d.aliases) map[a] = d;
  }
  studio.byName = map;
  studio.ready = true;
}

export function setCategories(list: Category[]): void {
  studio.categories = list;
}

/** Open the regex helper popover anchored under `el`, seeded with `seed`. `apply`
 *  writes the chosen pattern back to the field (only when the user commits). */
export function openRegexPopover(el: HTMLElement, seed: string, apply: (value: string) => void): void {
  const r = el.getBoundingClientRect();
  studio.regexPopover = {
    open: true,
    seed,
    anchor: { x: r.left, y: r.bottom, w: r.width, h: r.height },
    apply,
  };
}

export function closeRegexPopover(): void {
  studio.regexPopover = { open: false, seed: '', anchor: null, apply: null };
}

/** Switch the top-level tab, keeping the sub-view valid for that tab. */
export function setTab(tab: 'browse' | 'edit' | 'run'): void {
  studio.tab = tab;
  if (tab === 'edit' && studio.mode !== 'preview' && studio.mode !== 'test' && studio.mode !== 'check') {
    studio.mode = 'preview';
  }
  if (tab === 'run' && studio.mode !== 'impact' && studio.mode !== 'batch') {
    studio.mode = 'impact';
  }
}

/** True for the fields that take a regex, so their inputs open the helper popover:
 *  `find` in the regex replacers, and `pattern` in the line `*_matching` actions.
 *  (Step-level only/except_lines_matching are always regex and wired directly.) */
export function isRegexField(actionName: string, key: string): boolean {
  if (key === 'find' && (actionName === 'replace_regex' || actionName === 'replace_regex_multiline')) {
    return true;
  }
  if (key === 'pattern' && actionName.endsWith('_matching')) return true;
  return false;
}

/** Display label for a category: the engine registry's label if loaded, else the
 *  curated/title-cased fallback. */
export function catLabel(cat: string): string {
  return studio.categories.find((c) => c.name === cat)?.label ?? categoryLabel(cat);
}

/** The categories present among the loaded descriptors, ordered by the engine
 *  registry's rank when available; any category not in the registry (or when the
 *  registry failed to load) keeps first-seen descriptor order and sorts last. So
 *  the palette always shows every category the engine ships, never gated by a list. */
export function orderedCategories(): string[] {
  const present: string[] = [];
  for (const d of studio.descriptors) {
    if (!present.includes(d.category)) present.push(d.category);
  }
  const rank = new Map(studio.categories.map((c) => [c.name, c.rank]));
  const known = present.filter((c) => rank.has(c)).sort((a, b) => rank.get(a)! - rank.get(b)!);
  const unknown = present.filter((c) => !rank.has(c));
  return [...known, ...unknown];
}

/** Default options for a fresh step of this action. */
export function defaultOptions(desc: ActionDescriptor): Record<string, JsonValue> {
  const o: Record<string, JsonValue> = {};
  for (const p of desc.params) {
    if (p.default !== undefined && p.default !== null) {
      o[p.key] = p.default;
    } else if (p.required && p.kind !== 'bool') {
      o[p.key] = '';
    }
    // int_opt (optional, no default) is intentionally left absent.
  }
  return o;
}

export function makeStep(desc: ActionDescriptor): Step {
  return {
    action: desc.name,
    disabled: false,
    match_ignore_case: false,
    options: defaultOptions(desc),
    _uid: newUid(),
  };
}

export function addStep(desc: ActionDescriptor, at?: number): void {
  const step = makeStep(desc);
  if (at === undefined || at < 0 || at > studio.mog.steps.length) {
    studio.mog.steps.push(step);
  } else {
    studio.mog.steps.splice(at, 0, step);
  }
  studio.selectedUid = step._uid;
  markDirty();
}

export function removeStep(uid: string): void {
  const i = studio.mog.steps.findIndex((s) => s._uid === uid);
  if (i >= 0) {
    studio.mog.steps.splice(i, 1);
    if (studio.selectedUid === uid) studio.selectedUid = null;
    if (studio.stopAfter !== null && studio.stopAfter >= studio.mog.steps.length) {
      studio.stopAfter = null;
    }
    markDirty();
  }
}

/** Move step `fromUid` to just before/after step `toUid` (drag-and-drop reorder). */
export function reorderToTarget(fromUid: string, toUid: string, after: boolean): void {
  if (fromUid === toUid) return;
  const steps = studio.mog.steps;
  const from = steps.findIndex((s) => s._uid === fromUid);
  if (from < 0) return;
  const [moved] = steps.splice(from, 1);
  const to = steps.findIndex((s) => s._uid === toUid);
  if (to < 0) {
    steps.splice(from, 0, moved); // target vanished: undo
    return;
  }
  steps.splice(after ? to + 1 : to, 0, moved);
  markDirty();
}

export function markDirty(): void {
  studio.dirty = true;
}

function isDefault(desc: ActionDescriptor | undefined, key: string, value: JsonValue): boolean {
  if (!desc) return false;
  const p = desc.params.find((x) => x.key === key);
  if (!p || p.default === undefined || p.default === null) return false;
  return p.default === value;
}

function cleanStep(s: Step): Record<string, JsonValue | Record<string, JsonValue>> {
  const out: Record<string, JsonValue | Record<string, JsonValue>> = {};
  if (s.description) out.description = s.description;
  if (s.section) out.section = s.section;
  if (s.action) out.action = s.action;
  if (s.disabled) out.disabled = true;
  if (s.only_lines_matching) out.only_lines_matching = s.only_lines_matching;
  if (s.except_lines_matching) out.except_lines_matching = s.except_lines_matching;
  if (s.match_ignore_case) out.match_ignore_case = true;

  const desc = s.action ? studio.byName[s.action] : undefined;
  const opts: Record<string, JsonValue> = {};
  for (const [k, v] of Object.entries(s.options)) {
    if (v === undefined || v === null) continue;
    if (isDefault(desc, k, v)) continue; // drop values equal to the engine default
    opts[k] = v;
  }
  if (Object.keys(opts).length > 0) out.options = opts;
  return out;
}

/** Canonical .mog JSON for saving or for feeding a transform. */
export function serializeMog(): string {
  const doc: Record<string, unknown> = {};
  if (studio.mog.name) doc.name = studio.mog.name;
  if (studio.mog.description) doc.description = studio.mog.description;
  doc.steps = studio.mog.steps.map(cleanStep);
  return JSON.stringify(doc, null, 2);
}

interface RawStep {
  description?: string;
  section?: string;
  action?: string;
  disabled?: boolean;
  only_lines_matching?: string;
  except_lines_matching?: string;
  match_ignore_case?: boolean;
  options?: Record<string, JsonValue>;
}

/** Load a parsed .mog document into the editor, attaching UI ids. */
export function loadMog(doc: { name?: string; description?: string; steps?: RawStep[] }, path: string | null): void {
  const steps: Step[] = (doc.steps ?? []).map((r) => ({
    description: r.description,
    section: r.section,
    action: r.action,
    disabled: r.disabled ?? false,
    only_lines_matching: r.only_lines_matching,
    except_lines_matching: r.except_lines_matching,
    match_ignore_case: r.match_ignore_case ?? false,
    options: r.options ?? {},
    _uid: newUid(),
    _collapsed: true, // action steps start collapsed; expand to edit
  }));
  studio.mog = { name: doc.name ?? '', description: doc.description ?? '', steps };
  studio.currentPath = path;
  studio.sourceName = null;
  studio.selectedUid = null;
  studio.stopAfter = null;
  studio.dirty = false;
}

export function newPipeline(): void {
  studio.mog = { name: '', description: '', steps: [] };
  studio.currentPath = null;
  studio.sourceName = null;
  studio.selectedUid = null;
  studio.stopAfter = null;
  studio.dirty = false;
}
