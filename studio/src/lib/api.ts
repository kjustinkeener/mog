// Typed wrappers over the Tauri commands. Argument keys are snake_case to match
// the Rust command parameters exactly.

import { invoke } from '@tauri-apps/api/core';
import type { ActionDescriptor, Category } from './types';

export interface TransformResult {
  output: string;
  is_diff: boolean;
  changed: boolean;
}

export interface BatchResult {
  lines: string[];
  errors: string[];
  ok: boolean;
}

export function listActions(): Promise<ActionDescriptor[]> {
  return invoke<ActionDescriptor[]>('list_actions');
}

/** Append one line to the studio's debug log file (only used when file logging is on). */
export function appendLog(line: string): Promise<void> {
  return invoke('append_log', { line });
}

/** Absolute path the debug log writes to (for display in Settings). */
export function logPath(): Promise<string> {
  return invoke<string>('log_path');
}

/** The engine's ordered, labeled category registry. Rejects on older engines that
 *  lack `--list-categories`; callers should degrade to descriptor order. */
export function listCategories(): Promise<Category[]> {
  return invoke<Category[]>('list_categories');
}

export function runTransform(
  pipeline: string,
  input: string,
  diff: boolean,
  stopAfter: number | null,
): Promise<TransformResult> {
  return invoke<TransformResult>('run_transform', {
    pipeline,
    input,
    diff,
    stop_after: stopAfter,
  });
}

export interface BatchArgs {
  pipeline: string;
  inputs: string[];
  in_place: boolean;
  out_dir: string | null;
  dry_run: boolean;
  /** mog --output-encoding: 'preserve' (default) or utf-8 / utf-8-bom / utf-16le / utf-16be / ansi. */
  output_encoding: string;
  backup: string | null;
  overwrite: boolean;
  excludes: string[];
  jobs: number | null;
}

export function runBatch(args: BatchArgs): Promise<BatchResult> {
  return invoke<BatchResult>('run_batch', { ...args });
}

// -- Convergence panels (5c) --------------------------------------------------

export interface TestOutcome {
  pass: boolean;
  /** Unified diff (expected -> actual) on a mismatch, else null. */
  diff: string | null;
  message: string | null;
}

/** Run the current recipe + input + expected output as a `mog --test` case. */
export function runRecipeTest(
  pipeline: string,
  input: string,
  expected: string,
): Promise<TestOutcome> {
  return invoke<TestOutcome>('run_recipe_test', { pipeline, input, expected });
}

export interface FlagRow {
  tag: string;
  line: number;
  message: string;
}

export interface FlagCheck {
  clean: boolean;
  would_change: boolean;
  flags: FlagRow[];
  /** (tag, count) pairs. */
  counts: [string, number][];
  error: string | null;
}

/** Run the recipe over `input` under `--json --check`; returns flags + the gate. */
export function flagCheck(pipeline: string, input: string): Promise<FlagCheck> {
  return invoke<FlagCheck>('flag_check', { pipeline, input });
}

export interface FileImpact {
  path: string;
  changed: boolean;
  lines_added: number;
  lines_removed: number;
  diff: string | null;
  error: string | null;
}

export interface BatchImpact {
  total: number;
  changed: number;
  errors: number;
  files: FileImpact[];
}

/** Dry-run the recipe over `inputs`; returns per-file impact (writes nothing). */
export function runBatchReport(
  pipeline: string,
  inputs: string[],
  excludes: string[],
  jobs: number | null,
): Promise<BatchImpact> {
  return invoke<BatchImpact>('run_batch_report', { pipeline, inputs, excludes, jobs });
}

/** The unified diff for one on-disk file under the current recipe (fetched lazily). */
export function diffFile(pipeline: string, path: string): Promise<string> {
  return invoke<string>('diff_file', { pipeline, path });
}

// One recipe as returned by `mog market search/list --json`.
export interface RecipeHit {
  name: string;
  description: string;
  tags: string[];
  featured: boolean;
  /** Seeded popularity count from the catalog; used to sort and rank recipes. */
  download_count: number;
  status: string;
  version: number;
}

interface MarketSearchResult {
  count: number;
  results: RecipeHit[];
}

/** Ranked recipe search; empty query lists the whole catalog (featured first). */
export async function marketSearch(query: string): Promise<RecipeHit[]> {
  const r = await invoke<MarketSearchResult>('market_search', { query });
  return r.results ?? [];
}

export interface RecipeDetail {
  name: string;
  /** The recipe's .mog document, as a JSON string. */
  content: string;
  description: string;
  tags: string[];
  /** Resolvable name, e.g. "comment-out.mog". */
  resolvable: string | null;
  /** Sibling torture fixture, when the recipe has one. */
  fixture: { pass: boolean } | null;
  /** The fixture's TestInput content, read by the backend (null if none). */
  fixture_input: string | null;
  /** The recipe's rendered README markdown, when the engine provides one. May be
   *  absent/null (older engines, or a recipe with no doc); render gracefully. */
  readme?: string | null;
}

/** One recipe's full detail, including its `.mog` content. */
export function marketShow(name: string): Promise<RecipeDetail> {
  return invoke<RecipeDetail>('market_show', { name });
}

// One recipe change reported by `mog update`.
export interface UpdatePlan {
  added: string[];
  changed: string[];
  revoked: string[];
  orphaned: string[];
  catalog: number;
}

export interface UpdateResult {
  check: boolean;
  recipes: UpdatePlan | null;
  engine: { update_available: boolean; applied: boolean; version: string } | null;
}

/** Sync marketplace recipes (by content hash) and, unless recipesOnly, self-replace
 *  the engine binary. check=true reports without writing. The running sidecar keeps
 *  its loaded image until the app restarts, so an engine swap applies on next launch. */
export function mogUpdate(
  check: boolean,
  recipesOnly: boolean,
  prune: boolean,
): Promise<UpdateResult> {
  return invoke<UpdateResult>('mog_update', { check, recipes_only: recipesOnly, prune });
}

/** Full path to save `filename` into the user's mog library (the Save As default).
 *  Resolves null when no library root exists, or rejects on engines/builds without
 *  the command; callers fall back to the bare filename. */
export function userRecipePath(filename: string): Promise<string | null> {
  return invoke<string | null>('user_recipe_path', { filename });
}
