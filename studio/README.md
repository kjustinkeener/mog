# Mog Studio

A desktop GUI for the Mog text-transform engine. Build, preview, and run `.mog`
pipelines with a schema-driven form for every action, a live diff, and batch runs
across many files.

Mog Studio runs natively on Windows and Linux. Windows releases are portable
`.exe` installers; Linux releases use the system WebKitGTK runtime and are
distributed as standard Tauri bundles such as `.deb` and AppImage.

![Mog Studio browsing the mog marketplace](docs/screenshot.png)

## What it does

- **Build a mog without memorising the format.** Every action renders as a
  form generated from its schema, so the available options and their types come
  from the engine rather than from documentation that can drift.
- **See the change before making it.** A live diff shows exactly what a step
  would do to the current input, updated as you edit.
- **Run across many files at once**, with an impact summary per file before
  anything is written.
- **Test mogs against their goldens** and check that a mog would leave a
  set of files unchanged.
- **Browse and install shared mogs** from the Mog marketplace.
- **A regex tester in the engine's exact flavour**, so a pattern that works in
  the tester works in a mog.

Built with Tauri v2 (Rust) and Svelte 5 (runes) + TypeScript + Vite.

## Privacy

Studio does not collect analytics, telemetry or crash reports, and has no
account or sign-in.

It reads and writes the files you point it at, and it talks to the network only
when you browse the marketplace or check for updates. Both of those run in the
Rust process, not in the page.

This is enforced rather than asserted: `src-tauri/capabilities/default.json`
grants the webview only the core, dialog and filesystem permissions. There is no
shell permission and no HTTP permission, so the page cannot run a command or
open a network connection of its own.

## How it ships

Mog Studio ships as a single portable `.exe`. The `mog` engine binary is embedded
inside it (gzip-compressed, baked in at build time), so there is nothing else to
download.

On first launch from outside its install directory, Studio shows an installer
card: it copies itself into a per-user location, extracts the engine next to it,
and lets the engine register itself (PATH, mog library, MCP server, and the
Add/Remove Programs entry). Everything is per-user, so no administrator rights
are needed.

At runtime Studio drives the engine as a subprocess (it does not link it as a
library). The binary is resolved in this order:

1. `MOG_BIN` (explicit override, handy in development),
2. `mog.exe` next to the Studio executable (the installed layout),
3. `%LOCALAPPDATA%\mog\mog.exe`,
4. `mog` on `PATH`.

## Prerequisites

- Node.js and npm.
- Rust >= 1.77.2. Windows builds need the MSVC toolchain
  (`x86_64-pc-windows-msvc`); Linux builds use the host toolchain.
- WebView2 runtime on Windows (preinstalled on current Windows 11), or the
  WebKitGTK / GTK3 development packages on Linux.
- For the regex tester: `wasm-pack` and the `wasm32-unknown-unknown` target.

  ```bash
  rustup target add wasm32-unknown-unknown
  cargo install wasm-pack
  ```

Build on a local (non-synced) folder. Building inside a cloud-synced folder can
break Rust and `node_modules` builds through file locking.

## Develop

The WASM regex tester is a build prerequisite: `src/wasm/regex` is generated, not
checked in, so run `build:wasm` once before the first dev or build run (and again
after changing anything under `regex-wasm/`).

```bash
npm install
npm run build:wasm
MOG_BIN=/path/to/mog npm run tauri dev
```

On Linux, build the engine first with `cargo build --bin mog` in `mog-rs/`, then
set `MOG_BIN` to its resulting `target/debug/mog` path. For a release bundle,
build the Linux engine and pass it as `MOG_DIST_BIN`; the normal Tauri bundler
will produce the platform's package formats.

## Checks

```bash
npm run check
npm run build
```

## Build release packages

A plain `npm run tauri build` produces a Studio build with **no embedded engine**:
it runs against an engine found on the machine, and its installer card will refuse
to install. To produce the real single-file download, the engine binary has to be
built and staged first. The owner's ship script (`build-ship.ps1`) does this end to
end, but it lives outside this repository, so a fresh clone reproduces it manually:

1. `cargo build --profile dist --bin mog` in `mog-rs/`.
2. Copy `mog-rs/target/dist/mog.exe` to `studio/src-tauri/binaries/mog-dist.exe`
   (or point `MOG_DIST_EXE` at it; `build.rs` honours either).
3. `npm run build` then `npx tauri build --no-bundle` in `studio/`.

On Linux, build `mog-rs/target/dist/mog`, set `MOG_DIST_BIN` to that path, then
run `npx tauri build`. This produces the configured Linux bundles. Signed release
manifests are emitted separately by the `market-admin` binary (`engine` and
`studio` subcommands).

## License

MIT. See [LICENSE](LICENSE).
