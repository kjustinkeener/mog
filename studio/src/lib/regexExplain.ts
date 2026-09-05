// Rule-based regex -> plain-English explainer for the Studio's regex tester.
// Deliberately client-side and AI-free (the engine stays free of NL): a small
// tokenizer that walks the pattern and describes each construct. It covers the
// common set; anything unrecognized falls back to "the character ...", so it never
// throws on a weird pattern.

export interface ExplainPart {
  /** Nesting depth (groups indent their contents). */
  depth: number;
  /** The raw pattern slice this line describes. */
  token: string;
  /** Plain-English description. */
  desc: string;
}

const ESCAPES: Record<string, string> = {
  d: 'a digit (0-9)',
  D: 'a non-digit',
  w: 'a word character (letter, digit, or underscore)',
  W: 'a non-word character',
  s: 'whitespace',
  S: 'non-whitespace',
  b: 'a word boundary',
  B: 'not a word boundary',
  A: 'start of text',
  z: 'end of text',
  n: 'a newline',
  t: 'a tab',
  r: 'a carriage return',
  f: 'a form feed',
  v: 'a vertical tab',
  '0': 'a null character',
};

/** Describe the inside of a `[...]` character class. */
function describeClassBody(body: string): string {
  const items: string[] = [];
  let j = 0;
  while (j < body.length) {
    if (body[j] === '\\' && j + 1 < body.length) {
      const e = body[j + 1];
      items.push(ESCAPES[e] ?? `"${e}"`);
      j += 2;
    } else if (j + 2 < body.length && body[j + 1] === '-') {
      items.push(`${body[j]} to ${body[j + 2]}`);
      j += 3;
    } else {
      items.push(`"${body[j]}"`);
      j += 1;
    }
  }
  return items.join(', ');
}

export function explainRegex(pattern: string): ExplainPart[] {
  const parts: ExplainPart[] = [];
  let i = 0;
  const n = pattern.length;
  let captureIndex = 0;

  // Consume a quantifier at the cursor, returning a human label ('' if none).
  function quantifier(): string {
    if (i >= n) return '';
    const c = pattern[i];
    let label = '';
    if (c === '*') {
      label = 'zero or more times';
      i += 1;
    } else if (c === '+') {
      label = 'one or more times';
      i += 1;
    } else if (c === '?') {
      label = 'optionally';
      i += 1;
    } else if (c === '{') {
      const close = pattern.indexOf('}', i);
      if (close > i) {
        const m = /^(\d+)(,(\d*))?$/.exec(pattern.slice(i + 1, close));
        if (m) {
          i = close + 1;
          if (m[2] === undefined) label = `exactly ${m[1]} times`;
          else if (!m[3]) label = `${m[1]} or more times`;
          else label = `between ${m[1]} and ${m[3]} times`;
        }
      }
    }
    if (label && pattern[i] === '?') {
      label += ' (as few as possible)';
      i += 1;
    } else if (label && pattern[i] === '+') {
      i += 1; // possessive; no extra label
    }
    return label;
  }

  function groupOpener(): { opener: string; desc: string } {
    const rest = pattern.slice(i);
    const fixed: [string, string][] = [
      ['(?:', 'group (non-capturing)'],
      ['(?<=', 'preceded by'],
      ['(?<!', 'not preceded by'],
      ['(?=', 'followed by'],
      ['(?!', 'not followed by'],
    ];
    for (const [pre, desc] of fixed) {
      if (rest.startsWith(pre)) {
        i += pre.length;
        return { opener: pre, desc };
      }
    }
    const named = /^\(\?P?<([A-Za-z_]\w*)>/.exec(rest);
    if (named) {
      i += named[0].length;
      captureIndex += 1;
      return { opener: named[0], desc: `capture group ${captureIndex} named "${named[1]}"` };
    }
    const flags = /^\(\?[a-zA-Z]+\)/.exec(rest);
    if (flags) {
      i += flags[0].length;
      return { opener: flags[0], desc: `set flags: ${flags[0].slice(2, -1)}` };
    }
    i += 1;
    captureIndex += 1;
    return { opener: '(', desc: `capture group ${captureIndex}` };
  }

  function parseSeq(depth: number): void {
    let lit = '';
    const flush = () => {
      if (lit.length) {
        parts.push({
          depth,
          token: lit,
          desc: lit.length === 1 ? `the character "${lit}"` : `the text "${lit}"`,
        });
        lit = '';
      }
    };

    while (i < n) {
      const c = pattern[i];

      if (c === ')') return;

      if (c === '|') {
        flush();
        parts.push({ depth, token: '|', desc: 'OR (either the above or the below)' });
        i += 1;
        continue;
      }

      if (c === '(') {
        flush();
        const start = i;
        const { opener, desc } = groupOpener();
        parts.push({ depth, token: opener, desc: `${desc}:` });
        parseSeq(depth + 1);
        if (pattern[i] === ')') i += 1;
        const q = quantifier();
        if (q) {
          parts.push({ depth, token: `${pattern.slice(start, i)}`.slice(-3), desc: `...the group above, ${q}` });
        }
        continue;
      }

      if (c === '[') {
        flush();
        const start = i;
        i += 1;
        let neg = false;
        if (pattern[i] === '^') {
          neg = true;
          i += 1;
        }
        let body = '';
        while (i < n && pattern[i] !== ']') {
          if (pattern[i] === '\\' && i + 1 < n) {
            body += pattern.slice(i, i + 2);
            i += 2;
          } else {
            body += pattern[i];
            i += 1;
          }
        }
        if (i < n) i += 1; // closing ]
        const token = pattern.slice(start, i);
        const q = quantifier();
        const base = (neg ? 'any character except ' : 'any character in ') + describeClassBody(body);
        parts.push({ depth, token, desc: q ? `${base}, ${q}` : base });
        continue;
      }

      if (c === '\\') {
        flush();
        const nx = pattern[i + 1] ?? '';
        i += 2;
        let d: string;
        if (nx >= '1' && nx <= '9') d = `backreference to group ${nx}`;
        else d = ESCAPES[nx] ?? `the character "${nx}"`;
        const q = quantifier();
        parts.push({ depth, token: `\\${nx}`, desc: q ? `${d}, ${q}` : d });
        continue;
      }

      if (c === '^') {
        flush();
        i += 1;
        parts.push({ depth, token: '^', desc: 'start of line/text' });
        continue;
      }
      if (c === '$') {
        flush();
        i += 1;
        parts.push({ depth, token: '$', desc: 'end of line/text' });
        continue;
      }
      if (c === '.') {
        flush();
        i += 1;
        const q = quantifier();
        const base = 'any character (except newline)';
        parts.push({ depth, token: '.', desc: q ? `${base}, ${q}` : base });
        continue;
      }

      // A quantifier right after a literal binds to that last literal character.
      if ((c === '*' || c === '+' || c === '?' || c === '{') && lit.length) {
        const last = lit[lit.length - 1];
        lit = lit.slice(0, -1);
        flush();
        const q = quantifier();
        parts.push({
          depth,
          token: last,
          desc: q ? `the character "${last}", ${q}` : `the character "${last}"`,
        });
        continue;
      }

      lit += c;
      i += 1;
    }
    flush();
  }

  parseSeq(0);
  return parts;
}
