// Theme preference: 'auto' follows the OS (prefers-color-scheme), 'light'/'dark'
// force it via a data-theme attribute the CSS keys off. Persisted in localStorage.

export type Theme = 'auto' | 'light' | 'dark';

export function getTheme(): Theme {
  try {
    const v = localStorage.getItem('mog.theme');
    if (v === 'light' || v === 'dark' || v === 'auto') return v;
  } catch {
    /* ignore */
  }
  return 'auto';
}

export function applyTheme(t: Theme): void {
  const el = document.documentElement;
  if (t === 'auto') el.removeAttribute('data-theme');
  else el.setAttribute('data-theme', t);
}

export function setTheme(t: Theme): void {
  try {
    localStorage.setItem('mog.theme', t);
  } catch {
    /* ignore */
  }
  applyTheme(t);
}

/** Apply the persisted theme; call once at startup. */
export function initTheme(): void {
  applyTheme(getTheme());
}
