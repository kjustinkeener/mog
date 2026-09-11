mod commands;
mod install;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Installer entry points, handled before any window is created so they run
    // headless and clean.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--uninstall") {
        install::run_uninstall();
        return;
    }
    if args.iter().any(|a| a == "--silent") {
        install::run_silent();
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            // Install-card mode: the installer is a dedicated frameless,
            // transparent window (its own rounded card floating on the desktop),
            // not the 1200x800 app window shrunk down. The main window stays
            // hidden and is closed so the app quits when the card is dismissed.
            // The real app boots next launch from the install dir. The frontend
            // routes to the Installer on the `#installer` hash (and on
            // setup_state().needs_setup as a fallback).
            use tauri::Manager as _;
            if install::needs_setup() {
                let built = tauri::WebviewWindowBuilder::new(
                    app,
                    "installer",
                    tauri::WebviewUrl::App("index.html#installer".into()),
                )
                .title("Install Mog Studio")
                .inner_size(460.0, 560.0)
                .resizable(false)
                .decorations(false)
                .transparent(true)
                .center()
                .visible(false)
                .build();
                match built {
                    Ok(win) => {
                        // decorations(false) has been observed not to stick on
                        // Windows from the builder alone; re-assert it.
                        let _ = win.set_decorations(false);
                        let _ = win.show();
                        let _ = win.set_focus();
                        // Drop the hidden main window so closing the card exits.
                        if let Some(main) = app.handle().get_webview_window("main") {
                            let _ = main.close();
                        }
                    }
                    Err(_) => {
                        // Fallback: reuse the main window as a framed card so a
                        // failed installer-window build never leaves no window.
                        if let Some(win) = app.handle().get_webview_window("main") {
                            let _ = win.set_resizable(false);
                            let _ = win.set_size(tauri::LogicalSize::new(460.0, 560.0));
                            let _ = win.center();
                            let _ = win.show();
                        }
                    }
                }
            } else if let Some(win) = app.handle().get_webview_window("main") {
                let _ = win.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_actions,
            commands::list_categories,
            commands::run_transform,
            commands::run_batch,
            commands::market_search,
            commands::market_show,
            commands::mog_update,
            commands::user_mog_path,
            commands::append_log,
            commands::log_path,
            commands::run_mog_test,
            commands::flag_check,
            commands::run_batch_report,
            commands::diff_file,
            install::setup_state,
            install::perform_install,
            install::launch_installed_and_exit,
            install::open_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
