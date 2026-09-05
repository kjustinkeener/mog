//! Build script: (1) tell cargo to rebuild when an embedded `include_dir!`
//! directory changes, and (2) embed the Windows application icon into `mog.exe`.
//!
//! The icon step only runs when the build host is Windows (the `winresource`
//! build-dependency is gated to `cfg(windows)` in Cargo.toml). On other hosts
//! that step is a no-op and the icon is simply not embedded.

fn main() {
    // src/setup.rs and src/report.rs embed these directories via include_dir! at
    // compile time. cargo cannot see through the macro, so adding or editing a
    // factory script (or a dashboard template) would NOT trigger a rebuild and
    // the change would silently fail to ship. Emit rerun-if-changed for the dirs
    // and every file under them so any such edit recompiles and re-embeds.
    for dir in ["factory", "templates/factory"] {
        rerun_if_changed_recursive(std::path::Path::new(dir));
    }

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/mog.ico");
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/mog.ico");
        if let Err(e) = res.compile() {
            // Don't fail the build if the resource compiler is unavailable;
            // the binary is still fully functional, just without an icon.
            println!("cargo:warning=icon embed skipped: {e}");
        }
    }
}

/// Emit `cargo:rerun-if-changed` for `path` and, recursively, everything under
/// it, so a change to any embedded file (not just the directory entry) forces a
/// rebuild. A bare directory rerun-if-changed misses in-place file edits on some
/// platforms; walking the tree makes content changes reliably invalidate.
fn rerun_if_changed_recursive(path: &std::path::Path) {
    println!("cargo:rerun-if-changed={}", path.display());
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                rerun_if_changed_recursive(&p);
            } else {
                println!("cargo:rerun-if-changed={}", p.display());
            }
        }
    }
}
