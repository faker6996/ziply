use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, Write},
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "linux")]
use std::{env, ffi::OsString};

use bzip2::{read::BzDecoder, write::BzEncoder, Compression as BzCompression};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use hmac::{Hmac, Mac};
use rar::Archive as RarArchive;
use sevenz_rust2::{
    decompress_file, decompress_file_with_password, encoder_options::AesEncoderOptions,
    ArchiveReader, ArchiveWriter, EncoderMethod, Password,
};
use sha2::{Digest, Sha256};
use tar::Builder as TarBuilder;
use walkdir::WalkDir;
use xz2::{read::XzDecoder, write::XzEncoder};
use zip::unstable::write::FileOptionsExt;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::models::{ArchiveFormat, ArchivePreviewEntry, ArchivePreviewResult, ConflictPolicy};

#[derive(Clone, Copy, Eq, PartialEq)]
enum RarVariant {
    Rar4,
    Rar5,
    Unknown,
}

pub(crate) fn normalize_source_paths(source_paths: &[String]) -> Result<Vec<PathBuf>, String> {
    let normalized: Vec<PathBuf> = source_paths
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect();

    if normalized.is_empty() {
        return Err("add at least one source path to compress.".to_string());
    }

    for path in &normalized {
        if !path.exists() {
            return Err(format!("source path does not exist: {}", path.display()));
        }
    }

    Ok(normalized)
}

pub(crate) fn normalize_destination_path(
    destination: &str,
    format: ArchiveFormat,
) -> Result<PathBuf, String> {
    let trimmed = destination.trim();
    if trimmed.is_empty() {
        return Err("choose an output archive path.".to_string());
    }

    let lower = trimmed.to_ascii_lowercase();
    if matches!(format, ArchiveFormat::TarGz) {
        if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
            return Ok(PathBuf::from(trimmed));
        }
        return Ok(PathBuf::from(format!(
            "{trimmed}{}",
            format.preferred_suffix()
        )));
    }

    if matches!(format, ArchiveFormat::TarXz) {
        if lower.ends_with(".tar.xz") || lower.ends_with(".txz") {
            return Ok(PathBuf::from(trimmed));
        }
        return Ok(PathBuf::from(format!(
            "{trimmed}{}",
            format.preferred_suffix()
        )));
    }

    if matches!(format, ArchiveFormat::TarBz2) {
        if lower.ends_with(".tar.bz2") || lower.ends_with(".tbz2") {
            return Ok(PathBuf::from(trimmed));
        }
        return Ok(PathBuf::from(format!(
            "{trimmed}{}",
            format.preferred_suffix()
        )));
    }

    if lower.ends_with(format.preferred_suffix()) {
        return Ok(PathBuf::from(trimmed));
    }

    Ok(PathBuf::from(format!(
        "{trimmed}{}",
        format.preferred_suffix()
    )))
}

pub(crate) fn normalize_archive_path(archive_path: &str) -> Result<PathBuf, String> {
    let trimmed = archive_path.trim();
    if trimmed.is_empty() {
        return Err("choose an archive file to extract.".to_string());
    }

    let path = PathBuf::from(trimmed);
    if !path.is_file() {
        return Err(format!("archive file was not found: {}", path.display()));
    }

    Ok(resolve_rar_archive_entry_path(&path))
}

pub(crate) fn normalize_directory_path(directory: &str) -> Result<PathBuf, String> {
    let trimmed = directory.trim();
    if trimmed.is_empty() {
        return Err("choose a destination directory.".to_string());
    }

    Ok(PathBuf::from(trimmed))
}

pub(crate) fn normalize_password(password: Option<&str>) -> Option<String> {
    password
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(crate) fn resolve_archive_output_path(
    destination_path: &Path,
    conflict_policy: ConflictPolicy,
) -> Result<PathBuf, String> {
    if !destination_path.exists() {
        return Ok(destination_path.to_path_buf());
    }

    match conflict_policy {
        ConflictPolicy::KeepBoth => Ok(unique_conflict_path(destination_path)),
        ConflictPolicy::Overwrite => {
            if destination_path.is_dir() {
                return Err(format!(
                    "destination archive path points to an existing directory: {}",
                    destination_path.display()
                ));
            }

            fs::remove_file(destination_path).map_err(|error| {
                format!(
                    "failed to replace existing archive {}: {error}",
                    destination_path.display()
                )
            })?;
            Ok(destination_path.to_path_buf())
        }
        ConflictPolicy::Stop => Err(format!(
            "destination archive already exists: {}",
            destination_path.display()
        )),
    }
}

pub(crate) fn prepare_extract_destination(
    destination_directory: &Path,
    conflict_policy: ConflictPolicy,
) -> Result<PathBuf, String> {
    if !destination_directory.exists() {
        fs::create_dir_all(destination_directory).map_err(|error| {
            format!(
                "failed to create extraction directory {}: {error}",
                destination_directory.display()
            )
        })?;
        return Ok(destination_directory.to_path_buf());
    }

    if destination_directory.is_dir() && directory_is_empty(destination_directory)? {
        return Ok(destination_directory.to_path_buf());
    }

    match conflict_policy {
        ConflictPolicy::KeepBoth => {
            let next_directory = unique_conflict_path(destination_directory);
            fs::create_dir_all(&next_directory).map_err(|error| {
                format!(
                    "failed to create extraction directory {}: {error}",
                    next_directory.display()
                )
            })?;
            Ok(next_directory)
        }
        ConflictPolicy::Overwrite => {
            remove_existing_path(destination_directory)?;
            fs::create_dir_all(destination_directory).map_err(|error| {
                format!(
                    "failed to create extraction directory {}: {error}",
                    destination_directory.display()
                )
            })?;
            Ok(destination_directory.to_path_buf())
        }
        ConflictPolicy::Stop => Err(format!(
            "destination folder already exists and is not empty: {}",
            destination_directory.display()
        )),
    }
}

pub(crate) fn create_zip_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
    password: Option<&str>,
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let writer = BufWriter::new(file);
    let mut archive = ZipWriter::new(writer);
    let normalized_password = normalize_password(password);

    for source_path in source_paths {
        append_zip_source(&mut archive, source_path, normalized_password.as_deref())?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize zip archive: {error}"))?;
    Ok(())
}

fn append_zip_source<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    source_path: &Path,
    password: Option<&str>,
) -> Result<(), String> {
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);
    let file_options = if let Some(password) = password {
        options.with_deprecated_encryption(password.as_bytes())
    } else {
        options
    };

    if source_path.is_file() {
        let archive_name = source_path
            .file_name()
            .ok_or_else(|| format!("source path has no file name: {}", source_path.display()))?;
        return append_zip_file(archive, source_path, Path::new(archive_name), file_options);
    }

    let source_parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    for entry in WalkDir::new(source_path) {
        let entry = entry.map_err(|error| format!("failed to walk source path: {error}"))?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(source_parent)
            .map_err(|error| format!("failed to compute relative path: {error}"))?;
        let archive_name = relative_path.to_string_lossy().replace('\\', "/");

        if entry.file_type().is_dir() {
            archive
                .add_directory(format!("{archive_name}/"), options)
                .map_err(|error| format!("failed to add directory to zip archive: {error}"))?;
            continue;
        }

        append_zip_file(archive, path, relative_path, file_options)?;
    }

    Ok(())
}

fn append_zip_file<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    source_path: &Path,
    relative_path: &Path,
    options: FileOptions,
) -> Result<(), String> {
    let archive_name = relative_path.to_string_lossy().replace('\\', "/");
    let mut file = BufReader::new(File::open(source_path).map_err(|error| {
        format!(
            "failed to open source file {}: {error}",
            source_path.display()
        )
    })?);

    archive
        .start_file(archive_name, options)
        .map_err(|error| format!("failed to start zip entry: {error}"))?;
    io::copy(&mut file, archive).map_err(|error| {
        format!(
            "failed to write archive contents for {}: {error}",
            source_path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn create_tar_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let writer = BufWriter::new(file);
    let mut archive = TarBuilder::new(writer);

    for source_path in source_paths {
        append_tar_source(&mut archive, source_path)?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize tar archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_tar_gz_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let encoder = GzEncoder::new(BufWriter::new(file), Compression::default());
    let mut archive = TarBuilder::new(encoder);

    for source_path in source_paths {
        append_tar_source(&mut archive, source_path)?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize tar.gz archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_tar_xz_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let encoder = XzEncoder::new(BufWriter::new(file), 6);
    let mut archive = TarBuilder::new(encoder);

    for source_path in source_paths {
        append_tar_source(&mut archive, source_path)?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize tar.xz archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_tar_bz2_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let encoder = BzEncoder::new(BufWriter::new(file), BzCompression::default());
    let mut archive = TarBuilder::new(encoder);

    for source_path in source_paths {
        append_tar_source(&mut archive, source_path)?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize tar.bz2 archive: {error}"))?;
    Ok(())
}

fn append_tar_source<W: Write>(
    archive: &mut TarBuilder<W>,
    source_path: &Path,
) -> Result<(), String> {
    let archive_name = source_path
        .file_name()
        .ok_or_else(|| format!("source path has no file name: {}", source_path.display()))?;

    if source_path.is_dir() {
        archive
            .append_dir_all(archive_name, source_path)
            .map_err(|error| format!("failed to append directory to tar archive: {error}"))?;
    } else {
        archive
            .append_path_with_name(source_path, archive_name)
            .map_err(|error| format!("failed to append file to tar archive: {error}"))?;
    }

    Ok(())
}

pub(crate) fn create_gz_archive(source_path: &Path, destination_path: &Path) -> Result<(), String> {
    let input_file = File::open(source_path).map_err(|error| {
        format!(
            "failed to open source file {}: {error}",
            source_path.display()
        )
    })?;
    let output_file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let mut reader = BufReader::new(input_file);
    let mut encoder = GzEncoder::new(BufWriter::new(output_file), Compression::default());

    io::copy(&mut reader, &mut encoder)
        .map_err(|error| format!("failed to write gz archive: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("failed to finalize gz archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_xz_archive(source_path: &Path, destination_path: &Path) -> Result<(), String> {
    let input_file = File::open(source_path).map_err(|error| {
        format!(
            "failed to open source file {}: {error}",
            source_path.display()
        )
    })?;
    let output_file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let mut reader = BufReader::new(input_file);
    let mut encoder = XzEncoder::new(BufWriter::new(output_file), 6);

    io::copy(&mut reader, &mut encoder)
        .map_err(|error| format!("failed to write xz archive: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("failed to finalize xz archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_bz2_archive(
    source_path: &Path,
    destination_path: &Path,
) -> Result<(), String> {
    let input_file = File::open(source_path).map_err(|error| {
        format!(
            "failed to open source file {}: {error}",
            source_path.display()
        )
    })?;
    let output_file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let mut reader = BufReader::new(input_file);
    let mut encoder = BzEncoder::new(BufWriter::new(output_file), BzCompression::default());

    io::copy(&mut reader, &mut encoder)
        .map_err(|error| format!("failed to write bz2 archive: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("failed to finalize bz2 archive: {error}"))?;
    Ok(())
}

pub(crate) fn create_7z_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
    password: Option<&str>,
) -> Result<(), String> {
    let mut writer = ArchiveWriter::create(destination_path)
        .map_err(|error| format!("failed to create 7z archive writer: {error}"))?;

    if let Some(password) = normalize_password(password) {
        writer.set_content_methods(vec![
            AesEncoderOptions::new(Password::new(&password)).into(),
            EncoderMethod::LZMA2.into(),
        ]);
    }

    for source_path in source_paths {
        writer
            .push_source_path_non_solid(source_path, |_| true)
            .map_err(|error| {
                format!(
                    "failed to add source path {} to 7z archive: {error}",
                    source_path.display()
                )
            })?;
    }

    writer
        .finish()
        .map_err(|error| format!("failed to finalize 7z archive: {error}"))?;
    Ok(())
}

pub(crate) fn extract_zip_archive(
    archive_path: &Path,
    destination_directory: &Path,
    password: Option<&str>,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("failed to read zip archive: {error}"))?;
    let normalized_password = normalize_password(password);
    let selected_entries = normalize_selected_entries(selected_entries);

    for index in 0..archive.len() {
        let mut entry = if let Some(password) = normalized_password.as_deref() {
            match archive.by_index_decrypt(index, password.as_bytes()) {
                Ok(Ok(entry)) => entry,
                Ok(Err(_)) => return Err("invalid password for zip archive.".to_string()),
                Err(error) => return Err(format!("failed to read zip entry: {error}")),
            }
        } else {
            archive
                .by_index(index)
                .map_err(|error| format!("failed to read zip entry: {error}"))?
        };
        let relative_path = entry
            .enclosed_name()
            .ok_or_else(|| format!("zip entry contains an unsafe path: {}", entry.name()))?;
        if !should_extract_entry(
            &relative_path.to_string_lossy(),
            selected_entries.as_deref(),
        ) {
            continue;
        }
        let output_path = destination_directory.join(relative_path);

        if entry.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| {
                format!(
                    "failed to create extracted directory {}: {error}",
                    output_path.display()
                )
            })?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create extracted file parent {}: {error}",
                    parent.display()
                )
            })?;
        }

        let mut output_file = BufWriter::new(File::create(&output_path).map_err(|error| {
            format!(
                "failed to create extracted file {}: {error}",
                output_path.display()
            )
        })?);
        io::copy(&mut entry, &mut output_file).map_err(|error| {
            format!(
                "failed to write extracted file {}: {error}",
                output_path.display()
            )
        })?;
    }

    Ok(())
}

pub(crate) fn extract_tar_archive(
    archive_path: &Path,
    destination_directory: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let reader = BufReader::new(file);
    let mut archive = tar::Archive::new(reader);
    unpack_tar_entries(&mut archive, destination_directory, selected_entries)
}

pub(crate) fn extract_tar_gz_archive(
    archive_path: &Path,
    destination_directory: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, destination_directory, selected_entries)
}

pub(crate) fn extract_tar_xz_archive(
    archive_path: &Path,
    destination_directory: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = XzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, destination_directory, selected_entries)
}

pub(crate) fn extract_tar_bz2_archive(
    archive_path: &Path,
    destination_directory: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = BzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, destination_directory, selected_entries)
}

fn unpack_tar_entries<R: Read>(
    archive: &mut tar::Archive<R>,
    destination_directory: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let selected_entries = normalize_selected_entries(selected_entries);
    let entries = archive
        .entries()
        .map_err(|error| format!("failed to read tar archive entries: {error}"))?;

    for entry in entries {
        let mut entry =
            entry.map_err(|error| format!("failed to inspect tar archive entry: {error}"))?;
        let relative_path = entry
            .path()
            .map_err(|error| format!("failed to read tar archive entry path: {error}"))?
            .into_owned();
        if !should_extract_entry(
            &relative_path.to_string_lossy(),
            selected_entries.as_deref(),
        ) {
            continue;
        }
        let output_path = safe_join(destination_directory, &relative_path)?;

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create extracted file parent {}: {error}",
                    parent.display()
                )
            })?;
        }

        entry.unpack(&output_path).map_err(|error| {
            format!(
                "failed to extract archive entry into {}: {error}",
                output_path.display()
            )
        })?;
    }

    Ok(())
}

pub(crate) fn extract_gz_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".gz"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "gz archive name must end with .gz".to_string())?;
    let output_path = destination_directory.join(output_name);

    let input_file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut decoder = GzDecoder::new(BufReader::new(input_file));
    let output_file = File::create(&output_path).map_err(|error| {
        format!(
            "failed to create extracted file {}: {error}",
            output_path.display()
        )
    })?;
    let mut writer = BufWriter::new(output_file);

    io::copy(&mut decoder, &mut writer).map_err(|error| {
        format!(
            "failed to extract gz archive into {}: {error}",
            output_path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn extract_xz_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".xz"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "xz archive name must end with .xz".to_string())?;
    let output_path = destination_directory.join(output_name);

    let input_file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut decoder = XzDecoder::new(BufReader::new(input_file));
    let output_file = File::create(&output_path).map_err(|error| {
        format!(
            "failed to create extracted file {}: {error}",
            output_path.display()
        )
    })?;
    let mut writer = BufWriter::new(output_file);

    io::copy(&mut decoder, &mut writer).map_err(|error| {
        format!(
            "failed to extract xz archive into {}: {error}",
            output_path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn extract_bz2_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".bz2"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "bz2 archive name must end with .bz2".to_string())?;
    let output_path = destination_directory.join(output_name);

    let input_file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut decoder = BzDecoder::new(BufReader::new(input_file));
    let output_file = File::create(&output_path).map_err(|error| {
        format!(
            "failed to create extracted file {}: {error}",
            output_path.display()
        )
    })?;
    let mut writer = BufWriter::new(output_file);

    io::copy(&mut decoder, &mut writer).map_err(|error| {
        format!(
            "failed to extract bz2 archive into {}: {error}",
            output_path.display()
        )
    })?;
    Ok(())
}

pub(crate) fn extract_7z_archive(
    archive_path: &Path,
    destination_directory: &Path,
    password: Option<&str>,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let password = normalize_password(password)
        .map(|value| Password::new(&value))
        .unwrap_or_else(Password::empty);
    let selected_entries = normalize_selected_entries(selected_entries);

    if selected_entries.is_none() {
        return if password.is_empty() {
            decompress_file(archive_path, destination_directory)
                .map_err(|error| format!("failed to extract 7z archive: {error}"))
        } else {
            decompress_file_with_password(archive_path, destination_directory, password)
                .map_err(|error| format!("failed to extract 7z archive: {error}"))
        };
    }

    let mut reader = ArchiveReader::open(archive_path, password)
        .map_err(|error| format!("failed to read 7z archive: {error}"))?;
    let selected_entries = selected_entries.expect("selection checked above");

    reader
        .for_each_entries(|entry, input| {
            if !should_extract_entry(entry.name(), Some(&selected_entries)) {
                return Ok(true);
            }

            let output_path = safe_join(destination_directory, Path::new(entry.name()))
                .map_err(io::Error::other)?;

            if entry.is_directory() {
                fs::create_dir_all(&output_path)?;
                return Ok(true);
            }

            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let output_file = File::create(&output_path)?;
            let mut writer = BufWriter::new(output_file);
            io::copy(input, &mut writer)?;
            Ok(true)
        })
        .map_err(|error| format!("failed to extract 7z archive: {error}"))
}

pub(crate) fn extract_rar_archive(
    archive_path: &Path,
    destination_directory: &Path,
    password: Option<&str>,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    let selected_entries = normalize_selected_entries(selected_entries);
    let password = normalize_password(password);
    let temp_extract_directory = unique_preview_directory("rar-extract");
    let extraction_result =
        extract_rar_archive_into(archive_path, &temp_extract_directory, password.as_deref());

    let result = match extraction_result {
        Ok(archive) => validate_rar_extraction(&archive, password.as_deref()).and_then(|_| {
            copy_selected_tree(
                &temp_extract_directory,
                destination_directory,
                selected_entries.as_deref(),
            )
        }),
        Err(error) => Err(error),
    };

    let _ = fs::remove_dir_all(&temp_extract_directory);
    result
}

fn preview_rar_archive(
    archive_path: &Path,
    limit: usize,
    password: Option<&str>,
) -> Result<ArchivePreviewResult, String> {
    let preview_workspace = unique_preview_directory("rar");
    let extract_result = extract_rar_archive(archive_path, &preview_workspace, password, None);

    let result = match extract_result {
        Ok(()) => collect_directory_preview(&preview_workspace, "rar", limit),
        Err(error) => Err(error),
    };

    let _ = fs::remove_dir_all(&preview_workspace);
    result
}

fn extract_rar_archive_into(
    archive_path: &Path,
    destination_directory: &Path,
    password: Option<&str>,
) -> Result<RarArchive, String> {
    validate_rar_entry_path(archive_path)?;
    RarArchive::extract_all(
        &path_to_string(archive_path),
        &path_to_string(destination_directory),
        password.unwrap_or(""),
    )
    .map_err(|error| refine_rar_error(archive_path, &error.to_string()))
}

fn refine_rar_error(archive_path: &Path, error: &str) -> String {
    let normalized = error.to_ascii_lowercase();
    let variant = detect_rar_variant(archive_path);

    if normalized.contains("password") || normalized.contains("encrypted") {
        return "invalid password for rar archive.".to_string();
    }

    if normalized.contains("unexpected end of file")
        || normalized.contains("failed to fill whole buffer")
        || normalized.contains("can't find volume")
    {
        return "ziply could not find every volume for this multipart rar archive. Open the first volume and keep the remaining parts in the same folder."
            .to_string();
    }

    if normalized.contains("can't read rar archive block")
        || normalized.contains("unsupported")
        || normalized.contains("unknown header")
    {
        return match variant {
            RarVariant::Rar4 => "this rar archive uses an older rar4 variant that Ziply cannot extract reliably yet."
                .to_string(),
            RarVariant::Rar5 => {
                "ziply could not read this rar5 archive cleanly. The file may be damaged or use an unsupported rar feature."
                    .to_string()
            }
            RarVariant::Unknown => {
                "ziply could not read this rar archive cleanly. The file may be damaged or use an unsupported rar feature."
                    .to_string()
            }
        };
    }

    format!("failed to extract rar archive: {error}")
}

fn validate_rar_entry_path(archive_path: &Path) -> Result<(), String> {
    if let Some(part_number) = rar_part_number(archive_path) {
        if part_number > 1 {
            return Err(format!(
                "open the first rar volume instead of {}.",
                archive_path.display()
            ));
        }
    }

    if is_old_style_rar_segment(archive_path) {
        return Err(format!(
            "open the main .rar file instead of {}.",
            archive_path.display()
        ));
    }

    Ok(())
}

fn validate_rar_extraction(archive: &RarArchive, password: Option<&str>) -> Result<(), String> {
    let contains_encrypted_files = archive
        .files
        .iter()
        .any(|file| file.extra.file_encryption.is_some());

    if contains_encrypted_files && password.is_none() {
        return Err("rar archive requires a password before extraction.".to_string());
    }

    for file in &archive.files {
        let Some(encryption) = file.extra.file_encryption.as_ref() else {
            continue;
        };
        let Some(password) = password else {
            return Err("rar archive requires a password before extraction.".to_string());
        };

        match verify_rar_password(
            password,
            &encryption.salt,
            encryption.kdf_count,
            &encryption.pw_check,
        ) {
            Ok(true) => {}
            Ok(false) => return Err("invalid password for rar archive.".to_string()),
            Err(error) => return Err(error),
        }
    }

    Ok(())
}

fn verify_rar_password(
    password: &str,
    salt: &[u8; 16],
    kdf_count: u8,
    stored_pw_check: &[u8; 12],
) -> Result<bool, String> {
    let checksum = Sha256::digest(&stored_pw_check[..8]);
    if checksum[..4] != stored_pw_check[8..12] {
        return Err(
            "ziply could not validate password-check metadata for this rar archive.".to_string(),
        );
    }

    let derived_pw_check = derive_rar_password_check(password, salt, kdf_count)?;
    Ok(derived_pw_check == stored_pw_check[..8])
}

fn derive_rar_password_check(
    password: &str,
    salt: &[u8; 16],
    kdf_count: u8,
) -> Result<[u8; 8], String> {
    let rounds = 1_u32
        .checked_shl(u32::from(kdf_count))
        .ok_or_else(|| "rar password KDF count is too large.".to_string())?;
    let pwd = password.as_bytes();

    let mut salt_data = [0_u8; 20];
    salt_data[..16].copy_from_slice(salt);
    salt_data[19] = 1;

    let mut u1 = hmac_sha256(pwd, &salt_data)?;
    let mut fn_value = u1;
    let counts = [rounds.saturating_sub(1), 16, 16];
    let mut pw_check_value = [0_u8; 32];

    for (stage, iterations) in counts.into_iter().enumerate() {
        for _ in 0..iterations {
            let next = hmac_sha256(pwd, &u1)?;
            u1 = next;
            for (lhs, rhs) in fn_value.iter_mut().zip(u1.iter()) {
                *lhs ^= *rhs;
            }
        }

        if stage == 2 {
            pw_check_value.copy_from_slice(&fn_value);
        }
    }

    let mut result = [0_u8; 8];
    for (index, byte) in pw_check_value.iter().enumerate() {
        result[index % 8] ^= *byte;
    }
    Ok(result)
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .map_err(|error| format!("failed to initialize HMAC-SHA256: {error}"))?;
    mac.update(data);
    let digest = mac.finalize().into_bytes();
    let mut result = [0_u8; 32];
    result.copy_from_slice(&digest);
    Ok(result)
}

fn detect_rar_variant(archive_path: &Path) -> RarVariant {
    let mut header = [0_u8; 8];
    let Ok(mut file) = File::open(archive_path) else {
        return RarVariant::Unknown;
    };
    let Ok(read) = file.read(&mut header) else {
        return RarVariant::Unknown;
    };

    if read >= 8 && header == [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x01, 0x00] {
        return RarVariant::Rar5;
    }

    if read >= 7 && header[..7] == [0x52, 0x61, 0x72, 0x21, 0x1a, 0x07, 0x00] {
        return RarVariant::Rar4;
    }

    RarVariant::Unknown
}

pub(crate) fn resolve_rar_archive_entry_path(path: &Path) -> PathBuf {
    if let Some(main_volume) = resolve_old_style_rar_main_volume(path) {
        return main_volume;
    }

    if let Some(first_volume) = resolve_parted_rar_first_volume(path) {
        return first_volume;
    }

    path.to_path_buf()
}

fn rar_part_number(archive_path: &Path) -> Option<u32> {
    let lower = archive_path.file_name()?.to_str()?.to_ascii_lowercase();
    let start = lower.find(".part")?;
    let digits = lower[start + 5..].strip_suffix(".rar")?;
    digits.parse::<u32>().ok()
}

fn is_old_style_rar_segment(archive_path: &Path) -> bool {
    archive_path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| {
            let value = value.to_ascii_lowercase();
            value.len() == 3
                && value.starts_with('r')
                && value[1..].chars().all(|ch| ch.is_ascii_digit())
        })
}

fn resolve_parted_rar_first_volume(path: &Path) -> Option<PathBuf> {
    let file_name = path.file_name()?.to_str()?;
    let lower = file_name.to_ascii_lowercase();
    let start = lower.find(".part")?;
    let number = lower[start + 5..].strip_suffix(".rar")?;
    if number.is_empty() || !number.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let first_name = format!("{}part1.rar", &file_name[..start + 1]);
    let candidate = path.with_file_name(first_name);
    candidate.is_file().then_some(candidate)
}

fn resolve_old_style_rar_main_volume(path: &Path) -> Option<PathBuf> {
    if !is_old_style_rar_segment(path) {
        return None;
    }

    let stem = path.file_stem()?.to_str()?;
    let candidate = path.with_file_name(format!("{stem}.rar"));
    candidate.is_file().then_some(candidate)
}

fn normalize_selected_entries(selected_entries: Option<&[String]>) -> Option<Vec<String>> {
    let selected_entries = selected_entries
        .unwrap_or_default()
        .iter()
        .map(|entry| normalize_archive_entry_name(entry))
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();

    if selected_entries.is_empty() {
        None
    } else {
        Some(selected_entries)
    }
}

fn normalize_archive_entry_name(value: &str) -> String {
    value
        .trim()
        .replace('\\', "/")
        .trim_matches('/')
        .to_string()
}

fn should_extract_entry(entry_path: &str, selected_entries: Option<&[String]>) -> bool {
    let Some(selected_entries) = selected_entries else {
        return true;
    };

    let normalized_entry = normalize_archive_entry_name(entry_path);
    selected_entries.iter().any(|selected_entry| {
        normalized_entry == *selected_entry
            || normalized_entry.starts_with(&format!("{selected_entry}/"))
    })
}

pub(crate) fn preview_archive(
    archive_path: &Path,
    limit: usize,
    password: Option<&str>,
) -> Result<ArchivePreviewResult, String> {
    let format = ArchiveFormat::detect_from_archive_path(archive_path)?;
    let visible_limit = limit.max(1);

    match format {
        ArchiveFormat::Zip => preview_zip_archive(archive_path, visible_limit, password),
        ArchiveFormat::Tar => preview_tar_archive(archive_path, visible_limit),
        ArchiveFormat::TarGz => preview_tar_gz_archive(archive_path, visible_limit),
        ArchiveFormat::TarBz2 => preview_tar_bz2_archive(archive_path, visible_limit),
        ArchiveFormat::TarXz => preview_tar_xz_archive(archive_path, visible_limit),
        ArchiveFormat::Xz => preview_xz_archive(archive_path),
        ArchiveFormat::Bz2 => preview_bz2_archive(archive_path),
        ArchiveFormat::Gz => preview_gz_archive(archive_path),
        ArchiveFormat::SevenZip => preview_7z_archive(archive_path, visible_limit, password),
        ArchiveFormat::Rar => preview_rar_archive(archive_path, visible_limit, password),
    }
}

fn preview_zip_archive(
    archive_path: &Path,
    limit: usize,
    password: Option<&str>,
) -> Result<ArchivePreviewResult, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("failed to read zip archive: {error}"))?;
    let total_entries = archive.len();
    let mut visible_entries = Vec::with_capacity(total_entries.min(limit));
    let normalized_password = normalize_password(password);

    for index in 0..total_entries.min(limit) {
        let entry = if let Some(password) = normalized_password.as_deref() {
            match archive.by_index_decrypt(index, password.as_bytes()) {
                Ok(Ok(entry)) => entry,
                Ok(Err(_)) => return Err("invalid password for zip archive.".to_string()),
                Err(error) => return Err(format!("failed to read zip entry: {error}")),
            }
        } else {
            archive
                .by_index(index)
                .map_err(|error| format!("failed to read zip entry: {error}"))?
        };
        visible_entries.push(ArchivePreviewEntry {
            path: entry.name().to_string(),
            kind: if entry.is_dir() { "directory" } else { "file" },
            size: if entry.is_dir() {
                None
            } else {
                Some(entry.size())
            },
        });
    }

    Ok(ArchivePreviewResult {
        format: "zip",
        total_entries,
        hidden_entry_count: total_entries.saturating_sub(visible_entries.len()),
        visible_entries,
        note: None,
    })
}

fn preview_tar_archive(archive_path: &Path, limit: usize) -> Result<ArchivePreviewResult, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let reader = BufReader::new(file);
    let archive = tar::Archive::new(reader);
    collect_tar_preview(archive, "tar", limit)
}

fn preview_tar_gz_archive(
    archive_path: &Path,
    limit: usize,
) -> Result<ArchivePreviewResult, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let archive = tar::Archive::new(decoder);
    collect_tar_preview(archive, "tar.gz", limit)
}

fn preview_tar_xz_archive(
    archive_path: &Path,
    limit: usize,
) -> Result<ArchivePreviewResult, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = XzDecoder::new(BufReader::new(file));
    let archive = tar::Archive::new(decoder);
    collect_tar_preview(archive, "tar.xz", limit)
}

fn preview_tar_bz2_archive(
    archive_path: &Path,
    limit: usize,
) -> Result<ArchivePreviewResult, String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = BzDecoder::new(BufReader::new(file));
    let archive = tar::Archive::new(decoder);
    collect_tar_preview(archive, "tar.bz2", limit)
}

fn collect_tar_preview<R: Read>(
    mut archive: tar::Archive<R>,
    format: &'static str,
    limit: usize,
) -> Result<ArchivePreviewResult, String> {
    let entries = archive
        .entries()
        .map_err(|error| format!("failed to read tar archive entries: {error}"))?;
    let mut total_entries = 0usize;
    let mut visible_entries = Vec::new();

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to inspect tar archive entry: {error}"))?;
        let path = entry
            .path()
            .map_err(|error| format!("failed to read tar archive entry path: {error}"))?
            .to_string_lossy()
            .into_owned();
        let header = entry.header();
        let is_directory = header.entry_type().is_dir();

        total_entries += 1;
        if visible_entries.len() < limit {
            visible_entries.push(ArchivePreviewEntry {
                path,
                kind: if is_directory { "directory" } else { "file" },
                size: if is_directory {
                    None
                } else {
                    Some(header.size().unwrap_or(0))
                },
            });
        }
    }

    Ok(ArchivePreviewResult {
        format,
        total_entries,
        hidden_entry_count: total_entries.saturating_sub(visible_entries.len()),
        visible_entries,
        note: None,
    })
}

fn collect_directory_preview(
    root: &Path,
    format: &'static str,
    limit: usize,
) -> Result<ArchivePreviewResult, String> {
    let mut total_entries = 0usize;
    let mut visible_entries = Vec::new();

    for entry in WalkDir::new(root).min_depth(1) {
        let entry =
            entry.map_err(|error| format!("failed to inspect extracted preview entry: {error}"))?;
        let relative_path = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| format!("failed to compute extracted preview path: {error}"))?;

        total_entries += 1;
        if visible_entries.len() >= limit {
            continue;
        }

        let metadata = entry
            .metadata()
            .map_err(|error| format!("failed to read extracted preview metadata: {error}"))?;
        visible_entries.push(ArchivePreviewEntry {
            path: relative_path.to_string_lossy().replace('\\', "/"),
            kind: if metadata.is_dir() {
                "directory"
            } else {
                "file"
            },
            size: if metadata.is_dir() {
                None
            } else {
                Some(metadata.len())
            },
        });
    }

    Ok(ArchivePreviewResult {
        format,
        total_entries,
        hidden_entry_count: total_entries.saturating_sub(visible_entries.len()),
        visible_entries,
        note: if format == "rar" {
            Some(
                "RAR preview is currently generated from Ziply's native extraction path."
                    .to_string(),
            )
        } else {
            None
        },
    })
}

fn copy_selected_tree(
    source_root: &Path,
    destination_root: &Path,
    selected_entries: Option<&[String]>,
) -> Result<(), String> {
    for entry in WalkDir::new(source_root).min_depth(1) {
        let entry =
            entry.map_err(|error| format!("failed to inspect extracted rar entry: {error}"))?;
        let relative_path = entry
            .path()
            .strip_prefix(source_root)
            .map_err(|error| format!("failed to compute selected rar entry path: {error}"))?;
        let relative_string = relative_path.to_string_lossy().replace('\\', "/");

        if !should_extract_entry(&relative_string, selected_entries) {
            continue;
        }

        let output_path = safe_join(destination_root, relative_path)?;
        let metadata = entry
            .metadata()
            .map_err(|error| format!("failed to read selected rar entry metadata: {error}"))?;

        if metadata.is_dir() {
            fs::create_dir_all(&output_path).map_err(|error| {
                format!(
                    "failed to create selected rar directory {}: {error}",
                    output_path.display()
                )
            })?;
            continue;
        }

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create selected rar parent directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        fs::copy(entry.path(), &output_path).map_err(|error| {
            format!(
                "failed to copy selected rar entry into {}: {error}",
                output_path.display()
            )
        })?;
    }

    Ok(())
}

fn unique_preview_directory(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ziply-preview-{label}-{suffix}"));
    let _ = fs::create_dir_all(&path);
    path
}

fn preview_gz_archive(archive_path: &Path) -> Result<ArchivePreviewResult, String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".gz"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "gz archive name must end with .gz".to_string())?;

    Ok(ArchivePreviewResult {
        format: "gz",
        total_entries: 1,
        hidden_entry_count: 0,
        visible_entries: vec![ArchivePreviewEntry {
            path: output_name.to_string(),
            kind: "file",
            size: None,
        }],
        note: Some(
            "Gzip archives usually contain a single file stream without folder structure."
                .to_string(),
        ),
    })
}

fn preview_bz2_archive(archive_path: &Path) -> Result<ArchivePreviewResult, String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".bz2"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "bz2 archive name must end with .bz2".to_string())?;

    Ok(ArchivePreviewResult {
        format: "bz2",
        total_entries: 1,
        hidden_entry_count: 0,
        visible_entries: vec![ArchivePreviewEntry {
            path: output_name.to_string(),
            kind: "file",
            size: None,
        }],
        note: Some(
            "Bzip2 archives usually contain a single file stream without folder structure."
                .to_string(),
        ),
    })
}

fn preview_xz_archive(archive_path: &Path) -> Result<ArchivePreviewResult, String> {
    let output_name = archive_path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_suffix(".xz"))
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "xz archive name must end with .xz".to_string())?;

    Ok(ArchivePreviewResult {
        format: "xz",
        total_entries: 1,
        hidden_entry_count: 0,
        visible_entries: vec![ArchivePreviewEntry {
            path: output_name.to_string(),
            kind: "file",
            size: None,
        }],
        note: Some(
            "XZ archives usually contain a single file stream without folder structure."
                .to_string(),
        ),
    })
}

fn preview_7z_archive(
    archive_path: &Path,
    limit: usize,
    password: Option<&str>,
) -> Result<ArchivePreviewResult, String> {
    let password = normalize_password(password)
        .map(|password| Password::new(&password))
        .unwrap_or_else(Password::empty);
    let reader = ArchiveReader::open(archive_path, password)
        .map_err(|error| format!("failed to read 7z archive: {error}"))?;
    let total_entries = reader.archive().files.len();
    let visible_entries = reader
        .archive()
        .files
        .iter()
        .take(limit)
        .map(|entry| ArchivePreviewEntry {
            path: entry.name.clone(),
            kind: if entry.is_directory {
                "directory"
            } else {
                "file"
            },
            size: if entry.is_directory {
                None
            } else {
                Some(entry.size)
            },
        })
        .collect::<Vec<_>>();

    Ok(ArchivePreviewResult {
        format: "7z",
        total_entries,
        hidden_entry_count: total_entries.saturating_sub(visible_entries.len()),
        visible_entries,
        note: None,
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn find_command(candidates: &[&str]) -> Option<PathBuf> {
    let path_os = env::var_os("PATH")?;
    let extensions = command_extensions();

    for directory in env::split_paths(&path_os) {
        for candidate in candidates {
            for extension in &extensions {
                let executable_name = if extension.is_empty() {
                    OsString::from(candidate)
                } else {
                    OsString::from(format!("{candidate}{extension}"))
                };
                let executable_path = directory.join(executable_name);
                if executable_path.is_file() {
                    return Some(executable_path);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn command_extensions() -> Vec<String> {
    vec![String::new()]
}

fn safe_join(base_directory: &Path, relative_path: &Path) -> Result<PathBuf, String> {
    let mut output_path = PathBuf::from(base_directory);

    for component in relative_path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(value) => output_path.push(value),
            Component::Prefix(_) | Component::RootDir | Component::ParentDir => {
                return Err(format!(
                    "archive entry contains an unsafe path: {}",
                    relative_path.display()
                ));
            }
        }
    }

    Ok(output_path)
}

fn remove_existing_path(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| {
            format!(
                "failed to replace existing directory {}: {error}",
                path.display()
            )
        })
    } else {
        fs::remove_file(path).map_err(|error| {
            format!(
                "failed to replace existing file {}: {error}",
                path.display()
            )
        })
    }
}

fn directory_is_empty(path: &Path) -> Result<bool, String> {
    let mut entries = fs::read_dir(path).map_err(|error| {
        format!(
            "failed to inspect destination folder {}: {error}",
            path.display()
        )
    })?;
    Ok(entries.next().is_none())
}

fn unique_conflict_path(path: &Path) -> PathBuf {
    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .unwrap_or("ziply-output");
    let (base_name, suffix) = split_conflict_name(file_name, path.is_dir());

    for index in 1..10_000 {
        let candidate_name = format!("{base_name} ({index}){suffix}");
        let candidate = parent.join(candidate_name);
        if !candidate.exists() {
            return candidate;
        }
    }

    parent.join(format!("{base_name}-copy{suffix}"))
}

fn split_conflict_name(file_name: &str, is_directory: bool) -> (String, String) {
    if is_directory {
        return (file_name.to_string(), String::new());
    }

    for suffix in [".tar.gz", ".tar.bz2", ".tar.xz"] {
        if let Some(base_name) = file_name.strip_suffix(suffix) {
            return (base_name.to_string(), suffix.to_string());
        }
    }

    if let Some((base_name, extension)) = file_name.rsplit_once('.') {
        if !base_name.is_empty() && !extension.is_empty() {
            return (base_name.to_string(), format!(".{extension}"));
        }
    }

    (file_name.to_string(), String::new())
}

pub(crate) fn is_supported_archive_path(path: &Path) -> bool {
    ArchiveFormat::detect_from_archive_path(path).is_ok()
}

pub(crate) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
