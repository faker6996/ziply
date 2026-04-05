import { useDeferredValue, useState, type FormEvent } from 'react'
import { conflictPolicyOptions } from '../app/defaults'
import type {
  ActionFeedback,
  ArchivePreviewResult,
  ConflictPolicy,
} from '../app/types'
import { archivePreviewSummary } from '../app/utils'
import { ActionBanner } from './ActionBanner'

interface ExtractFormProps {
  desktopShell: boolean
  extractSource: string
  extractDestination: string
  extractConflictPolicy: ConflictPolicy
  extractPassword: string
  selectedEntries: string[]
  feedback: ActionFeedback
  preview: ArchivePreviewResult | null
  previewLimit: number
  previewStatus: 'idle' | 'loading' | 'ready' | 'error'
  previewError: string
  supportsPasswordOnExtract: (path: string) => boolean
  supportsSelectiveExtract: (path: string) => boolean
  onSubmit: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onSubmitAll: (event?: FormEvent<HTMLFormElement>) => void | Promise<void>
  onSubmitSelected: () => void | Promise<void>
  onQueue: () => void
  onQueueAll: () => void
  onExtractSourceChange: (value: string) => void
  onExtractDestinationChange: (value: string) => void
  onExtractConflictPolicyChange: (value: ConflictPolicy) => void
  onExtractPasswordChange: (value: string) => void
  onToggleEntry: (path: string) => void
  onSelectAllVisibleEntries: (paths?: string[]) => void
  onClearSelection: () => void
  onLoadMoreEntries: () => void
  onPickExtractSource: () => void
  onPickExtractDestination: () => void
}

export function ExtractForm({
  desktopShell,
  extractSource,
  extractDestination,
  extractConflictPolicy,
  extractPassword,
  selectedEntries,
  feedback,
  preview,
  previewLimit,
  previewStatus,
  previewError,
  supportsPasswordOnExtract,
  supportsSelectiveExtract,
  onSubmit,
  onSubmitAll,
  onSubmitSelected,
  onQueue,
  onQueueAll,
  onExtractSourceChange,
  onExtractDestinationChange,
  onExtractConflictPolicyChange,
  onExtractPasswordChange,
  onToggleEntry,
  onSelectAllVisibleEntries,
  onClearSelection,
  onLoadMoreEntries,
  onPickExtractSource,
  onPickExtractDestination,
}: ExtractFormProps) {
  const [previewQuery, setPreviewQuery] = useState('')
  const deferredPreviewQuery = useDeferredValue(previewQuery.trim().toLowerCase())
  const canSelectEntries = preview != null && supportsSelectiveExtract(extractSource)
  const hasSelection = selectedEntries.length > 0
  const filteredEntries =
    !preview || deferredPreviewQuery.length === 0
      ? preview?.visibleEntries ?? []
      : preview.visibleEntries.filter((entry) => entry.path.toLowerCase().includes(deferredPreviewQuery))

  return (
    <form className="feature-card feature-card--extract tool-card" onSubmit={onSubmit}>
      <div className="tool-card__header">
        <div>
          <p className="card-label">Extract</p>
          <h2>Unpack an archive into a destination folder.</h2>
        </div>
      </div>

      <label className="field">
        <span>Archive file</span>
        <div className="input-action">
          <input
            className="text-input"
            onChange={(event) => {
              onExtractSourceChange(event.target.value)
            }}
            placeholder="/path/to/archive.zip"
            type="text"
            value={extractSource}
          />
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickExtractSource}
            type="button"
          >
            Browse
          </button>
        </div>
      </label>

      <label className="field">
        <span>Destination folder</span>
        <div className="input-action">
          <input
            className="text-input"
            onChange={(event) => {
              onExtractDestinationChange(event.target.value)
            }}
            placeholder="/path/to/output-folder"
            type="text"
            value={extractDestination}
          />
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickExtractDestination}
            type="button"
          >
            Browse
          </button>
        </div>
        <small>Supported now: zip, tar, tar.gz, tgz, tar.bz2, tbz2, tar.xz, txz, xz, bz2, gz, and 7z.</small>
        <small>
          RAR is not supported yet. Ziply only ships native archive handlers.
        </small>
      </label>

      <label className="field">
        <span>Conflict handling</span>
        <select
          className="text-input"
          onChange={(event) => {
            onExtractConflictPolicyChange(event.target.value as ConflictPolicy)
          }}
          value={extractConflictPolicy}
        >
          {conflictPolicyOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <small>
          {
            conflictPolicyOptions.find((option) => option.value === extractConflictPolicy)
              ?.description
          }
        </small>
      </label>

      <label className="field">
        <span>Archive password</span>
        <input
          className="text-input"
          onChange={(event) => {
            onExtractPasswordChange(event.target.value)
          }}
          placeholder="Optional password"
          type="password"
          value={extractPassword}
        />
        <small>
          {supportsPasswordOnExtract(extractSource)
            ? 'Use this for password-protected zip and 7z archives. The same password is used for preview and extraction.'
            : 'Password-based extraction is currently supported for zip and 7z archives only.'}
        </small>
      </label>

      <div className="archive-preview">
        <div className="archive-preview__header">
          <div>
            <span>Preview contents</span>
            <small>
              {previewStatus === 'loading'
                ? 'Inspecting archive contents...'
                : preview
                  ? `${preview.totalEntries} entries detected`
                  : 'Choose an archive to inspect its contents before extraction.'}
            </small>
          </div>
          {preview ? (
            <span className="chip chip--soft">
              {preview.format}
            </span>
          ) : null}
        </div>

        {canSelectEntries ? (
          <div className="archive-preview__actions">
            <span className="chip chip--warm">{selectedEntries.length} selected</span>
            <span className="chip chip--soft">{filteredEntries.length} visible</span>
            <button
              className="ghost-button"
              onClick={() => {
                onSelectAllVisibleEntries(filteredEntries.map((entry) => entry.path))
              }}
              type="button"
            >
              Select all visible
            </button>
            <button
              className="ghost-button"
              disabled={selectedEntries.length === 0}
              onClick={onClearSelection}
              type="button"
            >
              Clear selection
            </button>
          </div>
        ) : null}

        {previewStatus === 'error' ? (
          <p className="inline-note inline-note--warning">{previewError}</p>
        ) : null}

        {preview ? (
          <>
            {preview.note ? <p className="archive-preview__note">{preview.note}</p> : null}
            <label className="field">
              <span>Find entry</span>
              <input
                className="text-input"
                onChange={(event) => {
                  setPreviewQuery(event.target.value)
                }}
                placeholder="Filter by path or file name"
                type="text"
                value={previewQuery}
              />
              <small>
                {deferredPreviewQuery
                  ? `${filteredEntries.length} of ${preview.visibleEntries.length} loaded entries match this filter.`
                  : `Search filters the ${preview.visibleEntries.length} entries currently loaded in this preview panel.`}
              </small>
            </label>
            {!supportsSelectiveExtract(extractSource) && preview.format ? (
              <p className="archive-preview__note">
                Selective extract is currently available for zip, tar, tar.gz, tar.bz2, tar.xz, and 7z.
              </p>
            ) : null}
            <div className="archive-preview__list">
              {filteredEntries.map((entry) => (
                <label className="archive-preview__item" key={`${entry.kind}-${entry.path}`}>
                  {canSelectEntries ? (
                    <input
                      checked={selectedEntries.includes(entry.path)}
                      className="archive-preview__checkbox"
                      onChange={() => {
                        onToggleEntry(entry.path)
                      }}
                      type="checkbox"
                    />
                  ) : null}
                  <div className="archive-preview__copy">
                    <strong>{entry.path}</strong>
                    <span>{archivePreviewSummary(entry)}</span>
                  </div>
                </label>
              ))}
            </div>
            {filteredEntries.length === 0 ? (
              <p className="archive-preview__meta">
                No loaded entries match the current search filter.
              </p>
            ) : null}
            {preview.hiddenEntryCount > 0 ? (
              <div className="archive-preview__footer">
                <p className="archive-preview__meta">
                  + {preview.hiddenEntryCount} more entries hidden from this preview. Selective
                  extract only applies to the entries currently loaded here.
                </p>
                <button
                  className="ghost-button"
                  disabled={previewStatus === 'loading'}
                  onClick={onLoadMoreEntries}
                  type="button"
                >
                  {previewStatus === 'loading'
                    ? 'Loading more...'
                    : `Load ${Math.min(160, preview.totalEntries - previewLimit)} more`}
                </button>
              </div>
            ) : null}
          </>
        ) : null}
      </div>

      <div className="tool-card__footer">
        <div className="button-row">
          <button
            className="primary-button primary-button--cool"
            disabled={feedback.status === 'running'}
            onClick={() => {
              void onSubmitSelected()
            }}
            type="button"
          >
            {feedback.status === 'running'
              ? 'Extracting...'
              : hasSelection
                ? `Extract selected (${selectedEntries.length})`
                : 'Extract archive'}
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell || feedback.status === 'running' || !hasSelection}
            onClick={() => {
              void onSubmitAll()
            }}
            type="button"
          >
            Extract all
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onQueue}
            type="button"
          >
            {hasSelection ? 'Queue selected' : 'Add to queue'}
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell || !hasSelection}
            onClick={onQueueAll}
            type="button"
          >
            Queue all
          </button>
        </div>
        <ActionBanner feedback={feedback} />
      </div>
    </form>
  )
}
