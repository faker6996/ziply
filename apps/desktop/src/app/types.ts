export interface AppOverview {
  name: string
  tagline: string
  supportedPlatforms: string[]
  focusAreas: string[]
  activeFormats: string[]
  plannedFormats: string[]
}

export interface ArchiveCapabilities {
  nativeArchiveOnly: boolean
  unsupportedFormats: string[]
}

export interface ArchiveActionResult {
  operation: 'compress' | 'extract'
  format: string
  outputPath: string
  message: string
}

export interface ArchiveHistoryEntry {
  id: string
  operation: string
  format: string
  sourceSummary: string
  outputPath: string
  timestampMs: number
}

export interface ArchiveJobEvent {
  jobId: string
  operation: 'compress' | 'extract'
  format: string
  stage: 'queued' | 'preparing' | 'processing' | 'finalizing' | 'completed' | 'failed'
  status: 'queued' | 'running' | 'success' | 'error'
  message: string
  progress: number
  sourceSummary: string
  outputPath?: string | null
  timestampMs: number
}

export interface ShellIntent {
  action: 'extract' | 'extract-here' | 'compress'
  paths: string[]
  autoRun: boolean
  destinationPath?: string | null
}

export interface ShellIntegrationStatus {
  platform: string
  supported: boolean
  canInstall: boolean
  installed: boolean
  mode: string
  note: string
}

export type CompressFormat =
  | 'zip'
  | 'tar'
  | 'tar.gz'
  | 'tar.bz2'
  | 'tar.xz'
  | 'xz'
  | 'bz2'
  | 'gz'
  | '7z'
export type ConflictPolicy = 'keepBoth' | 'overwrite' | 'stop'
export type ActionStatus = 'idle' | 'running' | 'success' | 'error'
export type QueueJobStatus = 'queued' | 'running' | 'success' | 'error'

export interface ActionFeedback {
  status: ActionStatus
  message: string
  outputPath?: string
  recoveryHint?: string
}

export interface DragDropState {
  active: boolean
  intent: 'idle' | 'compress' | 'extract'
  message: string
}

export interface ArchivePreviewEntry {
  path: string
  kind: 'file' | 'directory'
  size?: number | null
}

export interface ArchivePreviewResult {
  format: string
  totalEntries: number
  visibleEntries: ArchivePreviewEntry[]
  hiddenEntryCount: number
  note?: string | null
}

export interface CompressArchiveRequest {
  sourcePaths: string[]
  destinationPath: string
  format: CompressFormat
  conflictPolicy: ConflictPolicy
  password?: string
}

export interface ExtractArchiveRequest {
  archivePath: string
  destinationDirectory: string
  conflictPolicy: ConflictPolicy
  password?: string
  selectedEntries?: string[]
}

export interface ArchivePreviewRequest {
  archivePath: string
  password?: string
  limit?: number
}

interface BaseQueueItem {
  id: string
  status: QueueJobStatus
  operation: 'compress' | 'extract'
  format: string
  sourceSummary: string
  outputPath: string
  message: string
  recoveryHint?: string
  passwordProtected: boolean
  createdAt: number
  startedAt?: number
  finishedAt?: number
}

export interface CompressQueueItem extends BaseQueueItem {
  operation: 'compress'
  request: CompressArchiveRequest
}

export interface ExtractQueueItem extends BaseQueueItem {
  operation: 'extract'
  request: ExtractArchiveRequest
}

export type ArchiveQueueItem = CompressQueueItem | ExtractQueueItem
