use rubynaut_core::PlatformInfo;

#[tauri::command]
pub fn detect_platform() -> Result<PlatformInfo, String> {
    rubynaut_core::detect_platform()
}
