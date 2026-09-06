//! The market workflow: the `ls` / `show` / `add` / `rm` / `bless` commands that
//! manage the on-disk library of `.mog` scripts and their fixtures.
//!
//! The market *model* (root, sources, resolution) lives in [`crate::library`];
//! this module is the *management surface* built on top of it. Every command
//! reuses the shared primitives it must not duplicate: [`library::resolve_script`]
//! for name resolution, [`testkit`] for fixture discovery (`sibling`), running
//! (`run_mog_test`), and diffing (`unified_diff`).
//!
//! Each command takes plain parameters (no clap types) and prints its own
//! human/JSON output, returning the process exit code. Errors bubble up as
//! `anyhow::Error`, which the CLI renders (as a JSON object under `--json`).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::library;
use crate::model::Tier;
use crate::testkit;

/// A single stored script, as seen by `mog ls` / `mog show`. Descriptive fields
/// are read from the `.mog`; `has_fixture` is a sibling scan. No script body is
/// carried here (that is `show`'s job) so a listing stays cheap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    /// The resolvable name the script answers to, e.g. `strip-ansi-logs.mog`.
    pub resolvable: String,
    /// The `.mog`'s `name` field, if present.
    pub name: Option<String>,
    /// The `.mog`'s `description` field, if present (the long, agent-oriented text).
    pub description: Option<String>,
    /// The `.mog`'s `summary` field, if present (the short human browse blurb).
    pub summary: Option<String>,
    /// The `.mog`'s `tags` (empty when absent).
    pub tags: Vec<String>,
    /// The `.mog`'s `task_phrases` (empty when absent): natural-language triggers
    /// that the ranker weights strongly for discovery.
    pub task_phrases: Vec<String>,
    /// The `.mog`'s `tier` (`core` / `full`).
    pub tier: Tier,
    /// True when BOTH `TestInput` and `TestExpectedOutput` siblings exist.
    pub has_fixture: bool,
    /// Absolute path to the `.mog` on disk (not serialized; internal).
    #[serde(skip)]
    pub path: PathBuf,
}

/// The descriptive metadata read from a `.mog` file for listing. Deserialized
/// directly (a partial view of [`crate::Mog`], no `deny_unknown_fields`) so only
/// these six header fields are parsed, not the whole action pipeline; `steps`,
/// `constants`, and the rest are simply ignored. `#[serde(default)]` makes every
/// omitted field default, matching `Mog`'s per-field defaults.
#[derive(Default, Deserialize)]
#[serde(default)]
struct MogMeta {
    name: Option<String>,
    summary: Option<String>,
    description: Option<String>,
    tags: Vec<String>,
    task_phrases: Vec<String>,
    tier: Tier,
}

/// Read the descriptive metadata (name/summary/description/tags/task_phrases/tier)
/// directly from a `.mog` file, without interpolating constants (so a script with
/// unbound `{{placeholders}}` still lists). A read or parse failure yields empty
/// metadata rather than dropping the entry (one broken mog must not abort a
/// whole listing), but it warns on stderr so the failure is not silent.
fn read_metadata(path: &Path) -> MogMeta {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("warning: cannot read '{}': {e}", path.display());
            return MogMeta::default();
        }
    };
    match serde_json::from_str::<MogMeta>(&text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "warning: '{}' is not a valid .mog, listing it with empty metadata: {e}",
                path.display()
            );
            MogMeta::default()
        }
    }
}

/// True when a `.mog` has both required fixture siblings.
fn has_fixture(mog: &Path) -> bool {
    testkit::sibling(mog, "TestInput").is_some()
        && testkit::sibling(mog, "TestExpectedOutput").is_some()
}

/// Build the resolvable `relpath` name for a `.mog` under the flat `root`,
/// using forward slashes so it is stable across platforms.
fn resolvable_name(root: &Path, mog: &Path) -> String {
    let rel = mog.strip_prefix(root).unwrap_or(mog);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Modified time of a directory-enumeration entry as nanoseconds since the Unix
/// epoch, or `0` when it is unavailable (platform without mtime, transient error).
/// Used only for the scan fingerprint, so a coarse `0` fallback simply makes that
/// entry not advance the newest-mtime bound.
///
/// Takes the mtime from the [`fs::DirEntry`]'s own metadata rather than a path
/// lookup: on Windows `DirEntry::metadata()` is served from the `FindNextFile`
/// record already produced by `read_dir`, so it costs NO extra syscall. A
/// path-based `fs::metadata` would issue a fresh `stat` per file -- ~575 of them
/// across the library -- which under Defender real-time scanning is the bulk of
/// the old ~230ms walk.
fn entry_mtime_ns(e: &fs::DirEntry) -> u128 {
    e.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// Whether a mog has BOTH fixtures, from a SINGLE `read_dir` of its `tests/`
/// directory (rather than the two `sibling` scans `has_fixture` would do). Only
/// called on a cache miss while building entries: on a hit the flag comes from
/// the cached entry, so this stays off the hot path.
fn fixture_present(mog: &Path) -> bool {
    let Some(dir) = mog.parent() else {
        return false;
    };
    if let Ok(rd) = fs::read_dir(dir.join("tests")) {
        let mut have_input = false;
        let mut have_expected = false;
        for e in rd.flatten() {
            if let Some(n) = e.file_name().to_str() {
                if n.starts_with("input.") {
                    have_input = true;
                } else if n.starts_with("expected.") {
                    have_expected = true;
                }
            }
        }
        if have_input && have_expected {
            return true;
        }
    }
    // No complete `tests/` pair: fall back to the full check (which also handles
    // the legacy flat `<stem>.<infix>.<ext>` layout beside the `.mog`).
    has_fixture(mog)
}

/// One `.mog` discovered by the store walk. The walk stats each `.mog` for its
/// mtime but never reads, parses, or scans its `tests/` dir, so it is cheap
/// enough to run on every `scan` to fingerprint the store. Fixture detection and
/// metadata reads happen only on a cache miss, when the entry is actually built.
struct Walked {
    resolvable: String,
    mog: PathBuf,
    mtime_ns: u128,
}

/// Non-mog siblings at the flat root that the walk must never descend into or
/// surface as mogs: derived caches, report templates/output, config, the
/// install manifest. Any dotfile/dotdir is also skipped (handled in the walk).
fn is_reserved_dir_name(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_string_lossy().as_ref(),
        ".cache" | "templates" | "reports"
    )
}

/// Walk the flat `root` once. Returns the raw discovery (no `.mog` bodies read)
/// plus, via the caller, the material for a cheap fingerprint (`.mog` count +
/// newest `.mog` mtime).
///
/// Both the directory recursion and each file's mtime are driven entirely off the
/// [`fs::DirEntry`] records `read_dir` yields, so the walk makes ONE syscall per
/// directory and NONE per file (on Windows the entry's `file_type()`/`metadata()`
/// come from the enumeration, not a fresh `stat`). This is what keeps the warm
/// listing near process-startup: the fingerprint still stats nothing extra, so an
/// in-place mog edit is still caught by the newest-mtime half exactly as before.
fn walk_sources(root: &Path) -> Vec<Walked> {
    let mut out: Vec<Walked> = Vec::new();
    collect_walked(root, root, &mut out);
    // Path-sorted ordering, so listings are byte-for-byte stable. A tree walk
    // visits each path once, so no dedup.
    out.sort_by(|a, b| a.mog.cmp(&b.mog));
    out
}

/// Recursively collect every `.mog` under `dir`, tagging each with its
/// enumeration-derived mtime. Directories that cannot be read simply contribute
/// nothing (a missing root, a permission error). Symlinked directories fall back
/// to a path `is_dir()` check so symlinks keep working (rare, and the only place
/// an extra `stat` is spent). At the flat root, reserved non-mog siblings
/// (`.cache`, `templates`, `reports`) and any dotdir are skipped.
fn collect_walked(root: &Path, dir: &Path, out: &mut Vec<Walked>) {
    let Ok(rd) = fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let Ok(ft) = e.file_type() else {
            continue;
        };
        let path = e.path();
        let name = e.file_name();
        let is_dir = ft.is_dir() || (ft.is_symlink() && path.is_dir());
        if is_dir {
            // A mog's fixtures live in its `tests/` dir (`input.*`/`expected.*`),
            // never a `.mog`; skipping that descent avoids ~one `read_dir` per
            // mog -- roughly halving the walk -- with no listing change.
            if name.eq_ignore_ascii_case("tests") {
                continue;
            }
            // Skip reserved non-mog siblings and any dotdir (the flat root now
            // holds mogs directly beside these, so they must not be walked).
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || is_reserved_dir_name(&name) {
                continue;
            }
            collect_walked(root, &path, out);
        } else if path.extension().is_some_and(|x| x == "mog") {
            let mtime_ns = entry_mtime_ns(&e);
            out.push(Walked {
                resolvable: resolvable_name(root, &path),
                mog: path,
                mtime_ns,
            });
        }
    }
}

/// The persisted scan index: the fully built listing plus the cheap fingerprint
/// it was built from. Reused verbatim while the fingerprint still matches.
#[derive(Serialize, Deserialize)]
struct ScanCache {
    /// Number of `.mog` files across all sources at cache time.
    count: usize,
    /// Newest mtime (ns since the Unix epoch) across every `.mog` at cache time.
    max_mtime_ns: u128,
    /// The built listing.
    entries: Vec<Entry>,
}

/// Where the scan index lives: under `root/.cache/`, which is never itself a
/// library source, so it can hold derived artifacts without polluting listings.
fn cache_path(root: &Path) -> PathBuf {
    root.join(".cache").join("scan-index.json")
}

/// Return the cached entries when a valid index's fingerprint matches `(count,
/// max_mtime_ns)`; otherwise `None` (missing, unreadable, unparsable, or stale).
fn load_cache(root: &Path, count: usize, max_mtime_ns: u128) -> Option<Vec<Entry>> {
    let text = fs::read_to_string(cache_path(root)).ok()?;
    let cache: ScanCache = serde_json::from_str(&text).ok()?;
    (cache.count == count && cache.max_mtime_ns == max_mtime_ns).then_some(cache.entries)
}

/// Persist the freshly built index (best effort: a read-only or racing library
/// simply skips the cache and recomputes next time). The write is staged to a
/// pid-suffixed temp file then renamed, so a concurrent reader never observes a
/// half-written index.
fn store_cache(root: &Path, count: usize, max_mtime_ns: u128, entries: &[Entry]) {
    // Never materialize a root that does not exist yet (a fresh, un-`setup` library):
    // the cache is an optimization, not a reason to create directories.
    if !root.is_dir() {
        return;
    }
    let cache = ScanCache {
        count,
        max_mtime_ns,
        entries: entries.to_vec(),
    };
    let Ok(json) = serde_json::to_string(&cache) else {
        return;
    };
    let dir = root.join(".cache");
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let tmp = dir.join(format!("scan-index.json.{}.tmp", std::process::id()));
    if fs::write(&tmp, json.as_bytes()).is_ok()
        && fs::rename(&tmp, dir.join("scan-index.json")).is_err()
    {
        let _ = fs::remove_file(&tmp);
    }
}

/// Scan the flat `root` and return one [`Entry`] per `.mog`, in path order. A
/// missing root simply contributes nothing.
///
/// Memoized: every call does a cheap stat-only walk to fingerprint the store
/// (`.mog` count + newest mtime); when that matches the persisted
/// `root/.cache/scan-index.json`, the built listing is reused without re-reading
/// or re-parsing any `.mog`. Only a fingerprint miss (a mog added, removed, or
/// edited) pays for the full read+parse pass, which then refreshes the cache.
pub fn scan(root: &Path) -> Vec<Entry> {
    // Mogs live under `<root>/mogs/market`; the scan cache stays at the outer
    // `<root>/.cache`, so the walk root and the cache root differ.
    let market = library::market_dir(root);
    let walked = walk_sources(&market);
    let count = walked.len();
    let max_mtime_ns = walked.iter().map(|w| w.mtime_ns).max().unwrap_or(0);

    if let Some(cached) = load_cache(root, count, max_mtime_ns) {
        return cached;
    }

    // Miss: read + parse each `.mog` for its metadata and refresh the cache.
    let mut entries: Vec<Entry> = Vec::with_capacity(count);
    for w in walked {
        let MogMeta {
            name,
            summary,
            description,
            tags,
            task_phrases,
            tier,
        } = read_metadata(&w.mog);
        entries.push(Entry {
            resolvable: w.resolvable,
            name,
            summary,
            description,
            tags,
            task_phrases,
            tier,
            has_fixture: fixture_present(&w.mog),
            path: w.mog,
        });
    }

    store_cache(root, count, max_mtime_ns, &entries);
    entries
}

/// Filters for [`ls`].
#[derive(Debug, Default, Clone)]
pub struct LsFilter {
    /// Require every one of these tags (AND). Empty means no tag filter.
    pub tags: Vec<String>,
    /// Restrict to a single tier. `None` shows everything.
    pub tier: Option<Tier>,
    /// Case-insensitive substring query matched against each script's name,
    /// description, and tags. `None` means no search filter.
    pub search: Option<String>,
}

/// The tier-visibility rule for the default listing: everything is visible by
/// default. An explicit `--tier` narrows: `full` shows everything, `core` shows
/// only core.
fn tier_visible(entry: &Entry, tier: Option<Tier>) -> bool {
    match tier {
        Some(Tier::Core) => entry.tier == Tier::Core,
        Some(Tier::Full) | None => true,
    }
}

// Ranked search: a task query is tokenized and synonym-expanded, then scored
// against each entry, so a natural phrasing ("strip color codes from output")
// finds the right script even when no single word is a substring of it. The
// `mog mcp` server drives this ranker directly (it is the engine), so there is
// no second copy to keep in sync.

/// Interchangeable task words: a query token in a group also searches for the
/// group's other members (weighted below the caller's literal terms). Keeps a
/// phrasing like "whitespace cleanup" reaching actions/scripts that never
/// contain the word "whitespace". Keep groups tight to protect precision.
const SYNONYM_GROUPS: &[&[&str]] = &[
    &[
        "whitespace",
        "space",
        "spaces",
        "tab",
        "tabs",
        "indent",
        "indentation",
        "blank",
        "empty",
    ],
    &["trim", "strip", "clean", "cleanup", "tidy"],
    &[
        "collapse", "squeeze", "reduce", "dedupe", "dedup", "compact",
    ],
    &[
        "eol",
        "newline",
        "newlines",
        "linebreak",
        "crlf",
        "lf",
        "ending",
        "endings",
    ],
    &[
        "case",
        "upper",
        "uppercase",
        "lower",
        "lowercase",
        "capitalize",
        "title",
    ],
    &["prefix", "suffix", "affix", "prepend", "append"],
    &["encode", "decode", "escape", "unescape", "base64"],
    &["line", "lines", "row", "rows"],
    &["sort", "order", "alphabetize"],
    &["replace", "substitute", "swap", "regex"],
    &[
        "redact", "mask", "sanitize", "sanitise", "scrub", "censor", "secret", "secrets", "pii",
    ],
    &[
        "ansi",
        "color",
        "colors",
        "colour",
        "colours",
        "terminal",
        "decolorize",
        "vt100",
    ],
    &["log", "logs", "output", "console", "stdout"],
];

/// Split free text into lowercase alphanumeric terms (any other char is a break).
pub(crate) fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Expand query tokens with their synonyms: literal tokens carry weight 2,
/// synonyms pulled in by a group carry weight 1, so a literal word always
/// outranks an inferred one.
pub(crate) fn expand_query(tokens: &[String]) -> HashMap<String, i32> {
    let mut terms: HashMap<String, i32> = HashMap::new();
    for t in tokens {
        terms.insert(t.clone(), 2);
    }
    for t in tokens {
        for group in SYNONYM_GROUPS {
            if group.contains(&t.as_str()) {
                for g in *group {
                    terms.entry((*g).to_string()).or_insert(1);
                }
            }
        }
    }
    terms
}

/// Score name/tags (strong) + description (weak) against the expanded query
/// terms. OR semantics: any term hit counts; a strong hit is worth double a weak
/// hit; each is weighted by the term's literal(2)/synonym(1) weight. Shared by
/// the local `ls` ranker and the marketplace index ranker (`store_client`) so the
/// two never drift.
pub(crate) fn score_fields(
    name: Option<&str>,
    tags: &[String],
    task_phrases: &[String],
    description: Option<&str>,
    terms: &HashMap<String, i32>,
) -> i32 {
    let mut strong = String::new();
    if let Some(name) = name {
        strong.push_str(&name.to_lowercase());
        strong.push('\n');
    }
    for t in tags {
        strong.push_str(&t.to_lowercase());
        strong.push('\n');
    }
    // Task phrases are authored to match a user's own words, so they count as
    // strong signal alongside name and tags.
    for p in task_phrases {
        strong.push_str(&p.to_lowercase());
        strong.push('\n');
    }
    let weak = description.unwrap_or("").to_lowercase();
    let mut score = 0;
    for (term, weight) in terms {
        let in_strong = strong.contains(term.as_str());
        let in_weak = !in_strong && weak.contains(term.as_str());
        if !in_strong && !in_weak {
            continue;
        }
        score += weight * if in_strong { 2 } else { 1 };
    }
    score
}

fn score_entry(entry: &Entry, terms: &HashMap<String, i32>) -> i32 {
    score_fields(
        entry.name.as_deref(),
        &entry.tags,
        &entry.task_phrases,
        entry.description.as_deref(),
        terms,
    )
}

/// `mog ls`: compact, cheap discovery listing. Never prints script bodies.
pub fn ls(lib_root: Option<&Path>, filter: &LsFilter, json: bool) -> Result<i32> {
    let all = match lib_root {
        Some(root) if library::market_dir(root).is_dir() => scan(root),
        _ => Vec::new(),
    };

    let mut selected: Vec<&Entry> = all
        .iter()
        .filter(|e| tier_visible(e, filter.tier))
        .filter(|e| {
            filter
                .tags
                .iter()
                .all(|want| e.tags.iter().any(|t| t.eq_ignore_ascii_case(want)))
        })
        .collect();

    // A non-empty `--search` query RANKS the candidates (tokenized + synonym
    // aware, name/tags strong, description weak) and keeps only those that score,
    // best match first. sort_by is stable, so ties keep scan order. An empty
    // query (used to list everything) leaves the set untouched.
    if let Some(query) = &filter.search {
        let tokens = tokenize(query);
        if !tokens.is_empty() {
            let terms = expand_query(&tokens);
            let mut scored: Vec<(&Entry, i32)> = selected
                .iter()
                .map(|e| (*e, score_entry(e, &terms)))
                .filter(|(_, s)| *s > 0)
                .collect();
            scored.sort_by_key(|b| std::cmp::Reverse(b.1));
            selected = scored.into_iter().map(|(e, _)| e).collect();
        }
    }

    if json {
        // Compact records only (no script bodies), per the spec.
        let recs: Vec<serde_json::Value> = selected
            .iter()
            .map(|e| {
                serde_json::json!({
                    "resolvable": e.resolvable,
                    "name": e.name,
                    "description": e.description,
                    "tags": e.tags,
                    "task_phrases": e.task_phrases,
                    "tier": e.tier.as_str(),
                    "has_fixture": e.has_fixture,
                })
            })
            .collect();
        let root = serde_json::json!({
            "scripts": recs,
            "summary": { "count": selected.len() },
        });
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(0);
    }

    if selected.is_empty() {
        println!("(no scripts)");
        return Ok(0);
    }
    for e in &selected {
        let fixture = if e.has_fixture { "fixture" } else { "no-fix " };
        let desc = e
            .name
            .as_deref()
            .or(e.summary.as_deref())
            .or(e.description.as_deref())
            .unwrap_or("");
        println!(
            "{:<32} {:<5} {}  {}",
            e.resolvable,
            e.tier.as_str(),
            fixture,
            desc
        );
    }
    Ok(0)
}

/// Locate `resolved` (an absolute `.mog` path) within the flat `root`, returning
/// its `resolvable` name. Returns `None` when the path is not under the root
/// (e.g. a local path resolved base-relative rather than from the library).
fn locate_in_library(root: Option<&Path>, resolved: &Path) -> Option<String> {
    let market = library::market_dir(root?);
    if resolved.starts_with(&market) {
        Some(resolvable_name(&market, resolved))
    } else {
        None
    }
}

/// `mog show <name>`: full detail for one script, resolved exactly like `-m`.
pub fn show(lib_root: Option<&Path>, name: &Path, no_content: bool, json: bool) -> Result<i32> {
    let cwd = std::env::current_dir().ok();
    let resolved = library::resolve_script(name, cwd.as_deref(), lib_root)?;

    let resolvable =
        locate_in_library(lib_root, &resolved).unwrap_or_else(|| resolved.display().to_string());
    let MogMeta {
        name: mname,
        summary,
        description,
        tags,
        task_phrases,
        tier,
    } = read_metadata(&resolved);

    let input = testkit::sibling(&resolved, "TestInput");
    let expected = testkit::sibling(&resolved, "TestExpectedOutput");
    // Only run the fixture when both siblings exist; report pass/fail then.
    let fixture_pass = if input.is_some() && expected.is_some() {
        Some(testkit::run_mog_test(&resolved, None, None).pass)
    } else {
        None
    };
    let content = if no_content {
        None
    } else {
        Some(
            fs::read_to_string(&resolved)
                .with_context(|| format!("failed to read '{}'", resolved.display()))?,
        )
    };

    if json {
        // Prefer rendering the README on demand from the mog itself, so it is
        // populated even for a mog served from the installed store (flat layout)
        // or the embedded set, where no `README.md` file exists beside the script.
        // Fall back to a sibling `README.md` if one happens to be on disk; `null`
        // (not an error) when neither is available.
        let readme: Option<String> = crate::docgen::render_mog_doc(&resolved).ok().or_else(|| {
            resolved
                .parent()
                .map(|p| p.join("README.md"))
                .and_then(|p| fs::read_to_string(p).ok())
        });
        let mut root = serde_json::json!({
            "resolvable": resolvable,
            "name": mname,
            "summary": summary,
            "description": description,
            "tags": tags,
            "task_phrases": task_phrases,
            "tier": tier.as_str(),
            "provenance": serde_json::Value::Null,
            "fixture": {
                "input": input.as_ref().map(|p| p.display().to_string()),
                "expected": expected.as_ref().map(|p| p.display().to_string()),
                "pass": fixture_pass,
            },
            "readme": readme,
        });
        if let Some(c) = &content {
            root["content"] = serde_json::json!(c);
        }
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(0);
    }

    println!("name:        {}", mname.as_deref().unwrap_or("(unnamed)"));
    println!("resolvable:  {resolvable}");
    println!("tier:        {}", tier.as_str());
    if !tags.is_empty() {
        println!("tags:        {}", tags.join(", "));
    }
    if !task_phrases.is_empty() {
        println!("task_phrases: {}", task_phrases.join("; "));
    }
    if let Some(s) = &summary {
        println!("summary:     {s}");
    }
    if let Some(d) = &description {
        println!("description: {d}");
    }
    match (&input, &expected, fixture_pass) {
        (Some(i), Some(e), Some(pass)) => {
            println!("fixture:     {}", if pass { "PASS" } else { "FAIL" });
            println!("  input:     {}", i.display());
            println!("  expected:  {}", e.display());
        }
        _ => println!("fixture:     (missing)"),
    }
    if let Some(c) = &content {
        println!("---");
        print!("{c}");
        if !c.ends_with('\n') {
            println!();
        }
    }
    Ok(0)
}

/// A file we are about to write, remembered so a failed verify can roll back.
struct Staged {
    dest: PathBuf,
    prior: Option<Vec<u8>>,
}

impl Staged {
    fn restore(&self) {
        match &self.prior {
            Some(bytes) => {
                let _ = fs::write(&self.dest, bytes);
            }
            None => {
                let _ = fs::remove_file(&self.dest);
            }
        }
    }
}

/// `mog add <file.mog>`: copy the script and its fixtures into the flat store,
/// then verify the fixture in place. A failing/missing fixture aborts and rolls
/// back.
pub fn add(
    lib_root: Option<&Path>,
    file: &Path,
    as_name: Option<&str>,
    force: bool,
    json: bool,
) -> Result<i32> {
    let root = lib_root.ok_or_else(|| {
        anyhow!("no library root: set MOG_HOME or pass --mog-dir to install into the store")
    })?;
    // Enforce the `.mog`-only rule before reading the file (matches resolution).
    if !file
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("mog"))
    {
        bail!(
            "Bad file extension: mog scripts must be .mog files (got '{}')",
            file.display()
        );
    }
    if !file.is_file() {
        bail!("source script not found: {}", file.display());
    }
    // Installs land in the managed mog dir (`<root>/mogs/market`).
    let src_dir = library::market_dir(root);

    // Destination name: --as (a bare name or a relative subpath), else the source
    // file name. Always ends in `.mog`.
    let dest_rel = match as_name {
        Some(a) => {
            let mut p = PathBuf::from(a);
            if p.extension().and_then(|e| e.to_str()) != Some("mog") {
                p.set_extension("mog");
            }
            p
        }
        None => PathBuf::from(
            file.file_name()
                .ok_or_else(|| anyhow!("source path has no file name"))?,
        ),
    };
    // `dest_mog` is the flat LOGICAL path (source/<...>/<stem>.mog), used only to
    // derive the resolvable name; the mog is written into its own directory.
    let dest_mog = src_dir.join(&dest_rel);
    let dest_stem = dest_mog
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("destination has no valid stem"))?
        .to_string();
    let dest_parent = dest_mog
        .parent()
        .ok_or_else(|| anyhow!("destination has no parent directory"))?
        .to_path_buf();
    // One-directory-per-mog: <dest_parent>/<stem>/<stem>.mog plus a tests/
    // subdir holding input.<ext> / expected.<ext>.
    let mog_dir = dest_parent.join(&dest_stem);
    let tests_dir = mog_dir.join("tests");
    let final_mog = mog_dir.join(format!("{dest_stem}.mog"));

    // Locate the source fixtures up front (both are required to verify).
    let src_input = testkit::sibling(file, "TestInput").ok_or_else(|| {
        anyhow!(
            "{}: no tests/input.<ext> fixture; a script cannot enter the market unverified",
            file.display()
        )
    })?;
    let src_expected = testkit::sibling(file, "TestExpectedOutput").ok_or_else(|| {
        anyhow!(
            "{}: no tests/expected.<ext> fixture; a script cannot enter the market unverified",
            file.display()
        )
    })?;
    let input_ext = src_input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt");
    let expected_ext = src_expected
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("txt");
    let dest_input = tests_dir.join(format!("input.{input_ext}"));
    let dest_expected = tests_dir.join(format!("expected.{expected_ext}"));

    let new_mog_bytes = fs::read(file)?;
    // Report the mog by its actual installed path (<name>/<name>.mog),
    // consistent with what `scan` / `market list` / `show` surface.
    let resolvable = resolvable_name(&src_dir, &final_mog);

    // Idempotency: identical .mog content is a no-op; differing content needs
    // --force.
    if final_mog.is_file() {
        let existing = fs::read(&final_mog)?;
        if existing == new_mog_bytes {
            return add_report(json, &resolvable, false, true, true);
        }
        if !force {
            bail!(
                "{resolvable} already exists with different content; pass --force to overwrite \
                 (existing {} bytes, new {} bytes)",
                existing.len(),
                new_mog_bytes.len()
            );
        }
    }

    fs::create_dir_all(&tests_dir)
        .with_context(|| format!("failed to create recipe dir '{}'", mog_dir.display()))?;

    // Stage all three writes so a failed verify rolls the market back exactly.
    let mut staged: Vec<Staged> = Vec::new();
    let copies = [
        (file.to_path_buf(), final_mog.clone()),
        (src_input.clone(), dest_input.clone()),
        (src_expected.clone(), dest_expected.clone()),
    ];
    for (src, dest) in copies {
        let prior = if dest.is_file() {
            fs::read(&dest).ok()
        } else {
            None
        };
        let bytes =
            fs::read(&src).with_context(|| format!("failed to read '{}'", src.display()))?;
        if let Err(e) = fs::write(&dest, &bytes) {
            // Undo whatever we already staged, then fail.
            for s in &staged {
                s.restore();
            }
            return Err(anyhow!("failed to write '{}': {e}", dest.display()));
        }
        staged.push(Staged { dest, prior });
    }

    // Verify the fixture in its installed location.
    let outcome = testkit::run_mog_test(&final_mog, None, None);
    if !outcome.pass {
        for s in &staged {
            s.restore();
        }
        let reason = outcome
            .message
            .unwrap_or_else(|| "fixture did not pass".to_string());
        if let Some(diff) = outcome.diff {
            bail!("install aborted: {reason}\n{diff}");
        }
        bail!("install aborted: {reason}");
    }

    add_report(json, &resolvable, true, false, true)
}

/// Emit the `add` result (human or JSON) and return exit 0.
fn add_report(
    json: bool,
    resolvable: &str,
    installed: bool,
    noop: bool,
    fixture_pass: bool,
) -> Result<i32> {
    if json {
        let root = serde_json::json!({
            "resolvable": resolvable,
            "installed": installed,
            "noop": noop,
            "fixture_pass": fixture_pass,
        });
        println!("{}", serde_json::to_string_pretty(&root)?);
    } else if noop {
        println!("no change: {resolvable} already installed (identical content)");
    } else {
        println!("installed {resolvable} (fixture verified)");
    }
    Ok(0)
}

/// `mog rm <name>`: remove a stored mog and its fixtures.
pub fn rm(lib_root: Option<&Path>, name: &Path, json: bool) -> Result<i32> {
    let root =
        lib_root.ok_or_else(|| anyhow!("no library root: set MOG_HOME or pass --mog-dir"))?;
    // Resolve WITHOUT a base dir so a bare store name never resolves to a stray
    // local file; ambiguity in the store still errors (matching resolution).
    let resolved = library::resolve_script(name, None, Some(root))?;

    let resolvable = locate_in_library(Some(root), &resolved).ok_or_else(|| {
        anyhow!(
            "'{}' did not resolve inside the library; nothing to remove",
            name.display()
        )
    })?;

    // The mog lives in its own directory (<name>/, holding the .mog
    // and its tests/ fixtures); remove the whole directory.
    let mog_dir = resolved
        .parent()
        .ok_or_else(|| anyhow!("resolved script has no parent directory"))?
        .to_path_buf();
    fs::remove_dir_all(&mog_dir)
        .with_context(|| format!("failed to remove '{}'", mog_dir.display()))?;
    let removed = vec![mog_dir.display().to_string()];

    if json {
        let root = serde_json::json!({
            "resolvable": resolvable,
            "removed": removed,
        });
        println!("{}", serde_json::to_string_pretty(&root)?);
    } else {
        println!("removed {resolvable}:");
        for r in &removed {
            println!("  {r}");
        }
    }
    Ok(0)
}

/// `mog bless <name>`: regenerate the golden from the current TestInput, showing
/// the diff first and only writing with confirmation (`--yes`).
pub fn bless(lib_root: Option<&Path>, name: &Path, yes: bool, json: bool) -> Result<i32> {
    let cwd = std::env::current_dir().ok();
    let resolved = library::resolve_script(name, cwd.as_deref(), lib_root)?;

    let input_path = testkit::sibling(&resolved, "TestInput").ok_or_else(|| {
        anyhow!(
            "{}: no TestInput.<ext> sibling to bless against",
            resolved.display()
        )
    })?;
    let input = fs::read_to_string(&input_path)
        .with_context(|| format!("failed to read '{}'", input_path.display()))?;

    let script = fs::read_to_string(&resolved)
        .with_context(|| format!("failed to read '{}'", resolved.display()))?;
    let parsed =
        crate::parse_mog(&script).with_context(|| format!("parse '{}'", resolved.display()))?;
    let produced = crate::execute_at(&parsed, &input, resolved.parent())
        .with_context(|| format!("run '{}'", resolved.display()))?;

    // The golden path: the existing fixture, else derived from the input's ext,
    // written into the mog's tests/ directory.
    let expected_path = testkit::sibling(&resolved, "TestExpectedOutput").unwrap_or_else(|| {
        let ext = input_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");
        resolved
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("tests")
            .join(format!("expected.{ext}"))
    });
    let current = fs::read_to_string(&expected_path).unwrap_or_default();

    // Compare EOL-normalized, exactly like the test harness does.
    let changed = testkit::lf(&produced) != testkit::lf(&current);
    let diff = if changed {
        Some(testkit::unified_diff(
            &expected_path.display().to_string(),
            &current,
            &produced,
        ))
    } else {
        None
    };

    // Ensure the mog's tests/ directory exists before any write (it always
    // does for an installed mog; this only matters when blessing a brand-new
    // golden whose fixture did not yet exist).
    if let Some(parent) = expected_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create '{}'", parent.display()))?;
    }

    if json {
        let root = serde_json::json!({
            "name": resolved.file_name().map(|n| n.to_string_lossy().to_string()),
            "expected": expected_path.display().to_string(),
            "changed": changed,
            "written": changed && yes,
            "diff": diff,
        });
        // Under --json we only write when --yes is also given.
        if changed && yes {
            fs::write(&expected_path, &produced)
                .with_context(|| format!("failed to write golden '{}'", expected_path.display()))?;
        }
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(0);
    }

    if !changed {
        println!(
            "no change: {} already matches its script output",
            expected_path.display()
        );
        return Ok(0);
    }

    print!("{}", diff.as_deref().unwrap_or(""));
    if yes {
        fs::write(&expected_path, &produced)
            .with_context(|| format!("failed to write golden '{}'", expected_path.display()))?;
        println!("blessed: wrote {}", expected_path.display());
    } else {
        println!(
            "not written (diff shown above); re-run with --yes to overwrite {}",
            expected_path.display()
        );
    }
    Ok(0)
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    use tempfile::tempdir;

    /// Write a minimal valid mog at `<root>/mogs/market/<name>/<name>.mog`.
    fn write_mog(root: &Path, name: &str, summary: &str) {
        let dir = library::market_dir(root).join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{name}.mog")),
            format!(r#"{{"name":"{name}","summary":"{summary}","steps":[]}}"#),
        )
        .unwrap();
    }

    #[test]
    fn scan_writes_and_reuses_the_index() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        let first = scan(root.path());
        assert_eq!(first.len(), 1);
        assert!(
            cache_path(root.path()).is_file(),
            "a cold scan must persist the index"
        );
        // A second scan (fingerprint unchanged) is served from the cache and is
        // byte-for-byte the same listing.
        let second = scan(root.path());
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].name.as_deref(), Some("alpha"));
        assert_eq!(second[0].summary.as_deref(), Some("one"));
    }

    #[test]
    fn adding_a_mog_invalidates_the_cache() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        assert_eq!(scan(root.path()).len(), 1);
        write_mog(root.path(), "beta", "two");
        let names: Vec<String> = scan(root.path())
            .into_iter()
            .filter_map(|e| e.name)
            .collect();
        assert_eq!(names.len(), 2);
        assert!(names.iter().any(|n| n == "beta"));
    }

    #[test]
    fn removing_a_mog_invalidates_the_cache() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        write_mog(root.path(), "beta", "two");
        assert_eq!(scan(root.path()).len(), 2);
        fs::remove_dir_all(library::market_dir(root.path()).join("beta")).unwrap();
        assert_eq!(scan(root.path()).len(), 1);
    }

    #[test]
    fn editing_a_mog_in_place_is_reflected() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        assert_eq!(scan(root.path())[0].summary.as_deref(), Some("one"));
        // Advance well past any filesystem mtime granularity, then edit in place
        // (count unchanged): the newest-mtime half of the fingerprint must catch it.
        std::thread::sleep(std::time::Duration::from_millis(1100));
        write_mog(root.path(), "alpha", "TWO");
        assert_eq!(scan(root.path())[0].summary.as_deref(), Some("TWO"));
    }

    #[test]
    fn a_corrupt_index_is_ignored_and_rebuilt() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        scan(root.path());
        fs::write(cache_path(root.path()), b"{ not valid json").unwrap();
        // A garbage index must never poison the scan; it recomputes and overwrites.
        let got = scan(root.path());
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].name.as_deref(), Some("alpha"));
        let rebuilt = fs::read_to_string(cache_path(root.path())).unwrap();
        assert!(
            rebuilt.contains("alpha"),
            "index should be rebuilt: {rebuilt}"
        );
    }

    #[test]
    fn load_cache_matches_only_on_an_identical_fingerprint() {
        let root = tempdir().unwrap();
        write_mog(root.path(), "alpha", "one");
        let entries = scan(root.path());
        store_cache(root.path(), entries.len(), 42, &entries);
        assert!(load_cache(root.path(), entries.len(), 42).is_some());
        assert!(
            load_cache(root.path(), entries.len() + 1, 42).is_none(),
            "a different count must miss"
        );
        assert!(
            load_cache(root.path(), entries.len(), 43).is_none(),
            "a different newest-mtime must miss"
        );
    }

    #[test]
    fn a_missing_root_is_not_created_by_scanning() {
        let parent = tempdir().unwrap();
        let missing = parent.path().join("no-such-lib");
        assert!(scan(&missing).is_empty());
        assert!(!missing.exists(), "scanning must not materialize the root");
    }
}
