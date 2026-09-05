//! mog CLI: batch text-processing driven by a .mog script.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use similar::{ChangeTag, TextDiff};

use mog::model::Tier;
use mog::report::{
    truncate_diff, FileReport, FlagRecord, ReportCaps, ReportMeta, RunReport, Summary,
};
use mog::{
    execute_guarded_with_sources, execute_guarded_with_sources_observed,
    load_mog_file_with_defines, Mog, RunLimits, Step,
};

/// Batch text-processing tool. Applies a .mog script to files or to stdin.
///
/// With no inputs (or an input of "-") mog reads STDIN and writes the
/// transformed text to STDOUT, so it works as a Unix filter:
///   cat notes.txt | mog -m tidy.mog
#[derive(Parser, Debug, Clone)]
#[command(name = "mog", version, about, long_about = None)]
struct Cli {
    /// A subcommand: `mog market` verbs, or setup/report/config. When omitted,
    /// mog runs in its default transform mode using the flags below.
    #[command(subcommand)]
    command: Option<Command>,

    /// The .mog script to run: a library script name, or a path. Optional only
    /// when using --list-actions.
    #[arg(short = 'm', long = "script", value_name = "SCRIPT.mog")]
    script: Option<PathBuf>,

    /// Print the action descriptor table as JSON and exit (for tooling/GUIs).
    #[arg(long = "list-actions")]
    list_actions: bool,

    /// Print the ordered, labeled category registry as JSON and exit:
    /// [{ "name", "label", "rank" }, ...] (for tooling/GUIs).
    #[arg(long = "list-categories")]
    list_categories: bool,

    /// With --list-actions, emit a compact index (name, aliases, tier, category,
    /// summary) with no params, for progressive disclosure. Errors if used
    /// without --list-actions.
    #[arg(long = "compact")]
    compact: bool,

    /// Print the full descriptor (params, example, tier, category, aliases) for a
    /// single action, then exit. Resolves aliases to the canonical action; an
    /// unknown name is an error (a JSON error object under --json).
    #[arg(long = "describe", value_name = "ACTION")]
    describe: Option<String>,

    /// Explain what the -m pipeline does in plain language (name, description, and
    /// each step) and exit, without processing inputs. Each step shows its own
    /// `description`, or a fallback from the action descriptor. `--json` for a
    /// structured explanation.
    #[arg(long = "explain")]
    explain: bool,

    /// Print a JSON Schema (draft-07) for the .mog file structure and exit, for
    /// editors and tooling (execution-free shape validation).
    #[arg(long = "schema")]
    schema: bool,

    /// Emit machine-readable JSON instead of the human-readable summary. Covers
    /// --list-actions, the run report, --check, --test, and every store
    /// subcommand. In modes where stdout carries transformed content (stdin
    /// filter, single-file to stdout), pair it with --dry-run or --check so the
    /// report does not collide with the content.
    #[arg(long = "json", global = true)]
    json: bool,

    /// Check mode: writes nothing (like --dry-run) but sets the process exit code
    /// for scripting: 0 = nothing would change and no flags, 1 = at least one file
    /// would change (or a flag marker is present), 2 = a processing error. Prints
    /// the offending file paths (unless --json, which lists them in the report).
    #[arg(long = "check")]
    check: bool,

    /// Test mode: run each mogfile's sibling torture fixtures
    /// (Foo.TestInput.<ext> -> Foo.TestExpectedOutput.<ext>) and report PASS/FAIL
    /// with a unified diff on failure. INPUTS are .mog files or directories to
    /// search (recursively). Exits nonzero if any test fails. Ignores -m.
    #[arg(long = "test")]
    test: bool,

    /// Library root for resolving script names and for every subcommand
    /// (overrides MOG_HOME and the per-user default of %APPDATA%\mog or ~/.mog).
    #[arg(long = "mog-dir", value_name = "DIR", global = true)]
    mog_dir: Option<PathBuf>,

    /// Define or override a script constant used by {{name}} placeholders
    /// (repeatable): --define target_schema=public.
    #[arg(short = 'D', long = "define", value_name = "NAME=VALUE")]
    define: Vec<String>,

    // ---- Determinism pins (`--pin-*`) -----------------------------------------
    // These fix a source of non-determinism so a run is reproducible (and so a
    // recipe that uses {{@now}}/{{@uuid}} can have a stable golden fixture). Named
    // with a shared `--pin-` prefix to make their purpose obvious.
    /// Determinism pin: fix the clock for the built-in {{@now}} / {{@today}} /
    /// {{@year}} / {{@epoch}} / {{@date}} placeholders. Accepts epoch seconds or an
    /// ISO date/datetime (YYYY-MM-DD[THH:MM:SS]). Omit to use the real UTC clock.
    #[arg(long = "pin-now", value_name = "WHEN")]
    now: Option<String>,

    /// Determinism pin: fix the RNG for the built-in {{@uuid}} so a run is
    /// reproducible. Omit for a random uuid each run.
    #[arg(long = "pin-seed", value_name = "N")]
    seed: Option<u64>,

    /// Bind an external data source for source-aware actions (e.g.
    /// fill_from_list): --source NAME=PATH (repeatable). Overrides a same-named
    /// `sources` entry declared in the .mog.
    #[arg(long = "source", value_name = "NAME=PATH")]
    source: Vec<String>,

    /// Input files or glob patterns (e.g. "src/**/*.sql"). Omit, or pass "-", to
    /// read from STDIN and write to STDOUT.
    #[arg(value_name = "INPUTS")]
    inputs: Vec<String>,

    /// Read additional input paths (one per line) from FILE, or from STDIN when
    /// FILE is "-". For CI changed-file runs, e.g.
    /// `git diff --name-only | mog -m fix.mog --in-place --files-from -`.
    #[arg(long = "files-from", value_name = "FILE")]
    files_from: Option<String>,

    /// Overwrite each input file in place with its transformed output. Pairs with
    /// --check for the CI report-then-apply flow (`--check` reports, `--in-place`
    /// applies).
    #[arg(short = 'i', long = "in-place", conflicts_with = "out_dir")]
    in_place: bool,

    /// When writing in place, first save a backup with this suffix (e.g. ".bak").
    #[arg(long = "backup", value_name = "SUFFIX", requires = "in_place")]
    backup: Option<String>,

    /// Write outputs into this directory, mirroring input file names.
    #[arg(short = 'o', long = "out-dir", value_name = "DIR")]
    out_dir: Option<PathBuf>,

    /// Report what would change without writing any files.
    #[arg(long = "dry-run")]
    dry_run: bool,

    /// Abort applying to any single input after this many seconds (wall-clock).
    /// Off by default; a guard for untrusted or pathological input (H2).
    #[arg(long = "timeout", value_name = "SECS")]
    timeout: Option<f64>,

    /// Reject a pipeline with more than this many steps (off by default; H3).
    #[arg(long = "max-steps", value_name = "N")]
    max_steps: Option<usize>,

    /// Reject any single input larger than this many bytes (off by default; H3).
    #[arg(long = "max-input-bytes", value_name = "N")]
    max_input_bytes: Option<usize>,

    /// Data-loss guard: fail a file if the transform drops more than this percent
    /// of its lines (e.g. --max-shrink 50). Off by default.
    #[arg(long = "max-shrink", value_name = "PCT")]
    max_shrink: Option<f64>,

    /// Refuse input that looks binary (contains a NUL byte) instead of transforming
    /// it.
    #[arg(long = "refuse-binary")]
    refuse_binary: bool,

    /// Process STDIN in bounded (constant) memory instead of loading it whole:
    /// read newline-delimited blocks, transform, and emit as they go. Requires an
    /// all-streamable pipeline (per-line, EOL-preserving actions) and UTF-8; mog
    /// errors with the offending step/reason otherwise. For huge or unbounded
    /// input piped through mog.
    #[arg(long = "stream")]
    stream: bool,

    /// Preview a transform on just the first N lines of the input (read-bounded, so
    /// it stays fast on a huge file) and print the result to stdout. A preview: it
    /// refuses the file-writing modes (--in-place / --out-dir). Pair with --diff to
    /// see the sample as a before/after diff.
    #[arg(long = "sample", value_name = "N")]
    sample: Option<usize>,

    /// Speed up a large SINGLE file by transforming it across CPU cores: split at
    /// line boundaries, process chunks in parallel, reassemble in order.
    /// Byte-identical to sequential. Applies only to streamable (line-local)
    /// pipelines; others run sequentially. `--parallel` uses all cores;
    /// `--parallel=N` uses N. (Inter-file parallelism is `-j/--jobs`.)
    #[arg(long = "parallel", value_name = "N", num_args = 0..=1, default_missing_value = "0")]
    parallel: Option<usize>,

    /// Print a unified diff (before -> after) for each changed file, to stdout.
    #[arg(long = "diff")]
    diff: bool,

    /// Print a line to stderr as each pipeline step starts (`step i/N: <action>`),
    /// so a slow or hanging step is visible. Opt-in; a single-file affordance
    /// (ignored for multi-file runs, `--stream`, and streamable `--parallel`).
    /// stdout content is unaffected.
    #[arg(long = "progress")]
    progress: bool,

    /// Computed (not a CLI flag): whether progress output is actually active after
    /// gating (single, non-stream file). Set in `main`; read by `run_pipeline`.
    #[arg(skip)]
    progress_active: bool,

    /// Opt-in: after the run, render an HTML dashboard of the report. Bare
    /// `--report` writes to MOG_HOME/reports/run-<timestamp>.html; `--report
    /// <FILE>` writes to FILE. Absent: no dashboard (unchanged behavior). Works
    /// with apply and dry-run.
    #[arg(long = "report", value_name = "FILE")]
    report: Option<Option<String>>,

    /// Dashboard template name for `--report` (and the `mog report` subcommand).
    /// Resolves MOG_HOME/templates/{user,community,factory}, then the embedded
    /// factory set. Default: default.
    #[arg(long = "template", value_name = "NAME", default_value = "default")]
    template: String,

    /// Human-only: lift the sampling caps (max_diff_files, max_files,
    /// total_diff_bytes) to unlimited for this run. The report can then grow with
    /// the run; the agent surface never passes this. Precedence: CLI > config >
    /// default. (Does not raise diff_skip_over_bytes, which still gates the differ.)
    #[arg(long = "report-full")]
    report_full: bool,

    /// Human-only: override one report cap for this run (repeatable), e.g.
    /// --report-limit max_diff_files=100. A value of 0 or "unlimited" lifts the
    /// cap. Highest precedence (over config.toml and --report-full). Raising
    /// diff_skip_over_bytes warns, since a huge file can then OOM or hang.
    #[arg(long = "report-limit", value_name = "KEY=VALUE")]
    report_limit: Vec<String>,

    /// Read input as UTF-8 lossily (replace invalid bytes) instead of erroring.
    #[arg(long = "lossy")]
    lossy: bool,

    /// Input encoding: "auto" (default: sniff BOM, else UTF-8, else ANSI), or a
    /// label like utf-8, utf-16le, utf-16be, ansi/windows-1252, shift_jis, ...
    #[arg(long = "encoding", value_name = "NAME", default_value = "auto")]
    encoding: String,

    /// Output encoding: "preserve" (default: match the input), or utf-8,
    /// utf-8-bom, utf-16le, utf-16be, ansi/windows-1252, or another label.
    #[arg(
        long = "output-encoding",
        value_name = "NAME",
        default_value = "preserve"
    )]
    output_encoding: String,

    /// Drop any input file whose path matches this glob (repeatable).
    #[arg(long = "exclude", value_name = "GLOB")]
    exclude: Vec<String>,

    /// Number of files to process in parallel (default: available parallelism).
    #[arg(short = 'j', long = "jobs", value_name = "N")]
    jobs: Option<usize>,

    /// Allow overwriting existing files in the output directory.
    #[arg(long = "overwrite")]
    overwrite: bool,
}

/// Tier choice for `mog ls --tier`.
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum TierArg {
    Core,
    Full,
}

impl From<TierArg> for Tier {
    fn from(t: TierArg) -> Self {
        match t {
            TierArg::Core => Tier::Core,
            TierArg::Full => Tier::Full,
        }
    }
}

/// Subcommands. When none is given, mog runs its default transform mode from the
/// top-level flags. The library + marketplace verbs all live under `mog market`
/// (search / list / show / install / update / add / rm / bless). `--json` and
/// `--mog-dir` are global (before or after the verb).
#[derive(Subcommand, Debug, Clone)]
enum Command {
    /// Bootstrap the store and register the MCP server for a fresh install.
    Setup {
        /// Dry run: report every action, write and run nothing.
        #[arg(long)]
        print: bool,
        /// Skip the store bootstrap (create sources + sync factory scripts).
        #[arg(long = "no-store")]
        no_store: bool,
        /// Skip the MCP server registration.
        #[arg(long = "no-mcp")]
        no_mcp: bool,
        /// Opt out of dashboards at install: write `reports = false` to config.toml.
        #[arg(long = "no-reports")]
        no_reports: bool,
    },
    /// Render a report JSON (read from stdin) into a self-contained HTML
    /// dashboard: `mog ... --json | mog report out.html`.
    Report {
        /// Template name (resolves user > community > factory > embedded factory).
        #[arg(long = "template", value_name = "NAME", default_value = "default")]
        template: String,
        /// Output HTML path; omit to write the rendered HTML to stdout.
        #[arg(value_name = "OUT.html")]
        out: Option<PathBuf>,
    },
    /// Read or write persistent preferences in MOG_HOME/config.toml.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// The marketplace: search, install, and browse reusable recipes.
    Market {
        #[command(subcommand)]
        action: MarketAction,
    },
    /// Bring this machine current with the registry: sync recipes (by content
    /// hash) and self-replace the engine binary if a newer signed build exists.
    Update {
        /// Report what would change; write nothing.
        #[arg(long)]
        check: bool,
        /// Skip the recipe sync (engine binary only).
        #[arg(long = "no-recipes")]
        no_recipes: bool,
        /// Skip the engine binary swap (recipes only).
        #[arg(long = "no-engine")]
        no_engine: bool,
        /// Remove recipes on disk that are gone from the catalog (off by default).
        #[arg(long)]
        prune: bool,
    },
    /// The MCP server: `mog mcp` speaks the MCP stdio protocol so an agent
    /// harness can drive Mog directly. Subcommands register/unregister it with
    /// MCP client apps.
    Mcp {
        #[command(subcommand)]
        action: Option<McpAction>,
    },
    /// Provision a per-user install (no admin): copy this binary into
    /// %LOCALAPPDATA%\mog, add it to PATH, seed the library, register MCP, and
    /// write an Add/Remove Programs entry. `mog install studio` also downloads the
    /// Studio GUI from the registry (requires internet).
    Install {
        /// What to install. Omit for the engine; `studio` to add the GUI.
        #[command(subcommand)]
        target: Option<InstallTarget>,
        /// Skip the MCP server registration (engine install only).
        #[arg(long = "no-mcp")]
        no_mcp: bool,
        /// Dry run: report every action, write and run nothing.
        #[arg(long)]
        print: bool,
    },
    /// Reverse `mog install`: remove the PATH entry, Add/Remove Programs key, MCP
    /// registration, and shortcuts, then delete the install dir. The recipe
    /// library under %APPDATA%\mog (your recipes) is left in place.
    Uninstall {
        /// Dry run: report every action, change nothing.
        #[arg(long)]
        print: bool,
    },
}

/// The `mog install` targets. None -> install the engine.
#[derive(Subcommand, Debug, Clone)]
enum InstallTarget {
    /// Download and install the Studio GUI from the registry (requires internet),
    /// wired to the installed mog.exe next to it.
    Studio {
        /// Dry run: report every action, download and write nothing.
        #[arg(long)]
        print: bool,
    },
}

/// The `mog mcp` verbs. None -> run the stdio server.
#[derive(Subcommand, Debug, Clone)]
enum McpAction {
    /// Register this binary as the `mog` MCP server with client apps by editing
    /// their config. Bare: detect installed clients (registers one; lists many).
    Install {
        /// Register with exactly this client (claude-code, claude-desktop,
        /// cursor, windsurf).
        #[arg(long = "client", value_name = "ID")]
        client: Option<String>,
        /// Register with every detected client.
        #[arg(long = "all")]
        all: bool,
    },
    /// Remove the `mog` MCP registration from client apps (mirror of install).
    Uninstall {
        #[arg(long = "client", value_name = "ID")]
        client: Option<String>,
        #[arg(long = "all")]
        all: bool,
    },
}

/// The `mog market` verbs (spec: store-marketplace-spec.md section 9).
#[derive(Subcommand, Debug, Clone)]
enum MarketAction {
    /// Ranked search of the catalog for a task.
    Search {
        #[arg(value_name = "QUERY")]
        query: String,
    },
    /// Browse the catalog (featured first).
    List,
    /// Show one recipe's detail (local content + fixture, or the catalog entry).
    Show {
        #[arg(value_name = "NAME")]
        name: String,
        /// Print metadata only, no script body (for a local recipe).
        #[arg(long = "no-content")]
        no_content: bool,
    },
    /// Download, verify (signature + hash), and install a recipe.
    Install {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Author a local recipe: install a .mog (+ its fixtures) into your library.
    Add {
        #[arg(value_name = "FILE")]
        file: PathBuf,
        /// Install under this name/subpath instead of the file's own name.
        #[arg(long = "as", value_name = "NAME")]
        as_name: Option<String>,
        /// Overwrite an existing entry whose content differs.
        #[arg(long)]
        force: bool,
    },
    /// Remove a locally-authored recipe (user source only).
    Rm {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Regenerate a local recipe's golden fixture from its TestInput.
    Bless {
        #[arg(value_name = "NAME")]
        name: String,
        /// Write the new golden without prompting (otherwise only the diff shows).
        #[arg(long)]
        yes: bool,
    },
    /// Submit a recipe for review (not yet available).
    Submit {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Generate a README.md beside every recipe under DIR from its metadata and
    /// golden fixture. `--check` writes nothing and fails if any committed doc is
    /// out of date (for CI).
    GenDocs {
        #[arg(value_name = "DIR")]
        dir: PathBuf,
        #[arg(long)]
        check: bool,
    },
}

/// The `mog config` verbs.
#[derive(Subcommand, Debug, Clone)]
enum ConfigAction {
    /// Print one key's value, or the whole config when no key is given.
    Get {
        /// The key to read: `reports`, `template`, or a `report.<cap>` key
        /// (e.g. `report.max_diff_files`).
        #[arg(value_name = "KEY")]
        key: Option<String>,
    },
    /// Set a key and persist it: `reports` = bool, `template` = name, or a
    /// `report.<cap>` key (a value of `0`/`unlimited` lifts that cap).
    Set {
        /// The key to write (`reports`, `template`, or `report.<cap>`).
        #[arg(value_name = "KEY")]
        key: String,
        /// The value (`reports` parses on/off, true/false; a `report.<cap>`
        /// takes a number, `0`, or `unlimited`).
        #[arg(value_name = "VALUE")]
        value: String,
    },
}

/// The output destination decided from the flags.
enum OutputMode {
    Stdout,
    InPlace,
    OutDir(PathBuf),
}

fn main() {
    use clap::{CommandFactory, FromArgMatches};
    // Parse via ArgMatches so we can tell whether --output-encoding was given
    // explicitly (it has a default value, so a recipe's `output_encoding` should
    // apply only when the flag was NOT passed on the command line).
    let matches = Cli::command().get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(c) => c,
        Err(e) => e.exit(),
    };
    let out_enc_explicit =
        matches.value_source("output_encoding") == Some(clap::parser::ValueSource::CommandLine);
    let code = match run(&cli, out_enc_explicit) {
        Ok(code) => code,
        Err(e) => {
            // Parse/validation errors are emitted as JSON under --json (so tooling
            // never has to parse stderr prose), else as the usual stderr line.
            if cli.json {
                print_json_error(&e);
            } else {
                eprintln!("error: {e:#}");
            }
            1
        }
    };
    std::process::exit(code);
}

/// Run the CLI, returning the process exit code (0 = success). Errors bubble up
/// to `main`, which renders them (as JSON under --json) and exits nonzero.
fn run(cli: &Cli, out_enc_explicit: bool) -> Result<i32> {
    // Own a mutable copy so the recipe's `output_encoding` can be folded into
    // `cli.output_encoding` before any encode step reads it (see below). Cheap;
    // `main` keeps the original for error rendering.
    let mut cli = cli.clone();
    // A store subcommand takes over entirely; the default transform flags do not
    // apply. `--json` / `--mog-dir` are global, so they are read from `cli`.
    if let Some(cmd) = &cli.command {
        return run_subcommand(cmd, &cli);
    }

    // --compact only makes sense alongside --list-actions.
    if cli.compact && !cli.list_actions {
        bail!("--compact only applies to --list-actions; pass both (mog --list-actions --compact)");
    }

    // --list-actions: emit the descriptor table as JSON and exit. (--json is
    // accepted for explicitness; JSON is the only format.) With --compact, emit
    // the compact index (no params) instead of the full table.
    if cli.list_actions {
        if cli.compact {
            println!("{}", mog::descriptors::compact_descriptors_json());
        } else {
            println!("{}", mog::descriptors_json());
        }
        return Ok(0);
    }

    // --list-categories: emit the ordered category registry as JSON and exit.
    // (--json is accepted for explicitness; JSON is the only format.)
    if cli.list_categories {
        println!("{}", mog::categories_json());
        return Ok(0);
    }

    // --schema: emit the .mog JSON Schema and exit (execution-free shape check).
    if cli.schema {
        println!("{}", mog::descriptors::schema_json());
        return Ok(0);
    }

    // --describe <ACTION>: emit one action's full descriptor as JSON and exit.
    // Aliases resolve to the canonical action; an unknown name bubbles up as an
    // error, rendered as a JSON error object under --json (see `main`).
    if let Some(name) = &cli.describe {
        let desc = mog::descriptors::find_descriptor(name)
            .ok_or_else(|| anyhow!("unknown action '{name}'"))?;
        println!("{}", serde_json::to_string_pretty(&desc)?);
        return Ok(0);
    }

    // --test: run mogfile torture fixtures instead of transforming inputs.
    if cli.test {
        return run_tests(&cli);
    }

    // Resolve the script by name/path against the working dir and the library.
    let script = cli
        .script
        .as_ref()
        .ok_or_else(|| anyhow!("no script given: pass -m <SCRIPT.mog> (or --list-actions)"))?;
    let lib_root = cli.mog_dir.clone().or_else(mog::library::default_root);
    let cwd = std::env::current_dir().ok();
    let script_path = mog::library::resolve_script(script, cwd.as_deref(), lib_root.as_deref())?;

    let mut defines = parse_defines(&cli.define)?;
    mog::builtins::inject_builtins(&mut defines, cli.now.as_deref(), cli.seed)?;
    let mog = load_mog_file_with_defines(&script_path, &defines)?;

    // Output encoding precedence: an explicit --output-encoding wins; otherwise a
    // recipe-declared `output_encoding` applies; otherwise the "preserve" default.
    // Fold the resolved value into `cli` so every downstream encode reads it.
    if !out_enc_explicit {
        if let Some(enc) = &mog.output_encoding {
            cli.output_encoding = enc.clone();
        }
    }
    mog::encoding::validate_output_choice(&cli.output_encoding)?;

    // --explain: describe the pipeline in plain language and exit (no inputs).
    if cli.explain {
        return explain_mog(&mog, &cli);
    }

    // Pre-flight perf warning (skipped under --json so the report stays clean).
    if !cli.json {
        warn_backtracking_regex(&mog)?;
    }
    // run_mog steps resolve relative to the resolved script's directory (then
    // fall back to the library).
    let base_dir = script_path.parent();

    // External data sources for source-aware actions (recipe-declared paths are
    // confined to the .mog's directory; --source binds any path and overrides).
    let sources = load_sources(&mog.sources, &cli.source, base_dir)?;

    // --sample N: read only the first N lines and preview the transform to stdout.
    if let Some(n) = cli.sample {
        return run_sample(&mog, base_dir, lib_root.as_deref(), &cli, &sources, n);
    }

    // --check and --dry-run both write nothing; --check additionally drives the
    // exit code (see `emit_check_report`).
    let no_write = cli.dry_run || cli.check;

    // Report metadata, consumed by both --json and --report. `mode` is derived
    // from the flags (apply vs dry-run); `when` is stamped when the report is
    // built. This is the report layer only and never feeds a transform.
    let meta = ReportMeta {
        name: mog.name.clone().unwrap_or_else(|| {
            script_path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default()
        }),
        path: script.display().to_string(),
        mode: if no_write { "dry-run" } else { "apply" }.to_string(),
    };

    // Assemble the effective input list: positional args plus any `--files-from`
    // entries (one path per line, from a file or STDIN when "-").
    let mut input_args = cli.inputs.clone();
    if let Some(src) = &cli.files_from {
        let listing = if src == "-" {
            use std::io::Read;
            let mut s = String::new();
            std::io::stdin()
                .read_to_string(&mut s)
                .context("failed to read --files-from list from stdin")?;
            s
        } else {
            std::fs::read_to_string(src)
                .with_context(|| format!("failed to read --files-from '{src}'"))?
        };
        for line in listing.lines() {
            let p = line.trim();
            if !p.is_empty() {
                input_args.push(p.to_string());
            }
        }
    }

    // An empty --files-from list (e.g. `git diff --name-only` with no changes) is
    // a no-op success, not an error -- the CI-friendly behavior.
    if cli.files_from.is_some() && input_args.is_empty() {
        eprintln!("no input files (empty --files-from list); nothing to do");
        return Ok(0);
    }

    // Filter mode: no inputs and no --files-from, or a single "-", means read
    // STDIN -> STDOUT. (With --files-from the inputs come from the list, so a
    // "-" positional there would be a path, not the stdin-content sentinel.)
    let stdin_filter_mode = cli.files_from.is_none()
        && (input_args.is_empty() || (input_args.len() == 1 && input_args[0] == "-"));
    if stdin_filter_mode {
        // stdin is a single stream, so --progress applies (unless --stream, which
        // has its own bounded path). This is the primary --progress use case.
        if cli.progress {
            if cli.stream {
                eprintln!("note: --progress is ignored under --stream");
            } else {
                cli.progress_active = true;
            }
        }
        return run_stdin_filter(
            &mog,
            base_dir,
            lib_root.as_deref(),
            &cli,
            no_write,
            &meta,
            &sources,
        );
    }

    let files = expand_inputs(&input_args)?;
    let files = apply_excludes(files, &cli.exclude)?;
    if files.is_empty() {
        bail!("no input files matched");
    }

    let mode = decide_output_mode(&cli, files.len(), no_write)?;

    // --progress is a single-file affordance: multi-file / -j runs interleave, and
    // --stream has its own path. Decide once here; run_pipeline reads progress_active.
    if cli.progress {
        if cli.stream {
            eprintln!("note: --progress is ignored under --stream");
        } else if files.len() != 1 {
            eprintln!(
                "note: --progress applies to single-file runs; ignored for {} files",
                files.len()
            );
        } else {
            cli.progress_active = true;
        }
    }

    // Bounded-memory streaming of a single file (to stdout, or in place).
    if cli.stream {
        validate_stream_eligibility(&cli, &mog)?;
        if files.len() != 1 {
            bail!(
                "--stream supports a single file input (or stdin); got {} files",
                files.len()
            );
        }
        match &mode {
            OutputMode::Stdout => {
                return run_file_stream(
                    &files[0],
                    &mog,
                    base_dir,
                    lib_root.as_deref(),
                    &cli,
                    &sources,
                )
            }
            OutputMode::InPlace => {
                return run_inplace_stream(
                    &files[0],
                    &mog,
                    base_dir,
                    lib_root.as_deref(),
                    &cli,
                    &sources,
                )
            }
            OutputMode::OutDir(dir) => {
                return run_outdir_stream(
                    &files[0],
                    dir,
                    &mog,
                    base_dir,
                    lib_root.as_deref(),
                    &cli,
                    &sources,
                )
            }
        }
    }

    // stdout is reserved for transformed content in Stdout mode; a JSON report
    // there would collide with it. Require a non-writing report mode instead.
    if cli.json && matches!(mode, OutputMode::Stdout) && !no_write {
        bail!(
            "--json prints a report to stdout, but this mode sends transformed \
             content there; add --dry-run or --check (or write files with \
             --in-place/--out-dir)"
        );
    }

    if let OutputMode::OutDir(dir) = &mode {
        if !no_write {
            std::fs::create_dir_all(dir)
                .with_context(|| format!("failed to create out-dir '{}'", dir.display()))?;
        }
    }

    if let Some(j) = cli.jobs {
        rayon::ThreadPoolBuilder::new()
            .num_threads(j.max(1))
            .build_global()
            .ok();
    }

    // A single Stdout file is processed directly to keep content ordering clean.
    // Only the plain (non-report) modes emit content, so --json/--check skip this.
    // (Multi-file Stdout only happens under --dry-run, which prints no content,
    // so it falls through to the reporting loop below.)
    if !cli.json
        && !cli.check
        && cli.report.is_none()
        && matches!(mode, OutputMode::Stdout)
        && files.len() == 1
    {
        let path = &files[0];
        match process_one(
            &mog,
            path,
            &mode,
            &cli,
            base_dir,
            lib_root.as_deref(),
            None,
            &sources,
        ) {
            Ok(res) => {
                if let Some(payload) = &res.payload {
                    std::io::stdout().write_all(payload)?;
                }
                eprintln!("{}", res.summary);
                report_flags(&path.display().to_string(), &res.flags);
                return Ok(0);
            }
            Err(e) => {
                eprintln!("error: {}: {e:#}", path.display());
                return Ok(1);
            }
        }
    }

    // The report (JSON or dashboard) is bounded by construction. Resolve the caps
    // once, then share a diff budget across the parallel pass so the differ is
    // never handed a giant file and at most `max_diff_files` diffs are computed.
    let want_report = cli.json || cli.report.is_some();
    let caps = if want_report {
        resolve_caps(&cli, lib_root.as_deref())?
    } else {
        ReportCaps::default()
    };
    let budget = want_report.then(|| DiffBudget::new(caps));

    let results: Vec<(PathBuf, Result<Processed>)> = files
        .par_iter()
        .map(|path| {
            (
                path.clone(),
                process_one(
                    &mog,
                    path,
                    &mode,
                    &cli,
                    base_dir,
                    lib_root.as_deref(),
                    budget.as_ref(),
                    &sources,
                ),
            )
        })
        .collect();

    // Build the run report once when either output (JSON) or an HTML dashboard
    // needs it, then render the dashboard (opt-in) before the mode branches. The
    // summary aggregates are exact over every file; `files[]` is capped.
    let run_report = want_report.then(|| build_report(meta.clone(), &results, &caps));
    if let Some(inner) = &cli.report {
        let report = run_report.as_ref().expect("built when --report is set");
        let out = mog::report::write_dashboard(
            report,
            inner.as_deref(),
            &cli.template,
            lib_root.as_deref(),
        )?;
        eprintln!("report: wrote {}", out.display());
    }

    if cli.json {
        return emit_json_report(
            run_report.as_ref().expect("built under --json"),
            &results,
            &cli,
        );
    }
    if cli.check {
        return emit_check_report(&results);
    }

    let mut changed = 0usize;
    let mut errors = 0usize;
    for (path, res) in &results {
        match res {
            Ok(p) => {
                if let Some(payload) = &p.payload {
                    std::io::stdout().write_all(payload)?;
                }
                println!("{}", p.summary);
                report_flags(&path.display().to_string(), &p.flags);
                if p.changed {
                    changed += 1;
                }
            }
            Err(e) => {
                errors += 1;
                eprintln!("error: {}: {e:#}", path.display());
            }
        }
    }

    let total = files.len();
    let verb = if cli.dry_run {
        "would change"
    } else {
        "changed"
    };
    println!("Processed {total} file(s): {changed} {verb}, {errors} error(s).");

    Ok(if errors > 0 { 1 } else { 0 })
}

/// Dispatch a store subcommand (`ls` / `show` / `add` / `rm` / `bless`) to the
/// `mog::store` module. The library root is resolved the same way the default
/// run path does it: `--mog-dir` > `MOG_HOME` > the per-user default.
fn run_subcommand(cmd: &Command, cli: &Cli) -> Result<i32> {
    let lib_root = cli.mog_dir.clone().or_else(mog::library::default_root);
    let json = cli.json;
    match cmd {
        Command::Market { action } => match action {
            MarketAction::Search { query } => {
                mog::market_client::search(lib_root.as_deref(), query, json)
            }
            MarketAction::List => mog::market_client::list(lib_root.as_deref(), json),
            MarketAction::Show { name, no_content } => {
                mog::market_client::show(lib_root.as_deref(), name, *no_content, json)
            }
            MarketAction::Install { name } => {
                let base = mog::market_client::market_base_url()?;
                mog::market_client::install(lib_root.as_deref(), &base, name, json)
            }
            MarketAction::Add {
                file,
                as_name,
                force,
            } => mog::market::add(lib_root.as_deref(), file, as_name.as_deref(), *force, json),
            MarketAction::Rm { name } => {
                mog::market::rm(lib_root.as_deref(), std::path::Path::new(name), json)
            }
            MarketAction::Bless { name, yes } => {
                mog::market::bless(lib_root.as_deref(), std::path::Path::new(name), *yes, json)
            }
            MarketAction::GenDocs { dir, check } => mog::docgen::gen_docs(dir, *check),
            MarketAction::Submit { .. } => bail!(
                "`mog market submit` is not available yet (it arrives with the review \
                 pipeline in the next milestone)"
            ),
        },
        Command::Setup {
            print,
            no_store,
            no_mcp,
            no_reports,
        } => {
            let opts = mog::setup::SetupOptions {
                mog_dir: cli.mog_dir.clone(),
                print: *print,
                no_store: *no_store,
                no_mcp: *no_mcp,
                no_reports: *no_reports,
                json,
            };
            mog::setup::run(&opts)
        }
        Command::Update {
            check,
            no_recipes,
            no_engine,
            prune,
        } => mog::updater::run(
            lib_root.as_deref(),
            &mog::updater::Options {
                check: *check,
                no_recipes: *no_recipes,
                no_engine: *no_engine,
                prune: *prune,
                json,
            },
        ),
        Command::Mcp { action } => match action {
            None => mog::mcp::run_server(),
            Some(McpAction::Install { client, all }) => {
                mog::mcp::install::install(client.as_deref(), *all, json)
            }
            Some(McpAction::Uninstall { client, all }) => {
                mog::mcp::install::uninstall(client.as_deref(), *all, json)
            }
        },
        Command::Install {
            target,
            no_mcp,
            print,
        } => match target {
            None => mog::install::install_engine(*no_mcp, *print, json),
            Some(InstallTarget::Studio {
                print: studio_print,
            }) => mog::install::install_studio(*print || *studio_print, json),
        },
        Command::Uninstall { print } => mog::install::uninstall(*print, json),
        Command::Report { template, out } => run_report_command(template, out.as_deref(), cli),
        Command::Config { action } => match action {
            ConfigAction::Get { key } => {
                mog::config::get(lib_root.as_deref(), key.as_deref(), json)
            }
            ConfigAction::Set { key, value } => {
                mog::config::set(lib_root.as_deref(), key, value, json)
            }
        },
    }
}

/// `mog report [--template <NAME>] [<OUT.html>]`: read a report JSON from stdin
/// and render it into a self-contained HTML dashboard. Writes to `out`, or to
/// stdout when omitted. Reuses the same renderer the run path uses.
fn run_report_command(template: &str, out: Option<&Path>, cli: &Cli) -> Result<i32> {
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("failed to read report JSON from stdin")?;
    // Parse to validate, then re-serialize compactly for a clean injection.
    let value: serde_json::Value =
        serde_json::from_str(&buf).context("stdin is not valid report JSON")?;
    let json = serde_json::to_string(&value)?;
    let lib_root = cli.mog_dir.clone().or_else(mog::library::default_root);
    let html = mog::report::render(&json, template, lib_root.as_deref())?;
    match out {
        Some(p) => {
            std::fs::write(p, html)
                .with_context(|| format!("failed to write '{}'", p.display()))?;
            eprintln!("report: wrote {}", p.display());
        }
        None => print!("{html}"),
    }
    Ok(0)
}

/// Build the run report from all processed results, bounded by `caps`. The
/// `summary` aggregates are computed EXACTLY over every file (cheap counts and
/// byte sums), while `files[]` is enumerated only up to `caps.max_files` (the
/// remainder is counted in the summary but not listed individually). Per-file
/// diffs were already gated and capped during processing. Flags are collected
/// from every file so the flag report stays complete.
fn build_report(
    meta: ReportMeta,
    results: &[(PathBuf, Result<Processed>)],
    caps: &ReportCaps,
) -> RunReport {
    let mut summary = Summary {
        files: results.len(),
        changed: 0,
        errors: 0,
        lines_added: 0,
        lines_removed: 0,
    };
    let mut files: Vec<FileReport> = Vec::new();
    let mut flags: Vec<FlagRecord> = Vec::new();
    let limit = caps.max_files.unwrap_or(usize::MAX);

    for (i, (path, res)) in results.iter().enumerate() {
        let p = path.display().to_string();
        match res {
            Ok(pr) => {
                if pr.changed {
                    summary.changed += 1;
                }
                // Line counts exist only for the diffed sample; summing them here
                // reflects that sample (they are not in the exact-cost set).
                summary.lines_added += pr.lines_added;
                summary.lines_removed += pr.lines_removed;
                for f in &pr.flags {
                    flags.push(FlagRecord {
                        tag: f.tag.clone(),
                        file: p.clone(),
                        line: f.line,
                        message: f.message.clone(),
                    });
                }
                if i < limit {
                    files.push(FileReport {
                        path: p,
                        changed: pr.changed,
                        bytes_before: pr.bytes_before,
                        bytes_after: pr.bytes_after,
                        lines_added: pr.lines_added,
                        lines_removed: pr.lines_removed,
                        diff: pr.diff.clone(),
                        error: None,
                    });
                }
            }
            Err(e) => {
                summary.errors += 1;
                if i < limit {
                    files.push(FileReport {
                        path: p,
                        changed: false,
                        bytes_before: 0,
                        bytes_after: 0,
                        lines_added: 0,
                        lines_removed: 0,
                        diff: None,
                        error: Some(format!("{e:#}")),
                    });
                }
            }
        }
    }
    RunReport::from_parts(meta, summary, files, flags)
}

/// The offending paths (changed, or carrying a flag) over ALL results, for the
/// `--check` contract. Computed straight from the results (not the capped report)
/// so `--check` stays exact even when `files[]` is enumeration-capped.
fn offending_from_results(results: &[(PathBuf, Result<Processed>)]) -> Vec<String> {
    results
        .iter()
        .filter_map(|(path, res)| match res {
            Ok(p) if p.changed || !p.flags.is_empty() => Some(path.display().to_string()),
            _ => None,
        })
        .collect()
}

/// Emit the model-facing (lean) run report as a single JSON object on stdout: no
/// per-file diff text, `files[]` capped, but the summary exact. Returns the exit
/// code: with --check applied, 0 clean / 1 would-change or flags / 2 error;
/// otherwise 0, or 1 when any file errored.
fn emit_json_report(
    report: &RunReport,
    results: &[(PathBuf, Result<Processed>)],
    cli: &Cli,
) -> Result<i32> {
    let errors = report.summary.errors;
    let mut root = report.to_lean_value()?;
    let offending = if cli.check {
        offending_from_results(results)
    } else {
        Vec::new()
    };
    if cli.check {
        root["check"] = serde_json::json!({
            "clean": offending.is_empty() && errors == 0,
            "offending": offending,
        });
    }
    println!("{}", serde_json::to_string_pretty(&root)?);

    if cli.check {
        Ok(check_exit_code(errors, offending.is_empty()))
    } else {
        Ok(if errors > 0 { 1 } else { 0 })
    }
}

/// Emit the `--check` report (offending paths, gofmt -l / black --check style)
/// and return the exit code: 0 clean, 1 would-change or a flag present, 2 error.
fn emit_check_report(results: &[(PathBuf, Result<Processed>)]) -> Result<i32> {
    let mut offending: Vec<String> = Vec::new();
    let mut errors = 0usize;
    for (path, res) in results {
        match res {
            Ok(p) => {
                if p.changed || !p.flags.is_empty() {
                    offending.push(path.display().to_string());
                }
            }
            Err(e) => {
                errors += 1;
                eprintln!("error: {}: {e:#}", path.display());
            }
        }
    }
    for p in &offending {
        println!("{p}");
    }
    Ok(check_exit_code(errors, offending.is_empty()))
}

/// The `--check` exit-code contract in one place: 2 on any error, else 1 when
/// something would change or is flagged, else 0.
fn check_exit_code(errors: usize, clean: bool) -> i32 {
    if errors > 0 {
        2
    } else if clean {
        0
    } else {
        1
    }
}

/// Render a parse/validation error as a JSON object on stdout, best-effort
/// enriched with the failing step number and action name when the message
/// carries them.
fn print_json_error(e: &anyhow::Error) {
    let msg = format!("{e:#}");
    let mut obj = serde_json::json!({ "error": true, "message": &msg });
    if let Some((step, action)) = extract_step_action(&msg) {
        if let Some(s) = step {
            obj["step"] = serde_json::json!(s);
        }
        if let Some(a) = action {
            obj["action"] = serde_json::json!(a);
        }
    }
    let text =
        serde_json::to_string_pretty(&obj).unwrap_or_else(|_| "{\"error\":true}".to_string());
    println!("{text}");
}

/// Best-effort parse of the failing step number / action name from an engine
/// error message. Recognizes "Step N ('action') failed: ..." and "Unknown
/// action 'name' in step N.".
fn extract_step_action(msg: &str) -> Option<(Option<i64>, Option<String>)> {
    use fancy_regex::Regex;
    if let Ok(Some(c)) = Regex::new(r"Step (\d+) \('([^']*)'\)")
        .expect("static regex")
        .captures(msg)
    {
        let step = c.get(1).and_then(|m| m.as_str().parse::<i64>().ok());
        let action = c.get(2).map(|m| m.as_str().to_string());
        return Some((step, action));
    }
    if let Ok(Some(c)) = Regex::new(r"Unknown action '([^']*)' in step (\d+)")
        .expect("static regex")
        .captures(msg)
    {
        let action = c.get(1).map(|m| m.as_str().to_string());
        let step = c.get(2).and_then(|m| m.as_str().parse::<i64>().ok());
        return Some((step, action));
    }
    None
}

/// `mog --test <PATHS...>`: run each mogfile's sibling torture fixtures and
/// report PASS/FAIL. Returns 0 when all pass, 1 when any fails.
fn run_tests(cli: &Cli) -> Result<i32> {
    if cli.inputs.is_empty() {
        bail!("--test requires a .mog file or directory to test");
    }
    let mut mogs: Vec<PathBuf> = Vec::new();
    for inp in &cli.inputs {
        let p = PathBuf::from(inp);
        if !p.exists() {
            bail!("--test path does not exist: {}", p.display());
        }
        mogs.extend(mog::testkit::find_mogs(&p));
    }
    mogs.sort();
    mogs.dedup();
    if mogs.is_empty() {
        bail!("no .mog files found to test");
    }

    let outcomes: Vec<mog::testkit::TestOutcome> = mogs
        .iter()
        .map(|m| mog::testkit::run_mog_test(m, cli.now.as_deref(), cli.seed))
        .collect();
    let passed = outcomes.iter().filter(|o| o.pass).count();

    if cli.json {
        let recs: Vec<serde_json::Value> = outcomes
            .iter()
            .map(|o| {
                let mut v = serde_json::json!({
                    "mog": o.mog.display().to_string(),
                    "pass": o.pass,
                });
                if let Some(d) = &o.diff {
                    v["diff"] = serde_json::json!(d);
                }
                if let Some(m) = &o.message {
                    v["message"] = serde_json::json!(m);
                }
                v
            })
            .collect();
        let root = serde_json::json!({
            "tests": recs,
            "summary": {
                "total": outcomes.len(),
                "passed": passed,
                "failed": outcomes.len() - passed,
            },
        });
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(if passed == outcomes.len() { 0 } else { 1 });
    }

    for o in &outcomes {
        if o.pass {
            println!("PASS {}", o.mog.display());
        } else {
            let reason = o.message.as_deref().unwrap_or("mismatch");
            println!("FAIL {} ({reason})", o.mog.display());
            if let Some(d) = &o.diff {
                print!("{d}");
            }
        }
    }
    let failed = outcomes.len() - passed;
    println!("{passed} passed, {failed} failed of {}", outcomes.len());
    Ok(if failed > 0 { 1 } else { 0 })
}

/// Parse repeated `--define NAME=VALUE` args into a constant-override map.
/// Splits on the first `=`, so values may themselves contain `=`.
fn parse_defines(defs: &[String]) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();
    for d in defs {
        let (k, v) = d
            .split_once('=')
            .ok_or_else(|| anyhow!("--define expects NAME=VALUE, got '{d}'"))?;
        let k = k.trim();
        if k.is_empty() {
            bail!("--define has an empty name in '{d}'");
        }
        map.insert(k.to_string(), v.to_string());
    }
    Ok(map)
}

/// Load external data sources for source-aware actions. Recipe-declared paths
/// (from the .mog's `sources`) are confined to the .mog's directory; `--source
/// NAME=PATH` binds any path and overrides a same-named declaration. Each source
/// becomes its list of lines.
fn load_sources(
    recipe: &BTreeMap<String, String>,
    cli_sources: &[String],
    base_dir: Option<&Path>,
) -> Result<Arc<BTreeMap<String, Vec<String>>>> {
    // Recipe-declared sources are confined to the .mog's directory (shared with
    // the `mog --test` harness so a golden threads sources exactly as a run does).
    let mut out = mog::sources::load_recipe_sources(recipe, base_dir)?;
    // CLI `--source NAME=PATH` binds any path and overrides a same-named declaration.
    for entry in cli_sources {
        let (k, v) = entry
            .split_once('=')
            .ok_or_else(|| anyhow!("--source expects NAME=PATH, got '{entry}'"))?;
        let k = k.trim();
        if k.is_empty() {
            bail!("--source has an empty name in '{entry}'");
        }
        let lines = mog::sources::read_source_lines(Path::new(v), k)?;
        out.insert(k.to_string(), lines);
    }
    Ok(Arc::new(out))
}

/// Pre-flight perf warning: a `replace_regex` / `replace_regex_multiline` step
/// whose pattern uses backreferences or lookaround can only run on fancy-regex's
/// backtracking engine, which is O(n^2) on large input (the linear-time `regex`
/// crate fast path handles everything else). Warn once per such step, on stderr.
/// Scans top-level steps only; nested compose steps are not inspected.
fn warn_backtracking_regex(mog: &Mog) -> Result<()> {
    for (i, step) in mog.steps.iter().enumerate() {
        let action = match step.action.as_deref() {
            Some(a) => a,
            None => continue,
        };
        let multiline = match action {
            "replace_regex" | "rr" => false,
            "replace_regex_multiline" | "rrm" => true,
            _ => continue,
        };
        let find = match step.get_string("find") {
            Some(f) => f,
            None => continue,
        };
        let ignore_case = step.get_bool("ignore_case", false)?;
        if mog::actions::replace::pattern_needs_backtracking(&find, ignore_case, multiline) {
            eprintln!(
                "warning: step {} ({action}) uses backreferences/lookaround; it runs on a \
                 backtracking regex engine that can be slow (O(n^2)) on very large inputs",
                i + 1
            );
        }
    }
    Ok(())
}

/// One-line explanation of a step: its own `description` when present, else a
/// deterministic fallback from the action descriptor ("Label -- summary"). This is
/// the engine-side, AI-free explanation; prose polishing lives out-of-engine.
fn step_explanation(step: &Step) -> String {
    if let Some(d) = &step.description {
        if !d.trim().is_empty() {
            return d.clone();
        }
    }
    let action = match step.action.as_deref() {
        Some(a) => a,
        None => return "(no action)".to_string(),
    };
    match mog::descriptors::find_descriptor(action) {
        Some(desc) => format!("{} -- {}", desc.label, desc.summary),
        None => format!("{action} (unknown action)"),
    }
}

/// `--explain`: render the pipeline's intent (name, description, and each step) in
/// plain text, or as a JSON object under `--json`. No inputs are processed.
fn explain_mog(mog: &Mog, cli: &Cli) -> Result<i32> {
    if cli.json {
        let steps: Vec<serde_json::Value> = mog
            .steps
            .iter()
            .enumerate()
            .map(|(i, s)| {
                serde_json::json!({
                    "n": i + 1,
                    "action": s.action.as_deref().unwrap_or("(no action)"),
                    "text": step_explanation(s),
                })
            })
            .collect();
        let root = serde_json::json!({
            "name": mog.name,
            "description": mog.description,
            "steps": steps,
        });
        println!("{}", serde_json::to_string_pretty(&root)?);
    } else {
        if let Some(name) = &mog.name {
            println!("{name}");
        }
        if let Some(desc) = &mog.description {
            println!("{desc}");
        }
        println!("\nSteps:");
        for (i, s) in mog.steps.iter().enumerate() {
            println!("  {}. {}", i + 1, step_explanation(s));
        }
    }
    Ok(0)
}

/// Write `bytes` to `path` atomically: write a temp file alongside it, then rename
/// over the target (same-directory rename is atomic). On failure the temp file is
/// removed and the original is left intact -- a crash mid-write can't corrupt it.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(format!(".mog-tmp.{}", std::process::id()));
    let tmp = std::path::PathBuf::from(tmp);
    std::fs::write(&tmp, bytes)
        .with_context(|| format!("failed to write temp file '{}'", tmp.display()))?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e).with_context(|| format!("failed to replace '{}'", path.display()));
    }
    Ok(())
}

/// Resolve the requested intra-file parallel degree: `None` when `--parallel` is
/// absent; the machine's core count for bare `--parallel` (encoded as 0); else N.
fn parallel_degree(cli: &Cli) -> Option<usize> {
    cli.parallel.map(|n| {
        if n == 0 {
            rayon::current_num_threads().max(1)
        } else {
            n.max(1)
        }
    })
}

/// Run the pipeline over `input`. With `--parallel` on a streamable pipeline, use
/// intra-file parallelism (byte-identical to sequential); otherwise the normal
/// guarded whole-file path.
fn run_pipeline(
    mog: &Mog,
    input: &str,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<String> {
    if let Some(deg) = parallel_degree(cli) {
        if deg > 1 && mog::stream::pipeline_is_streamable(mog) {
            return mog::stream::execute_parallel(mog, input, deg);
        }
    }
    let on_step = cli.progress_active.then(make_step_observer);
    execute_guarded_with_sources_observed(
        mog,
        input,
        base_dir,
        lib_root,
        &run_limits(cli),
        sources.clone(),
        on_step,
        cli.seed,
    )
}

/// Build the `--progress` step observer: one line per step to stderr, showing the
/// 1-based position, the action, and the byte size entering the step.
fn make_step_observer() -> mog::StepObserver {
    std::sync::Arc::new(|p: mog::StepProgress| {
        eprintln!(
            "step {}/{}: {} ({} B in)",
            p.index, p.total, p.action, p.input_bytes
        );
    })
}

/// Resolve the script to run for one input `path`, expanding the per-file
/// placeholders (`{{@filename}}` / `{{@basename}}` / `{{@ext}}` / `{{@dirname}}`).
/// Clones the shared script only when it actually uses one of them; otherwise it is
/// borrowed unchanged. `path` is `None` for stdin (the placeholders become empty).
fn mog_for_file<'a>(mog: &'a Mog, path: Option<&Path>) -> Result<std::borrow::Cow<'a, Mog>> {
    if mog::interpolate::uses_file_context(mog) {
        let mut owned = mog.clone();
        mog::interpolate::expand_file_context(&mut owned, path)?;
        Ok(std::borrow::Cow::Owned(owned))
    } else {
        Ok(std::borrow::Cow::Borrowed(mog))
    }
}

/// Read raw bytes from `reader` up to and including the Nth `\n` (or EOF). This is
/// read-bounded, so previewing the first N lines of a huge file does not read it
/// whole.
fn read_first_lines(reader: &mut dyn std::io::Read, n: usize) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    if n == 0 {
        return Ok(buf);
    }
    let mut chunk = [0u8; 64 * 1024];
    let mut lines = 0usize;
    loop {
        let k = reader.read(&mut chunk).context("failed to read input")?;
        if k == 0 {
            break;
        }
        for &b in &chunk[..k] {
            buf.push(b);
            if b == b'\n' {
                lines += 1;
                if lines >= n {
                    return Ok(buf);
                }
            }
        }
    }
    Ok(buf)
}

/// `--sample N`: preview the transform on the first N lines of a single input (or
/// stdin), printed to stdout. Read-only -- the file-writing modes are refused.
fn run_sample(
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
    n: usize,
) -> Result<i32> {
    use std::io::Write;

    if cli.in_place || cli.out_dir.is_some() {
        bail!(
            "--sample is a read-only preview; it cannot write files (drop --in-place / --out-dir)"
        );
    }
    if cli.json || cli.check || cli.report.is_some() {
        bail!("--sample previews a partial file to stdout; it is incompatible with --json / --check / --report");
    }

    // Source: stdin (no inputs, or a single "-") or a single file/glob (first match).
    let stdin_mode = cli.inputs.is_empty() || (cli.inputs.len() == 1 && cli.inputs[0] == "-");
    let (bytes, label) = if stdin_mode {
        let mut r = std::io::stdin().lock();
        (read_first_lines(&mut r, n)?, "stdin".to_string())
    } else {
        if cli.inputs.len() != 1 {
            bail!(
                "--sample previews a single input; got {} (pass one file or stdin)",
                cli.inputs.len()
            );
        }
        let files = expand_inputs(&cli.inputs)?;
        let path = files
            .first()
            .ok_or_else(|| anyhow!("no input file matched"))?;
        let f = std::fs::File::open(path)
            .with_context(|| format!("failed to open '{}'", path.display()))?;
        let mut r = std::io::BufReader::new(f);
        (read_first_lines(&mut r, n)?, path.display().to_string())
    };

    check_binary(cli, &bytes, &label)?;
    let decoded = mog::encoding::decode(&bytes, Some(&cli.encoding), cli.lossy)
        .with_context(|| format!("failed to decode '{label}'"))?;
    let input = decoded.text.as_str();
    let output = run_pipeline(mog, input, base_dir, lib_root, cli, sources)?;
    guard_output(cli, input, &output, &label)?;

    let mut out = std::io::stdout();
    if cli.diff {
        out.write_all(unified_diff(&label, input, &output).as_bytes())?;
    } else {
        let encoded = mog::encoding::encode(&output, &cli.output_encoding, &decoded)?;
        out.write_all(&encoded)?;
    }
    eprintln!(
        "{label}: previewed first {n} line(s) ({} -> {} bytes)",
        input.len(),
        output.len()
    );
    Ok(0)
}

/// Refuse input that looks binary (contains a NUL byte), if `--refuse-binary`.
fn check_binary(cli: &Cli, bytes: &[u8], label: &str) -> Result<()> {
    if cli.refuse_binary && bytes.contains(&0) {
        bail!(
            "{label} looks binary (contains a NUL byte); refused (drop --refuse-binary to force)"
        );
    }
    Ok(())
}

/// Data-loss guard: with `--max-shrink PCT`, fail if the output dropped more than
/// PCT percent of the input's lines (a likely-mistake backstop for apply).
fn guard_output(cli: &Cli, input: &str, output: &str, label: &str) -> Result<()> {
    if let Some(pct) = cli.max_shrink {
        let in_lines = input.lines().count();
        if in_lines > 0 {
            let out_lines = output.lines().count();
            let dropped = in_lines.saturating_sub(out_lines);
            let shrink = dropped as f64 / in_lines as f64 * 100.0;
            if shrink > pct {
                bail!(
                    "{label}: output dropped {shrink:.0}% of lines ({in_lines} -> {out_lines}), \
                     over --max-shrink {pct}%; refusing"
                );
            }
        }
    }
    Ok(())
}

/// Read STDIN, run the pipeline, write the result (or a diff) to STDOUT. The
/// per-run summary goes to stderr so redirects stay clean. Under --json/--check
/// stdout carries the report instead (content is suppressed); since stdin has no
/// writing mode, --json here requires --dry-run or --check.
fn run_stdin_filter(
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    no_write: bool,
    meta: &ReportMeta,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    // Bounded-memory streaming path (opt-in). Validated eligible, it never loads
    // the whole of stdin.
    if cli.stream {
        validate_stream_eligibility(cli, mog)?;
        return run_stdin_stream(mog, base_dir, lib_root, cli, sources);
    }

    let mut buf = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buf)
        .context("failed to read stdin")?;
    check_binary(cli, &buf, "stdin")?;
    let decoded = mog::encoding::decode(&buf, Some(&cli.encoding), cli.lossy)
        .context("failed to decode stdin")?;
    let input = decoded.text.as_str();
    let mog_file = mog_for_file(mog, None)?;
    let output = run_pipeline(&mog_file, input, base_dir, lib_root, cli, sources)?;
    guard_output(cli, input, &output, "stdin")?;

    let changed = input != output;
    let flags = mog::actions::flag::find_flags(&output);
    let dirty = changed || !flags.is_empty();

    // Build the one-file "stdin" report once when either output (JSON) or an HTML
    // dashboard needs it. The single file's diff is still bounded: gated by
    // diff_skip_over_bytes and truncated to the per-file line/byte caps.
    let run_report = if cli.json || cli.report.is_some() {
        let caps = resolve_caps(cli, lib_root)?;
        let (lines_added, lines_removed, diff) =
            if changed && !caps.skips_diff(input.len(), output.len()) {
                let (d, added, removed) = unified_diff_with_counts("stdin", input, &output);
                let d = truncate_diff(d, caps.max_diff_lines, caps.max_diff_bytes);
                (added, removed, Some(d))
            } else {
                (0, 0, None)
            };
        let file = FileReport {
            path: "stdin".to_string(),
            changed,
            bytes_before: input.len(),
            bytes_after: output.len(),
            lines_added,
            lines_removed,
            diff,
            error: None,
        };
        let flag_recs: Vec<FlagRecord> = flags
            .iter()
            .map(|f| FlagRecord {
                tag: f.tag.clone(),
                file: "stdin".to_string(),
                line: f.line,
                message: f.message.clone(),
            })
            .collect();
        Some(RunReport::new(meta.clone(), vec![file], flag_recs))
    } else {
        None
    };

    // Opt-in HTML dashboard. Writing a side file never collides with stdout
    // content, so --report works here regardless of --dry-run/--check.
    if let Some(inner) = &cli.report {
        let report = run_report.as_ref().expect("built when --report is set");
        let out = mog::report::write_dashboard(report, inner.as_deref(), &cli.template, lib_root)?;
        eprintln!("report: wrote {}", out.display());
    }

    if cli.json {
        if !no_write {
            bail!(
                "--json prints a report to stdout, but stdin filter mode sends \
                 transformed content there; add --dry-run or --check"
            );
        }
        let report = run_report.as_ref().expect("built under --json");
        let mut root = report.to_lean_value()?;
        if cli.check {
            root["check"] = serde_json::json!({
                "clean": !dirty,
                "offending": if dirty { vec!["stdin".to_string()] } else { vec![] },
            });
        }
        println!("{}", serde_json::to_string_pretty(&root)?);
        return Ok(if cli.check {
            check_exit_code(0, !dirty)
        } else {
            0
        });
    }

    if cli.check {
        if dirty {
            println!("stdin");
        }
        return Ok(check_exit_code(0, !dirty));
    }

    let mut stdout = std::io::stdout();
    if cli.diff {
        if changed {
            stdout.write_all(unified_diff("stdin", input, &output).as_bytes())?;
        }
    } else {
        let bytes = mog::encoding::encode(&output, &cli.output_encoding, &decoded)?;
        stdout.write_all(&bytes)?;
    }

    eprintln!(
        "stdin: {} -> {} bytes [{} -> {}]{}",
        input.len(),
        output.len(),
        decoded.encoding,
        out_encoding_label(&cli.output_encoding, &decoded),
        if changed { "" } else { " (unchanged)" }
    );
    report_flags("stdin", &flags);
    Ok(0)
}

/// Reject an ineligible `--stream` run with a specific reason (never silently fall
/// back to the whole-file path -- streaming is an explicit promise of bounded
/// memory and byte-identical output).
fn validate_stream_eligibility(cli: &Cli, mog: &Mog) -> Result<()> {
    if cli.json || cli.check || cli.dry_run || cli.report.is_some() || cli.diff {
        bail!("--stream produces plain streamed content; it is incompatible with --dry-run / --json / --check / --report / --diff (those need the whole output at once)");
    }
    if cli.parallel.is_some() {
        bail!("--stream and --parallel are different strategies (bounded-memory sequential vs whole-file multi-core); use one");
    }
    if !cli.encoding.eq_ignore_ascii_case("utf-8") {
        bail!(
            "--stream requires --encoding utf-8 (got '{}'); per-block auto-detection is not byte-identical to whole-file",
            cli.encoding
        );
    }
    let out_enc = cli.output_encoding.to_ascii_lowercase();
    if out_enc != "utf-8" && out_enc != "preserve" {
        bail!(
            "--stream requires --output-encoding utf-8 or preserve (got '{}')",
            cli.output_encoding
        );
    }
    if !mog::stream::pipeline_is_streamable(mog) {
        // Name the first offending step so the fix is obvious.
        let offender = mog.steps.iter().enumerate().find(|(_, s)| {
            s.only_lines_matching.is_some()
                || s.except_lines_matching.is_some()
                || s.scope.is_some()
                || !s
                    .action
                    .as_deref()
                    .map(mog::stream::action_is_streamable)
                    .unwrap_or(false)
        });
        if let Some((i, s)) = offender {
            let action = s.action.as_deref().unwrap_or("(no action)");
            let why = if s.only_lines_matching.is_some()
                || s.except_lines_matching.is_some()
                || s.scope.is_some()
            {
                "it is scoped (scoped steps re-split and rejoin lines)"
            } else {
                "it is whole-file, cross-line, EOL-normalizing, or pattern-based"
            };
            bail!(
                "--stream needs an all-streamable pipeline, but step {} ('{}') is not streamable: {}. Re-run without --stream.",
                i + 1,
                action,
                why
            );
        }
        bail!("--stream needs an all-streamable pipeline");
    }
    Ok(())
}

/// Target block size for streaming: read at least this many bytes before flushing
/// a block (cut back to the last newline so blocks end on line boundaries).
const STREAM_BLOCK_BYTES: usize = 256 * 1024;

/// Bounded-memory stdin -> stdout executor for an all-streamable pipeline. Reads
/// newline-delimited blocks, transforms each with the SAME engine as the
/// whole-file path, and writes as it goes. Memory is O(block + longest line), not
/// O(file). Output is byte-identical to the whole-file path (guaranteed by the
/// chunk-concat-safety of every step; see `mog::stream` + tests/streaming.rs).
///
/// Guardrails still apply, with the streaming caveat that earlier blocks may
/// already be written when a later block trips one: `--refuse-binary` is checked
/// per block; `--max-shrink` on the accumulated line counts at the end.
fn run_stdin_stream(
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    let mog_file = mog_for_file(mog, None)?;
    let mog = &*mog_file;
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    run_stream(
        &mut reader,
        &mut out,
        "stdin",
        mog,
        base_dir,
        lib_root,
        cli,
        sources,
    )
}

/// Stream a single input FILE to stdout in bounded memory (the file-input analogue
/// of `run_stdin_stream`). Same eligibility + byte-identity guarantees.
fn run_file_stream(
    path: &Path,
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    let mog_file = mog_for_file(mog, Some(path))?;
    let mog = &*mog_file;
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open '{}'", path.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    run_stream(
        &mut reader,
        &mut out,
        &path.display().to_string(),
        mog,
        base_dir,
        lib_root,
        cli,
        sources,
    )
}

/// Accumulated stats from a streaming run.
struct StreamStats {
    total_in: usize,
    total_out: usize,
    changed: bool,
}

/// Stream `reader` -> `out` in bounded memory, returning stats. Does NOT print a
/// summary (the caller does, since it knows the destination). Flushes `out`.
/// `label` names the source in messages ("stdin" or a path).
#[allow(clippy::too_many_arguments)]
fn stream_core(
    reader: &mut dyn std::io::Read,
    out: &mut dyn std::io::Write,
    label: &str,
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<StreamStats> {
    let limits = run_limits(cli);

    let mut buf: Vec<u8> = Vec::with_capacity(STREAM_BLOCK_BYTES * 2);
    let mut read_chunk = [0u8; 64 * 1024];

    let mut total_in = 0usize;
    let mut total_out = 0usize;
    let mut in_lines = 0usize;
    let mut out_lines = 0usize;
    let mut changed = false;

    // Process one newline-terminated block: transform and write, accumulating
    // stats. `block` always ends at a line boundary (or is the final remainder).
    let emit = |block: &[u8],
                out: &mut dyn std::io::Write,
                total_in: &mut usize,
                total_out: &mut usize,
                in_lines: &mut usize,
                out_lines: &mut usize,
                changed: &mut bool|
     -> Result<()> {
        if cli.refuse_binary && block.contains(&0) {
            bail!("{label} looks binary (contains a NUL byte); refused (drop --refuse-binary to force)");
        }
        let decoded = mog::encoding::decode(block, Some(&cli.encoding), cli.lossy)
            .with_context(|| format!("failed to decode a {label} block"))?;
        let input = decoded.text.as_str();
        let output = execute_guarded_with_sources(
            mog,
            input,
            base_dir,
            lib_root,
            &limits,
            sources.clone(),
            cli.seed,
        )?;
        *total_in += input.len();
        *total_out += output.len();
        *in_lines += input.lines().count();
        *out_lines += output.lines().count();
        if input != output {
            *changed = true;
        }
        let bytes = mog::encoding::encode(&output, &cli.output_encoding, &decoded)?;
        out.write_all(&bytes).context("failed to write output")?;
        Ok(())
    };

    loop {
        let n = reader
            .read(&mut read_chunk)
            .with_context(|| format!("failed to read {label}"))?;
        if n == 0 {
            break; // EOF
        }
        buf.extend_from_slice(&read_chunk[..n]);
        // Flush whole blocks: once the buffer is big enough, cut at its last
        // newline so the flushed block ends on a line boundary. A buffer past the
        // threshold with NO newline is one very long line -- keep reading.
        while buf.len() >= STREAM_BLOCK_BYTES {
            match buf.iter().rposition(|&b| b == b'\n') {
                Some(idx) => {
                    let block: Vec<u8> = buf.drain(..=idx).collect();
                    emit(
                        &block,
                        &mut *out,
                        &mut total_in,
                        &mut total_out,
                        &mut in_lines,
                        &mut out_lines,
                        &mut changed,
                    )?;
                }
                None => break,
            }
        }
    }
    // Final remainder (last line without a trailing newline, or a leftover under
    // the block threshold).
    if !buf.is_empty() {
        emit(
            &buf,
            &mut *out,
            &mut total_in,
            &mut total_out,
            &mut in_lines,
            &mut out_lines,
            &mut changed,
        )?;
    }

    // --max-shrink on accumulated line counts (mirrors guard_output). Streamable
    // actions never drop lines, so this is a backstop.
    if let Some(pct) = cli.max_shrink {
        if in_lines > 0 {
            let dropped = in_lines.saturating_sub(out_lines);
            let shrink = dropped as f64 / in_lines as f64 * 100.0;
            if shrink > pct {
                bail!(
                    "{label}: output dropped {shrink:.0}% of lines ({in_lines} -> {out_lines}), \
                     over --max-shrink {pct}%; refusing"
                );
            }
        }
    }

    out.flush().context("failed to flush output")?;
    Ok(StreamStats {
        total_in,
        total_out,
        changed,
    })
}

/// Stream to stdout (stdin or file -> stdout), printing the streamed summary.
#[allow(clippy::too_many_arguments)]
fn run_stream(
    reader: &mut dyn std::io::Read,
    out: &mut dyn std::io::Write,
    label: &str,
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    let s = stream_core(reader, out, label, mog, base_dir, lib_root, cli, sources)?;
    eprintln!(
        "{label}: {} -> {} bytes [UTF-8 -> UTF-8, streamed]{}",
        s.total_in,
        s.total_out,
        if s.changed { "" } else { " (unchanged)" }
    );
    Ok(0)
}

/// Stream a single file IN PLACE in bounded memory: transform into a temp file in
/// the same directory, then (only if anything changed) back up the original and
/// atomically replace it. On any error or guardrail trip the temp file is removed
/// and the original is left untouched -- a cleaner failure mode than the stdout
/// path, where earlier blocks are already written.
fn run_inplace_stream(
    path: &Path,
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    let mog_file = mog_for_file(mog, Some(path))?;
    let mog = &*mog_file;
    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open '{}'", path.display()))?;
    let mut reader = std::io::BufReader::new(file);

    // Temp file alongside the target (same filesystem => atomic rename).
    let tmp_path = {
        let mut p = path.as_os_str().to_owned();
        p.push(format!(".mog-tmp.{}", std::process::id()));
        std::path::PathBuf::from(p)
    };
    let label = path.display().to_string();

    let stats = {
        let tmp = std::fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create temp file '{}'", tmp_path.display()))?;
        let mut writer = std::io::BufWriter::new(tmp);
        match stream_core(
            &mut reader,
            &mut writer,
            &label,
            mog,
            base_dir,
            lib_root,
            cli,
            sources,
        ) {
            Ok(s) => s,
            Err(e) => {
                // Drop the writer, then remove the temp file; original untouched.
                drop(writer);
                let _ = std::fs::remove_file(&tmp_path);
                return Err(e);
            }
        }
        // writer (BufWriter<File>) drops here, closing the temp file before rename.
    };

    if !stats.changed {
        // Nothing changed: discard the temp copy, leave the original as-is (and
        // write no backup) -- matching the whole-file in-place path.
        let _ = std::fs::remove_file(&tmp_path);
        eprintln!(
            "{label}: {} -> {} bytes [streamed, in-place] (unchanged)",
            stats.total_in, stats.total_out
        );
        return Ok(0);
    }

    if let Some(suffix) = &cli.backup {
        let bak = append_suffix(path, suffix);
        std::fs::copy(path, &bak)
            .with_context(|| format!("failed to write backup '{}'", bak.display()))?;
    }
    std::fs::rename(&tmp_path, path)
        .with_context(|| format!("failed to replace '{}'", path.display()))?;
    eprintln!(
        "{label}: {} -> {} bytes [streamed, in-place]",
        stats.total_in, stats.total_out
    );
    Ok(0)
}

/// Stream a single input FILE into an output directory in bounded memory (the
/// `--stream` analogue of the whole-file `--out-dir` mode). Writes the result to
/// `dir/<input name>` atomically via a temp file in that directory, refusing to
/// clobber an existing destination unless `--overwrite` is set.
fn run_outdir_stream(
    path: &Path,
    dir: &Path,
    mog: &Mog,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    cli: &Cli,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<i32> {
    let mog_file = mog_for_file(mog, Some(path))?;
    let mog = &*mog_file;
    let name = path
        .file_name()
        .ok_or_else(|| anyhow!("input '{}' has no file name", path.display()))?;
    let dest = dir.join(name);
    if dest.exists() && !cli.overwrite {
        bail!(
            "refusing to overwrite '{}' (pass --overwrite to allow)",
            dest.display()
        );
    }
    std::fs::create_dir_all(dir)
        .with_context(|| format!("failed to create out-dir '{}'", dir.display()))?;

    let file = std::fs::File::open(path)
        .with_context(|| format!("failed to open '{}'", path.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let label = path.display().to_string();

    // Temp file in the destination directory (same filesystem => atomic rename).
    let tmp_path = {
        let mut p = dest.as_os_str().to_owned();
        p.push(format!(".mog-tmp.{}", std::process::id()));
        std::path::PathBuf::from(p)
    };

    let stats = {
        let tmp = std::fs::File::create(&tmp_path)
            .with_context(|| format!("failed to create temp file '{}'", tmp_path.display()))?;
        let mut writer = std::io::BufWriter::new(tmp);
        match stream_core(
            &mut reader,
            &mut writer,
            &label,
            mog,
            base_dir,
            lib_root,
            cli,
            sources,
        ) {
            Ok(s) => s,
            Err(e) => {
                drop(writer);
                let _ = std::fs::remove_file(&tmp_path);
                return Err(e);
            }
        }
    };

    std::fs::rename(&tmp_path, &dest)
        .with_context(|| format!("failed to write '{}'", dest.display()))?;
    eprintln!(
        "{label}: {} -> {} bytes [streamed, out-dir]{}",
        stats.total_in,
        stats.total_out,
        if stats.changed { "" } else { " (unchanged)" }
    );
    Ok(0)
}

/// The encoding name to report for output: the chosen label, or the input's
/// encoding when the choice is `preserve`.
fn out_encoding_label(choice: &str, decoded: &mog::encoding::Decoded) -> String {
    if choice.eq_ignore_ascii_case("preserve") {
        decoded.encoding.clone()
    } else {
        choice.to_string()
    }
}

fn decide_output_mode(cli: &Cli, num_inputs: usize, no_write: bool) -> Result<OutputMode> {
    if cli.in_place {
        Ok(OutputMode::InPlace)
    } else if let Some(dir) = &cli.out_dir {
        Ok(OutputMode::OutDir(dir.clone()))
    } else if num_inputs == 1 || no_write {
        // A single input goes to stdout; --dry-run/--check only report (write
        // nothing), so they never need an output mode even for many inputs.
        Ok(OutputMode::Stdout)
    } else {
        Err(anyhow!(
            "multiple inputs require an output mode: use --in-place or --out-dir"
        ))
    }
}

/// Expand each input arg as a glob (if it contains glob metacharacters) or a
/// literal path. De-duplicates while preserving order.
fn expand_inputs(inputs: &[String]) -> Result<Vec<PathBuf>> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for pat in inputs {
        if pat == "-" {
            // A "-" mixed with real files has no clean output semantics; the
            // pure stdin case is handled earlier.
            bail!("'-' (stdin) cannot be mixed with file inputs");
        }
        if pat.contains('*') || pat.contains('?') || pat.contains('[') {
            let entries =
                glob::glob(pat).with_context(|| format!("invalid glob pattern '{pat}'"))?;
            for entry in entries {
                let p = entry.with_context(|| format!("error reading glob '{pat}'"))?;
                if p.is_file() && seen.insert(p.clone()) {
                    out.push(p);
                }
            }
        } else {
            let p = PathBuf::from(pat);
            if seen.insert(p.clone()) {
                out.push(p);
            }
        }
    }
    Ok(out)
}

/// Drop any file whose path matches one of the `--exclude` globs.
fn apply_excludes(files: Vec<PathBuf>, excludes: &[String]) -> Result<Vec<PathBuf>> {
    if excludes.is_empty() {
        return Ok(files);
    }
    let patterns: Vec<glob::Pattern> = excludes
        .iter()
        .map(|g| glob::Pattern::new(g).with_context(|| format!("invalid --exclude glob '{g}'")))
        .collect::<Result<_>>()?;
    Ok(files
        .into_iter()
        .filter(|p| !patterns.iter().any(|pat| pat.matches_path(p)))
        .collect())
}

/// The result of processing a single file: a summary line, whether the content
/// changed, any bytes to emit to stdout (encoded content or a UTF-8 diff), and
/// any mog flags found in the output (to report for review).
struct Processed {
    summary: String,
    changed: bool,
    payload: Option<Vec<u8>>,
    flags: Vec<mog::actions::flag::Flag>,
    bytes_before: usize,
    bytes_after: usize,
    lines_added: usize,
    lines_removed: usize,
    /// The unified diff for a changed file, computed only when a report (`--json`
    /// or `--report`) will consume it. `None` for unchanged files or plain runs.
    diff: Option<String>,
}

/// Print a prominent, listed report of any mog flags to stderr.
fn report_flags(label: &str, flags: &[mog::actions::flag::Flag]) {
    if flags.is_empty() {
        return;
    }
    eprintln!();
    eprintln!("!! mog: {} flag(s) need review in {label}:", flags.len());
    // A breakdown per tag type, with a count and the first few line numbers.
    for &tag in mog::actions::flag::TAGS {
        let lines: Vec<usize> = flags
            .iter()
            .filter(|f| f.tag == tag)
            .map(|f| f.line)
            .collect();
        if lines.is_empty() {
            continue;
        }
        let shown: Vec<String> = lines.iter().take(5).map(|l| l.to_string()).collect();
        let more = if lines.len() > 5 {
            format!(" (+{} more)", lines.len() - 5)
        } else {
            String::new()
        };
        eprintln!(
            "   {:<6} {:>3}: lines {}{}",
            tag,
            lines.len(),
            shown.join(", "),
            more
        );
    }
    eprintln!();
}

/// Build the optional apply resource bounds (H2/H3) from the CLI flags. All off
/// unless the corresponding flag is set, so a normal run is unaffected.
fn run_limits(cli: &Cli) -> RunLimits {
    RunLimits {
        max_input_bytes: cli.max_input_bytes,
        max_steps: cli.max_steps,
        timeout: cli.timeout.map(std::time::Duration::from_secs_f64),
    }
}

/// Process a single file.
#[allow(clippy::too_many_arguments)]
fn process_one(
    mog: &Mog,
    path: &Path,
    mode: &OutputMode,
    cli: &Cli,
    base_dir: Option<&Path>,
    lib_root: Option<&Path>,
    budget: Option<&DiffBudget>,
    sources: &Arc<BTreeMap<String, Vec<String>>>,
) -> Result<Processed> {
    let bytes =
        std::fs::read(path).with_context(|| format!("failed to read '{}'", path.display()))?;
    let label = path.display().to_string();
    check_binary(cli, &bytes, &label)?;
    let decoded = mog::encoding::decode(&bytes, Some(&cli.encoding), cli.lossy)
        .with_context(|| format!("failed to decode '{}'", path.display()))?;
    let input = decoded.text.as_str();
    let mog_file = mog_for_file(mog, Some(path))?;
    let output = run_pipeline(&mog_file, input, base_dir, lib_root, cli, sources)?;
    guard_output(cli, input, &output, &label)?;

    // --check writes nothing, exactly like --dry-run.
    let dry = cli.dry_run || cli.check;

    let did_change = input != output;
    // Bytes actually written to disk / stdout, in the chosen output encoding.
    let out_bytes = mog::encoding::encode(&output, &cli.output_encoding, &decoded)?;
    let in_len = bytes.len();
    let out_len = out_bytes.len();
    // The line delta and per-file diff are only consumed by a report, and only
    // for the bounded sample: the diff budget gates the differ by file size and
    // caps how many diffs are computed, so a giant or late file is stats-only
    // (diff `None`, line counts left at 0). No budget = plain run, no diff work.
    let (lines_added, lines_removed, report_diff) = match budget {
        Some(b) if did_change => match b.sample(&path.display().to_string(), input, &output) {
            Some((diff, added, removed)) => (added, removed, Some(diff)),
            None => (0, 0, None),
        },
        _ => (0, 0, None),
    };

    // A diff (when requested and there is a change) is the stdout payload for any
    // mode; otherwise, in Stdout mode the transformed content is the payload.
    let payload = if cli.diff {
        if did_change {
            Some(unified_diff(&path.display().to_string(), input, &output).into_bytes())
        } else {
            None
        }
    } else if matches!(mode, OutputMode::Stdout) && !dry {
        Some(out_bytes.clone())
    } else {
        None
    };

    let summary = match mode {
        OutputMode::Stdout => format!(
            "{}: {} bytes -> {} bytes{}",
            path.display(),
            in_len,
            out_len,
            if did_change { "" } else { " (unchanged)" }
        ),
        OutputMode::InPlace => {
            if !dry && did_change {
                if let Some(suffix) = &cli.backup {
                    let bak = append_suffix(path, suffix);
                    std::fs::copy(path, &bak)
                        .with_context(|| format!("failed to write backup '{}'", bak.display()))?;
                }
                write_atomic(path, &out_bytes)?;
            }
            summary(path, in_len, out_len, did_change, dry)
        }
        OutputMode::OutDir(dir) => {
            let name = path
                .file_name()
                .ok_or_else(|| anyhow!("input '{}' has no file name", path.display()))?;
            let dest = dir.join(name);
            if !dry {
                if dest.exists() && !cli.overwrite {
                    bail!(
                        "output '{}' already exists (use --overwrite)",
                        dest.display()
                    );
                }
                std::fs::write(&dest, &out_bytes)
                    .with_context(|| format!("failed to write '{}'", dest.display()))?;
            }
            format!(
                "{} -> {}: {} -> {} bytes{}",
                path.display(),
                dest.display(),
                in_len,
                out_len,
                if did_change { "" } else { " (unchanged)" }
            )
        }
    };

    Ok(Processed {
        summary,
        changed: did_change,
        payload,
        flags: mog::actions::flag::find_flags(&output),
        bytes_before: in_len,
        bytes_after: out_len,
        lines_added,
        lines_removed,
        diff: report_diff,
    })
}

/// Build a plain unified diff (before -> after) labelled with `name`.
fn unified_diff(name: &str, before: &str, after: &str) -> String {
    let diff = TextDiff::from_lines(before, after);
    let mut ud = diff.unified_diff();
    ud.header(&format!("a/{name}"), &format!("b/{name}"));
    ud.to_string()
}

/// A unified diff (before -> after) plus its inserted/deleted line counts, from a
/// single `TextDiff` pass (so the report does not diff twice for the sample).
fn unified_diff_with_counts(name: &str, before: &str, after: &str) -> (String, usize, usize) {
    let diff = TextDiff::from_lines(before, after);
    let mut added = 0usize;
    let mut removed = 0usize;
    for ch in diff.iter_all_changes() {
        match ch.tag() {
            ChangeTag::Insert => added += 1,
            ChangeTag::Delete => removed += 1,
            ChangeTag::Equal => {}
        }
    }
    let mut ud = diff.unified_diff();
    ud.header(&format!("a/{name}"), &format!("b/{name}"));
    (ud.to_string(), added, removed)
}

/// A shared, thread-safe diff budget for the parallel processing pass. It bounds
/// BOTH the number of diffs computed (`max_diff_files`) and their total size
/// (`total_diff_bytes`), and gates the differ by file size
/// (`diff_skip_over_bytes`). So `similar` is never handed a giant file and the
/// diff compute cost is O(caps), not O(run size).
struct DiffBudget {
    caps: ReportCaps,
    files_done: AtomicUsize,
    bytes_done: AtomicUsize,
}

impl DiffBudget {
    fn new(caps: ReportCaps) -> Self {
        DiffBudget {
            caps,
            files_done: AtomicUsize::new(0),
            bytes_done: AtomicUsize::new(0),
        }
    }

    /// Produce the capped diff (plus its line counts) for one changed file, or
    /// `None` when the file is stats-only: too large to diff, past the diff-file
    /// count, or once the byte budget is spent. Under parallelism the sampled set
    /// is "up to the caps", not a fixed identity, which the report layer tolerates.
    fn sample(&self, name: &str, before: &str, after: &str) -> Option<(String, usize, usize)> {
        // Size gate: never hand a huge file to the differ (this is where a diff
        // could OOM or hang).
        if self.caps.skips_diff(before.len(), after.len()) {
            return None;
        }
        // File-count gate: only the first N changed files get a diff computed.
        // The over-increment past the cap is harmless (no diff is computed).
        if let Some(maxf) = self.caps.max_diff_files {
            if self.files_done.fetch_add(1, Ordering::Relaxed) >= maxf {
                return None;
            }
        }
        let (diff, added, removed) = unified_diff_with_counts(name, before, after);
        let diff = truncate_diff(diff, self.caps.max_diff_lines, self.caps.max_diff_bytes);
        // Byte-budget gate: once the running total is spent, later files are
        // stats-only.
        if let Some(total) = self.caps.total_diff_bytes {
            if self.bytes_done.fetch_add(diff.len(), Ordering::Relaxed) >= total {
                return None;
            }
        }
        Some((diff, added, removed))
    }
}

/// Resolve the effective report caps. Precedence (low to high): the aggressive
/// defaults, then `config.toml [report]`, then the CLI (`--report-full`, then the
/// granular `--report-limit`). Warns when a CLI override raises
/// `diff_skip_over_bytes`, where a huge file can genuinely OOM or hang.
fn resolve_caps(cli: &Cli, lib_root: Option<&Path>) -> Result<ReportCaps> {
    let mut caps = ReportCaps::default();
    let default_skip = caps.diff_skip_over_bytes;

    // config.toml [report], if any (this is a separate concern from config
    // `reports`, which the opt-in CLI still does not read).
    if let Some(root) = lib_root {
        let cfg = mog::config::load(root)?;
        if let Some(rc) = &cfg.report {
            rc.apply_to_caps(&mut caps);
        }
    }

    // --report-full lifts the sampling / enumeration / total-budget caps (but not
    // diff_skip_over_bytes, which still gates the differ).
    if cli.report_full {
        caps.max_diff_files = None;
        caps.max_files = None;
        caps.total_diff_bytes = None;
    }

    // --report-limit KEY=VALUE: granular, highest precedence.
    let mut raised_skip = false;
    for item in &cli.report_limit {
        let (k, v) = item
            .split_once('=')
            .ok_or_else(|| anyhow!("--report-limit expects KEY=VALUE, got '{item}'"))?;
        let k = k.trim();
        let cap = mog::report::parse_cap(v)?;
        caps.set_key(k, cap)?;
        if k == "diff_skip_over_bytes" {
            raised_skip = match (cap, default_skip) {
                (None, _) => true,           // lifted to unlimited
                (Some(n), Some(d)) => n > d, // raised above the default
                (Some(_), None) => false,
            };
        }
    }
    if raised_skip {
        eprintln!(
            "warning: report cap diff_skip_over_bytes raised above the default; \
             huge files may OOM or hang when diffed"
        );
    }
    Ok(caps)
}

fn summary(path: &Path, in_len: usize, out_len: usize, changed: bool, dry: bool) -> String {
    let state = match (changed, dry) {
        (true, true) => "would change",
        (true, false) => "changed",
        (false, _) => "unchanged",
    };
    format!(
        "{}: {} -> {} bytes ({state})",
        path.display(),
        in_len,
        out_len
    )
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}
