use std::fs;

use tauri::AppHandle;

use crate::{
    archive::{
        create_7z_archive, create_gz_archive, create_tar_archive, create_tar_gz_archive,
        create_tar_xz_archive, create_zip_archive, extract_7z_archive, extract_gz_archive,
        extract_rar_archive, extract_tar_archive, extract_tar_gz_archive, extract_tar_xz_archive,
        extract_zip_archive, normalize_archive_path, normalize_destination_path,
        normalize_directory_path, normalize_source_paths, path_to_string,
        prepare_extract_destination, resolve_archive_output_path,
    },
    history::{
        append_archive_history, archive_history_id, archive_job_id, emit_archive_job_event,
        summarize_paths, unix_timestamp_ms,
    },
    models::{
        ArchiveActionResult, ArchiveFormat, ArchiveHistoryEntry, ArchiveJobEvent, CompressRequest,
        ConflictPolicy, ExtractRequest,
    },
};

#[tauri::command]
pub(crate) fn compress_archive(
    app: AppHandle,
    request: CompressRequest,
) -> Result<ArchiveActionResult, String> {
    let format = ArchiveFormat::from_compress_input(&request.format)?;
    let source_paths = normalize_source_paths(&request.source_paths)?;
    let conflict_policy = ConflictPolicy::from_input(request.conflict_policy.as_deref())?;
    let destination_path = resolve_archive_output_path(
        &normalize_destination_path(&request.destination_path, format)?,
        conflict_policy,
    )?;
    let job_id = archive_job_id();
    let source_summary = summarize_paths(&source_paths);

    emit_archive_job_event(
        &app,
        ArchiveJobEvent {
            job_id: job_id.clone(),
            operation: "compress".to_string(),
            format: format.label().to_string(),
            stage: "queued".to_string(),
            status: "queued".to_string(),
            message: "Compression job queued.".to_string(),
            progress: 4,
            source_summary: source_summary.clone(),
            output_path: Some(path_to_string(&destination_path)),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    if matches!(format, ArchiveFormat::Gz) {
        if source_paths.len() != 1 {
            return Err("gz compression currently supports exactly one source file.".to_string());
        }
        if source_paths[0].is_dir() {
            return Err(
                "gz compression only works with a single file, not a directory.".to_string(),
            );
        }
    }

    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create destination directory {}: {error}",
                parent.display()
            )
        })?;
    }

    emit_archive_job_event(
        &app,
        ArchiveJobEvent {
            job_id: job_id.clone(),
            operation: "compress".to_string(),
            format: format.label().to_string(),
            stage: "preparing".to_string(),
            status: "running".to_string(),
            message: "Validating sources and preparing destination.".to_string(),
            progress: 16,
            source_summary: source_summary.clone(),
            output_path: Some(path_to_string(&destination_path)),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    let execution =
        (|| -> Result<ArchiveActionResult, String> {
            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id: job_id.clone(),
                    operation: "compress".to_string(),
                    format: format.label().to_string(),
                    stage: "processing".to_string(),
                    status: "running".to_string(),
                    message: format!("Creating {} archive contents.", format.label()),
                    progress: 58,
                    source_summary: source_summary.clone(),
                    output_path: Some(path_to_string(&destination_path)),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );

            match format {
                ArchiveFormat::Zip => create_zip_archive(&source_paths, &destination_path)?,
                ArchiveFormat::Tar => create_tar_archive(&source_paths, &destination_path)?,
                ArchiveFormat::TarGz => create_tar_gz_archive(&source_paths, &destination_path)?,
                ArchiveFormat::TarXz => create_tar_xz_archive(&source_paths, &destination_path)?,
                ArchiveFormat::Gz => create_gz_archive(&source_paths[0], &destination_path)?,
                ArchiveFormat::SevenZip => create_7z_archive(&source_paths, &destination_path)?,
                ArchiveFormat::Rar => return Err(
                    "rar compression is not supported. Use zip, tar, tar.gz, gz, or 7z instead."
                        .to_string(),
                ),
            }

            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id: job_id.clone(),
                    operation: "compress".to_string(),
                    format: format.label().to_string(),
                    stage: "finalizing".to_string(),
                    status: "running".to_string(),
                    message: "Finalizing archive and saving recent activity.".to_string(),
                    progress: 88,
                    source_summary: source_summary.clone(),
                    output_path: Some(path_to_string(&destination_path)),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );

            let result = ArchiveActionResult {
                operation: "compress",
                format: format.label(),
                output_path: path_to_string(&destination_path),
                message: format!(
                    "Created {} archive at {}",
                    format.label(),
                    destination_path.display()
                ),
            };

            append_archive_history(
                &app,
                ArchiveHistoryEntry {
                    id: archive_history_id(),
                    operation: "compress".to_string(),
                    format: format.label().to_string(),
                    source_summary: source_summary.clone(),
                    output_path: result.output_path.clone(),
                    timestamp_ms: unix_timestamp_ms(),
                },
            )?;

            Ok(result)
        })();

    match execution {
        Ok(result) => {
            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id,
                    operation: "compress".to_string(),
                    format: format.label().to_string(),
                    stage: "completed".to_string(),
                    status: "success".to_string(),
                    message: result.message.clone(),
                    progress: 100,
                    source_summary,
                    output_path: Some(result.output_path.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            Ok(result)
        }
        Err(error) => {
            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id,
                    operation: "compress".to_string(),
                    format: format.label().to_string(),
                    stage: "failed".to_string(),
                    status: "error".to_string(),
                    message: error.clone(),
                    progress: 100,
                    source_summary,
                    output_path: Some(path_to_string(&destination_path)),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            Err(error)
        }
    }
}

#[tauri::command]
pub(crate) fn extract_archive(
    app: AppHandle,
    request: ExtractRequest,
) -> Result<ArchiveActionResult, String> {
    let archive_path = normalize_archive_path(&request.archive_path)?;
    let conflict_policy = ConflictPolicy::from_input(request.conflict_policy.as_deref())?;
    let destination_directory = prepare_extract_destination(
        &normalize_directory_path(&request.destination_directory)?,
        conflict_policy,
    )?;
    let format = ArchiveFormat::detect_from_archive_path(&archive_path)?;
    let job_id = archive_job_id();
    let source_summary = path_to_string(&archive_path);

    emit_archive_job_event(
        &app,
        ArchiveJobEvent {
            job_id: job_id.clone(),
            operation: "extract".to_string(),
            format: format.label().to_string(),
            stage: "queued".to_string(),
            status: "queued".to_string(),
            message: "Extraction job queued.".to_string(),
            progress: 4,
            source_summary: source_summary.clone(),
            output_path: Some(path_to_string(&destination_directory)),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    emit_archive_job_event(
        &app,
        ArchiveJobEvent {
            job_id: job_id.clone(),
            operation: "extract".to_string(),
            format: format.label().to_string(),
            stage: "preparing".to_string(),
            status: "running".to_string(),
            message: "Preparing extraction destination.".to_string(),
            progress: 16,
            source_summary: source_summary.clone(),
            output_path: Some(path_to_string(&destination_directory)),
            timestamp_ms: unix_timestamp_ms(),
        },
    );

    let execution = (|| -> Result<ArchiveActionResult, String> {
        emit_archive_job_event(
            &app,
            ArchiveJobEvent {
                job_id: job_id.clone(),
                operation: "extract".to_string(),
                format: format.label().to_string(),
                stage: "processing".to_string(),
                status: "running".to_string(),
                message: format!("Unpacking {} archive contents.", format.label()),
                progress: 62,
                source_summary: source_summary.clone(),
                output_path: Some(path_to_string(&destination_directory)),
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        match format {
            ArchiveFormat::Zip => extract_zip_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::Tar => extract_tar_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::TarGz => extract_tar_gz_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::TarXz => extract_tar_xz_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::Gz => extract_gz_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::SevenZip => extract_7z_archive(&archive_path, &destination_directory)?,
            ArchiveFormat::Rar => extract_rar_archive(&archive_path, &destination_directory)?,
        }

        emit_archive_job_event(
            &app,
            ArchiveJobEvent {
                job_id: job_id.clone(),
                operation: "extract".to_string(),
                format: format.label().to_string(),
                stage: "finalizing".to_string(),
                status: "running".to_string(),
                message: "Recording extraction in recent activity.".to_string(),
                progress: 90,
                source_summary: source_summary.clone(),
                output_path: Some(path_to_string(&destination_directory)),
                timestamp_ms: unix_timestamp_ms(),
            },
        );

        let result = ArchiveActionResult {
            operation: "extract",
            format: format.label(),
            output_path: path_to_string(&destination_directory),
            message: format!(
                "Extracted {} archive into {}",
                format.label(),
                destination_directory.display()
            ),
        };

        append_archive_history(
            &app,
            ArchiveHistoryEntry {
                id: archive_history_id(),
                operation: "extract".to_string(),
                format: format.label().to_string(),
                source_summary: source_summary.clone(),
                output_path: result.output_path.clone(),
                timestamp_ms: unix_timestamp_ms(),
            },
        )?;

        Ok(result)
    })();

    match execution {
        Ok(result) => {
            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id,
                    operation: "extract".to_string(),
                    format: format.label().to_string(),
                    stage: "completed".to_string(),
                    status: "success".to_string(),
                    message: result.message.clone(),
                    progress: 100,
                    source_summary,
                    output_path: Some(result.output_path.clone()),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            Ok(result)
        }
        Err(error) => {
            emit_archive_job_event(
                &app,
                ArchiveJobEvent {
                    job_id,
                    operation: "extract".to_string(),
                    format: format.label().to_string(),
                    stage: "failed".to_string(),
                    status: "error".to_string(),
                    message: error.clone(),
                    progress: 100,
                    source_summary,
                    output_path: Some(path_to_string(&destination_directory)),
                    timestamp_ms: unix_timestamp_ms(),
                },
            );
            Err(error)
        }
    }
}
