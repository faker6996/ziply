import type { FormEvent } from 'react'
import { conflictPolicyOptions } from '../app/defaults'
import type {
  ActionFeedback,
  ArchiveCapabilities,
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
  capabilities: ArchiveCapabilities
  feedback: ActionFeedback
  preview: ArchivePreviewResult | null
  previewStatus: 'idle' | 'loading' | 'ready' | 'error'
  previewError: string
  supportsPasswordOnExtract: (path: string) => boolean
  onSubmit: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onQueue: () => void
  onExtractSourceChange: (value: string) => void
  onExtractDestinationChange: (value: string) => void
  onExtractConflictPolicyChange: (value: ConflictPolicy) => void
  onExtractPasswordChange: (value: string) => void
  onPickExtractSource: () => void
  onPickExtractDestination: () => void
}

export function ExtractForm({
  desktopShell,
  extractSource,
  extractDestination,
  extractConflictPolicy,
  extractPassword,
  capabilities,
  feedback,
  preview,
  previewStatus,
  previewError,
  supportsPasswordOnExtract,
  onSubmit,
  onQueue,
  onExtractSourceChange,
  onExtractDestinationChange,
  onExtractConflictPolicyChange,
  onExtractPasswordChange,
  onPickExtractSource,
  onPickExtractDestination,
}: ExtractFormProps) {
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
        <small>Supported now: zip, tar, tar.gz, tgz, tar.xz, txz, gz, and 7z.</small>
        <small>
          {capabilities.rarExtractionAvailable
            ? `rar extraction is available through ${capabilities.rarExtractorLabel}.`
            : 'rar extraction needs an installed backend such as unar, 7zz, 7z, or unrar.'}
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

        {previewStatus === 'error' ? (
          <p className="inline-note inline-note--warning">{previewError}</p>
        ) : null}

        {preview ? (
          <>
            {preview.note ? <p className="archive-preview__note">{preview.note}</p> : null}
            <div className="archive-preview__list">
              {preview.visibleEntries.map((entry) => (
                <div className="archive-preview__item" key={`${entry.kind}-${entry.path}`}>
                  <strong>{entry.path}</strong>
                  <span>{archivePreviewSummary(entry)}</span>
                </div>
              ))}
            </div>
            {preview.hiddenEntryCount > 0 ? (
              <p className="archive-preview__meta">
                + {preview.hiddenEntryCount} more entries hidden from this preview.
              </p>
            ) : null}
          </>
        ) : null}
      </div>

      <div className="tool-card__footer">
        <div className="button-row">
          <button
            className="primary-button primary-button--cool"
            disabled={feedback.status === 'running'}
            type="submit"
          >
            {feedback.status === 'running' ? 'Extracting...' : 'Extract archive'}
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onQueue}
            type="button"
          >
            Add to queue
          </button>
        </div>
        <ActionBanner feedback={feedback} />
      </div>
    </form>
  )
}
