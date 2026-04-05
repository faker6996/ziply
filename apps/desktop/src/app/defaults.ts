import type {
  ActionFeedback,
  AppOverview,
  ArchiveCapabilities,
  ArchiveHistoryEntry,
  ArchiveJobEvent,
  CompressFormat,
  ShellIntent,
  ShellIntegrationStatus,
} from './types'

export const fallbackOverview: AppOverview = {
  name: 'Ziply',
  tagline: 'Compress and extract archives from one desktop workspace.',
  supportedPlatforms: ['macOS', 'Windows', 'Linux'],
  focusAreas: [
    'Create archives from files and folders',
    'Extract common archive formats',
    'Keep one workflow across three desktop operating systems',
  ],
  activeFormats: ['zip', 'tar', 'tar.gz', 'tgz', 'tar.xz', 'txz', 'gz', '7z'],
  plannedFormats: ['rar'],
}

export const emptyFeedback: ActionFeedback = {
  status: 'idle',
  message: '',
}

export const fallbackCapabilities: ArchiveCapabilities = {
  rarExtractionAvailable: false,
  rarExtractorLabel: null,
}

export const fallbackHistory: ArchiveHistoryEntry[] = []
export const fallbackLiveJobs: ArchiveJobEvent[] = []
export const fallbackShellIntents: ShellIntent[] = []

export const fallbackShellIntegration: ShellIntegrationStatus = {
  platform: 'unknown',
  supported: false,
  canInstall: false,
  installed: false,
  mode: 'none',
  note: 'Shell integration is only available inside the desktop app.',
}

export const compressFormatOptions: CompressFormat[] = [
  'zip',
  'tar',
  'tar.gz',
  'tar.xz',
  'gz',
  '7z',
]
