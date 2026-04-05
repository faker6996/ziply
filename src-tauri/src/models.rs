use std::{path::Path, sync::Mutex};

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AppOverview {
    pub(crate) name: &'static str,
    pub(crate) tagline: &'static str,
    pub(crate) supported_platforms: Vec<&'static str>,
    pub(crate) focus_areas: Vec<&'static str>,
    pub(crate) active_formats: Vec<&'static str>,
    pub(crate) planned_formats: Vec<&'static str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveCapabilities {
    pub(crate) native_archive_only: bool,
    pub(crate) unsupported_formats: Vec<&'static str>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CompressRequest {
    pub(crate) source_paths: Vec<String>,
    pub(crate) destination_path: String,
    pub(crate) format: String,
    #[serde(default)]
    pub(crate) conflict_policy: Option<String>,
    #[serde(default)]
    pub(crate) password: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExtractRequest {
    pub(crate) archive_path: String,
    pub(crate) destination_directory: String,
    #[serde(default)]
    pub(crate) conflict_policy: Option<String>,
    #[serde(default)]
    pub(crate) password: Option<String>,
    #[serde(default)]
    pub(crate) selected_entries: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivePreviewRequest {
    pub(crate) archive_path: String,
    #[serde(default)]
    pub(crate) password: Option<String>,
    #[serde(default)]
    pub(crate) limit: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveActionResult {
    pub(crate) operation: &'static str,
    pub(crate) format: &'static str,
    pub(crate) output_path: String,
    pub(crate) message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivePreviewEntry {
    pub(crate) path: String,
    pub(crate) kind: &'static str,
    pub(crate) size: Option<u64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchivePreviewResult {
    pub(crate) format: &'static str,
    pub(crate) total_entries: usize,
    pub(crate) visible_entries: Vec<ArchivePreviewEntry>,
    pub(crate) hidden_entry_count: usize,
    pub(crate) note: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveHistoryEntry {
    pub(crate) id: String,
    pub(crate) operation: String,
    pub(crate) format: String,
    pub(crate) source_summary: String,
    pub(crate) output_path: String,
    pub(crate) timestamp_ms: u128,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShellIntent {
    pub(crate) action: String,
    pub(crate) paths: Vec<String>,
    pub(crate) auto_run: bool,
    pub(crate) destination_path: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ShellIntegrationStatus {
    pub(crate) platform: &'static str,
    pub(crate) supported: bool,
    pub(crate) can_install: bool,
    pub(crate) installed: bool,
    pub(crate) mode: &'static str,
    pub(crate) note: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArchiveJobEvent {
    pub(crate) job_id: String,
    pub(crate) operation: String,
    pub(crate) format: String,
    pub(crate) stage: String,
    pub(crate) status: String,
    pub(crate) message: String,
    pub(crate) progress: u8,
    pub(crate) source_summary: String,
    pub(crate) output_path: Option<String>,
    pub(crate) timestamp_ms: u128,
}

#[derive(Clone, Copy)]
pub(crate) enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    Xz,
    Bz2,
    Gz,
    SevenZip,
    Rar,
}

impl ArchiveFormat {
    pub(crate) fn from_compress_input(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "zip" => Ok(Self::Zip),
            "tar" => Ok(Self::Tar),
            "tar.gz" | "tgz" => Ok(Self::TarGz),
            "tar.bz2" | "tbz2" => Ok(Self::TarBz2),
            "tar.xz" | "txz" => Ok(Self::TarXz),
            "xz" => Ok(Self::Xz),
            "bz2" => Ok(Self::Bz2),
            "gz" => Ok(Self::Gz),
            "7z" => Ok(Self::SevenZip),
            other => Err(format!("unsupported archive format: {other}")),
        }
    }

    pub(crate) fn detect_from_archive_path(path: &Path) -> Result<Self, String> {
        let lower_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .ok_or_else(|| "archive path must end with a valid file name".to_string())?;

        if lower_name.ends_with(".tar.gz") || lower_name.ends_with(".tgz") {
            return Ok(Self::TarGz);
        }

        if lower_name.ends_with(".tar.bz2") || lower_name.ends_with(".tbz2") {
            return Ok(Self::TarBz2);
        }

        if lower_name.ends_with(".tar.xz") || lower_name.ends_with(".txz") {
            return Ok(Self::TarXz);
        }

        if lower_name.ends_with(".tar") {
            return Ok(Self::Tar);
        }

        if lower_name.ends_with(".zip") {
            return Ok(Self::Zip);
        }

        if lower_name.ends_with(".7z") {
            return Ok(Self::SevenZip);
        }

        if lower_name.ends_with(".rar")
            || is_rar_volume_name(&lower_name)
            || is_old_style_rar_segment_name(&lower_name)
        {
            return Ok(Self::Rar);
        }

        if lower_name.ends_with(".xz") {
            return Ok(Self::Xz);
        }

        if lower_name.ends_with(".bz2") {
            return Ok(Self::Bz2);
        }

        if lower_name.ends_with(".gz") {
            return Ok(Self::Gz);
        }

        Err("unsupported archive extension. Ziply currently supports zip, tar, tar.gz, tgz, tar.bz2, tbz2, tar.xz, txz, xz, bz2, gz, 7z, and rar extraction.".to_string())
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Zip => "zip",
            Self::Tar => "tar",
            Self::TarGz => "tar.gz",
            Self::TarBz2 => "tar.bz2",
            Self::TarXz => "tar.xz",
            Self::Xz => "xz",
            Self::Bz2 => "bz2",
            Self::Gz => "gz",
            Self::SevenZip => "7z",
            Self::Rar => "rar",
        }
    }

    pub(crate) fn preferred_suffix(self) -> &'static str {
        match self {
            Self::Zip => ".zip",
            Self::Tar => ".tar",
            Self::TarGz => ".tar.gz",
            Self::TarBz2 => ".tar.bz2",
            Self::TarXz => ".tar.xz",
            Self::Xz => ".xz",
            Self::Bz2 => ".bz2",
            Self::Gz => ".gz",
            Self::SevenZip => ".7z",
            Self::Rar => ".rar",
        }
    }
}

fn is_rar_volume_name(name: &str) -> bool {
    let Some(start) = name.find(".part") else {
        return false;
    };
    let Some(number) = name[start + 5..].strip_suffix(".rar") else {
        return false;
    };
    !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit())
}

fn is_old_style_rar_segment_name(name: &str) -> bool {
    let Some(extension) = name.rsplit('.').next() else {
        return false;
    };
    extension.len() == 3
        && extension.starts_with('r')
        && extension[1..].chars().all(|ch| ch.is_ascii_digit())
}

#[derive(Clone, Copy)]
pub(crate) enum ConflictPolicy {
    KeepBoth,
    Overwrite,
    Stop,
}

impl ConflictPolicy {
    pub(crate) fn from_input(value: Option<&str>) -> Result<Self, String> {
        match value.map(|item| item.trim().to_ascii_lowercase()) {
            None => Ok(Self::KeepBoth),
            Some(value) if value.is_empty() => Ok(Self::KeepBoth),
            Some(value) if value == "keepboth" || value == "keep-both" || value == "keep_both" => {
                Ok(Self::KeepBoth)
            }
            Some(value) if value == "overwrite" => Ok(Self::Overwrite),
            Some(value) if value == "stop" || value == "error" || value == "cancel" => {
                Ok(Self::Stop)
            }
            Some(other) => Err(format!("unsupported conflict policy: {other}")),
        }
    }
}

pub(crate) struct PendingShellIntents(pub(crate) Mutex<Vec<ShellIntent>>);
