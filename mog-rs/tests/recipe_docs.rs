//! Drift check: every factory recipe's committed `README.md` must match what
//! `mog market gen-docs` would regenerate from the recipe (metadata + golden
//! fixture). This keeps the docs honest the same way the golden fixtures keep the
//! recipes honest. When it fails, regenerate:
//!
//!     mog market gen-docs mog-rs/factory

use std::path::PathBuf;

use mog::docgen::render_mog_doc;
use mog::testkit::find_mogs;

fn norm(s: &str) -> String {
    s.replace("\r\n", "\n")
}

#[test]
fn mog_readmes_are_up_to_date() {
    let factory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("factory");
    let mogs = find_mogs(&factory);
    assert!(
        !mogs.is_empty(),
        "no factory recipes found under {factory:?}"
    );

    let mut drifted = Vec::new();
    for mog in &mogs {
        let expected = render_mog_doc(mog).expect("render recipe doc");
        let readme = mog.parent().unwrap().join("README.md");
        let actual = std::fs::read_to_string(&readme).unwrap_or_default();
        if norm(&actual) != norm(&expected) {
            drifted.push(readme.display().to_string());
        }
    }

    assert!(
        drifted.is_empty(),
        "{} recipe README(s) are out of date; regenerate with \
         `mog market gen-docs mog-rs/factory`:\n{}",
        drifted.len(),
        drifted.join("\n")
    );
}
