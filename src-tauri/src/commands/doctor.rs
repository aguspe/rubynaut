use rubynaut_core::{DiagnosticResult};

#[tauri::command]
pub fn run_doctor() -> Result<Vec<DiagnosticResult>, String> {
    rubynaut_core::run_doctor()
}

#[tauri::command]
pub async fn run_doctor_fix(command: String) -> Result<String, String> {
    rubynaut_core::run_doctor_fix(command).await
}
