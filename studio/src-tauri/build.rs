use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // Re-embed the window/exe icon when it changes (tauri-build does not watch it).
    println!("cargo:rerun-if-changed=icons/icon.ico");
    // Embed the dist `mog.exe` so Studio is the whole product in one download.
    embed_engine();
    // Optional build date for the install card (CI can set MOG_STUDIO_BUILD_DATE).
    let date = std::env::var("MOG_STUDIO_BUILD_DATE").unwrap_or_else(|_| "dev".into());
    println!("cargo:rustc-env=MOG_STUDIO_BUILD_DATE={date}");
    println!("cargo:rerun-if-env-changed=MOG_STUDIO_BUILD_DATE");
    tauri_build::build()
}

/// Gzip the freshly-built dist `mog.exe` into `$OUT_DIR/mog.exe.gz`, which
/// `install.rs` pulls in with `include_bytes!` and extracts on install.
///
/// Source resolution: `MOG_DIST_EXE` (set by CI, pointing at the `--profile dist`
/// build) wins; otherwise a locally staged `binaries/mog-dist.exe`. When neither
/// exists (a plain `cargo tauri dev` loop), we embed an empty payload so the crate
/// still compiles; `install.rs::extract_engine` then reports "no embedded engine"
/// and the dev instance falls back to a PATH / `%LOCALAPPDATA%` `mog.exe`.
fn embed_engine() {
    println!("cargo:rerun-if-env-changed=MOG_DIST_EXE");
    println!("cargo:rerun-if-changed=binaries/mog-dist.exe");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let dest = out.join("mog.exe.gz");

    let src: Option<PathBuf> = std::env::var_os("MOG_DIST_EXE")
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| {
            let staged = Path::new("binaries/mog-dist.exe");
            staged.is_file().then(|| staged.to_path_buf())
        });

    let bytes = match &src {
        Some(p) => {
            let b = std::fs::read(p).expect("read engine exe to embed");
            println!(
                "cargo:warning=Studio embedding engine from {} ({} bytes uncompressed)",
                p.display(),
                b.len()
            );
            b
        }
        None => {
            println!(
                "cargo:warning=no dist mog.exe found (set MOG_DIST_EXE or stage \
                 binaries/mog-dist.exe); building Studio WITHOUT an embedded engine"
            );
            Vec::new()
        }
    };

    let f = std::fs::File::create(&dest).expect("create mog.exe.gz");
    let mut enc = flate2::write::GzEncoder::new(f, flate2::Compression::best());
    enc.write_all(&bytes).expect("gzip engine");
    enc.finish().expect("finish gzip");
}
