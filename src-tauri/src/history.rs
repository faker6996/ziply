use std::{fs, path::PathBuf};

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    archive::path_to_string,
    models::{ArchiveHistoryEntry, ArchiveJobEvent, ShellIntent},
};

fn archive_history_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data directory: {error}"))?;
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "failed to create app data directory {}: {error}",
            directory.display()
        )
    })?;
    Ok(directory.join("archive-history.json"))
}

pub(crate) fn load_archive_history(app: &AppHandle) -> Result<Vec<ArchiveHistoryEntry>, String> {
    let path = archive_history_file_path(app)?;
    if !path.exists() {
        return Ok(vec![]);
    }

    let bytes = fs::read(&path)
        .map_err(|error| format!("failed to read archive history {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse archive history file {}: {error}",
            path.display()
        )
    })
}

pub(crate) fn persist_archive_history(
    app: &AppHandle,
    history: &[ArchiveHistoryEntry],
) -> Result<(), String> {
    let path = archive_history_file_path(app)?;
    let bytes = serde_json::to_vec_pretty(history)
        .map_err(|error| format!("failed to serialize archive history: {error}"))?;
    fs::write(&path, bytes)
        .map_err(|error| format!("failed to write archive history {}: {error}", path.display()))
}

pub(crate) fn append_archive_history(
    app: &AppHandle,
    entry: ArchiveHistoryEntry,
) -> Result<(), String> {
    let mut history = load_archive_history(app)?;
    history.insert(0, entry);
    history.truncate(24);
    persist_archive_history(app, &history)
}

pub(crate) fn emit_archive_job_event(app: &AppHandle, event: ArchiveJobEvent) {
    let _ = app.emit("archive-job", event);
}

pub(crate) fn emit_shell_intent(app: &AppHandle, intent: ShellIntent) {
    let _ = app.emit("shell-intent", intent);
}

pub(crate) fn summarize_paths(paths: &[PathBuf]) -> String {
    match paths {
        [] => String::new(),
        [single] => path_to_string(single),
        [first, ..] => format!("{} and {} more", first.display(), paths.len() - 1),
    }
}

pub(crate) fn unix_timestamp_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(crate) fn archive_history_id() -> String {
    format!("job-{}", unix_timestamp_ms())
}

pub(crate) fn archive_job_id() -> String {
    format!("live-{}", unix_timestamp_ms())
}
