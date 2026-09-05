// Detects mog flag markers in transformed output. Mirrors the Rust
// `find_flags` scanner (same canonical `<TAG>(mog): <message>` format), so the
// studio lists exactly what the CLI reports.

export interface Flag {
  line: number; // 1-based
  tag: string;
  message: string;
}

// The fixed tag set (matches `actions::flag::TAGS`).
export const FLAG_TAGS = ['FIXME', 'TODO', 'NOTE', 'HACK', 'XXX', 'WARN'] as const;

const MARKER = /\b(FIXME|TODO|NOTE|HACK|XXX|WARN)\(mog\): (.*)$/;

export function findFlags(text: string): Flag[] {
  const flags: Flag[] = [];
  const lines = text.split(/\r\n|\r|\n/);
  for (let i = 0; i < lines.length; i++) {
    const m = MARKER.exec(lines[i]);
    if (m) flags.push({ line: i + 1, tag: m[1], message: m[2] });
  }
  return flags;
}

// Per-tag counts in the fixed tag order (only tags that occur).
export function flagCounts(flags: Flag[]): { tag: string; count: number }[] {
  return FLAG_TAGS.map((tag) => ({
    tag,
    count: flags.filter((f) => f.tag === tag).length,
  })).filter((c) => c.count > 0);
}
