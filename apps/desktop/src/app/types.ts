export interface AppOverview {
  name: string
  tagline: string
  supportedPlatforms: string[]
  focusAreas: string[]
  activeFormats: string[]
  plannedFormats: string[]
}

export interface ArchiveCapabilities {
  rarExtractionAvailable: boolean
  rarExtractorLabel: string | null
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

export type CompressFormat = 'zip' | 'tar' | 'tar.gz' | 'tar.xz' | 'gz' | '7z'
export type ActionStatus = 'idle' | 'running' | 'success' | 'error'

export interface ActionFeedback {
  status: ActionStatus
  message: string
  outputPath?: string
}
