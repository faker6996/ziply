use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::fs;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;

use tauri::{AppHandle, Manager};

use crate::{
    archive::{is_supported_archive_path, path_to_string, resolve_rar_archive_entry_path},
    history::emit_shell_intent,
    models::{PendingShellIntents, ShellIntegrationStatus, ShellIntent},
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
    let archive_path = resolve_rar_archive_entry_path(&PathBuf::from(path));
    ShellIntent {
        action: if auto_run {
            "extract-here".to_string()
        } else {
            "extract".to_string()
        },
        paths: vec![path_to_string(&archive_path)],
        auto_run,
        destination_path: default_shell_extract_destination(&archive_path, auto_run)
            .map(|value| path_to_string(&value)),
    }
}

pub(crate) fn store_shell_intents(
    app: &AppHandle,
    intents: Vec<ShellIntent>,
) -> Result<(), String> {
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
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tgz", ".tbz2", ".txz", ".zip", ".tar", ".bz2", ".gz",
        ".7z", ".xz", ".rar",
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

    if let Some(start) = lower.find(".part") {
        if let Some(number) = lower[start + 5..].strip_suffix(".rar") {
            if !number.is_empty() && number.chars().all(|ch| ch.is_ascii_digit()) {
                let trimmed = &file_name[..start];
                return if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                };
            }
        }
    }

    if let Some(extension) = path.extension().and_then(|value| value.to_str()) {
        let extension = extension.to_ascii_lowercase();
        if extension.len() == 3
            && extension.starts_with('r')
            && extension[1..].chars().all(|ch| ch.is_ascii_digit())
        {
            return path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string());
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
    let archive_extensions = [
        ".zip", ".7z", ".tar", ".tar.gz", ".tar.bz2", ".tar.xz", ".gz", ".bz2", ".tgz", ".tbz2",
        ".txz", ".xz", ".rar",
    ];

    for extension in archive_extensions {
        let base_key = format!(r"HKCU\Software\Classes\SystemFileAssociations\{extension}\shell");
        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtract"),
            None,
            "Extract with Ziply",
        )?;
        add_windows_registry_value(
            &format!(r"{base_key}\ZiplyExtract"),
            Some("Icon"),
            &path_to_string(executable),
        )?;
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
        "[Desktop Entry]\nType=Application\nName=Ziply\nComment=Compress and extract archives with Ziply\nExec=\"{exe}\" %F\nTerminal=false\nCategories=Utility;Archiving;\nMimeType=application/zip;application/x-7z-compressed;application/x-tar;application/gzip;application/x-bzip2;application/x-bzip-compressed-tar;application/x-compressed-tar;application/x-xz-compressed-tar;\nActions=ExtractWithZiply;ExtractHereWithZiply;CompressWithZiply;\n\n[Desktop Action ExtractWithZiply]\nName=Extract with Ziply\nExec=\"{exe}\" --extract %f\n\n[Desktop Action ExtractHereWithZiply]\nName=Extract here with Ziply\nExec=\"{exe}\" --extract-here %f\n\n[Desktop Action CompressWithZiply]\nName=Compress with Ziply\nExec=\"{exe}\" --compress %F\n",
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
    let services_dir = macos_services_directory();
    let installed = macos_quick_action_paths().iter().all(|path| path.is_dir());

    ShellIntegrationStatus {
        platform: "macos",
        supported: true,
        can_install: true,
        installed,
        mode: "quick-actions",
        note: if installed {
            format!(
                "Finder Quick Actions are installed in {} for Extract with Ziply and Extract here with Ziply. If Finder does not show them immediately, reopen Finder and check the Quick Actions section.",
                services_dir.display()
            )
        } else {
            format!(
                "Install Finder Quick Actions into {} for Extract with Ziply and Extract here with Ziply on supported archive files.",
                services_dir.display()
            )
        },
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn install_current_shell_integration() -> Result<(), String> {
    let executable = current_executable_path()?;
    let services_dir = macos_services_directory();
    fs::create_dir_all(&services_dir).map_err(|error| {
        format!(
            "failed to create macOS Services directory {}: {error}",
            services_dir.display()
        )
    })?;

    for definition in [
        MacOsQuickActionDefinition {
            workflow_name: "Extract with Ziply.workflow",
            bundle_identifier: "com.tranvanbach.ziply.service.extract",
            menu_title: "Extract with Ziply",
            cli_flag: "--extract",
        },
        MacOsQuickActionDefinition {
            workflow_name: "Extract here with Ziply.workflow",
            bundle_identifier: "com.tranvanbach.ziply.service.extracthere",
            menu_title: "Extract here with Ziply",
            cli_flag: "--extract-here",
        },
    ] {
        install_macos_quick_action(&services_dir, &executable, definition)?;
    }

    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "macos"))]
fn current_executable_path() -> Result<PathBuf, String> {
    env::current_exe()
        .map_err(|error| format!("failed to resolve current executable path: {error}"))
}

#[cfg(target_os = "macos")]
struct MacOsQuickActionDefinition {
    workflow_name: &'static str,
    bundle_identifier: &'static str,
    menu_title: &'static str,
    cli_flag: &'static str,
}

#[cfg(target_os = "macos")]
fn macos_services_directory() -> PathBuf {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join("Library/Services")
}

#[cfg(target_os = "macos")]
fn macos_quick_action_paths() -> [PathBuf; 2] {
    let services_dir = macos_services_directory();
    [
        services_dir.join("Extract with Ziply.workflow"),
        services_dir.join("Extract here with Ziply.workflow"),
    ]
}

#[cfg(target_os = "macos")]
fn install_macos_quick_action(
    services_dir: &Path,
    executable: &Path,
    definition: MacOsQuickActionDefinition,
) -> Result<(), String> {
    let workflow_dir = services_dir.join(definition.workflow_name);
    let resources_dir = workflow_dir.join("Contents/Resources");

    if workflow_dir.exists() {
        fs::remove_dir_all(&workflow_dir).map_err(|error| {
            format!(
                "failed to replace existing workflow {}: {error}",
                workflow_dir.display()
            )
        })?;
    }

    fs::create_dir_all(&resources_dir).map_err(|error| {
        format!(
            "failed to create workflow resources directory {}: {error}",
            resources_dir.display()
        )
    })?;

    let info_plist =
        macos_quick_action_info_plist(definition.bundle_identifier, definition.menu_title);
    fs::write(workflow_dir.join("Contents/Info.plist"), info_plist).map_err(|error| {
        format!(
            "failed to write workflow Info.plist for {}: {error}",
            definition.menu_title
        )
    })?;

    let version_plist = macos_quick_action_version_plist();
    fs::write(workflow_dir.join("Contents/version.plist"), version_plist).map_err(|error| {
        format!(
            "failed to write workflow version.plist for {}: {error}",
            definition.menu_title
        )
    })?;

    let document_wflow = macos_quick_action_document_wflow(executable, definition.cli_flag);
    fs::write(resources_dir.join("document.wflow"), document_wflow).map_err(|error| {
        format!(
            "failed to write workflow document.wflow for {}: {error}",
            definition.menu_title
        )
    })?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_quick_action_info_plist(bundle_identifier: &str, menu_title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en_US</string>
	<key>CFBundleIdentifier</key>
	<string>{bundle_identifier}</string>
	<key>CFBundleName</key>
	<string>{menu_title}</string>
	<key>CFBundlePackageType</key>
	<string>BNDL</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0</string>
	<key>NSServices</key>
	<array>
		<dict>
			<key>NSMenuItem</key>
			<dict>
				<key>default</key>
				<string>{menu_title}</string>
			</dict>
			<key>NSMessage</key>
			<string>runWorkflowAsService</string>
			<key>NSRequiredContext</key>
			<dict>
				<key>NSApplicationIdentifier</key>
				<string>com.apple.finder</string>
			</dict>
			<key>NSSendFileTypes</key>
			<array>
				<string>public.archive</string>
			</array>
		</dict>
	</array>
</dict>
</plist>
"#,
        bundle_identifier = xml_escape(bundle_identifier),
        menu_title = xml_escape(menu_title),
    )
}

#[cfg(target_os = "macos")]
fn macos_quick_action_version_plist() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>ProjectName</key>
	<string>Ziply</string>
	<key>SourceVersion</key>
	<string>1.0</string>
</dict>
</plist>
"#
    .to_string()
}

#[cfg(target_os = "macos")]
fn macos_quick_action_document_wflow(executable: &Path, cli_flag: &str) -> String {
    let command = format!(
        "exe={exe}\nwhile IFS= read -r item; do\n  [ -n \"$item\" ] || continue\n  \"$exe\" {cli_flag} \"$item\" >/dev/null 2>&1 &\ndone\n",
        exe = shell_single_quote(&path_to_string(executable)),
        cli_flag = cli_flag,
    );

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>AMApplicationBuild</key>
	<string>346</string>
	<key>AMApplicationVersion</key>
	<string>2.3</string>
	<key>AMDocumentVersion</key>
	<string>2</string>
	<key>actions</key>
	<array>
		<dict>
			<key>action</key>
			<dict>
				<key>AMAccepts</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Optional</key>
					<true/>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.path</string>
					</array>
				</dict>
				<key>AMActionVersion</key>
				<string>2.0.3</string>
				<key>AMApplication</key>
				<array>
					<string>Automator</string>
				</array>
				<key>AMParameterProperties</key>
				<dict>
					<key>COMMAND_STRING</key>
					<dict/>
					<key>CheckedForUserDefaultShell</key>
					<dict/>
					<key>inputMethod</key>
					<dict/>
					<key>shell</key>
					<dict/>
					<key>source</key>
					<dict/>
				</dict>
				<key>AMProvides</key>
				<dict>
					<key>Container</key>
					<string>List</string>
					<key>Types</key>
					<array>
						<string>com.apple.cocoa.path</string>
					</array>
				</dict>
				<key>ActionBundlePath</key>
				<string>/System/Library/Automator/Run Shell Script.action</string>
				<key>ActionName</key>
				<string>Run Shell Script</string>
				<key>ActionParameters</key>
				<dict>
					<key>COMMAND_STRING</key>
					<string>{command}</string>
					<key>CheckedForUserDefaultShell</key>
					<true/>
					<key>inputMethod</key>
					<integer>0</integer>
					<key>shell</key>
					<string>/bin/bash</string>
					<key>source</key>
					<string></string>
				</dict>
				<key>BundleIdentifier</key>
				<string>com.apple.RunShellScript</string>
				<key>CFBundleVersion</key>
				<string>2.0.3</string>
				<key>CanShowSelectedItemsWhenRun</key>
				<false/>
				<key>CanShowWhenRun</key>
				<true/>
				<key>Category</key>
				<array>
					<string>AMCategoryUtilities</string>
				</array>
				<key>Class Name</key>
				<string>RunShellScriptAction</string>
				<key>InputUUID</key>
				<string>F781F79A-5673-41C0-9CC0-684F0A0BB001</string>
				<key>Keywords</key>
				<array>
					<string>Shell</string>
					<string>Script</string>
					<string>Command</string>
					<string>Run</string>
					<string>Ziply</string>
				</array>
				<key>OutputUUID</key>
				<string>F781F79A-5673-41C0-9CC0-684F0A0BB002</string>
				<key>UUID</key>
				<string>F781F79A-5673-41C0-9CC0-684F0A0BB003</string>
				<key>UnlocalizedApplications</key>
				<array>
					<string>Automator</string>
				</array>
				<key>arguments</key>
				<dict>
					<key>0</key>
					<dict>
						<key>default value</key>
						<integer>0</integer>
						<key>name</key>
						<string>inputMethod</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>0</string>
					</dict>
					<key>1</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>source</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>1</string>
					</dict>
					<key>2</key>
					<dict>
						<key>default value</key>
						<false/>
						<key>name</key>
						<string>CheckedForUserDefaultShell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>2</string>
					</dict>
					<key>3</key>
					<dict>
						<key>default value</key>
						<string></string>
						<key>name</key>
						<string>COMMAND_STRING</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>3</string>
					</dict>
					<key>4</key>
					<dict>
						<key>default value</key>
						<string>/bin/sh</string>
						<key>name</key>
						<string>shell</string>
						<key>required</key>
						<string>0</string>
						<key>type</key>
						<string>0</string>
						<key>uuid</key>
						<string>4</string>
					</dict>
				</dict>
			</dict>
			<key>isViewVisible</key>
			<true/>
		</dict>
	</array>
	<key>connectors</key>
	<dict/>
	<key>workflowMetaData</key>
	<dict>
		<key>serviceApplicationBundleID</key>
		<string>com.apple.finder</string>
		<key>serviceApplicationPath</key>
		<string>/System/Library/CoreServices/Finder.app</string>
		<key>serviceInputTypeIdentifier</key>
		<string>public.archive</string>
		<key>serviceOutputTypeIdentifier</key>
		<string>com.apple.Automator.nothing</string>
		<key>serviceProcessesInput</key>
		<integer>0</integer>
		<key>workflowTypeIdentifier</key>
		<string>com.apple.Automator.servicesMenu</string>
	</dict>
</dict>
</plist>
"#,
        command = xml_escape(&command),
    )
}

#[cfg(target_os = "macos")]
fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_info_plist_targets_public_archive_files() {
        let info = macos_quick_action_info_plist(
            "com.tranvanbach.ziply.service.extract",
            "Extract with Ziply",
        );

        assert!(info.contains("<string>public.archive</string>"));
        assert!(info.contains("<string>com.apple.finder</string>"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_document_wflow_embeds_cli_flag() {
        let document = macos_quick_action_document_wflow(
            Path::new("/Applications/Ziply.app/Contents/MacOS/ziply-desktop"),
            "--extract-here",
        );

        assert!(document.contains("--extract-here"));
        assert!(document.contains("/System/Library/Automator/Run Shell Script.action"));
        assert!(document.contains("<string>public.archive</string>"));
    }
}
