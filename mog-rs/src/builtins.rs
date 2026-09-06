//! The built-in `@`-namespaced placeholders ({{@today}}, {{@now}}, {{@year}},
//! {{@epoch}}, {{@date}}, {{@uuid}}) and the determinism pins that fix them. The
//! engine itself never reads the clock or RNG; these values are injected as
//! constant overrides before any action runs, so `--pin-now` / `--pin-seed` make a
//! run reproducible. `mog --test` pins to the fixed [`TEST_NOW`] / [`TEST_SEED`] by
//! default so a mog using these placeholders can still have a stable golden.

use std::collections::BTreeMap;

use anyhow::Result;

/// The fixed clock `mog --test` pins by default (overridable with `--pin-now`).
/// This is Go's reference time; keep it STABLE -- changing it rewrites the goldens
/// of every mog that reads {{@now}}/{{@today}}/{{@year}}/{{@epoch}}/{{@date}}.
pub const TEST_NOW: &str = "2006-01-02T15:04:05";
/// The fixed RNG seed `mog --test` pins by default (overridable with `--pin-seed`).
pub const TEST_SEED: u64 = 0;

/// Inject the built-in `@` placeholders into `defines` (a user `--define @x=...`
/// wins). `now` pins the clock (epoch or ISO date/datetime); `seed` pins the uuid.
pub fn inject_builtins(
    defines: &mut BTreeMap<String, String>,
    now: Option<&str>,
    seed: Option<u64>,
) -> Result<()> {
    let epoch = match now {
        Some(s) => crate::datetime::parse_now(s)?,
        None => crate::datetime::now_epoch(),
    };
    let date = crate::datetime::format_date(epoch);
    let builtins = [
        ("@epoch", epoch.to_string()),
        ("@today", date.clone()),
        ("@date", date),
        ("@now", crate::datetime::format_datetime(epoch)),
        ("@year", crate::datetime::year(epoch).to_string()),
        ("@uuid", make_uuid(seed)),
        ("@random", make_random(seed)),
    ];
    for (k, v) in builtins {
        defines.entry(k.to_string()).or_insert(v);
    }
    Ok(())
}

/// A random (or seeded) 128-bit token as 32 lowercase hex characters, for the
/// `{{@random}}` placeholder. Seeded via `--pin-seed` for a reproducible run; the
/// seed is offset so `@random` differs from `@uuid` under the same pin.
pub fn make_random(seed: Option<u64>) -> String {
    use rand::{RngCore, SeedableRng};
    let mut bytes = [0u8; 16];
    match seed {
        Some(s) => {
            let mut rng = rand::rngs::StdRng::seed_from_u64(s.wrapping_add(0x9E37_79B9_7F4A_7C15));
            rng.fill_bytes(&mut bytes);
        }
        None => rand::thread_rng().fill_bytes(&mut bytes),
    }
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A random (or seeded) UUID v4, formatted 8-4-4-4-12. Uses `rand` (already a
/// dependency); no uuid crate needed.
pub fn make_uuid(seed: Option<u64>) -> String {
    use rand::SeedableRng;
    match seed {
        Some(s) => uuid_from_rng(&mut rand::rngs::StdRng::seed_from_u64(s)),
        None => uuid_from_rng(&mut rand::thread_rng()),
    }
}

/// Draw one UUID v4 (8-4-4-4-12, lowercase hex, version 4 / variant 1) from `rng`,
/// consuming exactly 16 bytes. This is the SINGLE uuid formatter shared by the
/// `{{@uuid}}` placeholder ([`make_uuid`]) and the per-occurrence `$uuid`
/// replacement token, so both emit byte-identical shapes and a seeded stream's
/// first draw equals `make_uuid(Some(seed))`.
pub fn uuid_from_rng<R: rand::RngCore>(rng: &mut R) -> String {
    let mut bytes = [0u8; 16];
    rng.fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 0x0F) | 0x40; // version 4
    bytes[8] = (bytes[8] & 0x3F) | 0x80; // variant 1
    let mut out = String::with_capacity(36);
    for (i, b) in bytes.iter().enumerate() {
        if matches!(i, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        out.push_str(&format!("{b:02x}"));
    }
    out
}

/// The RNG backing the per-occurrence `$uuid` replacement token: a reproducible
/// `StdRng` stream when a run is seed-pinned (`--pin-seed` / `mog --test`), else a
/// fresh entropy-seeded stream (random each run). Callers draw one uuid per regex
/// match via [`uuid_from_rng`], advancing the stream so each match differs; under a
/// pin the sequence is identical run-to-run (stable factory goldens).
pub fn uuid_stream(seed: Option<u64>) -> rand::rngs::StdRng {
    use rand::SeedableRng;
    match seed {
        Some(s) => rand::rngs::StdRng::seed_from_u64(s),
        None => rand::rngs::StdRng::from_entropy(),
    }
}
