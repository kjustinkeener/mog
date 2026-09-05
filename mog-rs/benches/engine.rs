//! Phase 4 baseline benchmarks.
//!
//! Establishes a perf baseline for the current whole-file engine path (each file
//! loads into one `String`; each action rewrites the whole text) BEFORE any
//! streaming work. The point is to measure where scale actually hurts, and to
//! give the streaming path a byte-for-byte throughput target to beat (or at
//! least not regress) on typical files.
//!
//! Run with `cargo bench`. Four representative actions across two file sizes:
//!   - to_upper        line-local, trivial (case map)
//!   - trim_whitespace line-local (per-line edit)
//!   - replace_regex   line-local but regex-bound (the realistic hot path)
//!   - sort_lines      whole-file / blocking (cannot ever stream; the ceiling)

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mog::{execute, parse_mog, Mog};

/// Deterministic synthetic corpus of roughly `target` bytes. Each line carries
/// mixed case, leading/trailing whitespace, and a `word=digits` field so every
/// benchmarked action has real work to do. A cheap LCG scrambles a counter into
/// the line's sort key so the corpus is NOT pre-sorted (sort_lines must actually
/// move data). Deterministic so runs are comparable across machines and commits.
fn synth(target: usize) -> String {
    let mut s = String::with_capacity(target + 64);
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut i: u64 = 0;
    while s.len() < target {
        // LCG (Knuth MMIX constants); high bits are the well-scrambled ones.
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let key = state >> 33;
        // `count={i}` is the only `\w+=\d+` match per line (the `=` around
        // Value_ has spaces, so it does not match).
        s.push_str(&format!("  Field_{key} = Value_{i} count={i}  \n"));
        i += 1;
    }
    s
}

fn pipeline(json: &str) -> Mog {
    parse_mog(json).expect("bench pipeline parses")
}

fn bench_actions(c: &mut Criterion) {
    let cases: &[(&str, &str)] = &[
        ("to_upper", r#"{"steps":[{"action":"to_upper"}]}"#),
        (
            "trim_whitespace",
            r#"{"steps":[{"action":"trim_whitespace"}]}"#,
        ),
        (
            "replace_regex",
            r#"{"steps":[{"action":"replace_regex","options":{"find":"(\\w+)=(\\d+)","replace_with":"$2:$1"}}]}"#,
        ),
        ("sort_lines", r#"{"steps":[{"action":"sort_lines"}]}"#),
    ];
    let sizes: &[(&str, usize)] = &[("1MiB", 1 << 20), ("8MiB", 8 << 20)];

    for (size_label, size) in sizes {
        let input = synth(*size);
        let mut group = c.benchmark_group(format!("engine/{size_label}"));
        // Report MB/s so a streaming rewrite can be compared apples-to-apples.
        group.throughput(Throughput::Bytes(input.len() as u64));
        for (name, json) in cases {
            let mog = pipeline(json);
            group.bench_with_input(BenchmarkId::from_parameter(name), &input, |b, input| {
                b.iter(|| execute(&mog, input).expect("bench run succeeds"));
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_actions);
criterion_main!(benches);
