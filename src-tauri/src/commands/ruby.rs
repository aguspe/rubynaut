use rubynaut_core::{
    GemInfo, ProjectGem, ProjectScanResult, RubyVersion, ShellHookStatus,
};

#[tauri::command]
pub fn get_installed_rubies() -> Result<Vec<RubyVersion>, String> {
    rubynaut_core::get_installed_rubies()
}

#[tauri::command]
pub async fn get_available_rubies() -> Result<Vec<RubyVersion>, String> {
    rubynaut_core::get_available_rubies().await
}

#[tauri::command]
pub fn get_active_version() -> Result<Option<String>, String> {
    rubynaut_core::get_active_version()
}

#[tauri::command]
pub fn set_global_version(version: String) -> Result<(), String> {
    rubynaut_core::set_global_version(version)
}

#[tauri::command]
pub fn set_local_version(path: String, version: String) -> Result<(), String> {
    rubynaut_core::set_local_version(path, version)
}

#[tauri::command]
pub fn uninstall_ruby(version: String) -> Result<(), String> {
    rubynaut_core::uninstall_ruby(version)
}

#[tauri::command]
pub async fn install_ruby(version: String, window: tauri::Window) -> Result<(), String> {
    use tauri::Emitter;
    let progress: rubynaut_core::ProgressCallback =
        Box::new(move |stage: &str, percent: u8, message: &str| {
            let _ = window.emit(
                "install-progress",
                serde_json::json!({
                    "stage": stage,
                    "percent": percent,
                    "message": message
                }),
            );
        });
    rubynaut_core::install_ruby(version, Some(progress)).await
}

#[tauri::command]
pub fn get_gems_for_version(version: String) -> Result<Vec<GemInfo>, String> {
    rubynaut_core::get_gems_for_version(version)
}

#[tauri::command]
pub async fn install_gem(
    ruby_version: String,
    gem_name: String,
    gem_version: Option<String>,
) -> Result<String, String> {
    rubynaut_core::install_gem(ruby_version, gem_name, gem_version).await
}

#[tauri::command]
pub async fn uninstall_gem(ruby_version: String, gem_name: String) -> Result<String, String> {
    rubynaut_core::uninstall_gem(ruby_version, gem_name).await
}

#[tauri::command]
pub fn get_project_gems(project_path: String) -> Result<Vec<ProjectGem>, String> {
    rubynaut_core::get_project_gems(project_path)
}

#[tauri::command]
pub async fn bundle_install(
    ruby_version: String,
    project_path: String,
    window: tauri::Window,
) -> Result<String, String> {
    use tauri::Emitter;
    let progress: rubynaut_core::ProgressCallback =
        Box::new(move |stage: &str, _percent: u8, message: &str| {
            let _ = window.emit(
                "bundle-progress",
                serde_json::json!({
                    "path": stage,
                    "status": stage,
                    "output": message
                }),
            );
        });
    rubynaut_core::bundle_install(ruby_version, project_path, Some(progress)).await
}

#[tauri::command]
pub fn scan_project(path: String) -> Result<ProjectScanResult, String> {
    rubynaut_core::scan_project(path)
}

#[tauri::command]
pub fn get_tracked_projects() -> Result<Vec<ProjectScanResult>, String> {
    rubynaut_core::get_tracked_projects()
}

#[tauri::command]
pub fn add_tracked_project(path: String) -> Result<Vec<ProjectScanResult>, String> {
    rubynaut_core::add_tracked_project(path)
}

#[tauri::command]
pub fn remove_tracked_project(path: String) -> Result<Vec<ProjectScanResult>, String> {
    rubynaut_core::remove_tracked_project(path)
}

#[tauri::command]
pub fn check_shell_hook() -> Result<Vec<ShellHookStatus>, String> {
    rubynaut_core::check_shell_hook()
}

#[tauri::command]
pub fn install_shell_hook(shell: String) -> Result<String, String> {
    rubynaut_core::install_shell_hook(shell)
}

#[tauri::command]
pub fn get_shell_hook(shell: String) -> Result<String, String> {
    rubynaut_core::get_shell_hook(shell)
}
