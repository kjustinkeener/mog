//! `mog install` / `mog uninstall`: provision (or remove) a per-user install of
//! mog, no admin required.
//!
//! `mog install` makes a downloaded, loose `mog.exe` into a real install:
//!   1. copy this binary into the per-user install dir (`%LOCALAPPDATA%\mog`),
//!   2. put that dir on the user PATH (so `mog` works in every new shell),
//!   3. seed the mog library and register the MCP server (by re-running the
//!      *installed* binary's `mog setup`, so MCP points at the installed path),
//!   4. write an Add/Remove Programs entry whose uninstall calls `mog uninstall`.
//!
//! `mog install studio` additionally downloads the Studio GUI from the registry
//! (a signed `studio.json`, mirroring the engine track of `mog update`) and drops
//! a Start-Menu shortcut. It requires internet.
//!
//! `mog uninstall` reverses all of it: PATH entry, Add/Remove key, MCP
//! registration, shortcuts, and finally the install dir itself (via a detached
//! command, since the running binary usually lives inside the dir it deletes).
//! The mog library under `%APPDATA%\mog` is user data and is left in place.
//!
//! The registry mechanics reuse the App-Patterns Self-Installer shapes: install
//! dir = `%LOCALAPPDATA%\<App>` (per-user, no UAC), shortcuts via WScript.Shell,
//! the Add/Remove key under HKCU, and the detached ping+rmdir delete-self trick.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::market_index::{self, sha256_hex};

/// The product name used for the install dir, shortcuts, and the ARP entry.
const APP: &str = "mog";
/// The display name shown in Add/Remove Programs and on shortcuts.
const DISPLAY_NAME: &str = "Mog";
/// The Studio binary's filename inside the install dir.
const STUDIO_EXE: &str = "mog-studio.exe";

/// The per-user install dir for binaries: `%LOCALAPPDATA%\mog` on Windows, or
/// `$XDG_DATA_HOME/mog` (`~/.local/share/mog`) elsewhere. `None` when no base is
/// resolvable. This is the single choke point for "where the binaries live".
pub fn install_dir() -> Option<PathBuf> {
    if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA")
            .filter(|v| !v.is_empty())
            .map(|base| PathBuf::from(base).join(APP))
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("share"))
            })
            .map(|base| base.join(APP))
    }
}

/// The installed engine binary path (`<install_dir>\mog.exe`).
fn installed_exe(dir: &Path) -> PathBuf {
    dir.join(if cfg!(windows) { "mog.exe" } else { "mog" })
}

/// True when the running binary already lives inside the install dir (so an
/// install is a no-op copy, and Studio's install card should not appear).
pub fn is_installed() -> bool {
    let (Some(dir), Ok(exe)) = (install_dir(), std::env::current_exe()) else {
        return false;
    };
    match (dir.canonicalize(), exe.canonicalize()) {
        (Ok(d), Ok(e)) => e.starts_with(d),
        _ => false,
    }
}

/// Two paths that resolve to the same file (canonicalized). Used to guard against
/// copying the binary onto itself when `mog install` is run from the install dir.
fn same_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Copy the running binary into the install dir (creating it). Returns the
/// installed path. A no-op when already running from that path.
fn place_self(dir: &Path, print: bool) -> Result<PathBuf> {
    let src = std::env::current_exe().context("locate the running mog binary")?;
    let dest = installed_exe(dir);
    if same_file(&src, &dest) {
        return Ok(dest);
    }
    if !print {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("create install dir '{}'", dir.display()))?;
        std::fs::copy(&src, &dest)
            .with_context(|| format!("copy '{}' -> '{}'", src.display(), dest.display()))?;
    }
    Ok(dest)
}

/// `mog install` (bare): provision the engine. Copies self into the install dir,
/// adds it to PATH, seeds the library + registers MCP, and writes the ARP entry.
pub fn install_engine(no_mcp: bool, print: bool, json_out: bool) -> Result<i32> {
    let dir = install_dir()
        .ok_or_else(|| anyhow!("could not determine an install dir (LOCALAPPDATA unset)"))?;
    let exe = place_self(&dir, print)?;

    let path_added = add_to_user_path(&dir, print)?;
    let arp = write_arp_entry(&dir, &exe, print)?;

    // Seed library + register MCP by running the INSTALLED binary's `mog setup`,
    // so `current_exe()` inside setup is the installed path (correct MCP command).
    let setup = run_installed_setup(&exe, no_mcp, print)?;

    let summary = json!({
        "install_dir": dir.display().to_string(),
        "exe": exe.display().to_string(),
        "path_added": path_added,
        "arp": arp,
        "setup": setup,
        "wrote": !print,
    });
    if json_out {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else {
        print_engine_human(&dir, &exe, path_added, print);
    }
    Ok(0)
}

/// Run `<installed>\mog.exe setup [--no-mcp]`, inheriting stdio. Best-effort under
/// `print`: reports the command instead of running it.
fn run_installed_setup(exe: &Path, no_mcp: bool, print: bool) -> Result<serde_json::Value> {
    let mut args = vec!["setup".to_string()];
    if no_mcp {
        args.push("--no-mcp".to_string());
    }
    if print {
        return Ok(
            json!({ "ran": false, "would_run": format!("{} {}", exe.display(), args.join(" ")) }),
        );
    }
    let mut cmd = std::process::Command::new(exe);
    cmd.args(&args);
    // When launched from a GUI (e.g. Mog Studio's installer) there is no console
    // to inherit, so a console-subsystem child would flash its own window. Suppress
    // it in that case; a real terminal (CLI `mog install`) keeps setup's output.
    #[cfg(windows)]
    {
        use std::io::IsTerminal as _;
        if !std::io::stdout().is_terminal() {
            hide_window(&mut cmd);
        }
    }
    let status = cmd
        .status()
        .with_context(|| format!("run '{} setup'", exe.display()))?;
    Ok(json!({ "ran": true, "ok": status.success() }))
}

/// `mog install studio`: download the Studio GUI from the registry and install it
/// beside the engine, with a Start-Menu shortcut. Requires internet.
pub fn install_studio(print: bool, json_out: bool) -> Result<i32> {
    let dir = install_dir()
        .ok_or_else(|| anyhow!("could not determine an install dir (LOCALAPPDATA unset)"))?;
    // Ensure the engine is present first (so Studio has its sibling mog.exe).
    let exe = place_self(&dir, print)?;
    if !is_installed() && !print {
        // We just copied ourselves in but are still running from elsewhere; make
        // sure PATH + ARP exist too, so `install studio` from a loose exe yields a
        // complete install rather than a stray Studio.
        let _ = add_to_user_path(&dir, print)?;
        let _ = write_arp_entry(&dir, &exe, print)?;
        let _ = run_installed_setup(&exe, false, print)?;
    }

    let base = crate::market_client::market_base_url().context(
        "`mog install studio` needs a registry URL (set MOG_MARKET_URL or use a build with a \
         default registry); Studio is fetched over the internet",
    )?;
    let plat = fetch_studio_platform(&base)?;

    let dest = dir.join(STUDIO_EXE);
    if print {
        println!("would download Studio {} -> {}", plat.path, dest.display());
    } else {
        let bytes = fetch_bytes(&base, &plat.path)?;
        let got = sha256_hex(&bytes);
        if got != plat.sha256 {
            bail!(
                "hash mismatch for the downloaded Studio binary: expected {} got {} (refusing)",
                plat.sha256,
                got
            );
        }
        std::fs::write(&dest, &bytes).with_context(|| format!("write '{}'", dest.display()))?;
    }

    let shortcut = create_start_menu_shortcut(DISPLAY_NAME, &dest, &dir, print)?;
    let summary = json!({
        "install_dir": dir.display().to_string(),
        "studio_exe": dest.display().to_string(),
        "shortcut": shortcut,
        "version": plat.version,
        "wrote": !print,
    });
    if json_out {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if print {
        println!(
            "mog install studio (dry run): would install Studio into {}",
            dir.display()
        );
    } else {
        println!("Installed Mog Studio into {}", dir.display());
        println!("  Start-Menu shortcut: {shortcut}");
        println!("  Studio runs the installed mog.exe next to it (no separate engine).");
    }
    Ok(0)
}

/// `mog uninstall`: reverse the install. Removes PATH entry, ARP key, MCP
/// registration, shortcuts, then deletes the install dir via a detached command.
/// Leaves the mog library under `%APPDATA%\mog` (user data) intact.
pub fn uninstall(print: bool, json_out: bool) -> Result<i32> {
    let dir = install_dir()
        .ok_or_else(|| anyhow!("could not determine an install dir (LOCALAPPDATA unset)"))?;

    let path_removed = remove_from_user_path(&dir, print)?;
    let arp_removed = remove_arp_entry(print)?;
    // Best-effort MCP unregister (never fails uninstall).
    let mcp = if print {
        json!({ "ran": false })
    } else {
        match crate::mcp::install::uninstall(None, false, false) {
            Ok(_) => json!({ "ran": true }),
            Err(e) => json!({ "ran": true, "note": e.to_string() }),
        }
    };
    let shortcut_removed = remove_start_menu_shortcut(DISPLAY_NAME, print)?;

    // Delete the install dir last, detached, so the running exe (inside it) exits
    // before the dir is removed.
    let dir_deleted = if print {
        false
    } else {
        schedule_dir_delete(&dir)?;
        true
    };

    let summary = json!({
        "install_dir": dir.display().to_string(),
        "path_removed": path_removed,
        "arp_removed": arp_removed,
        "mcp": mcp,
        "shortcut_removed": shortcut_removed,
        "dir_delete_scheduled": dir_deleted,
        "library_kept": true,
    });
    if json_out {
        println!("{}", serde_json::to_string_pretty(&summary)?);
    } else if print {
        println!("mog uninstall (dry run): would remove PATH entry, Add/Remove key, MCP, shortcuts, and delete {}", dir.display());
    } else {
        println!("Uninstalling Mog.");
        println!(
            "  PATH entry: {}",
            if path_removed {
                "removed"
            } else {
                "not present"
            }
        );
        println!(
            "  Add/Remove Programs: {}",
            if arp_removed {
                "removed"
            } else {
                "not present"
            }
        );
        println!(
            "  shortcuts: {}",
            if shortcut_removed { "removed" } else { "none" }
        );
        println!("  install dir: scheduled for deletion ({})", dir.display());
        println!(
            "  mog library kept at %APPDATA%\\mog (delete it by hand to remove your mogs)."
        );
    }
    Ok(0)
}

fn print_engine_human(dir: &Path, exe: &Path, path_added: bool, print: bool) {
    if print {
        println!("mog install (dry run: nothing written or run)");
    } else {
        println!("mog installed to {}", dir.display());
    }
    println!("  engine: {}", exe.display());
    if path_added {
        println!(
            "  PATH: added {} (open a NEW terminal to pick it up)",
            dir.display()
        );
    } else {
        println!("  PATH: {} already present", dir.display());
    }
    println!("  Add/Remove Programs: entry written (uninstall with `mog uninstall`)");
    println!();
    println!(
        "Next: run `mog install studio` to add the GUI, or `mog market list` to browse mogs."
    );
}

// ---- The signed Studio manifest (mirrors selfupdate::EngineManifest) ----------

/// One platform's Studio binary in `studio.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StudioPlatform {
    path: String,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StudioManifest {
    #[serde(default)]
    version: String,
    /// Keyed by `"<os>-<arch>"`, e.g. `"windows-x86_64"`.
    platforms: std::collections::BTreeMap<String, StudioPlatform>,
}

/// A resolved Studio download for the current platform.
struct ResolvedStudio {
    path: String,
    sha256: String,
    version: String,
}

/// Fetch + verify `studio.json` and pick this platform's entry.
fn fetch_studio_platform(base: &str) -> Result<ResolvedStudio> {
    let bytes = fetch_bytes(base, "studio.json")
        .context("fetch studio.json from the registry (Studio is not published there yet?)")?;
    let vk = resolve_pubkey()?;
    let sig = String::from_utf8(fetch_bytes(base, "studio.json.sig")?)
        .context("studio.json.sig is not valid utf-8")?;
    market_index::verify(&vk, &bytes, sig.trim())
        .context("Studio manifest signature verification failed (refusing to trust it)")?;
    let manifest: StudioManifest = serde_json::from_slice(&bytes).context("parse studio.json")?;
    let key = crate::selfupdate::platform_key();
    let plat = manifest
        .platforms
        .get(&key)
        .ok_or_else(|| anyhow!("no Studio build published for {key}"))?;
    Ok(ResolvedStudio {
        path: plat.path.clone(),
        sha256: plat.sha256.clone(),
        version: manifest.version,
    })
}

/// The verifying key for signed manifests: `MOG_MARKET_PUBKEY` (mirror/dev) else
/// the key compiled into this build. Mirrors `selfupdate::resolve_pubkey`.
fn resolve_pubkey() -> Result<ed25519_dalek::VerifyingKey> {
    if let Ok(k) = std::env::var("MOG_MARKET_PUBKEY") {
        if !k.trim().is_empty() {
            return market_index::verifying_key_from_b64(k.trim());
        }
    }
    market_index::embedded_public_key().ok_or_else(|| {
        anyhow!("this mog build has no marketplace public key (and MOG_MARKET_PUBKEY is unset); cannot verify a Studio download")
    })
}

/// Fetch bytes from the registry: HTTP(S) via ureq, or a local filesystem path
/// (mirrors `selfupdate::fetch_bytes`; the local case makes E2E testing trivial).
fn fetch_bytes(base: &str, rel: &str) -> Result<Vec<u8>> {
    let is_http = base.starts_with("http://") || base.starts_with("https://");
    if is_http {
        let url = format!(
            "{}/{}",
            base.trim_end_matches('/'),
            rel.trim_start_matches('/')
        );
        let resp = ureq::get(&url)
            .call()
            .map_err(|e| anyhow!("GET {url}: {e}"))?;
        let mut buf = Vec::new();
        std::io::Read::read_to_end(&mut resp.into_reader(), &mut buf)
            .with_context(|| format!("read body of {url}"))?;
        Ok(buf)
    } else {
        let path = Path::new(base).join(rel.trim_start_matches('/'));
        std::fs::read(&path).with_context(|| format!("read '{}'", path.display()))
    }
}

// ---- Windows registry: user PATH + Add/Remove Programs ------------------------

#[cfg(windows)]
fn add_to_user_path(dir: &Path, print: bool) -> Result<bool> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let dir_s = dir.to_string_lossy().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("open HKCU\\Environment")?;
    let current: String = env.get_value("Path").unwrap_or_default();
    if path_contains(&current, &dir_s) {
        return Ok(false);
    }
    if !print {
        let joined = if current.trim().is_empty() {
            dir_s.clone()
        } else {
            format!("{};{}", current.trim_end_matches(';'), dir_s)
        };
        // Preserve REG_EXPAND_SZ semantics: user Path commonly holds %VARS%.
        let val = winreg::RegValue {
            bytes: to_utf16_bytes(&joined),
            vtype: winreg::enums::RegType::REG_EXPAND_SZ,
        };
        env.set_raw_value("Path", &val)
            .context("write HKCU\\Environment\\Path")?;
        broadcast_env_change();
    }
    Ok(true)
}

#[cfg(windows)]
fn remove_from_user_path(dir: &Path, print: bool) -> Result<bool> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE};
    use winreg::RegKey;

    let dir_s = dir.to_string_lossy().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let env = hkcu
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("open HKCU\\Environment")?;
    let current: String = env.get_value("Path").unwrap_or_default();
    if !path_contains(&current, &dir_s) {
        return Ok(false);
    }
    if !print {
        let kept: Vec<&str> = current
            .split(';')
            .filter(|p| !p.is_empty() && !path_eq(p, &dir_s))
            .collect();
        let joined = kept.join(";");
        let val = winreg::RegValue {
            bytes: to_utf16_bytes(&joined),
            vtype: winreg::enums::RegType::REG_EXPAND_SZ,
        };
        env.set_raw_value("Path", &val)
            .context("write HKCU\\Environment\\Path")?;
        broadcast_env_change();
    }
    Ok(true)
}

/// Case-insensitive, separator-aware membership test for a PATH string.
fn path_contains(path: &str, dir: &str) -> bool {
    path.split(';').any(|p| path_eq(p, dir))
}

fn path_eq(a: &str, b: &str) -> bool {
    a.trim()
        .trim_end_matches('\\')
        .eq_ignore_ascii_case(b.trim().trim_end_matches('\\'))
}

/// Encode a string as UTF-16LE bytes with a trailing NUL, as the registry wants.
#[cfg(windows)]
fn to_utf16_bytes(s: &str) -> Vec<u8> {
    let mut v: Vec<u8> = s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    v.push(0);
    v.push(0);
    v
}

/// Tell running processes (Explorer, shells) that the environment changed, so new
/// shells inherit the updated PATH without a logoff. Best-effort via PowerShell
/// P/Invoke (avoids adding the `windows`/`winapi` FFI crate for one call).
#[cfg(windows)]
fn broadcast_env_change() {
    let script = r#"
$sig = '[DllImport("user32.dll", SetLastError=true, CharSet=CharSet.Auto)] public static extern IntPtr SendMessageTimeout(IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam, uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);'
$t = Add-Type -MemberDefinition $sig -Name Win32SendMessageTimeout -Namespace Win32Functions -PassThru
$HWND_BROADCAST = [IntPtr]0xffff; $WM_SETTINGCHANGE = 0x1a
$r = [UIntPtr]::Zero
[void]$t::SendMessageTimeout($HWND_BROADCAST, $WM_SETTINGCHANGE, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$r)
"#;
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    hide_window(&mut cmd);
    let _ = cmd.status();
}

/// Spawn without a console window flashing (the updater is windowless; install
/// must be too). CREATE_NO_WINDOW keeps the child's console hidden.
#[cfg(windows)]
fn hide_window(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(windows)]
fn arp_key_path() -> String {
    format!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\{APP}")
}

#[cfg(windows)]
fn write_arp_entry(dir: &Path, exe: &Path, print: bool) -> Result<serde_json::Value> {
    if print {
        return Ok(json!({ "wrote": false, "key": format!("HKCU\\{}", arp_key_path()) }));
    }
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(arp_key_path())
        .context("create Add/Remove Programs key")?;
    key.set_value("DisplayName", &DISPLAY_NAME)?;
    key.set_value("DisplayVersion", &env!("CARGO_PKG_VERSION"))?;
    key.set_value("Publisher", &"Justin Keener")?;
    key.set_value("DisplayIcon", &exe.to_string_lossy().to_string())?;
    key.set_value("InstallLocation", &dir.to_string_lossy().to_string())?;
    key.set_value(
        "UninstallString",
        &format!("\"{}\" uninstall", exe.to_string_lossy()),
    )?;
    key.set_value("NoModify", &1u32)?;
    key.set_value("NoRepair", &1u32)?;
    Ok(json!({ "wrote": true, "key": format!("HKCU\\{}", arp_key_path()) }))
}

#[cfg(windows)]
fn remove_arp_entry(print: bool) -> Result<bool> {
    if print {
        return Ok(true);
    }
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    match hkcu.delete_subkey_all(arp_key_path()) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(anyhow!("delete Add/Remove Programs key: {e}")),
    }
}

// ---- Shortcuts (WScript.Shell via PowerShell) ---------------------------------

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

#[cfg(windows)]
fn create_start_menu_shortcut(
    name: &str,
    target: &Path,
    working_dir: &Path,
    print: bool,
) -> Result<String> {
    let dir = start_menu_dir().ok_or_else(|| anyhow!("APPDATA unset; cannot place a shortcut"))?;
    let lnk = dir.join(format!("{name}.lnk"));
    if print {
        return Ok(lnk.display().to_string());
    }
    std::fs::create_dir_all(&dir).ok();
    // Single-quote-escape every path for the PowerShell string literals.
    let esc = |p: &Path| p.to_string_lossy().replace('\'', "''");
    let script = format!(
        "$w = New-Object -ComObject WScript.Shell; \
         $s = $w.CreateShortcut('{lnk}'); \
         $s.TargetPath = '{target}'; \
         $s.WorkingDirectory = '{wd}'; \
         $s.IconLocation = '{target},0'; \
         $s.Save()",
        lnk = esc(&lnk),
        target = esc(target),
        wd = esc(working_dir),
    );
    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    hide_window(&mut cmd);
    let status = cmd
        .status()
        .context("run WScript.Shell to create shortcut")?;
    if !status.success() {
        bail!("creating the Start-Menu shortcut failed");
    }
    Ok(lnk.display().to_string())
}

#[cfg(windows)]
fn remove_start_menu_shortcut(name: &str, print: bool) -> Result<bool> {
    let Some(dir) = start_menu_dir() else {
        return Ok(false);
    };
    let lnk = dir.join(format!("{name}.lnk"));
    if !lnk.exists() {
        return Ok(false);
    }
    if !print {
        std::fs::remove_file(&lnk).with_context(|| format!("remove '{}'", lnk.display()))?;
    }
    Ok(true)
}

/// Delete the install dir after this process exits, via a detached command that
/// waits briefly then removes the tree. Needed because the running binary lives
/// inside the dir it is deleting.
#[cfg(windows)]
fn schedule_dir_delete(dir: &Path) -> Result<()> {
    use std::os::windows::process::CommandExt;
    // ping as a portable ~2s sleep, then rmdir the tree.
    let cmd = format!(
        "ping 127.0.0.1 -n 3 >nul & rmdir /s /q \"{}\"",
        dir.display()
    );
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("cmd")
        .args(["/c", &cmd])
        .creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW)
        .spawn()
        .context("spawn detached uninstall command")?;
    Ok(())
}

// ---- Non-Windows stubs (this is a Windows product; keep it cross-compiling) ---

#[cfg(not(windows))]
fn add_to_user_path(_dir: &Path, _print: bool) -> Result<bool> {
    Ok(false)
}
#[cfg(not(windows))]
fn remove_from_user_path(_dir: &Path, _print: bool) -> Result<bool> {
    Ok(false)
}
#[cfg(not(windows))]
fn write_arp_entry(_dir: &Path, _exe: &Path, _print: bool) -> Result<serde_json::Value> {
    Ok(json!({ "wrote": false, "note": "Add/Remove Programs is Windows-only" }))
}
#[cfg(not(windows))]
fn remove_arp_entry(_print: bool) -> Result<bool> {
    Ok(false)
}
#[cfg(not(windows))]
fn create_start_menu_shortcut(
    _name: &str,
    _target: &Path,
    _working_dir: &Path,
    _print: bool,
) -> Result<String> {
    Ok(String::new())
}
#[cfg(not(windows))]
fn remove_start_menu_shortcut(_name: &str, _print: bool) -> Result<bool> {
    Ok(false)
}
#[cfg(not(windows))]
fn schedule_dir_delete(dir: &Path) -> Result<()> {
    std::fs::remove_dir_all(dir).ok();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_membership_is_case_and_slash_insensitive() {
        let p = r"C:\Users\x\AppData\Local\mog;C:\Windows";
        assert!(path_contains(p, r"c:\users\x\appdata\local\mog"));
        assert!(path_contains(p, r"C:\Users\x\AppData\Local\mog\"));
        assert!(!path_contains(p, r"C:\Users\x\AppData\Local\other"));
    }

    #[test]
    fn install_dir_uses_localappdata_on_windows() {
        if cfg!(windows) {
            std::env::set_var("LOCALAPPDATA", r"C:\tmp\LA");
            assert_eq!(install_dir().unwrap(), PathBuf::from(r"C:\tmp\LA\mog"));
        }
    }
}
