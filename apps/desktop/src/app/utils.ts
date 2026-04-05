import { invoke } from '@tauri-apps/api/core'
import type {
  AppOverview,
  ArchiveCapabilities,
  ArchiveHistoryEntry,
  ArchiveJobEvent,
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

export function liveJobStatusChipClass(status: ArchiveJobEvent['status']) {
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

export function liveJobProgressClass(status: ArchiveJobEvent['status']) {
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
