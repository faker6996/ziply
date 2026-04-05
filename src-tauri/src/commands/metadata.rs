use crate::{
    archive::rar_extractor_label,
    models::{AppOverview, ArchiveCapabilities},
};

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
        active_formats: ["zip", "tar", "tar.gz", "tgz", "tar.xz", "txz", "gz", "7z"],
        planned_formats: ["rar"],
    }
}

#[tauri::command]
pub(crate) fn archive_capabilities() -> ArchiveCapabilities {
    let rar_extractor = rar_extractor_label();
    ArchiveCapabilities {
        rar_extraction_available: rar_extractor.is_some(),
        rar_extractor_label: rar_extractor,
    }
}
