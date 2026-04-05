use std::{
    env,
    ffi::OsString,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Seek, Write},
    path::{Component, Path, PathBuf},
    process::Command,
};

use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use sevenz_rust2::{decompress_file, ArchiveWriter};
use tar::Builder as TarBuilder;
use walkdir::WalkDir;
use xz2::{read::XzDecoder, write::XzEncoder};
use zip::{write::FileOptions, CompressionMethod, ZipArchive, ZipWriter};

use crate::models::{ArchiveFormat, ConflictPolicy};

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
) -> Result<(), String> {
    let file = File::create(destination_path).map_err(|error| {
        format!(
            "failed to create archive {}: {error}",
            destination_path.display()
        )
    })?;
    let writer = BufWriter::new(file);
    let mut archive = ZipWriter::new(writer);

    for source_path in source_paths {
        append_zip_source(&mut archive, source_path)?;
    }

    archive
        .finish()
        .map_err(|error| format!("failed to finalize zip archive: {error}"))?;
    Ok(())
}

fn append_zip_source<W: Write + Seek>(
    archive: &mut ZipWriter<W>,
    source_path: &Path,
) -> Result<(), String> {
    let options = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    if source_path.is_file() {
        let archive_name = source_path
            .file_name()
            .ok_or_else(|| format!("source path has no file name: {}", source_path.display()))?;
        return append_zip_file(archive, source_path, Path::new(archive_name), options);
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

        append_zip_file(archive, path, relative_path, options)?;
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

pub(crate) fn create_7z_archive(
    source_paths: &[PathBuf],
    destination_path: &Path,
) -> Result<(), String> {
    let mut writer = ArchiveWriter::create(destination_path)
        .map_err(|error| format!("failed to create 7z archive writer: {error}"))?;

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
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let mut archive =
        ZipArchive::new(file).map_err(|error| format!("failed to read zip archive: {error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("failed to read zip entry: {error}"))?;
        let relative_path = entry
            .enclosed_name()
            .ok_or_else(|| format!("zip entry contains an unsafe path: {}", entry.name()))?;
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
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let reader = BufReader::new(file);
    let mut archive = tar::Archive::new(reader);
    unpack_tar_entries(&mut archive, destination_directory)
}

pub(crate) fn extract_tar_gz_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = GzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, destination_directory)
}

pub(crate) fn extract_tar_xz_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = XzDecoder::new(BufReader::new(file));
    let mut archive = tar::Archive::new(decoder);
    unpack_tar_entries(&mut archive, destination_directory)
}

fn unpack_tar_entries<R: Read>(
    archive: &mut tar::Archive<R>,
    destination_directory: &Path,
) -> Result<(), String> {
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

pub(crate) fn extract_7z_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    decompress_file(archive_path, destination_directory)
        .map_err(|error| format!("failed to extract 7z archive: {error}"))
}

pub(crate) fn extract_rar_archive(
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let extractor = detect_rar_extractor().ok_or_else(|| {
        "rar extraction requires an external tool. Install one of: unar, 7zz, 7z, or unrar."
            .to_string()
    })?;

    run_external_rar_extract(&extractor, archive_path, destination_directory)
}

pub(crate) fn rar_extractor_label() -> Option<String> {
    detect_rar_extractor().map(|extractor| extractor.label().to_string())
}

#[derive(Clone)]
enum RarExtractor {
    Unar(PathBuf),
    SevenZip(PathBuf),
    Unrar(PathBuf),
}

impl RarExtractor {
    fn label(&self) -> &str {
        match self {
            Self::Unar(_) => "unar",
            Self::SevenZip(_) => "7z-compatible backend",
            Self::Unrar(_) => "unrar",
        }
    }
}

fn detect_rar_extractor() -> Option<RarExtractor> {
    find_command(&["unar"])
        .map(RarExtractor::Unar)
        .or_else(|| find_command(&["7zz", "7z"]).map(RarExtractor::SevenZip))
        .or_else(|| find_command(&["unrar"]).map(RarExtractor::Unrar))
}

fn run_external_rar_extract(
    extractor: &RarExtractor,
    archive_path: &Path,
    destination_directory: &Path,
) -> Result<(), String> {
    let mut command = match extractor {
        RarExtractor::Unar(binary) => {
            let mut command = Command::new(binary);
            command.arg("-force-overwrite");
            command.arg("-output-directory");
            command.arg(destination_directory);
            command.arg(archive_path);
            command
        }
        RarExtractor::SevenZip(binary) => {
            let mut command = Command::new(binary);
            command.arg("x");
            command.arg(archive_path);
            command.arg(format!("-o{}", destination_directory.display()));
            command.arg("-y");
            command
        }
        RarExtractor::Unrar(binary) => {
            let mut command = Command::new(binary);
            command.arg("x");
            command.arg("-o+");
            command.arg(archive_path);
            command.arg(destination_directory);
            command
        }
    };

    let output = command.output().map_err(|error| {
        format!(
            "failed to start {} for rar extraction: {error}",
            extractor.label()
        )
    })?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(if detail.is_empty() {
        format!(
            "{} failed while extracting the rar archive",
            extractor.label()
        )
    } else {
        format!(
            "{} failed while extracting the rar archive: {detail}",
            extractor.label()
        )
    })
}

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

#[cfg(windows)]
fn command_extensions() -> Vec<String> {
    env::var("PATHEXT")
        .ok()
        .map(|value| {
            value
                .split(';')
                .filter(|entry| !entry.is_empty())
                .map(|entry| entry.to_ascii_lowercase())
                .collect()
        })
        .filter(|extensions: &Vec<String>| !extensions.is_empty())
        .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string()])
}

#[cfg(not(windows))]
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

    for suffix in [".tar.gz", ".tar.xz"] {
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
