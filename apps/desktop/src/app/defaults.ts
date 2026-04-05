import type {
  ActionFeedback,
  AppOverview,
  ArchiveCapabilities,
  ArchiveHistoryEntry,
  ArchiveJobEvent,
  CompressFormat,
  ConflictPolicy,
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
  activeFormats: ['zip', 'tar', 'tar.gz', 'tgz', 'tar.bz2', 'tbz2', 'tar.xz', 'txz', 'xz', 'bz2', 'gz', '7z', 'rar'],
  plannedFormats: [],
}

export const emptyFeedback: ActionFeedback = {
  status: 'idle',
  message: '',
}

export const fallbackCapabilities: ArchiveCapabilities = {
  nativeArchiveOnly: true,
  unsupportedFormats: [],
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
  'tar.bz2',
  'tar.xz',
  'xz',
  'bz2',
  'gz',
  '7z',
]

export const conflictPolicyOptions: Array<{
  value: ConflictPolicy
  label: string
  description: string
}> = [
  {
    value: 'keepBoth',
    label: 'Keep both',
    description: 'Create a new archive or folder name when the destination already exists.',
  },
  {
    value: 'overwrite',
    label: 'Overwrite',
    description: 'Replace the existing destination before the archive job runs.',
  },
  {
    value: 'stop',
    label: 'Stop on conflict',
    description: 'Abort the job if the destination already exists.',
  },
]
