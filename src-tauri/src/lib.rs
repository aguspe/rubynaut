mod commands;

use tauri_plugin_dialog::DialogExt;

#[tauri::command]
async fn pick_folder(app: tauri::AppHandle, title: String) -> Result<Option<String>, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .set_title(&title)
        .pick_folder(move |folder| {
            let path = folder.map(|f| f.to_string());
            let _ = sender.send(path);
        });

    receiver.await.map_err(|e| format!("Dialog error: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::platform::detect_platform,
            commands::ruby::get_installed_rubies,
            commands::ruby::get_available_rubies,
            commands::ruby::get_active_version,
            commands::ruby::set_global_version,
            commands::ruby::set_local_version,
            commands::ruby::uninstall_ruby,
            commands::ruby::install_ruby,
            commands::ruby::get_shell_hook,
            commands::ruby::check_shell_hook,
            commands::ruby::install_shell_hook,
            commands::ruby::get_gems_for_version,
            commands::ruby::install_gem,
            commands::ruby::uninstall_gem,
            commands::ruby::get_project_gems,
            commands::ruby::bundle_install,
            commands::ruby::scan_project,
            commands::ruby::get_tracked_projects,
            commands::ruby::add_tracked_project,
            commands::ruby::remove_tracked_project,
            commands::ruby::get_config,
            commands::ruby::set_wizard_completed,
            commands::ruby::get_latest_stable_version,
            commands::doctor::run_doctor,
            commands::doctor::run_doctor_fix,
            pick_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
