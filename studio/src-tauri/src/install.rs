//! Self-installer for Mog Studio: the download is one portable `mog-studio.exe`
//! that carries the engine inside it and installs the whole product per-user, no
//! admin required.
//!
//! Direction (owner, 2026-09-02): **Studio embeds mog.exe** (not the reverse).
//! The `--profile dist` engine binary is gzip-embedded at build time (see
//! `build.rs`); on install we extract it beside Studio and hand PATH / library /
//! MCP / Add-Remove provisioning to `mog.exe install` itself, so the engine's
//! install logic stays the single source of truth. Studio only adds its own
//! Start-Menu shortcut and Add/Remove entry on top.
//!
//! First launch from outside the install dir (`%LOCALAPPDATA%\mog`, shared with
//! the engine so the two binaries sit together) shows the install card instead of
//! the app. Clicking Install:
//!   1. copies `mog-studio.exe` into the install dir,
//!   2. extracts the embedded `mog.exe` next to it,
//!   3. runs `mog.exe install` (windowless) for PATH + library + MCP + engine ARP,
//!   4. writes a Studio Start-Menu shortcut and a Studio Add/Remove entry,
//!   5. relaunches the installed Studio and exits.
//!
//! `--silent` does all of that headlessly (scripted installs). `--uninstall`
//! removes the Studio shortcut + ARP entry, then runs `mog.exe uninstall`, which
//! tears down the engine and deletes the whole install dir (Studio included). The
//! mog library under `%APPDATA%\mog` is user data and is left in place.

use std::path::{Path, PathBuf};

use serde::Serialize;

/// The gzip-compressed dist `mog.exe`, embedded at build time. Empty in dev
/// builds that had no engine to embed (see `build.rs`).
const ENGINE_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mog.exe.gz"));

/// The install dir: `%LOCALAPPDATA%\mog`, shared with the engine so `mog.exe` and
/// `mog-studio.exe` live side by side. Matches `mog::install::install_dir()`.
const APP_DIR: &str = "mog";
/// Studio's own executable name inside the install dir.
const STUDIO_EXE: &str = "mog-studio.exe";
/// The engine executable name (what we extract and what the ARP shortcut targets).
const ENGINE_EXE: &str = "mog.exe";
/// Display name on the card, the shortcut, and Add/Remove Programs.
const DISPLAY_NAME: &str = "Mog Studio";
/// Studio's Add/Remove Programs subkey (the engine uses `mog`; this is separate).
const ARP_KEY: &str = "mog-studio";

/// `%LOCALAPPDATA%\mog`, the per-user install dir. `None` if LOCALAPPDATA is unset.
pub fn install_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .filter(|v| !v.is_empty())
        .map(|base| PathBuf::from(base).join(APP_DIR))
}

/// The installed Studio path (`<install_dir>\mog-studio.exe`).
fn installed_studio(dir: &Path) -> PathBuf {
    dir.join(STUDIO_EXE)
}

/// Are we running from inside the install dir (i.e. already installed)?
/// Canonicalizes both sides so casing / `..` / short-path forms don't false-miss.
fn is_installed() -> bool {
    match (std::env::current_exe(), install_dir()) {
        (Ok(cur), Some(dir)) => {
            let cur = cur.canonicalize().unwrap_or(cur);
            let dir = dir.canonicalize().unwrap_or(dir);
            cur.starts_with(&dir)
        }
        _ => false,
    }
}

/// Show the install card? True only in release builds run from outside the
/// install dir. False in dev (so `cargo tauri dev` isn't interrupted) and once
/// installed.
pub fn needs_setup() -> bool {
    !cfg!(debug_assertions) && !is_installed()
}

// ---- Frontend command surface -------------------------------------------------

#[derive(Serialize)]
pub struct SetupState {
    /// Show the install card.
    pub needs_setup: bool,
    /// Running from the install dir already.
    pub installed: bool,
    /// This build carries an embedded engine (false in dev builds).
    pub has_engine: bool,
    pub version: String,
    pub build_date: String,
    /// Shown on the card.
    pub install_dir: String,
}

#[tauri::command]
pub fn setup_state() -> SetupState {
    SetupState {
        needs_setup: needs_setup(),
        installed: is_installed(),
        has_engine: !ENGINE_GZ.is_empty(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        build_date: env!("MOG_STUDIO_BUILD_DATE").to_string(),
        install_dir: install_dir()
            .map(|d| d.display().to_string())
            .unwrap_or_default(),
    }
}

/// Perform the install. Returns the installed Studio exe path for the frontend to
/// relaunch. `desktop_shortcut` adds a Desktop `.lnk` in addition to Start Menu.
#[tauri::command]
pub fn perform_install(
    desktop_shortcut: bool,
    register_mcp: bool,
    add_to_path: bool,
) -> Result<String, String> {
    do_install(desktop_shortcut, register_mcp, add_to_path).map(|p| p.display().to_string())
}

/// Open a URL in the user's default browser (Windows `start`). Used by the
/// installer's footer links; only http(s) and mailto URLs are allowed, and a URL
/// with whitespace is refused, so a stray value cannot launch an arbitrary program.
#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let scheme_ok =
        url.starts_with("https://") || url.starts_with("http://") || url.starts_with("mailto:");
    if !scheme_ok || url.split_whitespace().count() != 1 {
        return Err("refusing to open a non-http(s)/mailto URL".into());
    }
    let mut c = std::process::Command::new("cmd");
    // The empty "" is start's window-title arg, so a quoted URL is not eaten.
    c.args(["/c", "start", "", &url]);
    hide_window(&mut c);
    c.spawn().map_err(|e| format!("open url: {e}"))?;
    Ok(())
}

/// Spawn the installed Studio and exit this (loose) process.
#[tauri::command]
pub fn launch_installed_and_exit(app: tauri::AppHandle, exe: String) {
    let mut c = std::process::Command::new(&exe);
    hide_window(&mut c);
    let _ = c.spawn();
    app.exit(0);
}

// ---- Pre-Tauri entry points (called from lib.rs::run before any window) --------

/// `mog-studio.exe --silent`: install headlessly for scripted use, then exit.
pub fn run_silent() {
    match do_install(false, true, true) {
        Ok(p) => println!("Mog Studio installed to {}", p.display()),
        Err(e) => {
            eprintln!("install failed: {e}");
            std::process::exit(1);
        }
    }
    std::process::exit(0);
}

/// `mog-studio.exe --uninstall`: remove Studio's shortcut + ARP entry, then hand
/// off to `mog.exe uninstall` (which removes the engine and deletes the whole
/// install dir, this binary included). Runs headless, before any window.
pub fn run_uninstall() {
    let _ = remove_start_menu_shortcut(DISPLAY_NAME);
    let _ = remove_desktop_shortcut(DISPLAY_NAME);
    let _ = remove_arp_entry();

    if let Some(dir) = install_dir() {
        let engine = dir.join(ENGINE_EXE);
        let mut delegated = false;
        if engine.is_file() {
            let mut c = std::process::Command::new(&engine);
            c.arg("uninstall");
            hide_window(&mut c);
            // Fire and forget: the engine schedules a detached rmdir of the dir
            // we're running from, so we must exit promptly. We cannot wait for
            // its exit code, but a failed spawn is worth falling back on.
            delegated = c.spawn().is_ok();
        }
        if !delegated {
            // Either a partial install with no engine, or the engine would not
            // start. Do the engine's cleanup ourselves so an uninstall never
            // leaves a dangling PATH entry pointing at a deleted directory.
            let _ = remove_from_user_path(&dir);
            let _ = schedule_dir_delete(&dir);
        }
    }
    std::process::exit(0);
}

// ---- The install itself -------------------------------------------------------

fn do_install(
    desktop_shortcut: bool,
    register_mcp: bool,
    add_to_path: bool,
) -> Result<PathBuf, String> {
    let dir = install_dir().ok_or("could not determine an install dir (LOCALAPPDATA unset)")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("create install dir: {e}"))?;

    // 1. Copy Studio in (guard against copying onto ourselves).
    let target = installed_studio(&dir);
    let src = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    if !same_file(&src, &target) {
        std::fs::copy(&src, &target).map_err(|e| format!("copy studio exe: {e}"))?;
    }

    // 2. Extract the embedded engine beside Studio.
    let engine = extract_engine(&dir)?;

    // 3. Let the engine do PATH / library / MCP / engine ARP (windowless).
    run_engine_install(&engine, register_mcp, add_to_path)?;

    // 4. Studio's own shortcut(s) + Add/Remove entry. Everything below is
    //    best-effort on purpose: the product is installed and working at this
    //    point, and reporting a hard failure would leave the user with a
    //    complete install they were told to distrust (and, if the Add/Remove
    //    write is the thing that failed, no obvious way to remove it).
    if let Some(sm) = start_menu_dir() {
        let _ = std::fs::create_dir_all(&sm);
        let _ = create_shortcut(&sm.join(format!("{DISPLAY_NAME}.lnk")), &target, &dir);
    }
    if desktop_shortcut {
        if let Some(desk) = desktop_dir() {
            let _ = create_shortcut(&desk.join(format!("{DISPLAY_NAME}.lnk")), &target, &dir);
        }
    }
    let _ = register_uninstall(&dir, &target);

    Ok(target)
}

/// Decompress the embedded engine into `<dir>\mog.exe`. Errors if this build has
/// no embedded engine (a dev build).
fn extract_engine(dir: &Path) -> Result<PathBuf, String> {
    if ENGINE_GZ.is_empty() {
        return Err("this Studio build has no embedded engine (built without \
                    MOG_DIST_EXE / binaries/mog-dist.exe); rebuild with the dist \
                    mog.exe staged"
            .into());
    }
    use std::io::Read;
    let mut decoder = flate2::read::GzDecoder::new(ENGINE_GZ);
    let mut bytes = Vec::new();
    decoder
        .read_to_end(&mut bytes)
        .map_err(|e| format!("decompress embedded engine: {e}"))?;
    let dest = dir.join(ENGINE_EXE);
    std::fs::write(&dest, &bytes).map_err(|e| format!("write '{}': {e}", dest.display()))?;
    Ok(dest)
}

/// Run `<install_dir>\mog.exe install` windowless, so the engine provisions PATH,
/// the mog library, MCP registration, and its own Add/Remove entry.
fn run_engine_install(engine: &Path, register_mcp: bool, add_to_path: bool) -> Result<(), String> {
    let mut c = std::process::Command::new(engine);
    c.arg("install");
    if !register_mcp {
        c.arg("--no-mcp");
    }
    if !add_to_path {
        c.arg("--no-path");
    }
    hide_window(&mut c);
    let status = c
        .status()
        .map_err(|e| format!("run '{} install': {e}", engine.display()))?;
    if !status.success() {
        return Err(format!(
            "'{} install' exited with {status} (engine provisioning failed)",
            engine.display()
        ));
    }
    Ok(())
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

// ---- Windows shell work: shortcuts + Add/Remove Programs ----------------------

#[cfg(windows)]
fn hide_window(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}
#[cfg(not(windows))]
fn hide_window(_cmd: &mut std::process::Command) {}

#[cfg(windows)]
fn start_menu_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| {
        PathBuf::from(a)
            .join("Microsoft")
            .join("Windows")
            .join("Start Menu")
            .join("Programs")
    })
}
#[cfg(not(windows))]
fn start_menu_dir() -> Option<PathBuf> {
    None
}

fn desktop_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("Desktop"))
}

#[cfg(windows)]
fn create_shortcut(lnk: &Path, target: &Path, working_dir: &Path) -> Result<(), String> {
    let esc = |p: &Path| p.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
         $s = $w.CreateShortcut('{lnk}'); \
         $s.TargetPath = '{target}'; \
         $s.WorkingDirectory = '{wd}'; \
         $s.IconLocation = '{target},0'; \
         $s.Description = '{desc}'; \
         $s.Save()",
        lnk = esc(lnk),
        target = esc(target),
        wd = esc(working_dir),
        desc = DISPLAY_NAME,
    );
    let mut c = std::process::Command::new("powershell");
    c.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    hide_window(&mut c);
    let status = c.status().map_err(|e| format!("create shortcut: {e}"))?;
    if !status.success() {
        return Err("creating the shortcut failed".into());
    }
    Ok(())
}
#[cfg(not(windows))]
fn create_shortcut(_lnk: &Path, _target: &Path, _working_dir: &Path) -> Result<(), String> {
    Ok(())
}

fn remove_start_menu_shortcut(name: &str) -> Result<bool, String> {
    remove_lnk(start_menu_dir(), name)
}
fn remove_desktop_shortcut(name: &str) -> Result<bool, String> {
    remove_lnk(desktop_dir(), name)
}
fn remove_lnk(dir: Option<PathBuf>, name: &str) -> Result<bool, String> {
    let Some(dir) = dir else { return Ok(false) };
    let lnk = dir.join(format!("{name}.lnk"));
    if !lnk.exists() {
        return Ok(false);
    }
    std::fs::remove_file(&lnk).map_err(|e| format!("remove '{}': {e}", lnk.display()))?;
    Ok(true)
}

#[cfg(windows)]
fn arp_key_path() -> String {
    format!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{ARP_KEY}")
}

#[cfg(windows)]
fn register_uninstall(dir: &Path, exe: &Path) -> Result<(), String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(arp_key_path())
        .map_err(|e| format!("create Add/Remove key: {e}"))?;
    let set = |k: &str, v: &str| key.set_value(k, &v.to_string());
    set("DisplayName", DISPLAY_NAME).map_err(|e| e.to_string())?;
    set("DisplayVersion", env!("CARGO_PKG_VERSION")).map_err(|e| e.to_string())?;
    set("Publisher", "Justin Keener").map_err(|e| e.to_string())?;
    set("DisplayIcon", &exe.to_string_lossy()).map_err(|e| e.to_string())?;
    set("InstallLocation", &dir.to_string_lossy()).map_err(|e| e.to_string())?;
    set(
        "UninstallString",
        &format!("\"{}\" --uninstall", exe.to_string_lossy()),
    )
    .map_err(|e| e.to_string())?;
    set(
        "QuietUninstallString",
        &format!("\"{}\" --uninstall", exe.to_string_lossy()),
    )
    .map_err(|e| e.to_string())?;
    set("InstallDate", &install_date_stamp()).map_err(|e| e.to_string())?;
    key.set_value("NoModify", &1u32)
        .map_err(|e| e.to_string())?;
    key.set_value("NoRepair", &1u32)
        .map_err(|e| e.to_string())?;
    // EstimatedSize is in KiB and is what Settings shows in the size column.
    let kib = (dir_size(dir) / 1024).min(u64::from(u32::MAX)) as u32;
    key.set_value("EstimatedSize", &kib)
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[cfg(not(windows))]
fn register_uninstall(_dir: &Path, _exe: &Path) -> Result<(), String> {
    Ok(())
}

/// Total bytes of everything in the install dir, for `EstimatedSize`.
fn dir_size(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|e| match e.metadata() {
            Ok(m) if m.is_dir() => dir_size(&e.path()),
            Ok(m) => m.len(),
            Err(_) => 0,
        })
        .sum()
}

/// Today as the `YYYYMMDD` string Add/Remove Programs expects. Computed from the
/// epoch directly rather than pulling in a date crate for one field.
fn install_date_stamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}{m:02}{d:02}")
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's civil_from_days).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Strip `dir` from the user's PATH. Normally the engine's own uninstall does
/// this; we only reach here when the engine is missing or will not start. This
/// edits the registry without broadcasting a settings change, so open programs
/// keep the stale value until they are restarted.
#[cfg(windows)]
fn remove_from_user_path(dir: &Path) -> Result<(), String> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let env = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .map_err(|e| format!("open Environment: {e}"))?;
    let current: String = env
        .get_value("Path")
        .map_err(|e| format!("read Path: {e}"))?;
    let want = dir.to_string_lossy().to_lowercase();
    let want = want.trim_end_matches('\\');
    let kept: Vec<&str> = current
        .split(';')
        .filter(|p| {
            let p = p.trim().trim_end_matches('\\').to_lowercase();
            !p.is_empty() && p != want
        })
        .collect();
    let before = current.split(';').filter(|p| !p.trim().is_empty()).count();
    if kept.len() == before {
        return Ok(()); // nothing of ours was there
    }
    env.set_value("Path", &kept.join(";"))
        .map_err(|e| format!("write Path: {e}"))
}
#[cfg(not(windows))]
fn remove_from_user_path(_dir: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn remove_arp_entry() -> Result<bool, String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.delete_subkey_all(arp_key_path()) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(format!("delete Add/Remove key: {e}")),
    }
}
#[cfg(not(windows))]
fn remove_arp_entry() -> Result<bool, String> {
    Ok(false)
}

/// Fallback used only when uninstall finds no engine to delegate to: schedule a
/// detached delete of the install dir (we're running from inside it).
#[cfg(windows)]
fn schedule_dir_delete(dir: &Path) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    // Retry: we are running from inside this directory, and the engine may be
    // too, so the first attempts routinely fail on a locked exe. A folder left
    // behind is what makes Windows report the uninstall as unsuccessful.
    let cmd = format!(
        "for /l %i in (1,1,10) do (ping 127.0.0.1 -n 3 >nul & rmdir /s /q \"{}\" 2>nul &          if not exist \"{}\" exit)",
        dir.display(),
        dir.display()
    );
    std::process::Command::new("cmd")
        .args(["/c", &cmd])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("spawn detached delete: {e}"))?;
    Ok(())
}
#[cfg(not(windows))]
fn schedule_dir_delete(dir: &Path) -> Result<(), String> {
    std::fs::remove_dir_all(dir).ok();
    Ok(())
}
