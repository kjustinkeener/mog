//! Focused tests for shipped factory recipes, for cases their golden fixtures
//! can't cover. The `factory_golden` suite already runs every recipe's
//! input -> expected golden; this file adds assertions that need to avoid a byte
//! in the committed fixture.

use std::path::PathBuf;

use mog::{execute, load_mog_file};

/// paste-cleanup maps the em dash (U+2014) to `--`. That codepoint is kept OUT of
/// the committed TestInput fixture because this repo's git hook blocks the raw
/// U+2014 byte; assert it here via a Rust `\u{}` escape (ASCII source, no raw
/// byte). The rest of the mappings are covered by the golden fixture.
#[test]
fn paste_cleanup_converts_em_dash() {
    let mog_doc =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("factory/paste-cleanup/paste-cleanup.mog");
    let mog = load_mog_file(&mog_doc).expect("recipe loads");
    let input = "a\u{2014}b, then c\u{2014}\u{2014}d";
    let out = execute(&mog, input).expect("recipe runs");
    assert_eq!(out, "a--b, then c----d");
}

/// emdash-cleanup's golden fixture uses U+2015 (horizontal bar) because this
/// repo's git hook blocks the raw U+2014 byte in committed files. Cover the em
/// dash itself here via a Rust `\u{}` escape.
#[test]
fn emdash_cleanup_converts_em_dash() {
    let mog_doc =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("factory/emdash-cleanup/emdash-cleanup.mog");
    let mog = load_mog_file(&mog_doc).expect("recipe loads");
    let input = "a\u{2014}b\n\u{2014} lead\ntrail \u{2014}\nrun a \u{2014}\u{2014} b";
    let out = execute(&mog, input).expect("recipe runs");
    assert_eq!(out, "a - b\n- lead\ntrail\nrun a - b");
}
