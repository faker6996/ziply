import { invoke } from '@tauri-apps/api/core'
import type {
  AppOverview,
  ArchiveCapabilities,
  ArchiveHistoryEntry,
  ArchiveJobEvent,
  ArchivePreviewEntry,
  ArchiveQueueItem,
  CompressFormat,
  ShellIntent,
  ShellIntegrationStatus,
} from './types'
import {
  fallbackCapabilities,
  fallbackHistory,
  fallbackOverview,
  fallbackShellIntegration,
  fallbackShellIntents,
} from './defaults'

export function isDesktopShell() {
  return typeof window !== 'undefined' && window.__TAURI_INTERNALS__ != null
}

export async function loadBootstrapData() {
  if (!isDesktopShell()) {
    return {
      overview: fallbackOverview,
      capabilities: fallbackCapabilities,
      history: fallbackHistory,
      shellIntegration: fallbackShellIntegration,
      shellIntents: fallbackShellIntents,
    }
  }

  const [overview, capabilities, history, shellIntegration, shellIntents] = await Promise.all([
    invoke<AppOverview>('app_overview'),
    invoke<ArchiveCapabilities>('archive_capabilities'),
    invoke<ArchiveHistoryEntry[]>('get_archive_history'),
    invoke<ShellIntegrationStatus>('shell_integration_status'),
    invoke<ShellIntent[]>('consume_shell_intents'),
  ])

  return { overview, capabilities, history, shellIntegration, shellIntents }
}

export function formatHistoryTimestamp(timestampMs: number) {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(timestampMs))
}

export function upsertLiveJob(currentJobs: ArchiveJobEvent[], nextJob: ArchiveJobEvent) {
  return [nextJob, ...currentJobs.filter((job) => job.jobId !== nextJob.jobId)]
    .sort((left, right) => right.timestampMs - left.timestampMs)
    .slice(0, 8)
}

export function formatLiveJobStatus(status: ArchiveJobEvent['status']) {
  if (status === 'queued') {
    return 'Queued'
  }

  if (status === 'running') {
    return 'Running'
  }

  if (status === 'success') {
    return 'Done'
  }

  return 'Failed'
}

export function liveJobStatusChipClass(status: ArchiveJobEvent['status'] | ArchiveQueueItem['status']) {
  if (status === 'success') {
    return 'chip chip--success'
  }

  if (status === 'error') {
    return 'chip chip--danger'
  }

  if (status === 'running') {
    return 'chip chip--warm'
  }

  return 'chip chip--soft'
}

export function liveJobProgressClass(status: ArchiveJobEvent['status'] | ArchiveQueueItem['status']) {
  if (status === 'success') {
    return 'progress-fill progress-fill--success'
  }

  if (status === 'error') {
    return 'progress-fill progress-fill--error'
  }

  return 'progress-fill'
}

export function normalizePaths(value: string) {
  return Array.from(
    new Set(
      value
        .split('\n')
        .map((line) => line.trim())
        .filter(Boolean),
    ),
  )
}

export function pathsToText(paths: string[]) {
  return paths.join('\n')
}

export function toDialogPaths(result: string | string[] | null) {
  if (result == null) {
    return []
  }

  return Array.isArray(result) ? result : [result]
}

export function preferredArchiveExtension(format: CompressFormat) {
  if (format === 'tar.gz') {
    return '.tar.gz'
  }

  if (format === 'tar.xz') {
    return '.tar.xz'
  }

  return `.${format}`
}

export function suggestArchiveName(format: CompressFormat, sourcePaths: string[]) {
  const segments = sourcePaths[0]?.split(/[\\/]/).filter(Boolean) ?? []
  const primarySource = segments[segments.length - 1]
  const baseName =
    sourcePaths.length === 1 && primarySource
      ? primarySource.replace(/\.[^.]+$/, '')
      : 'ziply-archive'

  return `${baseName}${preferredArchiveExtension(format)}`
}

function stripArchiveExtension(path: string) {
  return path.replace(/\.(tar\.gz|tar\.xz|tgz|txz|zip|tar|gz|7z|rar|xz)$/i, '')
}

function parentDirectory(path: string) {
  const normalized = path.trim().replace(/[\\/]+$/, '')
  const index = Math.max(normalized.lastIndexOf('/'), normalized.lastIndexOf('\\'))
  return index >= 0 ? normalized.slice(0, index) : ''
}

export function suggestExtractDestination(archivePath: string, extractHere: boolean) {
  const parent = parentDirectory(archivePath)
  if (extractHere) {
    return parent
  }

  const baseName = stripArchiveExtension(archivePath)
  return baseName === archivePath ? parent : baseName
}

export function isArchivePath(path: string) {
  return /\.(tar\.gz|tar\.xz|tgz|txz|zip|tar|gz|7z|rar)$/i.test(path.trim())
}

export function supportsArchivePasswordOnCompress(format: CompressFormat) {
  return format === '7z'
}

export function supportsArchivePasswordOnExtract(path: string) {
  return /\.(zip|7z)$/i.test(path.trim())
}

function pathSegments(path: string) {
  return path.trim().split(/[\\/]/).filter(Boolean)
}

export function fileNameFromPath(path: string) {
  const segments = pathSegments(path)
  return segments[segments.length - 1] ?? path.trim()
}

export function summarizePathList(paths: string[]) {
  if (paths.length === 0) {
    return 'No sources'
  }

  if (paths.length === 1) {
    return fileNameFromPath(paths[0])
  }

  return `${paths.length} items`
}

export function inferArchiveFormatFromPath(path: string) {
  const normalized = path.trim().toLowerCase()

  if (normalized.endsWith('.tar.gz') || normalized.endsWith('.tgz')) {
    return 'tar.gz'
  }

  if (normalized.endsWith('.tar.xz') || normalized.endsWith('.txz')) {
    return 'tar.xz'
  }

  const match = normalized.match(/\.(zip|tar|gz|7z|rar)$/)
  return match?.[1] ?? 'archive'
}

export function formatEntrySize(size?: number | null) {
  if (size == null) {
    return ''
  }

  if (size < 1024) {
    return `${size} B`
  }

  const units = ['KB', 'MB', 'GB', 'TB']
  let value = size / 1024
  let unitIndex = 0

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }

  return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unitIndex]}`
}

export function archivePreviewSummary(entry: ArchivePreviewEntry) {
  if (entry.kind === 'directory') {
    return 'Folder'
  }

  const sizeLabel = formatEntrySize(entry.size)
  return sizeLabel ? `File • ${sizeLabel}` : 'File'
}

export function recoveryHintForArchiveError(message: string) {
  const normalized = message.trim().toLowerCase()

  if (!normalized) {
    return undefined
  }

  if (
    normalized.includes('invalid password') ||
    normalized.includes('wrong password') ||
    normalized.includes('incorrect password')
  ) {
    return 'Check the archive password, then run the job again. Encrypted archives cannot be previewed or extracted without the correct password.'
  }

  if (normalized.includes('password-protected archive creation is currently supported for 7z only')) {
    return 'Switch the output format to 7z if you need encryption, or remove the password for zip, tar, tar.gz, tar.xz, or gz.'
  }

  if (normalized.includes('password-based extraction is currently supported for zip and 7z archives only')) {
    return 'Remove the password for this archive type, or use a zip or 7z archive if encrypted extraction is required.'
  }

  if (normalized.includes('destination archive already exists')) {
    return 'Choose Keep both or Overwrite, or change the output archive path before running the job again.'
  }

  if (normalized.includes('destination folder already exists and is not empty')) {
    return 'Choose Keep both or Overwrite, or extract into a different folder.'
  }

  if (normalized.includes('archive file was not found') || normalized.includes('source path does not exist')) {
    return 'Re-select the file or folder. The original path may have moved, been renamed, or is no longer mounted.'
  }

  if (normalized.includes('rar extraction requires an external tool')) {
    return 'Install one of these backends on this machine: unar, 7zz, 7z, or unrar, then refresh Ziply shell capabilities.'
  }

  if (normalized.includes('preview is not available for rar yet')) {
    return 'Extract the rar archive directly, or install a rar backend first if the archive cannot be opened yet.'
  }

  if (normalized.includes('unsupported archive extension') || normalized.includes('unsupported archive format')) {
    return 'Use one of the supported formats: zip, tar, tar.gz, tar.xz, gz, 7z, and rar extraction when a backend is installed.'
  }

  if (normalized.includes('rar compression is not supported')) {
    return 'Choose zip, tar, tar.gz, tar.xz, gz, or 7z as the output format. Rar creation is not available in Ziply.'
  }

  if (normalized.includes('gz compression currently supports exactly one source file')) {
    return 'Keep only one source file for gz compression. Use tar.gz if you need to package multiple files or folders together.'
  }

  if (normalized.includes('gz compression only works with a single file, not a directory')) {
    return 'Pick one file instead of a folder, or switch to tar.gz when you need directory compression.'
  }

  return undefined
}
