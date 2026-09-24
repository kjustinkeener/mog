use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // Re-embed the window/exe icon when it changes (tauri-build does not watch it).
    println!("cargo:rerun-if-changed=icons/icon.ico");
    // Embed the platform-native dist engine so Studio is the whole product in one download.
    embed_engine();
    // Optional build date for the install card (CI can set MOG_STUDIO_BUILD_DATE).
    let date = std::env::var("MOG_STUDIO_BUILD_DATE").unwrap_or_else(|_| "dev".into());
    println!("cargo:rustc-env=MOG_STUDIO_BUILD_DATE={date}");
    println!("cargo:rerun-if-env-changed=MOG_STUDIO_BUILD_DATE");
    tauri_build::build()
}

/// Gzip the freshly-built dist engine into `$OUT_DIR`, which `install.rs` pulls
/// in with `include_bytes!` and extracts on Windows installs.
///
/// `MOG_DIST_BIN` is the cross-platform CI override. `MOG_DIST_EXE` remains
/// supported for existing Windows release jobs. When neither it nor the
/// platform-specific staged binary exists, embed an empty payload so dev builds
/// continue to compile and resolve `mog` from `MOG_BIN` or PATH.
fn embed_engine() {
    let engine_name = if cfg!(windows) { "mog.exe" } else { "mog" };
    let staged_name = if cfg!(windows) {
        "mog-dist.exe"
    } else {
        "mog-dist"
    };
    let archive_name = if cfg!(windows) {
        "mog.exe.gz"
    } else {
        "mog.gz"
    };
    println!("cargo:rustc-env=MOG_ENGINE_GZ={archive_name}");
    println!("cargo:rerun-if-env-changed=MOG_DIST_BIN");
    println!("cargo:rerun-if-env-changed=MOG_DIST_EXE");
    println!("cargo:rerun-if-changed=binaries/{staged_name}");

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let dest = out.join(archive_name);

    let src: Option<PathBuf> = std::env::var_os("MOG_DIST_BIN")
        .or_else(|| std::env::var_os("MOG_DIST_EXE"))
        .map(PathBuf::from)
        .filter(|p| p.is_file())
        .or_else(|| {
            let staged = Path::new("binaries").join(staged_name);
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
                "cargo:warning=no dist {engine_name} found (set MOG_DIST_BIN or stage \
                 binaries/{staged_name}); building Studio WITHOUT an embedded engine"
            );
            Vec::new()
        }
    };

    let f = std::fs::File::create(&dest).expect("create mog.exe.gz");
    let mut enc = flate2::write::GzEncoder::new(f, flate2::Compression::best());
    enc.write_all(&bytes).expect("gzip engine");
    enc.finish().expect("finish gzip");
}
