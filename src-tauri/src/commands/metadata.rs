use crate::models::{AppOverview, ArchiveCapabilities};

#[tauri::command]
pub(crate) fn app_overview() -> AppOverview {
    AppOverview {
        name: "Ziply",
        tagline: "Compress and extract archives from one desktop workspace.",
        supported_platforms: vec!["macOS", "Windows", "Linux"],
        focus_areas: vec![
            "Create archives from files and folders",
            "Extract common archive formats",
            "Keep one workflow across three desktop operating systems",
        ],
        active_formats: vec![
            "zip", "tar", "tar.gz", "tgz", "tar.bz2", "tbz2", "tar.xz", "txz", "xz", "bz2", "gz",
            "7z", "rar",
        ],
        planned_formats: vec![],
    }
}

#[tauri::command]
pub(crate) fn archive_capabilities() -> ArchiveCapabilities {
    ArchiveCapabilities {
        native_archive_only: true,
        unsupported_formats: vec![],
    }
}
