//! The mog script library and name resolution.
//!
//! A `.mog` script named on the command line (`-m`) or by a `run_mog` step is
//! resolved by this order:
//!   1. an absolute path is used as-is;
//!   2. a path found relative to the current base (the working dir for `-m`, or
//!      the parent `.mog`'s directory for `run_mog`) is used;
//!   3. otherwise the library is searched.
//!
//! The store root is the default `%APPDATA%\mog` on Windows or `~/.mog`
//! elsewhere, overridable via `MOG_HOME` or the `--mog-dir` flag. The managed
//! recipe set lives under `<root>/mogs/market/`, one directory per recipe
//! (`<root>/mogs/market/<name>/<name>.mog`); there are no source subfolders.
//! The user's own recipes live under `<root>/mogs/user/`, which the engine never
//! scans, surfaces, overrides, or overwrites. Non-recipe siblings (`.cache`,
//! `templates`, `reports`, `config.toml`) stay at the outer root.
//!
//! mog only ever resolves and reads `.mog` files. Before any of the above steps
//! runs, the requested name is passed through the extension rule (see
//! [`require_mog_extension`]): a `.mog` name (any case) resolves as-is, an
//! extensionless name gets `.mog` appended (`tidy-list` -> `tidy-list.mog`), and
//! any other extension is refused before the file is ever opened.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

/// The default library root: `MOG_HOME` if set, else the per-user default
/// (`%APPDATA%\mog` on Windows, `$HOME/.mog` otherwise). Returned whether or not
/// the directory exists yet; callers treat a missing root as "no library".
pub fn default_root() -> Option<PathBuf> {
    if let Some(v) = std::env::var_os("MOG_HOME") {
        if !v.is_empty() {
            return Some(PathBuf::from(v));
        }
    }
    if cfg!(windows) {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("mog"))
    } else {
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".mog"))
    }
}

/// The managed-recipe directory under a store `root`: `<root>/mogs/market`. This
/// is the ONE place the engine reads, scans, resolves, installs, and removes
/// recipes; it holds the embedded factory set and market installs, and is what
/// `setup` overwrites wholesale. Every recipe path composes from this helper so
/// the `mogs/market` join is never scattered across the codebase.
pub fn market_dir(root: &Path) -> PathBuf {
    root.join("mogs").join("market")
}

/// The user's own recipe directory under a store `root`: `<root>/mogs/user`. The
/// engine never scans, surfaces, overrides, or overwrites it; `setup` only
/// ensures it exists. It is Studio's default save-dialog location.
pub fn user_dir(root: &Path) -> PathBuf {
    root.join("mogs").join("user")
}

/// Resolve a script name/path to a concrete file path (see module docs).
///
/// `name` is the raw `-m` value or `run_mog` `file`. `base` is the directory to
/// resolve relative paths against. `lib_root` is the library root, if any.
pub fn resolve_script(
    name: &Path,
    base: Option<&Path>,
    lib_root: Option<&Path>,
) -> Result<PathBuf> {
    // 0. Enforce the `.mog`-only rule before touching the filesystem. This may
    //    append `.mog` to an extensionless name; every step below uses the result.
    let name = require_mog_extension(name)?;
    let name = name.as_ref();

    // 1. Absolute path: use verbatim.
    if name.is_absolute() {
        return Ok(name.to_path_buf());
    }

    // 2. Relative to the current base.
    if let Some(base) = base {
        let candidate = base.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    // 3. Library lookup. The managed recipe set lives under `<root>/mogs/market`,
    //    so search there when it exists. Otherwise `lib_root` already IS a recipe
    //    directory (the torture harness passes a recipe's enclosing dir, and the
    //    repo `factory/` tree is searched directly), so fall back to it verbatim.
    if let Some(root) = lib_root {
        let market = market_dir(root);
        let search_root = if market.is_dir() {
            market.as_path()
        } else {
            root
        };
        if search_root.is_dir() {
            if let Some(found) = search_library(search_root, name)? {
                return Ok(found);
            }
        }
    }

    // Nothing resolved: build a helpful error.
    if base.is_none() && lib_root.is_none() {
        // Preserves the historical message for pipelines run without any context.
        bail!(
            "run_mog needs a base directory (the .mog file's location); \
             it cannot be used when the pipeline was run without one"
        );
    }
    let mut searched: Vec<String> = Vec::new();
    if let Some(base) = base {
        searched.push(format!("relative to '{}'", base.display()));
    }
    if let Some(root) = lib_root {
        let market = market_dir(root);
        let search_root = if market.is_dir() {
            market
        } else {
            root.to_path_buf()
        };
        searched.push(format!("library '{}'", search_root.display()));
    }
    bail!(
        "script '{}' not found ({})",
        name.display(),
        searched.join(", ")
    )
}

/// Reject a composition target that would escape the resolution roots: absolute
/// paths, rooted paths, and any `..` traversal. Store recipes reference their
/// children by name only, so a `run_mog` / `for_each_block` target must stay
/// within the base directory or the library. Top-level `mog -m <path>` is
/// unaffected; only child (composed) resolution is confined. Engine hardening H1
/// (store-marketplace-spec.md section 2.2).
pub fn ensure_confined(name: &Path) -> Result<()> {
    if name.is_absolute() || name.has_root() {
        bail!(
            "composed script path '{}' must be relative: absolute or rooted paths \
             are not allowed in run_mog / for_each_block",
            name.display()
        );
    }
    if name
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        bail!(
            "composed script path '{}' must not contain '..' (it could escape the library)",
            name.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod confine_tests {
    use super::ensure_confined;
    use std::path::Path;

    #[test]
    fn allows_plain_and_subpath_names() {
        assert!(ensure_confined(Path::new("tidy-list.mog")).is_ok());
        assert!(ensure_confined(Path::new("sql/strip-ansi.mog")).is_ok());
        assert!(ensure_confined(Path::new("sub/dir/x.mog")).is_ok());
    }

    #[test]
    fn rejects_escapes() {
        assert!(ensure_confined(Path::new("../secret.mog")).is_err());
        assert!(ensure_confined(Path::new("a/../../b.mog")).is_err());
        assert!(ensure_confined(Path::new("/etc/passwd.mog")).is_err());
        #[cfg(windows)]
        assert!(ensure_confined(Path::new("C:\\Windows\\x.mog")).is_err());
    }
}

/// Apply the `.mog`-only extension rule to a requested script name/path, before
/// any filesystem access. Only the FINAL path component's extension is inspected,
/// so subpath prefixes (`sql/...`) and internal dots (`my.v2`) are handled
/// correctly: `sql/` is a path component, not an extension, and `my.v2` has
/// the extension `v2`.
///
///   1. final component ends in `.mog` (case-insensitive) -> returned unchanged;
///   2. no extension at all (`tidy-list`, `sql/foo`) -> `.mog` appended;
///   3. any other extension (`notes.txt`, `my.v2`) -> refused, file never opened.
fn require_mog_extension(name: &Path) -> Result<Cow<'_, Path>> {
    match name.extension() {
        // Rule 1: already a .mog (any case). Use as-is.
        Some(ext) if ext.eq_ignore_ascii_case("mog") => Ok(Cow::Borrowed(name)),
        // Rule 3: a different extension. Refuse before reading anything.
        Some(_) => bail!(
            "Bad file extension: mog scripts must be .mog files (got '{}')",
            name.display()
        ),
        // Rule 2: no extension. Append `.mog` and resolve that.
        None => {
            let mut owned = name.to_path_buf();
            owned.set_extension("mog");
            Ok(Cow::Owned(owned))
        }
    }
}

/// Search the flat library `root` for `name`: first as a direct relative path,
/// then recursively by file name. Ambiguous basename matches error.
fn search_library(root: &Path, name: &Path) -> Result<Option<PathBuf>> {
    let direct = root.join(name);
    if direct.is_file() {
        return Ok(Some(direct));
    }
    let Some(basename) = name.file_name() else {
        return Ok(None);
    };
    let mut matches: Vec<PathBuf> = Vec::new();
    collect_by_name(root, basename, &mut matches)?;
    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches.pop().unwrap())),
        _ => {
            matches.sort();
            let list = matches
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(anyhow!(
                "ambiguous script '{}' in library '{}': {} (use a subpath to disambiguate)",
                name.display(),
                root.display(),
                list
            ))
        }
    }
}

/// Recursively collect files under `dir` whose file name equals `basename`.
fn collect_by_name(dir: &Path, basename: &std::ffi::OsStr, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()), // unreadable subdir: skip rather than fail the whole search
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_by_name(&path, basename, out)?;
        } else if path.file_name() == Some(basename) {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Build a store root whose managed recipe dir (`<root>/mogs/market`) holds
    /// the given relpath files (content `{}`). The library lookup resolves against
    /// `market_dir(root)`, so callers pass `root.path()` and expect matches under
    /// `market_dir(root.path())`.
    fn lib_with(files: &[&str]) -> tempfile::TempDir {
        let root = tempdir().unwrap();
        let market = market_dir(root.path());
        for rel in files {
            let p = market.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "{}").unwrap();
        }
        root
    }

    #[test]
    fn absolute_path_is_returned_verbatim() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("a.mog");
        assert_eq!(resolve_script(&f, None, None).unwrap(), f);
    }

    #[test]
    fn base_relative_is_used_when_present() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.mog"), "{}").unwrap();
        let got = resolve_script(Path::new("a.mog"), Some(dir.path()), None).unwrap();
        assert_eq!(got, dir.path().join("a.mog"));
    }

    #[test]
    fn recipe_resolves_from_its_own_directory() {
        // One directory per recipe: <root>/<name>/<name>.mog resolves by bare name.
        let root = lib_with(&["x/x.mog"]);
        let got = resolve_script(Path::new("x.mog"), None, Some(root.path())).unwrap();
        assert_eq!(got, market_dir(root.path()).join("x").join("x.mog"));
    }

    #[test]
    fn recursive_search_finds_nested_script() {
        let root = lib_with(&["sql/deep.mog"]);
        let got = resolve_script(Path::new("deep.mog"), None, Some(root.path())).unwrap();
        assert_eq!(got, market_dir(root.path()).join("sql").join("deep.mog"));
    }

    #[test]
    fn ambiguous_basename_errors() {
        let root = lib_with(&["a/dup.mog", "b/dup.mog"]);
        let err = resolve_script(Path::new("dup.mog"), None, Some(root.path())).unwrap_err();
        assert!(err.to_string().contains("ambiguous"), "{err}");
    }

    #[test]
    fn not_found_reports_searched_locations() {
        let root = lib_with(&["other.mog"]);
        let err = resolve_script(
            Path::new("missing.mog"),
            Some(root.path()),
            Some(root.path()),
        )
        .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not found") && msg.contains("library"),
            "{msg}"
        );
    }

    #[test]
    fn no_base_and_no_library_keeps_legacy_message() {
        let err = resolve_script(Path::new("x.mog"), None, None).unwrap_err();
        assert!(err.to_string().contains("base directory"), "{err}");
    }

    #[test]
    fn bare_name_appends_mog_and_resolves_base_relative() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.mog"), "{}").unwrap();
        // No extension: `a` should resolve exactly like `a.mog`.
        let got = resolve_script(Path::new("a"), Some(dir.path()), None).unwrap();
        assert_eq!(got, dir.path().join("a.mog"));
    }

    #[test]
    fn bare_name_appends_mog_and_resolves_in_library() {
        let root = lib_with(&["tidy-list/tidy-list.mog"]);
        let got = resolve_script(Path::new("tidy-list"), None, Some(root.path())).unwrap();
        assert_eq!(
            got,
            market_dir(root.path())
                .join("tidy-list")
                .join("tidy-list.mog")
        );
    }

    #[test]
    fn wrong_extension_is_refused_without_reading() {
        let dir = tempdir().unwrap();
        // The file exists, but the extension is wrong: we must refuse, not open it.
        fs::write(dir.path().join("notes.txt"), "not a mog").unwrap();
        let err = resolve_script(Path::new("notes.txt"), Some(dir.path()), None).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Bad file extension"), "{msg}");
        assert!(msg.contains("notes.txt"), "{msg}");
    }

    #[test]
    fn internal_dot_counts_as_extension_and_is_refused() {
        // `my.v2` has extension `v2`; it must NOT be turned into `my.v2.mog`.
        let err = resolve_script(Path::new("my.v2"), None, None).unwrap_err();
        assert!(err.to_string().contains("Bad file extension"), "{}", err);
    }

    #[test]
    fn mog_extension_is_case_insensitive() {
        let dir = tempdir().unwrap();
        let f = dir.path().join("a.MOG");
        // Absolute .MOG path is accepted verbatim (rule 1 is case-insensitive).
        assert_eq!(resolve_script(&f, None, None).unwrap(), f);
    }
}
