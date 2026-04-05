mod archive;
mod commands;
mod history;
mod models;
mod shell;

use std::sync::Mutex;

use tauri::Manager;

use crate::{
    archive::{is_supported_archive_path, path_to_string},
    commands::{
        archive as archive_commands, metadata as metadata_commands, shell as shell_commands,
    },
    models::PendingShellIntents,
    shell::{collect_launch_shell_intents, shell_extract_intent, store_shell_intents},
};

pub fn run() {
    let app = tauri::Builder::default()
        .manage(PendingShellIntents(Mutex::new(
            collect_launch_shell_intents(),
        )))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            metadata_commands::app_overview,
            metadata_commands::archive_capabilities,
            shell_commands::consume_shell_intents,
            shell_commands::shell_integration_status,
            shell_commands::install_shell_integration,
            shell_commands::get_archive_history,
            shell_commands::clear_archive_history,
            archive_commands::compress_archive,
            archive_commands::extract_archive
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Ziply");

    app.run(|app, event| {
        if let tauri::RunEvent::Opened { urls } = event {
            let intents = urls
                .into_iter()
                .filter_map(|url| url.to_file_path().ok())
                .filter(|path| is_supported_archive_path(path))
                .map(|path| shell_extract_intent(&path_to_string(&path), false))
                .collect::<Vec<_>>();

            let _ = store_shell_intents(app, intents);

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::archive::{
        create_7z_archive, create_gz_archive, create_tar_gz_archive, create_tar_xz_archive,
        create_zip_archive, extract_7z_archive, extract_gz_archive, extract_tar_gz_archive,
        extract_tar_xz_archive, extract_zip_archive, prepare_extract_destination,
        resolve_archive_output_path,
    };
    use super::models::ConflictPolicy;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    fn unique_temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ziply-{label}-{suffix}"));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, contents).expect("write file");
    }

    #[test]
    fn zip_archive_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("zip-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("docs/readme.txt");
        let root_file = source_directory.join("notes.txt");
        write_file(&nested_file, "nested zip content");
        write_file(&root_file, "top level zip content");

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory.clone()], &archive_path).expect("create zip archive");

        let extract_directory = workspace.join("extract");
        extract_zip_archive(&archive_path, &extract_directory).expect("extract zip archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/docs/readme.txt"))
                .expect("read nested extracted file"),
            "nested zip content"
        );
        assert_eq!(
            fs::read_to_string(extract_directory.join("source/notes.txt"))
                .expect("read top-level extracted file"),
            "top level zip content"
        );
    }

    #[test]
    fn tar_gz_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("tar-gz-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("images/logo.txt");
        write_file(&nested_file, "tar gz content");

        let archive_path = workspace.join("bundle.tar.gz");
        create_tar_gz_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.gz archive");

        let extract_directory = workspace.join("extract");
        extract_tar_gz_archive(&archive_path, &extract_directory).expect("extract tar.gz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/images/logo.txt"))
                .expect("read extracted tar.gz file"),
            "tar gz content"
        );
    }

    #[test]
    fn tar_xz_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("tar-xz-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("images/logo.txt");
        write_file(&nested_file, "tar xz content");

        let archive_path = workspace.join("bundle.tar.xz");
        create_tar_xz_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.xz archive");

        let extract_directory = workspace.join("extract");
        extract_tar_xz_archive(&archive_path, &extract_directory).expect("extract tar.xz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/images/logo.txt"))
                .expect("read extracted tar.xz file"),
            "tar xz content"
        );
    }

    #[test]
    fn gz_round_trip_restores_original_file() {
        let workspace = unique_temp_dir("gz-roundtrip");
        let source_file = workspace.join("hello.txt");
        write_file(&source_file, "hello from gzip");

        let archive_path = workspace.join("hello.txt.gz");
        create_gz_archive(&source_file, &archive_path).expect("create gz archive");

        let extract_directory = workspace.join("extract");
        fs::create_dir_all(&extract_directory).expect("create extract dir");
        extract_gz_archive(&archive_path, &extract_directory).expect("extract gz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("hello.txt"))
                .expect("read extracted gzip file"),
            "hello from gzip"
        );
    }

    #[test]
    fn seven_zip_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("seven-zip-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("reports/q1.txt");
        write_file(&nested_file, "7z content");

        let archive_path = workspace.join("bundle.7z");
        create_7z_archive(&[source_directory.clone()], &archive_path).expect("create 7z archive");

        let extract_directory = workspace.join("extract");
        extract_7z_archive(&archive_path, &extract_directory).expect("extract 7z archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("reports/q1.txt"))
                .expect("read extracted 7z file"),
            "7z content"
        );
    }

    #[test]
    fn keep_both_archive_conflict_uses_incremented_file_name() {
        let workspace = unique_temp_dir("archive-conflict-keep-both");
        let archive_path = workspace.join("bundle.tar.gz");
        write_file(&archive_path, "existing archive");

        let resolved = resolve_archive_output_path(&archive_path, ConflictPolicy::KeepBoth)
            .expect("resolve archive output path");

        assert_eq!(resolved, workspace.join("bundle (1).tar.gz"));
        assert!(!resolved.exists());
    }

    #[test]
    fn overwrite_extract_destination_clears_existing_contents() {
        let workspace = unique_temp_dir("extract-destination-overwrite");
        let destination = workspace.join("output");
        write_file(&destination.join("old.txt"), "old contents");

        let resolved = prepare_extract_destination(&destination, ConflictPolicy::Overwrite)
            .expect("prepare extract destination");

        assert_eq!(resolved, destination);
        assert!(destination.is_dir());
        assert!(!destination.join("old.txt").exists());
    }

    #[test]
    fn keep_both_extract_destination_uses_incremented_folder_name() {
        let workspace = unique_temp_dir("extract-destination-keep-both");
        let destination = workspace.join("extract");
        write_file(&destination.join("old.txt"), "old contents");

        let resolved = prepare_extract_destination(&destination, ConflictPolicy::KeepBoth)
            .expect("prepare extract destination");

        assert_eq!(resolved, workspace.join("extract (1)"));
        assert!(resolved.is_dir());
        assert!(!resolved.join("old.txt").exists());
    }
}
