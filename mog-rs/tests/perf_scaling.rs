//! Perf-regression gate: catch the O(n^2) class of bugs that the criterion
//! benchmarks (benches/engine.rs) first surfaced -- a whole-file regex `replace_all`
//! or a fancy-regex `(?m)$` scan that turns linear work quadratic on large input.
//!
//! Strategy: run a guarded action on two input sizes (N and 4N) and assert the
//! larger run is not dramatically super-linear. Linear work scales ~4x; an O(n^2)
//! regression scales ~16x. The threshold (8x) sits well between the two, so this is
//! robust to CI timing jitter (min-of-three per size) while still failing loudly on
//! a real regression. It is a coarse gate, not a micro-benchmark -- for precise
//! numbers use `cargo bench`.

use std::time::{Duration, Instant};

use mog::{execute, parse_mog};

/// Deterministic corpus of `lines` lines, each with leading/trailing whitespace
/// and a `word = value` field so trim and regex actions both have real work.
fn corpus(lines: usize) -> String {
    let mut s = String::with_capacity(lines * 24);
    for i in 0..lines {
        s.push_str("   Key_");
        s.push_str(&(i % 997).to_string());
        s.push_str(" = value here  \n");
    }
    s
}

/// Fastest of three runs (min is robust to transient CI slowdowns).
fn best_of_three(mog_json: &str, input: &str) -> Duration {
    let mog = parse_mog(mog_json).unwrap();
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let t = Instant::now();
        let out = execute(&mog, input).unwrap();
        std::hint::black_box(&out);
        best = best.min(t.elapsed());
    }
    best
}

/// Assert the action's runtime grows roughly linearly (not quadratically) with a
/// 4x input-size increase.
fn assert_subquadratic(name: &str, mog_json: &str) {
    let small = corpus(200_000);
    let large = corpus(800_000); // 4x the lines
                                 // Warm caches / branch predictors before timing.
    let _ = best_of_three(mog_json, &small);
    let ts = best_of_three(mog_json, &small);
    let tl = best_of_three(mog_json, &large);
    let ratio = tl.as_secs_f64() / ts.as_secs_f64().max(1e-6);
    assert!(
        ratio < 8.0,
        "{name}: 4x input took {ratio:.1}x the time (linear should be ~4x; an O(n^2) \
         regression is ~16x). small={ts:?} large={tl:?}"
    );
}

#[test]
fn trim_whitespace_scales_linearly() {
    // Regression guard: trim once ran a fancy-regex `(?m)$` scan (O(n^2)).
    assert_subquadratic(
        "trim_whitespace",
        r#"{"steps":[{"action":"trim_whitespace"}]}"#,
    );
}

#[test]
fn replace_regex_scales_linearly() {
    // Regression guard: replace_regex once used a whole-file fancy-regex
    // `replace_all` (O(n^2)); an ordinary pattern must route through the linear
    // `regex` engine.
    assert_subquadratic(
        "replace_regex",
        r#"{"steps":[{"action":"replace_regex","options":{"find":"value","replace_with":"VALUE"}}]}"#,
    );
}
