use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, Write},
    path::{Component, Path, PathBuf},
};

#[cfg(target_os = "linux")]
use std::{env, ffi::OsString};

use bzip2::{read::BzDecoder, write::BzEncoder, Compression as BzCompression};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use sevenz_rust2::{
    decompress_file, decompress_file_with_password, encoder_options::AesEncoderOptions,
    ArchiveReader, ArchiveWriter, EncoderMethod, Password,
};
use tar::Builder as TarBuilder;
use walkdir::WalkDir;
use xz2::{read::XzDecoder, write::XzEncoder};
use zip::unstable::write::FileOptionsExt;
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::models::{ArchiveFormat, ArchivePreviewEntry, ArchivePreviewResult, ConflictPolicy};

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

    Ok(path)
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
