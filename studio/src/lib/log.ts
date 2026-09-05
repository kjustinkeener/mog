// Central logger. Every call goes to the console; when file logging is enabled
// (a Settings toggle, persisted in localStorage) it is ALSO appended to a file on
// disk via the append_log Tauri command, so timings can be read back after a run.
// The append is fire-and-forget and needs the Tauri backend (a rebuild of the app
// if the running binary predates the append_log command).

import { appendLog } from './api';

let enabled = false;

/** Load the persisted enable flag; call once at startup. */
export function initFileLogging(): void {
  try {
    enabled = localStorage.getItem('mog.filelog') === '1';
  } catch {
    enabled = false;
  }
}

export function setFileLogging(on: boolean): void {
  enabled = on;
  try {
    localStorage.setItem('mog.filelog', on ? '1' : '0');
  } catch {
    /* ignore storage failures */
  }
}

export function isFileLogging(): boolean {
  return enabled;
}

function stamp(): string {
  const d = new Date();
  const p = (n: number, w = 2): string => String(n).padStart(w, '0');
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}.${p(d.getMilliseconds(), 3)}`;
}

/** Log a line to the console, and to the log file when file logging is enabled. */
export function logLine(line: string): void {
  console.log(line);
  if (enabled) appendLog(`${stamp()} ${line}`).catch(() => {});
}
