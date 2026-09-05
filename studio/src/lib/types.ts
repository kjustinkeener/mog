// The .mog model (mirrors the engine) plus the action-descriptor shapes the
// backend returns from `mog --list-actions --json`.

export type JsonValue = string | number | boolean | null;

export type ParamKind = 'text' | 'multiline_text' | 'bool' | 'int' | 'enum';

export interface ParamSpec {
  key: string;
  label: string;
  kind: ParamKind;
  required: boolean;
  default?: JsonValue;
  values?: string[];
  help: string;
}

/** Runtime complexity class in the input size, from the engine's descriptor JSON.
 * `linear` = O(n) streaming pass; `whole_file` = buffers the whole input
 * (sort/dedupe/columnar/compose); `regex_backtracking` = O(n^2) with
 * backreferences/lookaround. Optional so older engine builds still parse. */
export type Complexity = 'linear' | 'whole_file' | 'regex_backtracking';

export interface ActionDescriptor {
  name: string;
  aliases: string[];
  category: string;
  label: string;
  summary: string;
  complexity?: Complexity;
  params: ParamSpec[];
}

export interface Step {
  description?: string;
  /** Section header: begins a named section grouping this and following steps. */
  section?: string;
  action?: string;
  disabled: boolean;
  only_lines_matching?: string;
  except_lines_matching?: string;
  match_ignore_case: boolean;
  options: Record<string, JsonValue>;
  // UI-only, stripped on serialize:
  _uid: string;
  _collapsed?: boolean;
  _scopeOpen?: boolean;
}

export interface Mog {
  name?: string;
  description?: string;
  steps: Step[];
}

// Display labels for engine action categories. These are presentation overrides
// only (nice casing/wording); any category the engine emits that is not listed
// here falls back to a title-cased version of its name via categoryLabel(), so a
// newly added engine category still renders and is never silently dropped.
// Category ORDER is taken from the engine's own descriptor order at render time
// (see Palette), not hardcoded here, which is what used to make the palette drift.
export const CATEGORY_LABEL: Record<string, string> = {
  replace: 'Replace',
  line: 'Line',
  text: 'Text',
  data: 'Data',
  json: 'JSON',
  whitespace: 'Whitespace',
  eol: 'EOL',
  case: 'Case',
  affix: 'Affix',
  encode: 'Encode',
  compare: 'Compare',
  detect: 'Detect',
  assert: 'Assert',
  compose: 'Compose',
};

/** Human label for a category: a curated override, else the name title-cased. */
export function categoryLabel(cat: string): string {
  return CATEGORY_LABEL[cat] ?? cat.replace(/\b\w/g, (c) => c.toUpperCase());
}

/** One entry of the engine's category registry (`mog --list-categories --json`). */
export interface Category {
  name: string;
  label: string;
  rank: number;
}
