import { useState, type FormEvent } from 'react'
import type {
  ActionFeedback,
  ArchiveJobEvent,
  ArchiveQueueItem,
  ArchivePreviewResult,
  CompressFormat,
  ConflictPolicy,
} from '../app/types'
import { BatchQueuePanel } from './BatchQueuePanel'
import { CompressForm } from './CompressForm'
import { DropZonePanel } from './DropZonePanel'
import { ExtractForm } from './ExtractForm'
import { LiveJobsPanel } from './LiveJobsPanel'
import { CompressIcon, ExtractIcon } from './AppIcons'

interface WorkspaceScreenProps {
  desktopShell: boolean
  dragDropState: {
    active: boolean
    intent: 'idle' | 'compress' | 'extract'
    message: string
  }
  compressSources: string
  compressDestination: string
  compressFormat: CompressFormat
  compressConflictPolicy: ConflictPolicy
  compressPassword: string
  compressFeedback: ActionFeedback
  normalizedCompressSources: string[]
  gzipSourceCount: number
  extractSource: string
  extractDestination: string
  extractConflictPolicy: ConflictPolicy
  extractPassword: string
  extractFeedback: ActionFeedback
  extractPreview: ArchivePreviewResult | null
  extractPreviewStatus: 'idle' | 'loading' | 'ready' | 'error'
  extractPreviewError: string
  extractPreviewLimit: number
  extractSelectedEntries: string[]
  liveJobs: ArchiveJobEvent[]
  queueItems: ArchiveQueueItem[]
  onClearFinishedQueue: () => void
  onRemoveQueuedJob: (jobId: string) => void
  onRetryQueueJob: (jobId: string) => void
  onCompressSourcesChange: (value: string) => void
  onCompressDestinationChange: (value: string) => void
  onCompressFormatChange: (value: CompressFormat) => void
  onCompressConflictPolicyChange: (value: ConflictPolicy) => void
  onCompressPasswordChange: (value: string) => void
  onPickCompressFiles: () => void
  onPickCompressFolders: () => void
  onPickCompressDestination: () => void
  onRunCompress: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onQueueCompress: () => void
  onExtractSourceChange: (value: string) => void
  onExtractDestinationChange: (value: string) => void
  onExtractConflictPolicyChange: (value: ConflictPolicy) => void
  onExtractPasswordChange: (value: string) => void
  onPickExtractSource: () => void
  onPickExtractDestination: () => void
  onRunExtract: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onRunExtractAll: (event?: FormEvent<HTMLFormElement>) => void | Promise<void>
  onRunExtractSelected: () => void | Promise<void>
  onQueueExtract: () => void
  onQueueAllExtract: () => void
  onToggleEntry: (path: string) => void
  onSelectAllVisibleEntries: (paths?: string[]) => void
  onClearSelection: () => void
  onLoadMoreEntries: () => void
  supportsPasswordOnCompress: (format: CompressFormat) => boolean
  supportsPasswordOnExtract: (path: string) => boolean
  supportsSelectiveExtract: (path: string) => boolean
}

export function WorkspaceScreen({
  desktopShell,
  dragDropState,
  compressSources,
  compressDestination,
  compressFormat,
  compressConflictPolicy,
  compressPassword,
  compressFeedback,
  normalizedCompressSources,
  gzipSourceCount,
  extractSource,
  extractDestination,
  extractConflictPolicy,
  extractPassword,
  extractFeedback,
  extractPreview,
  extractPreviewStatus,
  extractPreviewError,
  extractPreviewLimit,
  extractSelectedEntries,
  liveJobs,
  queueItems,
  onClearFinishedQueue,
  onRemoveQueuedJob,
  onRetryQueueJob,
  onCompressSourcesChange,
  onCompressDestinationChange,
  onCompressFormatChange,
  onCompressConflictPolicyChange,
  onCompressPasswordChange,
  onPickCompressFiles,
  onPickCompressFolders,
  onPickCompressDestination,
  onRunCompress,
  onQueueCompress,
  onExtractSourceChange,
  onExtractDestinationChange,
  onExtractConflictPolicyChange,
  onExtractPasswordChange,
  onPickExtractSource,
  onPickExtractDestination,
  onRunExtract,
  onRunExtractAll,
  onRunExtractSelected,
  onQueueExtract,
  onQueueAllExtract,
  onToggleEntry,
  onSelectAllVisibleEntries,
  onClearSelection,
  onLoadMoreEntries,
  supportsPasswordOnCompress,
  supportsPasswordOnExtract,
  supportsSelectiveExtract,
}: WorkspaceScreenProps) {
  const [manualTool, setManualTool] = useState<'compress' | 'extract' | null>(null)
  const inferredTool =
    dragDropState.active
      ? dragDropState.intent === 'extract'
        ? 'extract'
        : 'compress'
      : extractSource.trim()
        ? 'extract'
        : compressSources.trim()
          ? 'compress'
          : 'compress'
  const activeTool = manualTool ?? inferredTool

  return (
    <section className="page-section page-section--workspace">
      <header className="page-header page-header--workspace">
        <div>
          <h1>Workspace</h1>
          <p>Drop files to compress, or drop an archive to preview and extract it.</p>
        </div>
      </header>

      <div className="workspace-dropzone">
        <DropZonePanel
          desktopShell={desktopShell}
          dragDropState={dragDropState}
          onPickArchive={onPickExtractSource}
          onPickFiles={onPickCompressFiles}
        />
      </div>

      <div className="workspace-tool-switcher">
        <button
          className={`workspace-tool-switcher__button ${activeTool === 'compress' ? 'workspace-tool-switcher__button--active' : ''}`}
          onClick={() => {
            setManualTool('compress')
          }}
          type="button"
        >
          <CompressIcon />
          <span>Compress</span>
        </button>
        <button
          className={`workspace-tool-switcher__button ${activeTool === 'extract' ? 'workspace-tool-switcher__button--active' : ''}`}
          onClick={() => {
            setManualTool('extract')
          }}
          type="button"
        >
          <ExtractIcon />
          <span>Extract</span>
        </button>
      </div>

      <div className="workspace-tool-panel">
        {activeTool === 'compress' ? (
          <CompressForm
            compressDestination={compressDestination}
            compressFormat={compressFormat}
            compressConflictPolicy={compressConflictPolicy}
            compressPassword={compressPassword}
            compressSources={compressSources}
            desktopShell={desktopShell}
            feedback={compressFeedback}
            gzipSourceCount={gzipSourceCount}
            normalizedCompressSources={normalizedCompressSources}
            onCompressDestinationChange={onCompressDestinationChange}
            onCompressFormatChange={onCompressFormatChange}
            onCompressConflictPolicyChange={onCompressConflictPolicyChange}
            onCompressPasswordChange={onCompressPasswordChange}
            onCompressSourcesChange={onCompressSourcesChange}
            onPickCompressDestination={onPickCompressDestination}
            onPickCompressFiles={onPickCompressFiles}
            onPickCompressFolders={onPickCompressFolders}
            onQueue={onQueueCompress}
            onSubmit={onRunCompress}
            supportsPasswordOnCompress={supportsPasswordOnCompress}
          />
        ) : null}

        {activeTool === 'extract' ? (
          <ExtractForm
            desktopShell={desktopShell}
            extractDestination={extractDestination}
            extractConflictPolicy={extractConflictPolicy}
            extractPassword={extractPassword}
            extractSource={extractSource}
            feedback={extractFeedback}
            onClearSelection={onClearSelection}
            onExtractConflictPolicyChange={onExtractConflictPolicyChange}
            onExtractDestinationChange={onExtractDestinationChange}
            onExtractPasswordChange={onExtractPasswordChange}
            onExtractSourceChange={onExtractSourceChange}
            onLoadMoreEntries={onLoadMoreEntries}
            onPickExtractDestination={onPickExtractDestination}
            onPickExtractSource={onPickExtractSource}
            onQueue={onQueueExtract}
            onQueueAll={onQueueAllExtract}
            onSelectAllVisibleEntries={onSelectAllVisibleEntries}
            onSubmit={onRunExtract}
            onSubmitAll={onRunExtractAll}
            onSubmitSelected={onRunExtractSelected}
            onToggleEntry={onToggleEntry}
            preview={extractPreview}
            previewError={extractPreviewError}
            previewLimit={extractPreviewLimit}
            previewStatus={extractPreviewStatus}
            selectedEntries={extractSelectedEntries}
            supportsPasswordOnExtract={supportsPasswordOnExtract}
            supportsSelectiveExtract={supportsSelectiveExtract}
          />
        ) : null}
      </div>

      <div className="workspace-detail-grid">
        <LiveJobsPanel liveJobs={liveJobs} />
        <BatchQueuePanel
          onClearFinished={onClearFinishedQueue}
          onRemoveQueuedJob={onRemoveQueuedJob}
          onRetryJob={onRetryQueueJob}
          queueItems={queueItems}
        />
      </div>
    </section>
  )
}
