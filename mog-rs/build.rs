//! Build script: (1) tell cargo to rebuild when an embedded `include_dir!`
//! directory changes, and (2) embed the Windows application icon into `mog.exe`.
//!
//! The icon step only runs when the build host is Windows (the `winresource`
//! build-dependency is gated to `cfg(windows)` in Cargo.toml). On other hosts
//! that step is a no-op and the icon is simply not embedded.
//!
//! It also packs the `factory/` bundle into `${OUT_DIR}/factory.pack`
//! (compressed and obfuscated) so its realistic secret-shaped fixtures do not
//! ship as plaintext in the binary; `src/setup.rs` decodes it at install time.
//! See factory_pack.rs.

#[path = "src/factory_pack.rs"]
mod factory_pack;

use std::path::Path;

fn main() {
    // src/report.rs embeds templates/factory via include_dir! at compile time,
    // and src/setup.rs embeds ${OUT_DIR}/factory.pack (built below from factory/).
    // cargo cannot see through the macro, so adding or editing a factory script
    // (or a dashboard template) would NOT trigger a rebuild and the change would
    // silently fail to ship. Emit rerun-if-changed for the dirs and every file
    // under them so any such edit recompiles and re-embeds.
    for dir in ["factory", "templates/factory"] {
        rerun_if_changed_recursive(std::path::Path::new(dir));
    }

    pack_factory();

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

/// Walk `factory/` (files sorted by relative forward-slash path for a stable,
/// reproducible archive), pack them via [`factory_pack::encode`], and write the
/// result to `${OUT_DIR}/factory.pack` for `src/setup.rs` to `include_bytes!`.
fn pack_factory() {
    let root = Path::new("factory");
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect_files(root, root, &mut files);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let packed = factory_pack::encode(&files);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR");
    let dest = Path::new(&out_dir).join("factory.pack");
    std::fs::write(&dest, &packed).expect("write factory.pack");
}

/// Recursively collect `(relative forward-slash path, contents)` for every file
/// under `dir`, relative to `base`.
fn collect_files(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_files(base, &p, out);
        } else {
            let rel = p
                .strip_prefix(base)
                .expect("under base")
                .to_string_lossy()
                .replace('\\', "/");
            let data = std::fs::read(&p).expect("read factory file");
            out.push((rel, data));
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
