//! Loading external data `sources` (name -> lines) for source-aware actions
//! (`fill_from_list`, and the two-input `compare` family). Mog-declared source
//! paths are confined to the .mog's directory; the CLI `--source NAME=PATH` binds
//! any path and overrides a same-named declaration.
//!
//! This is the ONE place the confined mog-source loading lives, shared by the
//! CLI (`main.rs`) and the torture harness (`testkit`), so `mog --test` threads a
//! mog's declared sources exactly as a real run would.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// A path is confined if it is relative and contains no `..` component.
pub fn is_confined(p: &Path) -> bool {
    !p.is_absolute()
        && !p
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
}

/// Read a file into its list of lines (terminators dropped).
pub fn read_source_lines(path: &Path, name: &str) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read source '{name}' from '{}'", path.display()))?;
    Ok(text.lines().map(|l| l.to_string()).collect())
}

/// Load a mog's own declared `sources` (name -> relative path), each confined
/// to `base_dir` (the .mog's directory) and read into its list of lines. No CLI
/// overrides are applied here; callers that support `--source` layer those on top.
pub fn load_mog_sources(
    mog: &BTreeMap<String, String>,
    base_dir: Option<&Path>,
) -> Result<BTreeMap<String, Vec<String>>> {
    let mut out = BTreeMap::new();
    for (name, path) in mog {
        let p = Path::new(path);
        if !is_confined(p) {
            bail!(
                "source '{name}': mog-declared path '{path}' must be relative and stay \
                 within the .mog's directory (bind an outside path with --source {name}=...)"
            );
        }
        let resolved: PathBuf = match base_dir {
            Some(b) => b.join(p),
            None => p.to_path_buf(),
        };
        out.insert(name.clone(), read_source_lines(&resolved, name)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confinement_rejects_absolute_and_parent() {
        assert!(is_confined(Path::new("list.txt")));
        assert!(is_confined(Path::new("sub/list.txt")));
        assert!(!is_confined(Path::new("../list.txt")));
        assert!(!is_confined(Path::new("a/../../list.txt")));
        #[cfg(windows)]
        assert!(!is_confined(Path::new("C:\\list.txt")));
        #[cfg(not(windows))]
        assert!(!is_confined(Path::new("/etc/list.txt")));
    }
}
