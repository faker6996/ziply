use tauri::State;

use crate::{
    history::{load_archive_history, persist_archive_history},
    models::{ArchiveHistoryEntry, PendingShellIntents, ShellIntent, ShellIntegrationStatus},
    shell::{current_shell_integration_status, install_current_shell_integration},
};

#[tauri::command]
pub(crate) fn consume_shell_intents(
    pending_shell_intents: State<'_, PendingShellIntents>,
) -> Result<Vec<ShellIntent>, String> {
    let mut intents = pending_shell_intents
        .0
        .lock()
        .map_err(|_| "failed to lock pending shell intents".to_string())?;
    Ok(std::mem::take(&mut *intents))
}

#[tauri::command]
pub(crate) fn shell_integration_status() -> ShellIntegrationStatus {
    current_shell_integration_status()
}

#[tauri::command]
pub(crate) fn install_shell_integration() -> Result<ShellIntegrationStatus, String> {
    install_current_shell_integration()?;
    Ok(current_shell_integration_status())
}

#[tauri::command]
pub(crate) fn get_archive_history(
    app: tauri::AppHandle,
) -> Result<Vec<ArchiveHistoryEntry>, String> {
    load_archive_history(&app)
}

#[tauri::command]
pub(crate) fn clear_archive_history(app: tauri::AppHandle) -> Result<(), String> {
    persist_archive_history(&app, &[])
}
