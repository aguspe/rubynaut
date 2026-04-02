mod commands;

use commands::doctor::{run_doctor, run_doctor_fix};
use commands::platform::detect_platform;
use commands::ruby::{
    add_tracked_project, bundle_install, check_shell_hook, get_active_version,
    get_available_rubies, get_gems_for_version, get_installed_rubies, get_project_gems,
    get_shell_hook, get_tracked_projects, install_gem, install_ruby, install_shell_hook,
    remove_tracked_project, scan_project, set_global_version, set_local_version, uninstall_gem,
    uninstall_ruby,
};

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
            detect_platform,
            get_installed_rubies,
            get_available_rubies,
            get_active_version,
            set_global_version,
            set_local_version,
            uninstall_ruby,
            install_ruby,
            get_shell_hook,
            check_shell_hook,
            install_shell_hook,
            get_gems_for_version,
            install_gem,
            uninstall_gem,
            run_doctor,
            run_doctor_fix,
            pick_folder,
            scan_project,
            get_tracked_projects,
            add_tracked_project,
            remove_tracked_project,
            bundle_install,
            get_project_gems,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
