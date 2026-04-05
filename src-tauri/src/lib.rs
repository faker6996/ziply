mod archive;
mod commands;
mod history;
mod models;
mod shell;
#[cfg(test)]
mod test_fixtures;

use std::sync::Mutex;

use tauri::Manager;

use crate::{
    commands::{
        archive as archive_commands, metadata as metadata_commands, shell as shell_commands,
    },
    models::PendingShellIntents,
    shell::{collect_launch_shell_intents, store_shell_intents},
};

#[cfg(target_os = "macos")]
use crate::{
    archive::{is_supported_archive_path, path_to_string},
    shell::shell_extract_intent,
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
            archive_commands::extract_archive,
            archive_commands::preview_archive_contents
        ])
        .build(tauri::generate_context!())
        .expect("failed to build Ziply");

    app.run(handle_run_event);
}

#[cfg(target_os = "macos")]
fn handle_run_event(app: &tauri::AppHandle, event: tauri::RunEvent) {
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
}

#[cfg(not(target_os = "macos"))]
fn handle_run_event(_app: &tauri::AppHandle, _event: tauri::RunEvent) {}

#[cfg(test)]
mod tests {
    use super::archive::{
        create_7z_archive, create_bz2_archive, create_gz_archive, create_tar_bz2_archive,
        create_tar_gz_archive, create_tar_xz_archive, create_xz_archive, create_zip_archive,
        extract_7z_archive, extract_bz2_archive, extract_gz_archive, extract_rar_archive,
        extract_tar_bz2_archive, extract_tar_gz_archive, extract_tar_xz_archive,
        extract_xz_archive, extract_zip_archive, normalize_archive_path,
        prepare_extract_destination, preview_archive, resolve_archive_output_path,
    };
    use super::models::{ConflictPolicy, ExtractRequest};
    use super::test_fixtures::{RAR4_SAVE_TXT, RAR5_SAVE_TXT, TEXT_TXT};
    use std::{
        fs,
        fs::File,
        io::{BufWriter, Write},
        path::{Path, PathBuf},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use zip::{write::FileOptions, CompressionMethod, ZipWriter};

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

    fn create_directory(path: &Path) {
        fs::create_dir_all(path).expect("create directory");
    }

    fn sample_binary_payload(size: usize) -> Vec<u8> {
        (0..size)
            .map(|index| ((index * 37 + 11) % 251) as u8)
            .collect()
    }

    fn compatibility_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/compat")
            .join(name)
    }

    fn rar_fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/rar")
            .join(name)
    }

    fn compatibility_raw_payload() -> Vec<u8> {
        (0..(65_536 + 19))
            .map(|index| ((index * 29 + 7) % 251) as u8)
            .collect()
    }

    fn write_bytes(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, contents).expect("write bytes");
    }

    fn write_rar_fixture(label: &str, file_name: &str, bytes: &[u8]) -> PathBuf {
        let workspace = unique_temp_dir(label);
        let archive_path = workspace.join(file_name);
        write_bytes(&archive_path, bytes);
        archive_path
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
        create_zip_archive(&[source_directory.clone()], &archive_path, None)
            .expect("create zip archive");

        let extract_directory = workspace.join("extract");
        extract_zip_archive(&archive_path, &extract_directory, None, None)
            .expect("extract zip archive");

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
    fn zip_round_trip_preserves_unicode_empty_file_and_empty_directory() {
        let workspace = unique_temp_dir("zip-unicode-empty");
        let source_directory = workspace.join("source");
        let unicode_file = source_directory.join("thử nghiệm/điện toán.txt");
        let empty_file = source_directory.join("empty/blank.txt");
        let empty_directory = source_directory.join("trống");

        write_file(&unicode_file, "unicode zip content");
        write_file(&empty_file, "");
        create_directory(&empty_directory);

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory.clone()], &archive_path, None)
            .expect("create zip archive");

        let extract_directory = workspace.join("extract");
        extract_zip_archive(&archive_path, &extract_directory, None, None)
            .expect("extract zip archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/thử nghiệm/điện toán.txt"))
                .expect("read unicode extracted file"),
            "unicode zip content"
        );
        assert_eq!(
            fs::metadata(extract_directory.join("source/empty/blank.txt"))
                .expect("stat empty extracted file")
                .len(),
            0
        );
        assert!(extract_directory.join("source/trống").is_dir());
    }

    #[test]
    fn zip_encrypted_round_trip_requires_password() {
        let workspace = unique_temp_dir("zip-encrypted-roundtrip");
        let source_directory = workspace.join("source");
        write_file(
            &source_directory.join("secure/plan.txt"),
            "encrypted zip content",
        );

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory], &archive_path, Some("ziply-secret"))
            .expect("create encrypted zip archive");

        assert!(preview_archive(&archive_path, 20, None).is_err());
        assert!(preview_archive(&archive_path, 20, Some("wrong-secret")).is_err());

        let preview =
            preview_archive(&archive_path, 20, Some("ziply-secret")).expect("preview zip archive");
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("source/secure/plan.txt")));

        let extract_directory = workspace.join("extract");
        extract_zip_archive(
            &archive_path,
            &extract_directory,
            Some("ziply-secret"),
            None,
        )
        .expect("extract encrypted zip archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/secure/plan.txt"))
                .expect("read extracted encrypted zip file"),
            "encrypted zip content"
        );
    }

    #[test]
    fn extract_request_defaults_delete_after_extraction_to_false() {
        let request: ExtractRequest = serde_json::from_value(serde_json::json!({
            "archivePath": "/tmp/archive.zip",
            "destinationDirectory": "/tmp/output"
        }))
        .expect("deserialize extract request");

        assert!(!request.delete_after_extraction);
    }

    #[test]
    fn extract_request_deserializes_delete_after_extraction_flag() {
        let request: ExtractRequest = serde_json::from_value(serde_json::json!({
            "archivePath": "/tmp/archive.zip",
            "destinationDirectory": "/tmp/output",
            "deleteAfterExtraction": true
        }))
        .expect("deserialize extract request");

        assert!(request.delete_after_extraction);
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
        extract_tar_gz_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.gz archive");

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
        extract_tar_xz_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.xz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/images/logo.txt"))
                .expect("read extracted tar.xz file"),
            "tar xz content"
        );
    }

    #[test]
    fn tar_xz_round_trip_preserves_unicode_empty_file_and_empty_directory() {
        let workspace = unique_temp_dir("tar-xz-unicode-empty");
        let source_directory = workspace.join("source");
        let unicode_file = source_directory.join("thư mục/ghi chú.txt");
        let empty_file = source_directory.join("empty/blank.txt");
        let empty_directory = source_directory.join("trống");

        write_file(&unicode_file, "unicode tar xz content");
        write_file(&empty_file, "");
        create_directory(&empty_directory);

        let archive_path = workspace.join("bundle.tar.xz");
        create_tar_xz_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.xz archive");

        let extract_directory = workspace.join("extract");
        extract_tar_xz_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.xz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/thư mục/ghi chú.txt"))
                .expect("read unicode extracted file"),
            "unicode tar xz content"
        );
        assert_eq!(
            fs::metadata(extract_directory.join("source/empty/blank.txt"))
                .expect("stat empty extracted file")
                .len(),
            0
        );
        assert!(extract_directory.join("source/trống").is_dir());
    }

    #[test]
    fn tar_bz2_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("tar-bz2-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("images/logo.txt");
        write_file(&nested_file, "tar bz2 content");

        let archive_path = workspace.join("bundle.tar.bz2");
        create_tar_bz2_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.bz2 archive");

        let extract_directory = workspace.join("extract");
        extract_tar_bz2_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.bz2 archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/images/logo.txt"))
                .expect("read extracted tar.bz2 file"),
            "tar bz2 content"
        );
    }

    #[test]
    fn tar_gz_round_trip_preserves_unicode_empty_file_and_empty_directory() {
        let workspace = unique_temp_dir("tar-gz-unicode-empty");
        let source_directory = workspace.join("source");
        let unicode_file = source_directory.join("thư mục/ghi chú.txt");
        let empty_file = source_directory.join("empty/blank.txt");
        let empty_directory = source_directory.join("trống");

        write_file(&unicode_file, "unicode tar gz content");
        write_file(&empty_file, "");
        create_directory(&empty_directory);

        let archive_path = workspace.join("bundle.tar.gz");
        create_tar_gz_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.gz archive");

        let extract_directory = workspace.join("extract");
        extract_tar_gz_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.gz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/thư mục/ghi chú.txt"))
                .expect("read unicode extracted file"),
            "unicode tar gz content"
        );
        assert_eq!(
            fs::metadata(extract_directory.join("source/empty/blank.txt"))
                .expect("stat empty extracted file")
                .len(),
            0
        );
        assert!(extract_directory.join("source/trống").is_dir());
    }

    #[test]
    fn tar_bz2_round_trip_preserves_unicode_empty_file_and_empty_directory() {
        let workspace = unique_temp_dir("tar-bz2-unicode-empty");
        let source_directory = workspace.join("source");
        let unicode_file = source_directory.join("thư mục/ghi chú.txt");
        let empty_file = source_directory.join("empty/blank.txt");
        let empty_directory = source_directory.join("trống");

        write_file(&unicode_file, "unicode tar bz2 content");
        write_file(&empty_file, "");
        create_directory(&empty_directory);

        let archive_path = workspace.join("bundle.tar.bz2");
        create_tar_bz2_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.bz2 archive");

        let extract_directory = workspace.join("extract");
        extract_tar_bz2_archive(&archive_path, &extract_directory, None)
            .expect("extract tar.bz2 archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("source/thư mục/ghi chú.txt"))
                .expect("read unicode extracted file"),
            "unicode tar bz2 content"
        );
        assert_eq!(
            fs::metadata(extract_directory.join("source/empty/blank.txt"))
                .expect("stat empty extracted file")
                .len(),
            0
        );
        assert!(extract_directory.join("source/trống").is_dir());
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
    fn bz2_round_trip_restores_original_file() {
        let workspace = unique_temp_dir("bz2-roundtrip");
        let source_file = workspace.join("hello.txt");
        write_file(&source_file, "hello from bzip2");

        let archive_path = workspace.join("hello.txt.bz2");
        create_bz2_archive(&source_file, &archive_path).expect("create bz2 archive");

        let extract_directory = workspace.join("extract");
        fs::create_dir_all(&extract_directory).expect("create extract dir");
        extract_bz2_archive(&archive_path, &extract_directory).expect("extract bz2 archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("hello.txt"))
                .expect("read extracted bzip2 file"),
            "hello from bzip2"
        );
    }

    #[test]
    fn xz_round_trip_restores_original_file() {
        let workspace = unique_temp_dir("xz-roundtrip");
        let source_file = workspace.join("hello.txt");
        write_file(&source_file, "hello from xz");

        let archive_path = workspace.join("hello.txt.xz");
        create_xz_archive(&source_file, &archive_path).expect("create xz archive");

        let extract_directory = workspace.join("extract");
        fs::create_dir_all(&extract_directory).expect("create extract dir");
        extract_xz_archive(&archive_path, &extract_directory).expect("extract xz archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("hello.txt"))
                .expect("read extracted xz file"),
            "hello from xz"
        );
    }

    #[test]
    fn raw_stream_round_trip_preserves_large_binary_payload() {
        let workspace = unique_temp_dir("raw-large-binary");
        let source_file = workspace.join("payload.bin");
        let payload = sample_binary_payload(1024 * 1024 + 321);
        fs::write(&source_file, &payload).expect("write binary payload");

        let gz_archive = workspace.join("payload.bin.gz");
        create_gz_archive(&source_file, &gz_archive).expect("create gz archive");
        let gz_extract_directory = workspace.join("extract-gz");
        fs::create_dir_all(&gz_extract_directory).expect("create gz extract dir");
        extract_gz_archive(&gz_archive, &gz_extract_directory).expect("extract gz archive");
        assert_eq!(
            fs::read(gz_extract_directory.join("payload.bin")).expect("read extracted gz payload"),
            payload
        );

        let bz2_archive = workspace.join("payload.bin.bz2");
        create_bz2_archive(&source_file, &bz2_archive).expect("create bz2 archive");
        let bz2_extract_directory = workspace.join("extract-bz2");
        fs::create_dir_all(&bz2_extract_directory).expect("create bz2 extract dir");
        extract_bz2_archive(&bz2_archive, &bz2_extract_directory).expect("extract bz2 archive");
        assert_eq!(
            fs::read(bz2_extract_directory.join("payload.bin"))
                .expect("read extracted bz2 payload"),
            payload
        );

        let xz_archive = workspace.join("payload.bin.xz");
        create_xz_archive(&source_file, &xz_archive).expect("create xz archive");
        let xz_extract_directory = workspace.join("extract-xz");
        fs::create_dir_all(&xz_extract_directory).expect("create xz extract dir");
        extract_xz_archive(&xz_archive, &xz_extract_directory).expect("extract xz archive");
        assert_eq!(
            fs::read(xz_extract_directory.join("payload.bin")).expect("read extracted xz payload"),
            payload
        );
    }

    #[test]
    fn compatibility_zip_fixtures_preview_and_extract() {
        for fixture_name in ["zip-cli.zip", "zip-ditto.zip"] {
            let archive_path = compatibility_fixture_path(fixture_name);
            assert!(
                archive_path.is_file(),
                "missing fixture {}",
                archive_path.display()
            );

            let preview = preview_archive(&archive_path, 20, None).expect("preview zip fixture");
            assert_eq!(preview.format, "zip");
            assert!(preview
                .visible_entries
                .iter()
                .any(|entry| entry.path.ends_with("compat-source/docs/readme.txt")));
            assert!(preview.visible_entries.iter().any(|entry| entry
                .path
                .ends_with("compat-source/docs/nested/config.json")));

            let extract_directory = unique_temp_dir("compat-zip-extract");
            extract_zip_archive(&archive_path, &extract_directory, None, None)
                .expect("extract zip fixture");

            assert_eq!(
                fs::read_to_string(extract_directory.join("compat-source/docs/readme.txt"))
                    .expect("read extracted file"),
                "Ziply compatibility fixture\n"
            );
            assert_eq!(
                fs::metadata(extract_directory.join("compat-source/docs/blank.txt"))
                    .expect("stat empty extracted file")
                    .len(),
                0
            );
            assert!(extract_directory.join("compat-source/empty-dir").is_dir());
        }
    }

    #[test]
    fn compatibility_encrypted_zip_fixture_requires_password() {
        let archive_path = compatibility_fixture_path("zip-cli-encrypted.zip");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        assert!(preview_archive(&archive_path, 20, None).is_err());
        assert!(preview_archive(&archive_path, 20, Some("wrong-secret")).is_err());

        let preview =
            preview_archive(&archive_path, 20, Some("ziply-secret")).expect("preview zip fixture");
        assert_eq!(preview.format, "zip");
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("compat-source/docs/readme.txt")));

        let extract_directory = unique_temp_dir("compat-zip-encrypted-extract");
        extract_zip_archive(
            &archive_path,
            &extract_directory,
            Some("ziply-secret"),
            None,
        )
        .expect("extract encrypted zip fixture");

        assert_eq!(
            fs::read_to_string(extract_directory.join("compat-source/docs/readme.txt"))
                .expect("read extracted file"),
            "Ziply compatibility fixture\n"
        );
    }

    #[test]
    fn compatibility_tar_fixtures_preview_and_extract() {
        for (fixture_name, expected_format) in [
            ("tar-cli.tar", "tar"),
            ("tar-cli.tar.gz", "tar.gz"),
            ("tar-bsdtar.tar.bz2", "tar.bz2"),
            ("tar-bsdtar.tar.xz", "tar.xz"),
        ] {
            let archive_path = compatibility_fixture_path(fixture_name);
            assert!(
                archive_path.is_file(),
                "missing fixture {}",
                archive_path.display()
            );

            let preview = preview_archive(&archive_path, 20, None).expect("preview tar fixture");
            assert_eq!(preview.format, expected_format);
            assert!(preview.visible_entries.iter().any(|entry| entry
                .path
                .ends_with("compat-source/docs/nested/config.json")));

            let extract_directory = unique_temp_dir("compat-tar-extract");
            match expected_format {
                "tar" => {
                    super::archive::extract_tar_archive(&archive_path, &extract_directory, None)
                        .expect("extract tar fixture")
                }
                "tar.gz" => extract_tar_gz_archive(&archive_path, &extract_directory, None)
                    .expect("extract tar.gz fixture"),
                "tar.bz2" => extract_tar_bz2_archive(&archive_path, &extract_directory, None)
                    .expect("extract tar.bz2 fixture"),
                "tar.xz" => extract_tar_xz_archive(&archive_path, &extract_directory, None)
                    .expect("extract tar.xz fixture"),
                _ => panic!("unexpected format"),
            }

            assert_eq!(
                fs::read_to_string(extract_directory.join("compat-source/docs/readme.txt"))
                    .expect("read extracted file"),
                "Ziply compatibility fixture\n"
            );
            assert!(extract_directory.join("compat-source/empty-dir").is_dir());
        }
    }

    #[test]
    fn compatibility_raw_stream_fixtures_preview_and_extract() {
        let expected_payload = compatibility_raw_payload();

        for (fixture_name, format) in [
            ("raw-gzip.gz", "gz"),
            ("raw-bzip2.bz2", "bz2"),
            ("raw-xz.xz", "xz"),
        ] {
            let archive_path = compatibility_fixture_path(fixture_name);
            assert!(
                archive_path.is_file(),
                "missing fixture {}",
                archive_path.display()
            );

            let preview = preview_archive(&archive_path, 20, None).expect("preview raw fixture");
            assert_eq!(preview.format, format);
            assert_eq!(preview.total_entries, 1);

            let extract_directory = unique_temp_dir("compat-raw-extract");
            match format {
                "gz" => extract_gz_archive(&archive_path, &extract_directory)
                    .expect("extract gz fixture"),
                "bz2" => extract_bz2_archive(&archive_path, &extract_directory)
                    .expect("extract bz2 fixture"),
                "xz" => extract_xz_archive(&archive_path, &extract_directory)
                    .expect("extract xz fixture"),
                _ => panic!("unexpected raw format"),
            }

            let extracted_name = match format {
                "gz" => "raw-gzip",
                "bz2" => "raw-bzip2",
                "xz" => "raw-xz",
                _ => unreachable!(),
            };
            assert_eq!(
                fs::read(extract_directory.join(extracted_name))
                    .expect("read extracted raw payload"),
                expected_payload
            );
        }
    }

    #[test]
    fn compatibility_7z_fixture_preview_and_extract() {
        let archive_path = compatibility_fixture_path("7zz-cli.7z");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        let preview = preview_archive(&archive_path, 20, None).expect("preview 7z fixture");
        assert_eq!(preview.format, "7z");
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("compat-source/docs/readme.txt")));
        assert!(preview.visible_entries.iter().any(|entry| entry
            .path
            .ends_with("compat-source/docs/nested/config.json")));

        let extract_directory = unique_temp_dir("compat-7z-extract");
        extract_7z_archive(&archive_path, &extract_directory, None, None)
            .expect("extract 7z fixture");

        assert_eq!(
            fs::read_to_string(extract_directory.join("compat-source/docs/readme.txt"))
                .expect("read extracted 7z file"),
            "Ziply compatibility fixture\n"
        );
        assert!(extract_directory.join("compat-source/empty-dir").is_dir());
    }

    #[test]
    fn compatibility_encrypted_7z_fixture_requires_password() {
        let archive_path = compatibility_fixture_path("7zz-cli-encrypted.7z");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        assert!(preview_archive(&archive_path, 20, None).is_err());
        assert!(preview_archive(&archive_path, 20, Some("wrong-secret")).is_err());

        let preview =
            preview_archive(&archive_path, 20, Some("ziply-secret")).expect("preview 7z fixture");
        assert_eq!(preview.format, "7z");
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("compat-source/docs/readme.txt")));

        let extract_directory = unique_temp_dir("compat-7z-encrypted-extract");
        extract_7z_archive(
            &archive_path,
            &extract_directory,
            Some("ziply-secret"),
            None,
        )
        .expect("extract encrypted 7z fixture");

        assert_eq!(
            fs::read_to_string(extract_directory.join("compat-source/docs/readme.txt"))
                .expect("read extracted encrypted 7z file"),
            "Ziply compatibility fixture\n"
        );
    }

    #[test]
    fn rar_fixture_extracts_contents_natively() {
        let archive_path = write_rar_fixture("rar5-extract-fixture", "fixture.rar", RAR5_SAVE_TXT);
        let extract_directory = unique_temp_dir("rar-extract");

        extract_rar_archive(&archive_path, &extract_directory, None, None)
            .expect("extract rar archive");

        let extracted = fs::read(extract_directory.join("text.txt")).expect("read extracted text");
        assert_eq!(extracted, TEXT_TXT);
    }

    #[test]
    fn rar4_fixture_is_rejected_cleanly() {
        let archive_path = write_rar_fixture("rar4-extract-fixture", "fixture.rar", RAR4_SAVE_TXT);
        let extract_directory = unique_temp_dir("rar4-extract");

        let error = extract_rar_archive(&archive_path, &extract_directory, None, None)
            .expect_err("rar4 fixture should currently fail");
        assert!(error.contains("older rar4 variant"));
    }

    #[test]
    fn rar4_preview_is_rejected_cleanly() {
        let archive_path = write_rar_fixture("rar4-preview-fixture", "fixture.rar", RAR4_SAVE_TXT);
        let error = match preview_archive(&archive_path, 20, None) {
            Ok(_) => panic!("rar4 preview should fail"),
            Err(error) => error,
        };
        assert!(error.contains("older rar4 variant"));
    }

    #[test]
    fn rar_selective_extract_copies_selected_entries() {
        let archive_path = write_rar_fixture("rar-selective-fixture", "fixture.rar", RAR5_SAVE_TXT);
        let extract_directory = unique_temp_dir("rar-selective-extract");

        extract_rar_archive(
            &archive_path,
            &extract_directory,
            None,
            Some(&["text.txt".to_string()]),
        )
        .expect("extract selected rar entry");

        let extracted = fs::read(extract_directory.join("text.txt")).expect("read extracted text");
        assert_eq!(extracted, TEXT_TXT);
    }

    #[test]
    fn rar_preview_lists_fixture_entries() {
        let archive_path = write_rar_fixture("rar-preview-fixture", "fixture.rar", RAR5_SAVE_TXT);
        let preview = preview_archive(&archive_path, 20, None).expect("preview rar archive");
        assert_eq!(preview.format, "rar");
        assert_eq!(preview.total_entries, 1);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("text.txt")));
    }

    #[test]
    fn rar5_password_fixture_requires_password() {
        let archive_path = rar_fixture_path("rar5-save-32mb-txt-png-pw-test.rar");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        let preview = preview_archive(&archive_path, 20, Some("test")).expect("preview rar");
        assert_eq!(preview.format, "rar");
        assert_eq!(preview.total_entries, 2);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("text.txt")));
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("photo.jpg")));

        let missing_password_error = extract_rar_archive(
            &archive_path,
            &unique_temp_dir("rar-encrypted-no-password"),
            None,
            None,
        )
        .expect_err("encrypted rar should require password");
        assert!(missing_password_error.contains("requires a password"));

        let wrong_password_error = extract_rar_archive(
            &archive_path,
            &unique_temp_dir("rar-encrypted-wrong-password"),
            Some("wrong-secret"),
            None,
        )
        .expect_err("wrong rar password should fail");
        assert!(wrong_password_error.contains("invalid password"));

        let extract_directory = unique_temp_dir("rar-encrypted-extract");
        extract_rar_archive(&archive_path, &extract_directory, Some("test"), None)
            .expect("extract encrypted rar");

        assert_eq!(
            fs::read(extract_directory.join("text.txt")).expect("read extracted text"),
            TEXT_TXT
        );
        assert_eq!(
            fs::metadata(extract_directory.join("photo.jpg"))
                .expect("stat extracted photo")
                .len(),
            2_149_083
        );
    }

    #[test]
    fn rar5_multipart_fixture_extracts_and_previews() {
        let archive_path = rar_fixture_path("rar5-save-32mb-txt-png-512kb.part1.rar");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        let preview = preview_archive(&archive_path, 20, None).expect("preview multipart rar");
        assert_eq!(preview.format, "rar");
        assert_eq!(preview.total_entries, 2);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("text.txt")));
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("photo.jpg")));

        let extract_directory = unique_temp_dir("rar-multipart-extract");
        extract_rar_archive(&archive_path, &extract_directory, None, None)
            .expect("extract multipart rar");

        assert_eq!(
            fs::read(extract_directory.join("text.txt")).expect("read extracted text"),
            TEXT_TXT
        );
        assert_eq!(
            fs::metadata(extract_directory.join("photo.jpg"))
                .expect("stat extracted photo")
                .len(),
            2_149_083
        );
    }

    #[test]
    fn rar_multipart_non_first_volume_is_rejected_cleanly() {
        let archive_path = rar_fixture_path("rar5-save-32mb-txt-png-512kb.part2.rar");
        assert!(
            archive_path.is_file(),
            "missing fixture {}",
            archive_path.display()
        );

        let error = extract_rar_archive(&archive_path, &unique_temp_dir("rar-part2"), None, None)
            .expect_err("later multipart volume should fail");
        assert!(error.contains("open the first rar volume"));
    }

    #[test]
    fn normalize_archive_path_resolves_rar_part_volume_to_first_volume() {
        let archive_path = rar_fixture_path("rar5-save-32mb-txt-png-512kb.part2.rar");
        let resolved =
            normalize_archive_path(&archive_path.to_string_lossy()).expect("normalize rar part");
        assert_eq!(
            resolved,
            rar_fixture_path("rar5-save-32mb-txt-png-512kb.part1.rar")
        );
    }

    #[test]
    fn normalize_archive_path_resolves_old_style_rar_segment_to_main_volume() {
        let workspace = unique_temp_dir("rar-old-style-normalize");
        let main_volume = workspace.join("bundle.rar");
        let segment = workspace.join("bundle.r00");
        write_bytes(&main_volume, RAR5_SAVE_TXT);
        write_bytes(&segment, RAR5_SAVE_TXT);

        let resolved =
            normalize_archive_path(&segment.to_string_lossy()).expect("normalize r00 segment");
        assert_eq!(resolved, main_volume);
    }

    #[test]
    fn seven_zip_round_trip_preserves_file_contents() {
        let workspace = unique_temp_dir("seven-zip-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("reports/q1.txt");
        write_file(&nested_file, "7z content");

        let archive_path = workspace.join("bundle.7z");
        create_7z_archive(&[source_directory.clone()], &archive_path, None)
            .expect("create 7z archive");

        let extract_directory = workspace.join("extract");
        extract_7z_archive(&archive_path, &extract_directory, None, None)
            .expect("extract 7z archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("reports/q1.txt"))
                .expect("read extracted 7z file"),
            "7z content"
        );
    }

    #[test]
    fn raw_stream_previews_report_single_visible_entry() {
        let workspace = unique_temp_dir("raw-preview");
        let source_file = workspace.join("unicode-report.txt");
        write_file(&source_file, "preview me");

        let gz_archive = workspace.join("unicode-report.txt.gz");
        create_gz_archive(&source_file, &gz_archive).expect("create gz archive");
        let gz_preview = preview_archive(&gz_archive, 20, None).expect("preview gz archive");
        assert_eq!(gz_preview.format, "gz");
        assert_eq!(gz_preview.total_entries, 1);
        assert_eq!(gz_preview.visible_entries[0].path, "unicode-report.txt");
        assert!(gz_preview.note.is_some());

        let bz2_archive = workspace.join("unicode-report.txt.bz2");
        create_bz2_archive(&source_file, &bz2_archive).expect("create bz2 archive");
        let bz2_preview = preview_archive(&bz2_archive, 20, None).expect("preview bz2 archive");
        assert_eq!(bz2_preview.format, "bz2");
        assert_eq!(bz2_preview.total_entries, 1);
        assert_eq!(bz2_preview.visible_entries[0].path, "unicode-report.txt");
        assert!(bz2_preview.note.is_some());

        let xz_archive = workspace.join("unicode-report.txt.xz");
        create_xz_archive(&source_file, &xz_archive).expect("create xz archive");
        let xz_preview = preview_archive(&xz_archive, 20, None).expect("preview xz archive");
        assert_eq!(xz_preview.format, "xz");
        assert_eq!(xz_preview.total_entries, 1);
        assert_eq!(xz_preview.visible_entries[0].path, "unicode-report.txt");
        assert!(xz_preview.note.is_some());
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

    #[test]
    fn zip_preview_lists_nested_entries() {
        let workspace = unique_temp_dir("zip-preview");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("docs/readme.txt"), "preview content");
        write_file(&source_directory.join("notes.txt"), "top level");

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory.clone()], &archive_path, None)
            .expect("create zip archive");

        let preview = preview_archive(&archive_path, 20, None).expect("preview zip archive");

        assert_eq!(preview.format, "zip");
        assert!(preview.total_entries >= 2);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("source/docs/readme.txt")));
    }

    #[test]
    fn zip_preview_limit_reports_hidden_entries() {
        let workspace = unique_temp_dir("zip-preview-limit");
        let source_directory = workspace.join("source");
        for index in 0..5 {
            write_file(
                &source_directory.join(format!("docs/file-{index}.txt")),
                &format!("preview content {index}"),
            );
        }

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory], &archive_path, None).expect("create zip archive");

        let preview = preview_archive(&archive_path, 2, None).expect("preview zip archive");

        assert_eq!(preview.format, "zip");
        assert_eq!(preview.visible_entries.len(), 2);
        assert!(preview.total_entries >= 5);
        assert!(preview.hidden_entry_count >= 3);
    }

    #[test]
    fn seven_zip_preview_lists_entries() {
        let workspace = unique_temp_dir("seven-zip-preview");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("reports/q1.txt"), "preview content");

        let archive_path = workspace.join("bundle.7z");
        create_7z_archive(&[source_directory], &archive_path, None).expect("create 7z archive");

        let preview = preview_archive(&archive_path, 20, None).expect("preview 7z archive");

        assert_eq!(preview.format, "7z");
        assert!(preview.total_entries >= 1);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("reports/q1.txt")));
    }

    #[test]
    fn tar_bz2_preview_lists_entries() {
        let workspace = unique_temp_dir("tar-bz2-preview");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("reports/q1.txt"), "preview content");

        let archive_path = workspace.join("bundle.tar.bz2");
        create_tar_bz2_archive(&[source_directory], &archive_path).expect("create tar.bz2 archive");

        let preview = preview_archive(&archive_path, 20, None).expect("preview tar.bz2 archive");

        assert_eq!(preview.format, "tar.bz2");
        assert!(preview.total_entries >= 1);
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("source/reports/q1.txt")));
    }

    #[test]
    fn seven_zip_encrypted_round_trip_requires_password() {
        let workspace = unique_temp_dir("seven-zip-encrypted-roundtrip");
        let source_directory = workspace.join("source");
        let nested_file = source_directory.join("secure/plan.txt");
        write_file(&nested_file, "encrypted 7z content");

        let archive_path = workspace.join("secure.7z");
        create_7z_archive(&[source_directory], &archive_path, Some("ziply-secret"))
            .expect("create encrypted 7z archive");

        assert!(preview_archive(&archive_path, 20, None).is_err());

        let preview =
            preview_archive(&archive_path, 20, Some("ziply-secret")).expect("preview 7z archive");
        assert!(preview
            .visible_entries
            .iter()
            .any(|entry| entry.path.ends_with("secure/plan.txt")));

        let extract_directory = workspace.join("extract");
        extract_7z_archive(
            &archive_path,
            &extract_directory,
            Some("ziply-secret"),
            None,
        )
        .expect("extract encrypted 7z archive");

        assert_eq!(
            fs::read_to_string(extract_directory.join("secure/plan.txt"))
                .expect("read extracted encrypted 7z file"),
            "encrypted 7z content"
        );
    }

    #[test]
    fn seven_zip_wrong_password_is_rejected() {
        let workspace = unique_temp_dir("seven-zip-wrong-password");
        let source_directory = workspace.join("source");
        write_file(
            &source_directory.join("secure/plan.txt"),
            "encrypted payload",
        );

        let archive_path = workspace.join("secure.7z");
        create_7z_archive(&[source_directory], &archive_path, Some("ziply-secret"))
            .expect("create encrypted 7z archive");

        let preview_error = match preview_archive(&archive_path, 20, Some("wrong-secret")) {
            Ok(_) => panic!("expected preview to reject wrong password"),
            Err(error) => error,
        };
        assert!(!preview_error.trim().is_empty());

        let extract_directory = workspace.join("extract");
        let extract_error = match extract_7z_archive(
            &archive_path,
            &extract_directory,
            Some("wrong-secret"),
            None,
        ) {
            Ok(_) => panic!("expected extract to reject wrong password"),
            Err(error) => error,
        };
        assert!(!extract_error.trim().is_empty());
    }

    #[test]
    fn zip_selective_extract_only_unpacks_selected_entries() {
        let workspace = unique_temp_dir("zip-selective-extract");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("docs/readme.txt"), "keep me");
        write_file(&source_directory.join("docs/guide.txt"), "skip me");
        write_file(&source_directory.join("notes.txt"), "skip me too");

        let archive_path = workspace.join("bundle.zip");
        create_zip_archive(&[source_directory.clone()], &archive_path, None)
            .expect("create zip archive");

        let extract_directory = workspace.join("extract");
        extract_zip_archive(
            &archive_path,
            &extract_directory,
            None,
            Some(&["source/docs/readme.txt".to_string()]),
        )
        .expect("extract selected zip entry");

        assert!(extract_directory.join("source/docs/readme.txt").exists());
        assert!(!extract_directory.join("source/docs/guide.txt").exists());
        assert!(!extract_directory.join("source/notes.txt").exists());
    }

    #[test]
    fn zip_extract_rejects_unsafe_paths() {
        let workspace = unique_temp_dir("zip-unsafe-path");
        let archive_path = workspace.join("unsafe.zip");
        let archive_file = File::create(&archive_path).expect("create zip file");
        let writer = BufWriter::new(archive_file);
        let mut archive = ZipWriter::new(writer);
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        archive
            .start_file("../escape.txt", options)
            .expect("start unsafe zip entry");
        archive
            .write_all(b"do not extract")
            .expect("write unsafe zip entry");
        archive.finish().expect("finalize unsafe zip archive");

        let extract_directory = workspace.join("extract");
        let error = extract_zip_archive(&archive_path, &extract_directory, None, None)
            .expect_err("reject unsafe zip entry");

        assert!(error.contains("unsafe path"));
        assert!(!workspace.join("escape.txt").exists());
    }

    #[test]
    fn tar_gz_selective_extract_only_unpacks_selected_directory() {
        let workspace = unique_temp_dir("tar-gz-selective-extract");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("images/logo.txt"), "keep tree");
        write_file(&source_directory.join("images/banner.txt"), "keep tree too");
        write_file(&source_directory.join("notes.txt"), "skip file");

        let archive_path = workspace.join("bundle.tar.gz");
        create_tar_gz_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.gz archive");

        let extract_directory = workspace.join("extract");
        extract_tar_gz_archive(
            &archive_path,
            &extract_directory,
            Some(&["source/images".to_string()]),
        )
        .expect("extract selected tar.gz directory");

        assert!(extract_directory.join("source/images/logo.txt").exists());
        assert!(extract_directory.join("source/images/banner.txt").exists());
        assert!(!extract_directory.join("source/notes.txt").exists());
    }

    #[test]
    fn tar_bz2_selective_extract_only_unpacks_selected_directory() {
        let workspace = unique_temp_dir("tar-bz2-selective-extract");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("images/logo.txt"), "keep tree");
        write_file(&source_directory.join("images/banner.txt"), "keep tree too");
        write_file(&source_directory.join("notes.txt"), "skip file");

        let archive_path = workspace.join("bundle.tar.bz2");
        create_tar_bz2_archive(&[source_directory.clone()], &archive_path)
            .expect("create tar.bz2 archive");

        let extract_directory = workspace.join("extract");
        extract_tar_bz2_archive(
            &archive_path,
            &extract_directory,
            Some(&["source/images".to_string()]),
        )
        .expect("extract selected tar.bz2 directory");

        assert!(extract_directory.join("source/images/logo.txt").exists());
        assert!(extract_directory.join("source/images/banner.txt").exists());
        assert!(!extract_directory.join("source/notes.txt").exists());
    }

    #[test]
    fn seven_zip_selective_extract_only_unpacks_selected_entries() {
        let workspace = unique_temp_dir("seven-zip-selective-extract");
        let source_directory = workspace.join("source");
        write_file(&source_directory.join("reports/q1.txt"), "keep me");
        write_file(&source_directory.join("reports/q2.txt"), "skip me");

        let archive_path = workspace.join("bundle.7z");
        create_7z_archive(&[source_directory], &archive_path, None).expect("create 7z archive");

        let extract_directory = workspace.join("extract");
        extract_7z_archive(
            &archive_path,
            &extract_directory,
            None,
            Some(&["reports/q1.txt".to_string()]),
        )
        .expect("extract selected 7z entry");

        assert!(extract_directory.join("reports/q1.txt").exists());
        assert!(!extract_directory.join("reports/q2.txt").exists());
    }
}
