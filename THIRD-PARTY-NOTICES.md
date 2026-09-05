# Third-party notices

Mog and Mog Studio statically link the Rust crates listed below into the shipped
executables, and the Studio download additionally embeds a complete copy of the
`mog` engine binary. That is redistribution, so those crates' licences travel
with the binary rather than only with the source.

Mog's own code is MIT licensed; see [LICENSE](LICENSE).

This file is generated from `cargo metadata`. Regenerate it whenever dependencies
change.

## Fonts

No fonts are redistributed. Both the engine and Studio use system font stacks,
resolved on the user's machine at render time. Fonts read from the system are not
redistributed and carry no notice obligation here.

## Runtime components not bundled

Studio renders through the Microsoft Edge WebView2 runtime, which is a component of
Windows supplied by Microsoft. It is used, not redistributed, and is governed by
Microsoft's own terms.

## Obligations worth knowing

Everything below is permissive and compatible with shipping a closed binary; nothing
here requires Mog's own source to be released under another licence. Two entries do
carry a condition beyond attribution:

- **MPL-2.0** (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`,
  `selectors`, all pulled in by Studio's webview stack). File-level copyleft: linking
  is unrestricted, but if any MPL-licensed *file* is modified, the modified file's
  source must be made available. Mog modifies none of them and consumes them as
  published crates, so the obligation is satisfied by this notice.
- **Unicode-3.0** (18 crates, all `icu_*` / Unicode character data). Requires the
  Unicode licence text and copyright notice to travel with the data, which this file
  and the crates' own bundled licence files do.

`webpki-roots` is CDLA-Permissive-2.0, which is attribution-only for data.

## Licence summary

| Licence | Crates |
|---|---|
| MIT OR Apache-2.0 | 210 |
| MIT | 114 |
| Apache-2.0 OR MIT | 47 |
| MIT/Apache-2.0 | 21 |
| Unicode-3.0 | 18 |
| Zlib OR Apache-2.0 OR MIT | 17 |
| Unlicense OR MIT | 9 |
| Apache-2.0 | 8 |
| Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 5 |
| BSD-3-Clause | 5 |
| MPL-2.0 | 5 |
| Unlicense/MIT | 4 |
| Apache-2.0/MIT | 3 |
| ISC | 3 |
| Apache-2.0 WITH LLVM-exception | 2 |
| BSD-2-Clause OR Apache-2.0 OR MIT | 2 |
| BSD-3-Clause OR MIT OR Apache-2.0 | 2 |
| MIT OR Apache-2.0 OR Zlib | 2 |
| (Apache-2.0 OR MIT) AND BSD-3-Clause | 1 |
| (MIT OR Apache-2.0) AND Unicode-3.0 | 1 |
| 0BSD OR MIT OR Apache-2.0 | 1 |
| Apache-2.0 / MIT | 1 |
| Apache-2.0 AND ISC | 1 |
| Apache-2.0 AND MIT | 1 |
| Apache-2.0 OR BSL-1.0 | 1 |
| Apache-2.0 OR ISC OR MIT | 1 |
| BSD-3-Clause AND MIT | 1 |
| BSD-3-Clause/MIT | 1 |
| CC0-1.0 OR MIT-0 OR Apache-2.0 | 1 |
| CDLA-Permissive-2.0 | 1 |
| MIT OR Apache-2.0 OR BSD-1-Clause | 1 |
| MIT OR Apache-2.0 OR LGPL-2.1-or-later | 1 |
| MIT OR Zlib OR Apache-2.0 | 1 |
| Zlib | 1 |

493 distinct third-party crates in total.

## Crates

| Crate | Version | Licence | Used by |
|---|---|---|---|
| `adler2` | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | Studio, engine (mog) |
| `aho-corasick` | 1.1.5 | Unlicense OR MIT | Studio, engine (mog), regex module (wasm) |
| `alloc-no-stdlib` | 2.0.4 | BSD-3-Clause | Studio |
| `alloc-stdlib` | 0.2.4 | BSD-3-Clause | Studio |
| `android_log-sys` | 0.3.2 | MIT OR Apache-2.0 | Studio |
| `android_logger` | 0.15.1 | MIT OR Apache-2.0 | Studio |
| `android_system_properties` | 0.1.6 | MIT OR Apache-2.0 | Studio |
| `anes` | 0.1.6 | MIT OR Apache-2.0 | engine (mog) |
| `anstream` | 1.0.0 | MIT OR Apache-2.0 | engine (mog) |
| `anstyle` | 1.0.14 | MIT OR Apache-2.0 | engine (mog) |
| `anstyle-parse` | 1.0.0 | MIT OR Apache-2.0 | engine (mog) |
| `anstyle-query` | 1.1.5 | MIT OR Apache-2.0 | engine (mog) |
| `anstyle-wincon` | 3.0.11 | MIT OR Apache-2.0 | engine (mog) |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `ar_archive_writer` | 0.5.3 | Apache-2.0 WITH LLVM-exception | engine (mog) |
| `assert_cmd` | 2.2.2 | MIT OR Apache-2.0 | engine (mog) |
| `atk` | 0.18.2 | MIT | Studio |
| `atk-sys` | 0.18.2 | MIT | Studio |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | Studio |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `base64` | 0.22.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `base64ct` | 1.8.3 | Apache-2.0 OR MIT | engine (mog) |
| `bit-set` | 0.8.0 | Apache-2.0 OR MIT | Studio, engine (mog), regex module (wasm) |
| `bit-vec` | 0.8.0 | Apache-2.0 OR MIT | Studio, engine (mog), regex module (wasm) |
| `bitflags` | 2.13.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `block2` | 0.6.2 | MIT | Studio |
| `brotli` | 8.0.4 | BSD-3-Clause AND MIT | Studio |
| `brotli-decompressor` | 5.0.3 | BSD-3-Clause/MIT | Studio |
| `bs58` | 0.5.1 | MIT/Apache-2.0 | Studio |
| `bstr` | 1.13.1 | MIT OR Apache-2.0 | engine (mog) |
| `bumpalo` | 3.20.3 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `bytemuck` | 1.25.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `byteorder` | 1.5.0 | Unlicense OR MIT | Studio |
| `bytes` | 1.12.1 | MIT | Studio |
| `cairo-rs` | 0.18.5 | MIT | Studio |
| `cairo-sys-rs` | 0.18.2 | MIT | Studio |
| `camino` | 1.2.5 | MIT OR Apache-2.0 | Studio |
| `cargo-platform` | 0.1.9 | MIT OR Apache-2.0 | Studio |
| `cargo_metadata` | 0.19.2 | MIT | Studio |
| `cargo_toml` | 0.22.3 | Apache-2.0 OR MIT | Studio |
| `cast` | 0.3.0 | MIT OR Apache-2.0 | engine (mog) |
| `cc` | 1.4.4 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `cesu8` | 1.1.0 | Apache-2.0/MIT | Studio |
| `cfb` | 0.7.3 | MIT | Studio |
| `cfg-expr` | 0.15.8 | MIT OR Apache-2.0 | Studio |
| `cfg-if` | 1.0.4 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | Studio |
| `ciborium` | 0.2.2 | Apache-2.0 | engine (mog) |
| `ciborium-io` | 0.2.2 | Apache-2.0 | engine (mog) |
| `ciborium-ll` | 0.2.2 | Apache-2.0 | engine (mog) |
| `clap` | 4.6.6 | MIT OR Apache-2.0 | engine (mog) |
| `clap_builder` | 4.6.6 | MIT OR Apache-2.0 | engine (mog) |
| `clap_derive` | 4.6.4 | MIT OR Apache-2.0 | engine (mog) |
| `clap_lex` | 1.1.0 | MIT OR Apache-2.0 | engine (mog) |
| `colorchoice` | 1.0.5 | MIT OR Apache-2.0 | engine (mog) |
| `combine` | 4.6.7 | MIT | Studio |
| `const-oid` | 0.9.6 | Apache-2.0 OR MIT | engine (mog) |
| `cookie` | 0.18.2 | MIT OR Apache-2.0 | Studio |
| `core-foundation` | 0.10.1 | MIT OR Apache-2.0 | Studio |
| `core-foundation-sys` | 0.8.7 | MIT OR Apache-2.0 | Studio |
| `core-graphics` | 0.25.0 | MIT OR Apache-2.0 | Studio |
| `core-graphics-types` | 0.2.0 | MIT OR Apache-2.0 | Studio |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `crc32fast` | 1.5.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `criterion` | 0.5.1 | Apache-2.0 OR MIT | engine (mog) |
| `criterion-plot` | 0.5.0 | MIT/Apache-2.0 | engine (mog) |
| `crossbeam-channel` | 0.5.16 | MIT OR Apache-2.0 | Studio |
| `crossbeam-deque` | 0.8.7 | MIT OR Apache-2.0 | engine (mog) |
| `crossbeam-epoch` | 0.9.20 | MIT OR Apache-2.0 | engine (mog) |
| `crossbeam-utils` | 0.8.22 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `crunchy` | 0.2.4 | MIT | engine (mog) |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `cssparser` | 0.36.0 | MPL-2.0 | Studio |
| `cssparser-macros` | 0.6.1 | MPL-2.0 | Studio |
| `csv` | 1.4.0 | Unlicense/MIT | engine (mog) |
| `csv-core` | 0.1.13 | Unlicense/MIT | engine (mog) |
| `ctor` | 0.8.0 | Apache-2.0 OR MIT | Studio |
| `ctor-proc-macro` | 0.0.7 | Apache-2.0 OR MIT | Studio |
| `curve25519-dalek` | 4.1.3 | BSD-3-Clause | engine (mog) |
| `curve25519-dalek-derive` | 0.1.1 | MIT/Apache-2.0 | engine (mog) |
| `darling` | 0.23.0 | MIT | Studio |
| `darling_core` | 0.23.0 | MIT | Studio |
| `darling_macro` | 0.23.0 | MIT | Studio |
| `dbus` | 0.9.12 | Apache-2.0/MIT | Studio |
| `defmt` | 1.1.1 | MIT OR Apache-2.0 | Studio |
| `defmt-macros` | 1.1.1 | MIT OR Apache-2.0 | Studio |
| `defmt-parser` | 1.0.0 | MIT OR Apache-2.0 | Studio |
| `der` | 0.7.10 | Apache-2.0 OR MIT | engine (mog) |
| `deranged` | 0.5.8 | MIT OR Apache-2.0 | Studio |
| `derive_more` | 2.1.1 | MIT | Studio |
| `derive_more-impl` | 2.1.1 | MIT | Studio |
| `difflib` | 0.4.0 | MIT | engine (mog) |
| `digest` | 0.10.7 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `dirs` | 6.0.0 | MIT OR Apache-2.0 | Studio |
| `dirs-sys` | 0.5.0 | MIT OR Apache-2.0 | Studio |
| `dispatch2` | 0.3.1 | Zlib OR Apache-2.0 OR MIT | Studio |
| `displaydoc` | 0.2.7 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `dlopen2` | 0.8.2 | MIT | Studio |
| `dlopen2_derive` | 0.4.3 | MIT | Studio |
| `dom_query` | 0.27.0 | MIT | Studio |
| `dpi` | 0.1.2 | Apache-2.0 AND MIT | Studio |
| `dtoa` | 1.0.11 | MIT OR Apache-2.0 | Studio |
| `dtoa-short` | 0.3.5 | MPL-2.0 | Studio |
| `dtor` | 0.3.0 | Apache-2.0 OR MIT | Studio |
| `dtor-proc-macro` | 0.0.6 | Apache-2.0 OR MIT | Studio |
| `dunce` | 1.0.5 | CC0-1.0 OR MIT-0 OR Apache-2.0 | Studio |
| `dyn-clone` | 1.0.20 | MIT OR Apache-2.0 | Studio |
| `ed25519` | 2.2.3 | Apache-2.0 OR MIT | engine (mog) |
| `ed25519-dalek` | 2.2.0 | BSD-3-Clause | engine (mog) |
| `either` | 1.17.0 | MIT OR Apache-2.0 | engine (mog) |
| `embed-resource` | 3.0.11 | MIT | Studio |
| `embed_plist` | 1.2.2 | MIT OR Apache-2.0 | Studio |
| `encoding_rs` | 0.8.35 | (Apache-2.0 OR MIT) AND BSD-3-Clause | engine (mog) |
| `env_filter` | 0.1.4 | MIT OR Apache-2.0 | Studio |
| `equivalent` | 1.0.2 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `erased-serde` | 0.4.10 | MIT OR Apache-2.0 | Studio |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | engine (mog) |
| `fancy-regex` | 0.14.0 | MIT | engine (mog), regex module (wasm) |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | Studio |
| `fern` | 0.7.1 | MIT | Studio |
| `fiat-crypto` | 0.2.9 | MIT OR Apache-2.0 OR BSD-1-Clause | engine (mog) |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 | Studio |
| `find-msvc-tools` | 0.1.11 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `flate2` | 1.1.9 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `float-cmp` | 0.10.0 | MIT | engine (mog) |
| `fnv` | 1.0.7 | Apache-2.0 / MIT | Studio |
| `foldhash` | 0.2.0 | Zlib | Studio |
| `foreign-types` | 0.5.0 | MIT/Apache-2.0 | Studio |
| `foreign-types-macros` | 0.2.4 | MIT/Apache-2.0 | Studio |
| `foreign-types-shared` | 0.3.1 | MIT/Apache-2.0 | Studio |
| `form_urlencoded` | 1.2.2 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `gdk` | 0.18.2 | MIT | Studio |
| `gdk-pixbuf` | 0.18.5 | MIT | Studio |
| `gdk-pixbuf-sys` | 0.18.0 | MIT | Studio |
| `gdk-sys` | 0.18.2 | MIT | Studio |
| `gdkwayland-sys` | 0.18.2 | MIT | Studio |
| `gdkx11` | 0.18.2 | MIT | Studio |
| `gdkx11-sys` | 0.18.2 | MIT | Studio |
| `generic-array` | 0.14.7 | MIT | Studio, engine (mog) |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `gio` | 0.18.4 | MIT | Studio |
| `gio-sys` | 0.18.1 | MIT | Studio |
| `glib` | 0.18.5 | MIT | Studio |
| `glib-macros` | 0.18.5 | MIT | Studio |
| `glib-sys` | 0.18.1 | MIT | Studio |
| `glob` | 0.3.4 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `gobject-sys` | 0.18.0 | MIT | Studio |
| `gtk` | 0.18.2 | MIT | Studio |
| `gtk-sys` | 0.18.2 | MIT | Studio |
| `gtk3-macros` | 0.18.2 | MIT | Studio |
| `half` | 2.7.1 | MIT OR Apache-2.0 | engine (mog) |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `hermit-abi` | 0.5.2 | MIT OR Apache-2.0 | engine (mog) |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `html5ever` | 0.38.0 | MIT OR Apache-2.0 | Studio |
| `http` | 1.5.0 | MIT OR Apache-2.0 | Studio |
| `http-body` | 1.1.0 | MIT | Studio |
| `http-body-util` | 0.1.5 | MIT | Studio |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | Studio |
| `hyper` | 1.11.0 | MIT | Studio |
| `hyper-util` | 0.1.20 | MIT | Studio |
| `iana-time-zone` | 0.1.65 | MIT OR Apache-2.0 | Studio |
| `iana-time-zone-haiku` | 0.1.2 | MIT OR Apache-2.0 | Studio |
| `ico` | 0.5.0 | MIT | Studio |
| `icu_collections` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_locale_core` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_normalizer` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_normalizer_data` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_properties` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_properties_data` | 2.3.0 | Unicode-3.0 | Studio, engine (mog) |
| `icu_provider` | 2.3.1 | Unicode-3.0 | Studio, engine (mog) |
| `ident_case` | 1.0.1 | MIT/Apache-2.0 | Studio |
| `idna` | 1.1.0 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `idna_adapter` | 1.2.2 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `include_dir` | 0.7.4 | MIT | engine (mog) |
| `include_dir_macros` | 0.7.4 | MIT | engine (mog) |
| `indexmap` | 2.14.0 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `infer` | 0.19.0 | MIT | Studio |
| `ipnet` | 2.12.1 | MIT OR Apache-2.0 | Studio |
| `is-terminal` | 0.4.17 | MIT | engine (mog) |
| `is_terminal_polyfill` | 1.70.2 | MIT OR Apache-2.0 | engine (mog) |
| `itertools` | 0.10.5 | MIT/Apache-2.0 | engine (mog) |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `javascriptcore-rs` | 1.1.2 | MIT | Studio |
| `javascriptcore-rs-sys` | 1.1.1 | MIT | Studio |
| `jiff` | 0.2.35 | Unlicense OR MIT | Studio |
| `jiff-core` | 0.1.0 | Unlicense OR MIT | Studio |
| `jiff-static` | 0.2.35 | Unlicense OR MIT | Studio |
| `jiff-tzdb` | 0.1.8 | Unlicense OR MIT | Studio |
| `jiff-tzdb-platform` | 0.1.3 | Unlicense OR MIT | Studio |
| `jni` | 0.21.1 | MIT/Apache-2.0 | Studio |
| `jni-sys` | 0.3.1 | MIT OR Apache-2.0 | Studio |
| `jni-sys-macros` | 0.4.1 | MIT OR Apache-2.0 | Studio |
| `js-sys` | 0.3.104 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `json-patch` | 3.0.1 | MIT/Apache-2.0 | Studio |
| `jsonptr` | 0.6.3 | MIT OR Apache-2.0 | Studio |
| `keyboard-types` | 0.7.0 | MIT OR Apache-2.0 | Studio |
| `libappindicator` | 0.9.0 | Apache-2.0 OR MIT | Studio |
| `libappindicator-sys` | 0.9.0 | Apache-2.0 OR MIT | Studio |
| `libc` | 0.2.189 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | Studio |
| `libloading` | 0.7.4 | ISC | Studio |
| `libredox` | 0.1.20 | MIT | Studio |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | engine (mog) |
| `litemap` | 0.8.3 | Unicode-3.0 | Studio, engine (mog) |
| `lock_api` | 0.4.14 | MIT OR Apache-2.0 | Studio |
| `log` | 0.4.34 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `markup5ever` | 0.38.0 | MIT OR Apache-2.0 | Studio |
| `memchr` | 2.8.3 | Unlicense OR MIT | Studio, engine (mog), regex module (wasm) |
| `memoffset` | 0.9.1 | MIT | Studio |
| `mime` | 0.3.17 | MIT OR Apache-2.0 | Studio |
| `miniz_oxide` | 0.8.9 | MIT OR Zlib OR Apache-2.0 | Studio, engine (mog) |
| `mio` | 1.2.2 | MIT | Studio |
| `muda` | 0.19.3 | Apache-2.0 OR MIT | Studio |
| `ndk` | 0.9.0 | MIT OR Apache-2.0 | Studio |
| `ndk-sys` | 0.6.0+11769913 | MIT OR Apache-2.0 | Studio |
| `new_debug_unreachable` | 1.0.6 | MIT | Studio |
| `normalize-line-endings` | 0.3.0 | Apache-2.0 | engine (mog) |
| `num-conv` | 0.2.2 | MIT OR Apache-2.0 | Studio |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `num_enum` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | Studio |
| `num_enum_derive` | 0.7.6 | BSD-3-Clause OR MIT OR Apache-2.0 | Studio |
| `num_threads` | 0.1.7 | MIT OR Apache-2.0 | Studio |
| `objc2` | 0.6.4 | MIT | Studio |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-cloud-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-data` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-foundation` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-graphics` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-image` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-location` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-core-text` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-encode` | 4.1.0 | MIT | Studio |
| `objc2-exception-helper` | 0.1.1 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-foundation` | 0.3.2 | MIT | Studio |
| `objc2-io-surface` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-quartz-core` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-ui-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-user-notifications` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `objc2-web-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | Studio |
| `object` | 0.39.1 | Apache-2.0 OR MIT | engine (mog) |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `once_cell_polyfill` | 1.70.2 | MIT OR Apache-2.0 | engine (mog) |
| `oorandom` | 11.1.5 | MIT | engine (mog) |
| `option-ext` | 0.2.0 | MPL-2.0 | Studio |
| `pango` | 0.18.3 | MIT | Studio |
| `pango-sys` | 0.18.0 | MIT | Studio |
| `parking_lot` | 0.12.5 | MIT OR Apache-2.0 | Studio |
| `parking_lot_core` | 0.9.12 | MIT OR Apache-2.0 | Studio |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `phf` | 0.13.1 | MIT | Studio |
| `phf_codegen` | 0.13.1 | MIT | Studio |
| `phf_generator` | 0.13.1 | MIT | Studio |
| `phf_macros` | 0.13.1 | MIT | Studio |
| `phf_shared` | 0.13.1 | MIT | Studio |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `pkcs8` | 0.10.2 | Apache-2.0 OR MIT | engine (mog) |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | Studio |
| `plist` | 1.10.0 | MIT | Studio |
| `plotters` | 0.3.7 | MIT | engine (mog) |
| `plotters-backend` | 0.3.7 | MIT | engine (mog) |
| `plotters-svg` | 0.3.7 | MIT | engine (mog) |
| `png` | 0.17.16 | MIT OR Apache-2.0 | Studio |
| `polyglot-sql` | 0.9.2 | MIT | engine (mog) |
| `polyglot-sql-ast-derive` | 0.9.2 | MIT | engine (mog) |
| `portable-atomic` | 1.15.0 | Apache-2.0 OR MIT | Studio |
| `portable-atomic-util` | 0.2.7 | Apache-2.0 OR MIT | Studio |
| `potential_utf` | 0.1.6 | Unicode-3.0 | Studio, engine (mog) |
| `powerfmt` | 0.2.0 | MIT OR Apache-2.0 | Studio |
| `ppv-lite86` | 0.2.21 | MIT OR Apache-2.0 | engine (mog) |
| `precomputed-hash` | 0.1.1 | MIT | Studio |
| `predicates` | 3.1.4 | MIT OR Apache-2.0 | engine (mog) |
| `predicates-core` | 1.0.10 | MIT OR Apache-2.0 | engine (mog) |
| `predicates-tree` | 1.0.13 | MIT OR Apache-2.0 | engine (mog) |
| `proc-macro-crate` | 1.3.1 | MIT OR Apache-2.0 | Studio |
| `proc-macro-error` | 1.0.4 | MIT OR Apache-2.0 | Studio |
| `proc-macro-error-attr` | 1.0.4 | MIT OR Apache-2.0 | Studio |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `psm` | 0.1.32 | MIT OR Apache-2.0 | engine (mog) |
| `quick-xml` | 0.41.0 | MIT | Studio |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | Studio, engine (mog) |
| `rand` | 0.8.7 | MIT OR Apache-2.0 | engine (mog) |
| `rand_chacha` | 0.3.1 | MIT OR Apache-2.0 | engine (mog) |
| `rand_core` | 0.6.4 | MIT OR Apache-2.0 | engine (mog) |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | Studio |
| `rayon` | 1.12.0 | MIT OR Apache-2.0 | engine (mog) |
| `rayon-core` | 1.13.0 | MIT OR Apache-2.0 | engine (mog) |
| `redox_syscall` | 0.5.18 | MIT | Studio |
| `redox_users` | 0.5.2 | MIT | Studio |
| `ref-cast` | 1.0.27 | MIT OR Apache-2.0 | Studio |
| `ref-cast-impl` | 1.0.27 | MIT OR Apache-2.0 | Studio |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `regex-automata` | 0.4.18 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `regex-syntax` | 0.8.11 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `reqwest` | 0.13.4 | MIT OR Apache-2.0 | Studio |
| `rfd` | 0.16.0 | MIT | Studio |
| `ring` | 0.17.14 | Apache-2.0 AND ISC | engine (mog) |
| `roxmltree` | 0.20.0 | MIT OR Apache-2.0 | engine (mog) |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | Studio |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `rustix` | 1.1.4 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | engine (mog) |
| `rustls` | 0.23.43 | Apache-2.0 OR ISC OR MIT | engine (mog) |
| `rustls-pki-types` | 1.15.1 | MIT OR Apache-2.0 | engine (mog) |
| `rustls-webpki` | 0.103.15 | ISC | engine (mog) |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `ryu` | 1.0.23 | Apache-2.0 OR BSL-1.0 | engine (mog) |
| `same-file` | 1.0.6 | Unlicense/MIT | Studio, engine (mog) |
| `schemars` | 0.8.22 | MIT | Studio |
| `schemars_derive` | 0.8.22 | MIT | Studio |
| `scopeguard` | 1.2.0 | MIT OR Apache-2.0 | Studio |
| `selectors` | 0.36.1 | MPL-2.0 | Studio |
| `self-replace` | 1.5.0 | Apache-2.0 | engine (mog) |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `serde-untagged` | 0.1.9 | MIT OR Apache-2.0 | Studio |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `serde_derive_internals` | 0.29.1 | MIT OR Apache-2.0 | Studio |
| `serde_json` | 1.0.151 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | Studio |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `serde_with` | 3.22.0 | MIT OR Apache-2.0 | Studio |
| `serde_with_macros` | 3.22.0 | MIT OR Apache-2.0 | Studio |
| `serde_yaml` | 0.9.34+deprecated | MIT OR Apache-2.0 | engine (mog) |
| `serialize-to-javascript` | 0.1.2 | MIT OR Apache-2.0 | Studio |
| `serialize-to-javascript-impl` | 0.1.2 | MIT OR Apache-2.0 | Studio |
| `servo_arc` | 0.4.3 | MIT OR Apache-2.0 | Studio |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `signature` | 2.2.0 | Apache-2.0 OR MIT | engine (mog) |
| `simd-adler32` | 0.3.10 | MIT | Studio, engine (mog) |
| `similar` | 2.7.0 | Apache-2.0 | engine (mog) |
| `siphasher` | 1.0.3 | MIT/Apache-2.0 | Studio |
| `slab` | 0.4.12 | MIT | Studio, engine (mog) |
| `smallvec` | 1.15.2 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | Studio |
| `softbuffer` | 0.4.8 | MIT OR Apache-2.0 | Studio |
| `soup3` | 0.5.0 | MIT | Studio |
| `soup3-sys` | 0.5.0 | MIT | Studio |
| `spki` | 0.7.3 | Apache-2.0 OR MIT | engine (mog) |
| `stable_deref_trait` | 1.2.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `stacker` | 0.1.25 | MIT OR Apache-2.0 | engine (mog) |
| `string_cache` | 0.9.0 | MIT OR Apache-2.0 | Studio |
| `string_cache_codegen` | 0.6.1 | MIT OR Apache-2.0 | Studio |
| `strsim` | 0.11.1 | MIT | Studio, engine (mog) |
| `subtle` | 2.6.1 | BSD-3-Clause | engine (mog) |
| `swift-rs` | 1.0.8 | MIT OR Apache-2.0 | Studio |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `sync_wrapper` | 1.0.2 | Apache-2.0 | Studio |
| `synstructure` | 0.13.2 | MIT | Studio, engine (mog) |
| `system-deps` | 6.2.2 | MIT OR Apache-2.0 | Studio |
| `tao` | 0.35.3 | Apache-2.0 | Studio |
| `tao-macros` | 0.1.4 | MIT OR Apache-2.0 | Studio |
| `target-lexicon` | 0.12.16 | Apache-2.0 WITH LLVM-exception | Studio |
| `tauri` | 2.11.5 | Apache-2.0 OR MIT | Studio |
| `tauri-build` | 2.6.3 | Apache-2.0 OR MIT | Studio |
| `tauri-codegen` | 2.6.3 | Apache-2.0 OR MIT | Studio |
| `tauri-macros` | 2.6.3 | Apache-2.0 OR MIT | Studio |
| `tauri-plugin` | 2.6.3 | Apache-2.0 OR MIT | Studio |
| `tauri-plugin-dialog` | 2.7.2 | Apache-2.0 OR MIT | Studio |
| `tauri-plugin-fs` | 2.5.1 | Apache-2.0 OR MIT | Studio |
| `tauri-plugin-log` | 2.9.0 | Apache-2.0 OR MIT | Studio |
| `tauri-runtime` | 2.11.3 | Apache-2.0 OR MIT | Studio |
| `tauri-runtime-wry` | 2.11.4 | Apache-2.0 OR MIT | Studio |
| `tauri-utils` | 2.9.3 | Apache-2.0 OR MIT | Studio |
| `tauri-winres` | 0.3.6 | MIT | Studio |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | engine (mog) |
| `tendril` | 0.5.1 | MIT OR Apache-2.0 | Studio |
| `termtree` | 0.5.1 | MIT | engine (mog) |
| `thiserror` | 1.0.69 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `thiserror-impl` | 1.0.69 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `time` | 0.3.55 | MIT OR Apache-2.0 | Studio |
| `time-core` | 0.1.9 | MIT OR Apache-2.0 | Studio |
| `time-macros` | 0.2.32 | MIT OR Apache-2.0 | Studio |
| `tinystr` | 0.8.4 | Unicode-3.0 | Studio, engine (mog) |
| `tinytemplate` | 1.2.1 | Apache-2.0 OR MIT | engine (mog) |
| `tinyvec` | 1.12.0 | Zlib OR Apache-2.0 OR MIT | Studio |
| `tinyvec_macros` | 0.1.1 | MIT OR Apache-2.0 OR Zlib | Studio |
| `tokio` | 1.53.1 | MIT | Studio |
| `tokio-util` | 0.7.19 | MIT | Studio |
| `toml` | 0.8.23 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `toml_datetime` | 0.6.11 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `toml_edit` | 0.22.27 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `toml_parser` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `toml_write` | 0.1.2 | MIT OR Apache-2.0 | engine (mog) |
| `toml_writer` | 1.1.2+spec-1.1.0 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `tower` | 0.5.3 | MIT | Studio |
| `tower-http` | 0.6.11 | MIT | Studio |
| `tower-layer` | 0.3.3 | MIT | Studio |
| `tower-service` | 0.3.3 | MIT | Studio |
| `tracing` | 0.1.44 | MIT | Studio |
| `tracing-core` | 0.1.36 | MIT | Studio |
| `tray-icon` | 0.24.2 | MIT OR Apache-2.0 | Studio |
| `try-lock` | 0.2.5 | MIT | Studio |
| `typeid` | 1.0.3 | MIT OR Apache-2.0 | Studio |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `unic-char-property` | 0.9.0 | MIT/Apache-2.0 | Studio |
| `unic-char-range` | 0.9.0 | MIT/Apache-2.0 | Studio |
| `unic-common` | 0.9.0 | MIT/Apache-2.0 | Studio |
| `unic-ucd-ident` | 0.9.0 | MIT/Apache-2.0 | Studio |
| `unic-ucd-version` | 0.9.0 | MIT/Apache-2.0 | Studio |
| `unicode-ident` | 1.0.24 | (MIT OR Apache-2.0) AND Unicode-3.0 | Studio, engine (mog), regex module (wasm) |
| `unicode-segmentation` | 1.13.3 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `unsafe-libyaml` | 0.2.11 | MIT | engine (mog) |
| `untrusted` | 0.9.0 | ISC | engine (mog) |
| `ureq` | 2.12.1 | MIT OR Apache-2.0 | engine (mog) |
| `url` | 2.5.8 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `urlencoding` | 2.1.3 | MIT | engine (mog) |
| `urlpattern` | 0.3.0 | MIT | Studio |
| `utf8_iter` | 1.0.4 | Apache-2.0 OR MIT | Studio, engine (mog) |
| `utf8parse` | 0.2.2 | Apache-2.0 OR MIT | engine (mog) |
| `uuid` | 1.24.1 | Apache-2.0 OR MIT | Studio |
| `version-compare` | 0.2.1 | MIT | Studio |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | Studio, engine (mog) |
| `vswhom` | 0.1.0 | MIT | Studio |
| `vswhom-sys` | 0.1.3 | MIT | Studio |
| `wait-timeout` | 0.2.1 | MIT/Apache-2.0 | engine (mog) |
| `walkdir` | 2.5.0 | Unlicense/MIT | Studio, engine (mog) |
| `want` | 0.3.1 | MIT | Studio |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Studio, engine (mog) |
| `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Studio |
| `wasm-bindgen` | 0.2.127 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `wasm-bindgen-futures` | 0.4.77 | MIT OR Apache-2.0 | Studio |
| `wasm-bindgen-macro` | 0.2.127 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `wasm-bindgen-macro-support` | 0.2.127 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `wasm-bindgen-shared` | 0.2.127 | MIT OR Apache-2.0 | Studio, engine (mog), regex module (wasm) |
| `wasm-streams` | 0.5.0 | MIT OR Apache-2.0 | Studio |
| `web-sys` | 0.3.104 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `web_atoms` | 0.2.6 | MIT OR Apache-2.0 | Studio |
| `webkit2gtk` | 2.0.2 | MIT | Studio |
| `webkit2gtk-sys` | 2.0.2 | MIT | Studio |
| `webpki-roots` | 0.26.11 | CDLA-Permissive-2.0 | engine (mog) |
| `webview2-com` | 0.38.2 | MIT | Studio |
| `webview2-com-macros` | 0.8.1 | MIT | Studio |
| `webview2-com-sys` | 0.38.2 | MIT | Studio |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | Studio |
| `winapi-i686-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | Studio |
| `winapi-util` | 0.1.11 | Unlicense OR MIT | Studio, engine (mog) |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | Studio |
| `window-vibrancy` | 0.6.0 | Apache-2.0 OR MIT | Studio |
| `windows` | 0.61.3 | MIT OR Apache-2.0 | Studio |
| `windows-collections` | 0.2.0 | MIT OR Apache-2.0 | Studio |
| `windows-core` | 0.61.2 | MIT OR Apache-2.0 | Studio |
| `windows-future` | 0.2.1 | MIT OR Apache-2.0 | Studio |
| `windows-implement` | 0.60.2 | MIT OR Apache-2.0 | Studio |
| `windows-interface` | 0.59.3 | MIT OR Apache-2.0 | Studio |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows-numerics` | 0.2.0 | MIT OR Apache-2.0 | Studio |
| `windows-result` | 0.3.4 | MIT OR Apache-2.0 | Studio |
| `windows-strings` | 0.4.2 | MIT OR Apache-2.0 | Studio |
| `windows-sys` | 0.48.0 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows-targets` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows-threading` | 0.1.0 | MIT OR Apache-2.0 | Studio |
| `windows-version` | 0.1.7 | MIT OR Apache-2.0 | Studio |
| `windows_aarch64_gnullvm` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_aarch64_msvc` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_i686_gnu` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_i686_msvc` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_x86_64_gnu` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_x86_64_gnullvm` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `windows_x86_64_msvc` | 0.48.5 | MIT OR Apache-2.0 | Studio, engine (mog) |
| `winnow` | 0.7.15 | MIT | Studio, engine (mog) |
| `winreg` | 0.52.0 | MIT | Studio, engine (mog) |
| `winresource` | 0.1.31 | MIT | engine (mog) |
| `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | Studio |
| `writeable` | 0.6.4 | Unicode-3.0 | Studio, engine (mog) |
| `wry` | 0.55.1 | Apache-2.0 OR MIT | Studio |
| `x11` | 2.21.0 | MIT | Studio |
| `x11-dl` | 2.21.0 | MIT | Studio |
| `yoke` | 0.8.3 | Unicode-3.0 | Studio, engine (mog) |
| `yoke-derive` | 0.8.2 | Unicode-3.0 | Studio, engine (mog) |
| `zerocopy` | 0.8.56 | BSD-2-Clause OR Apache-2.0 OR MIT | engine (mog) |
| `zerocopy-derive` | 0.8.56 | BSD-2-Clause OR Apache-2.0 OR MIT | engine (mog) |
| `zerofrom` | 0.1.8 | Unicode-3.0 | Studio, engine (mog) |
| `zerofrom-derive` | 0.1.7 | Unicode-3.0 | Studio, engine (mog) |
| `zeroize` | 1.9.0 | Apache-2.0 OR MIT | engine (mog) |
| `zerotrie` | 0.2.5 | Unicode-3.0 | Studio, engine (mog) |
| `zerovec` | 0.11.8 | Unicode-3.0 | Studio, engine (mog) |
| `zerovec-derive` | 0.11.6 | Unicode-3.0 | Studio, engine (mog) |
| `zmij` | 1.0.23 | MIT | Studio, engine (mog), regex module (wasm) |
