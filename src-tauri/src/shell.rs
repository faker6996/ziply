use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::{fs, process::Command};

use tauri::{AppHandle, Manager};

use crate::{
    archive::{is_supported_archive_path, path_to_string},
    history::emit_shell_intent,
    models::{PendingShellIntents, ShellIntent, ShellIntegrationStatus},
};

#[cfg(target_os = "linux")]
use crate::archive::find_command;

pub(crate) fn collect_launch_shell_intents() -> Vec<ShellIntent> {
    let args: Vec<String> = env::args_os()
        .skip(1)
        .map(|value| value.to_string_lossy().into_owned())
        .collect();

    if args.is_empty() {
        return vec![];
    }

    match args[0].as_str() {
        "--extract" => args
            .get(1)
            .map(|path| shell_extract_intent(path, false))
            .into_iter()
            .collect(),
        "--extract-here" => args
            .get(1)
            .map(|path| shell_extract_intent(path, true))
            .into_iter()
            .collect(),
        "--compress" => {
            let paths = args
                .into_iter()
                .skip(1)
                .filter(|path| !path.trim().is_empty())
                .collect::<Vec<_>>();
            if paths.is_empty() {
                vec![]
            } else {
                vec![ShellIntent {
                    action: "compress".to_string(),
                    paths,
                    auto_run: false,
                    destination_path: None,
                }]
            }
        }
        _ => args
            .iter()
            .filter(|value| is_supported_archive_path(Path::new(value)))
            .map(|path| shell_extract_intent(path, false))
            .collect(),
    }
}

pub(crate) fn shell_extract_intent(path: &str, auto_run: bool) -> ShellIntent {
    let archive_path = PathBuf::from(path);
    ShellIntent {
        action: if auto_run {
            "extract-here".to_string()
        } else {
            "extract".to_string()
        },
        paths: vec![path.to_string()],
        auto_run,
        destination_path: default_shell_extract_destination(&archive_path, auto_run)
            .map(|value| path_to_string(&value)),
    }
}

pub(crate) fn store_shell_intents(app: &AppHandle, intents: Vec<ShellIntent>) -> Result<(), String> {
    if intents.is_empty() {
        return Ok(());
    }

    let pending_shell_intents = app.state::<PendingShellIntents>();
    let mut pending = pending_shell_intents
        .0
        .lock()
        .map_err(|_| "failed to lock pending shell intents".to_string())?;

    for intent in intents {
        pending.push(intent.clone());
        emit_shell_intent(app, intent);
    }

    Ok(())
}

fn default_shell_extract_destination(path: &Path, extract_here: bool) -> Option<PathBuf> {
    let parent = path.parent()?.to_path_buf();
    if extract_here {
        return Some(parent);
    }

    let base_name = archive_display_name(path)?;
    Some(parent.join(base_name))
}

fn archive_display_name(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let lower = file_name.to_ascii_lowercase();
    let suffixes = [
        ".tar.gz", ".tar.xz", ".tgz", ".txz", ".zip", ".tar", ".gz", ".7z", ".rar", ".xz",
    ];

    for suffix in suffixes {
        if lower.ends_with(suffix) {
            let trimmed = &file_name[..file_name.len() - suffix.len()];
            return if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            };
        }
    }

    path.file_stem()
        .and_then(|value| value.to_str())
        .map(|value| value.to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn current_shell_integration_status() -> ShellIntegrationStatus {
    let installed = windows_registry_key_exists(
        r"HKCU\Software\Classes\SystemFileAssociations\.zip\shell\ZiplyExtract",
    );
    ShellIntegrationStatus {
        platform: "windows",
        supported: true,
        can_install: true,
        installed,
        mode: "context-menu",
        note: if installed {
            "Windows Explorer right-click entries are installed for supported archives, files, and folders.".to_string()
        } else {
            "Install Windows Explorer menu entries for Extract with Ziply, Extract here with Ziply, and Compress with Ziply.".to_string()
        },
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn install_current_shell_integration() -> Result<(), String> {
    let executable = current_executable_path()?;
    install_windows_shell_integration(&executable)
}

#[cfg(target_os = "windows")]
fn install_windows_shell_integration(executable: &Path) -> Result<(), String> {
    let extract_command = format!("\"{}\" --extract \"%1\"", executable.display());
    let extract_here_command = format!("\"{}\" --extract-here \"%1\"", executable.display());
    let compress_command = format!("\"{}\" --compress \"%1\"", executable.display());
    let archive_extensions = [".zip", ".7z", ".rar", ".tar", ".gz", ".tgz", ".txz", ".xz"];

    for extension in archive_extensions {
        let base_key = format!(r"HKCU\Software\Classes\SystemFileAssociations\{extension}\shell");
        add_windows_registry_value(&format!(r"{base_key}\ZiplyExtract"), None, "Extract with Ziply")?;
        add_windows_registry_value(&format!(r"{base_key}\ZiplyExtract"), Some("Icon"), &path_to_string(executable))?;
        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtract\command"),
            None,
            &extract_command,
        )?;

        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtractHere"),
            None,
            "Extract here with Ziply",
        )?;
        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtractHere"),
            Some("Icon"),
            &path_to_string(executable),
        )?;
        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtractHere\command"),
            None,
            &extract_here_command,
        )?;
    }

    for base_key in [
        r"HKCU\Software\Classes\*\shell\ZiplyCompress",
        r"HKCU\Software\Classes\Directory\shell\ZiplyCompress",
    ] {
        add_windows_registry_value(base_key, None, "Compress with Ziply")?;
        add_windows_registry_value(base_key, Some("Icon"), &path_to_string(executable))?;
        add_windows_registry_value(&format!(r"{base_key}\command"), None, &compress_command)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn add_windows_registry_value(key: &str, name: Option<&str>, value: &str) -> Result<(), String> {
    let mut command = Command::new("reg");
    command.arg("add").arg(key);
    if let Some(name) = name {
        command.arg("/v").arg(name);
    } else {
        command.arg("/ve");
    }
    command.arg("/d").arg(value).arg("/f");
    let output = command
        .output()
        .map_err(|error| format!("failed to start reg.exe for shell integration: {error}"))?;

    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "failed to update registry key {key}: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

#[cfg(target_os = "windows")]
fn windows_registry_key_exists(key: &str) -> bool {
    Command::new("reg")
        .arg("query")
        .arg(key)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
pub(crate) fn current_shell_integration_status() -> ShellIntegrationStatus {
    let desktop_path = linux_desktop_entry_path();
    let installed = desktop_path.is_file();
    ShellIntegrationStatus {
        platform: "linux",
        supported: true,
        can_install: true,
        installed,
        mode: "desktop-actions",
        note: if installed {
            format!(
                "Linux desktop actions are installed at {}.",
                desktop_path.display()
            )
        } else {
            "Install a desktop entry with Extract with Ziply, Extract here with Ziply, and Compress with Ziply actions for supported file managers.".to_string()
        },
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn install_current_shell_integration() -> Result<(), String> {
    let executable = current_executable_path()?;
    let desktop_path = linux_desktop_entry_path();
    let parent = desktop_path
        .parent()
        .ok_or_else(|| "failed to resolve Linux desktop entry parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "failed to create desktop applications directory {}: {error}",
            parent.display()
        )
    })?;

    let desktop_entry = format!(
        "[Desktop Entry]\nType=Application\nName=Ziply\nComment=Compress and extract archives with Ziply\nExec=\"{exe}\" %F\nTerminal=false\nCategories=Utility;Archiving;\nMimeType=application/zip;application/x-7z-compressed;application/vnd.rar;application/x-rar;application/x-tar;application/gzip;application/x-compressed-tar;application/x-xz-compressed-tar;\nActions=ExtractWithZiply;ExtractHereWithZiply;CompressWithZiply;\n\n[Desktop Action ExtractWithZiply]\nName=Extract with Ziply\nExec=\"{exe}\" --extract %f\n\n[Desktop Action ExtractHereWithZiply]\nName=Extract here with Ziply\nExec=\"{exe}\" --extract-here %f\n\n[Desktop Action CompressWithZiply]\nName=Compress with Ziply\nExec=\"{exe}\" --compress %F\n",
        exe = executable.display()
    );

    fs::write(&desktop_path, desktop_entry).map_err(|error| {
        format!(
            "failed to write Linux desktop entry {}: {error}",
            desktop_path.display()
        )
    })?;

    if let Some(update_desktop_database) = find_command(&["update-desktop-database"]) {
        let _ = Command::new(update_desktop_database).arg(parent).output();
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn linux_desktop_entry_path() -> PathBuf {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local/share/applications/com.tranvanbach.ziply.desktop")
}

#[cfg(target_os = "macos")]
pub(crate) fn current_shell_integration_status() -> ShellIntegrationStatus {
    ShellIntegrationStatus {
        platform: "macos",
        supported: false,
        can_install: false,
        installed: false,
        mode: "open-with",
        note: "Ziply declares archive file associations for macOS bundles, so Finder can offer Open With Ziply after install. Custom Finder right-click extraction commands need a dedicated Finder extension or Quick Action and are not installed in this beta.".to_string(),
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn install_current_shell_integration() -> Result<(), String> {
    Err("macOS custom Finder context commands are not installed in this beta build yet.".to_string())
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn current_executable_path() -> Result<PathBuf, String> {
    env::current_exe()
        .map_err(|error| format!("failed to resolve current executable path: {error}"))
}
