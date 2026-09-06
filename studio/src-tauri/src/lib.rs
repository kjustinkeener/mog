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
            // Install-card mode: shrink to card size, no resize. The frontend
            // routes to the Installer on setup_state().needs_setup; the real app
            // boots next launch from the install dir.
            if install::needs_setup() {
                use tauri::Manager as _;
                if let Some(win) = app.handle().get_webview_window("main") {
                    let _ = win.set_resizable(false);
                    let _ = win.set_size(tauri::LogicalSize::new(460.0, 460.0));
                    let _ = win.center();
                    let _ = win.show();
                }
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
