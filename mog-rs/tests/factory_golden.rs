//! Runs every factory recipe against its golden fixture.
//!
//! Each recipe lives in its own directory under `factory/`:
//! `factory/<name>/<name>.mog` with `factory/<name>/tests/input.<ext>` and
//! `factory/<name>/tests/expected.<ext>`. This suite discovers every `.mog` under
//! `factory/` (recursively, via `mog::testkit::find_mogs`) and asserts each one
//! reproduces its golden. It closes the coverage gap where `cargo test` only
//! exercised the handful of sample recipes, and it proves the per-recipe
//! directory migration preserved all recipes byte-for-byte.

use std::path::PathBuf;

use mog::testkit::{find_mogs, run_mog_test};

/// `mog-rs/factory`, resolved from the crate manifest dir.
fn factory_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("factory")
}

#[test]
fn every_factory_mog_matches_its_golden() {
    let root = factory_root();
    let mogs = find_mogs(&root);
    assert!(
        !mogs.is_empty(),
        "no factory recipes found under '{}'",
        root.display()
    );

    // Collect every failure so a single run reports all broken recipes rather
    // than stopping at the first.
    let mut failures: Vec<String> = Vec::new();
    for mog in &mogs {
        let outcome = run_mog_test(mog, None, None);
        if !outcome.pass {
            failures.push(format!(
                "{}: {}\n{}",
                mog.display(),
                outcome.message.as_deref().unwrap_or("mismatch"),
                outcome.diff.as_deref().unwrap_or("")
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} factory recipes failed their golden:\n\n{}",
        failures.len(),
        mogs.len(),
        failures.join("\n")
    );
}
