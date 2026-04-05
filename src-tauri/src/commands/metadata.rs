use crate::models::{AppOverview, ArchiveCapabilities};

#[tauri::command]
pub(crate) fn app_overview() -> AppOverview {
    AppOverview {
        name: "Ziply",
        tagline: "Compress and extract archives from one desktop workspace.",
        supported_platforms: ["macOS", "Windows", "Linux"],
        focus_areas: [
            "Create archives from files and folders",
            "Extract common archive formats",
            "Keep one workflow across three desktop operating systems",
        ],
        active_formats: [
            "zip", "tar", "tar.gz", "tgz", "tar.bz2", "tbz2", "tar.xz", "txz", "xz", "bz2", "gz",
            "7z",
        ],
        planned_formats: ["rar"],
    }
}

#[tauri::command]
pub(crate) fn archive_capabilities() -> ArchiveCapabilities {
    ArchiveCapabilities {
        native_archive_only: true,
        unsupported_formats: ["rar"],
    }
}
