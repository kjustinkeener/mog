// Wrapper over the `regex-wasm` module: the exact Rust regex flavor mog uses,
// compiled to WASM (see regex-wasm/). Loading is async (the .wasm must be fetched
// and instantiated); once initialized, `analyzeRegex` is a fast synchronous call.
// Rebuild the wasm with `npm run build:wasm` after changing the crate.

import init, {
  analyze as wasmAnalyze,
  substitute as wasmSubstitute,
} from '../wasm/regex/regex_wasm.js';

/**
 * Which engine compiled the pattern:
 * - `linear`: the `regex` crate (fast path). What mog uses for ordinary patterns.
 * - `backtracking`: only `fancy-regex` accepts it (backreferences / lookaround);
 *   mog falls back to this and it is O(n^2) on large input.
 * - `invalid`: neither engine accepts it (`error` is set, matching mog's message).
 * - `empty`: no pattern typed yet.
 */
export type RegexEngine = 'linear' | 'backtracking' | 'invalid' | 'empty';

export interface CaptureGroup {
  name: string | null;
  /** UTF-16 offset into the test text (JS string index); null if group absent. */
  start: number | null;
  end: number | null;
}

export interface RegexMatch {
  /** UTF-16 offsets of the whole match. */
  start: number;
  end: number;
  /** Capture groups; index 0 is the whole match. */
  groups: CaptureGroup[];
}

export interface RegexAnalysis {
  ok: boolean;
  engine: RegexEngine;
  error: string | null;
  /** Capture-group names by index (index 0 = whole match, always null). */
  group_names: (string | null)[];
  matches: RegexMatch[];
  /** True when matching stopped at the internal match cap. */
  truncated: boolean;
}

let ready: Promise<void> | null = null;

/** Initialize the wasm module once; safe to call repeatedly (returns the same promise). */
export function initRegexEngine(): Promise<void> {
  if (!ready) {
    ready = init().then(() => undefined);
  }
  return ready;
}

/**
 * Analyze `find` against `text`. Must be called only after `initRegexEngine()`
 * has resolved. Never throws: a malformed pattern comes back as
 * `{ ok: false, engine: 'invalid', error }`.
 */
export function analyzeRegex(
  find: string,
  text: string,
  ignoreCase: boolean,
  multiline: boolean,
): RegexAnalysis {
  const json = wasmAnalyze(find, text, ignoreCase, multiline);
  return JSON.parse(json) as RegexAnalysis;
}

export interface RegexSubstitution {
  ok: boolean;
  engine: RegexEngine;
  error: string | null;
  /** The substituted text (unchanged input on error / empty pattern). */
  output: string;
  changed: boolean;
}

/**
 * Preview replacing every match of `find` with `replace`. Byte-identical to mog's
 * `replace_regex` (`$1` / `${name}` / `$$` expansion). Must be called only after
 * `initRegexEngine()` resolves. Never throws.
 */
export function substituteRegex(
  find: string,
  replace: string,
  text: string,
  ignoreCase: boolean,
  multiline: boolean,
): RegexSubstitution {
  const json = wasmSubstitute(find, replace, text, ignoreCase, multiline);
  return JSON.parse(json) as RegexSubstitution;
}
