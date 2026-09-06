# The Mog marketplace (`mog market`)

`mog market` is Mog's library and marketplace in one command. Your local library
is simply the **already-installed slice of the marketplace**, so the same verbs
browse what you have, search the catalog, and install more. Discovery is
**local-first**: `list` and `search` work offline against your installed mogs
and add catalog mogs on top when a registry is configured.

## Trust model

You never have to trust the host a mog came from. The catalog (`index.json`)
is signed with an ed25519 key whose **public half is compiled into `mog`**;
`install`/`update` verify that signature, then verify each downloaded mog
against the SHA-256 in the now-trusted catalog. A tampered index or mog is
refused. A signed revocation list lets a bad mog be pulled after publication.

## Verbs

| Command | What it does |
|---|---|
| `mog market search <query>` | Ranked, synonym-aware search over your installed mogs plus the catalog. Multi-word task phrasings work (e.g. `"strip color codes from output"`). |
| `mog market list` | Browse, featured first. Each result is marked `installed`, `local` (authored by you, not in the catalog), or `available`. |
| `mog market show <name>` | Detail for one mog: local content + fixture status if installed, otherwise the catalog entry. |
| `mog market install <name>` | Download, verify (signature + hash), and install a mog and its dependencies. |
| `mog market update` | Refresh the catalog and upgrade installed mogs (revisions are backward-compatible only). |
| `mog market add <file.mog>` | Add a local mog (with its two fixtures) to your library; the fixture is verified on install. |
| `mog market rm <name>` | Remove a locally-authored (`user`) mog. |
| `mog market bless <name>` | Regenerate a local mog's golden fixture from its `TestInput`. |
| `mog market submit <file.mog>` | Submit a mog for review (not available yet). |

Add `--json` to any of these for machine-readable output.

## Configuration

| Variable | Purpose |
|---|---|
| `MOG_HOME` / `--mog-dir` | The library root (default `%APPDATA%\mog` or `~/.mog`). |
| `MOG_MARKET_URL` | The registry base (an `http(s)` URL, or a local directory). A build ships a default; this overrides it (e.g. a private mirror). |
| `MOG_MARKET_PUBKEY` | Override the compiled-in verifying key (dev / mirror). Verification is never skipped. |

Without a registry configured, `list`/`search`/`show`/`add`/`rm`/`bless` still
work against your local library; only `install`/`update` need one.

## From an agent (MCP)

The `mog_market` MCP tool exposes the same surface. It infers the operation when
you omit `op`:

- `mog_market { query }` -> `search`
- `mog_market { name }` -> `show`
- `mog_market {}` -> `list`
- `mog_market { op: "install", name }` -> download + verify + install
- `mog_market { op: "update" }` -> refresh + upgrade

Authoring a new mog from scratch goes through the other tools instead:
`mog_actions` (find actions) -> `mog_validate` -> `mog_preview` -> `mog_apply`.

## Publishing

The catalog lives in the companion registry repo. Mogs are submitted as pull
requests, reviewed (LLM-assisted human), and merged; a CI job then regenerates
and re-signs the index. See that repo's README for the maintainer/go-live steps.
