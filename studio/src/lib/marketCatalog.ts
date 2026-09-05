// Session cache + prewarm for the marketplace catalog. The full catalog list is the
// expensive part of opening Browse (a `mog market list` subprocess that walks the
// on-disk recipe tree). RecipeBrowser unmounts on every tab switch, so without a
// shared cache each reopen would re-spawn it. We also prewarm at app launch so the
// first Browse open is instant instead of paying the subprocess on the click.
//
// Recipes are static for the session, so the list is cached once. A text search still
// queries the engine live (see RecipeBrowser). A recipe saved mid-session won't appear
// until restart.

import { marketSearch, type RecipeHit } from './api';

let cache: RecipeHit[] | null = null;
let inflight: Promise<RecipeHit[]> | null = null;

/** The cached catalog if already loaded, else null (no fetch). */
export function cachedCatalog(): RecipeHit[] | null {
  return cache;
}

/** Load the catalog once, reusing an in-flight or completed fetch. */
export function loadCatalog(): Promise<RecipeHit[]> {
  if (cache) return Promise.resolve(cache);
  if (inflight) return inflight;
  inflight = marketSearch('')
    .then((list) => {
      cache = list;
      return list;
    })
    .finally(() => {
      inflight = null;
    });
  return inflight;
}

/** Fire-and-forget prewarm for app startup; swallows errors (best-effort). */
export function prewarmCatalog(): void {
  if (cache || inflight) return;
  loadCatalog().catch(() => {});
}
