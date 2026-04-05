import type { FormEvent } from 'react'
import { conflictPolicyOptions } from '../app/defaults'
import type { ActionFeedback, ArchiveCapabilities, ConflictPolicy } from '../app/types'
import { ActionBanner } from './ActionBanner'

interface ExtractFormProps {
  desktopShell: boolean
  extractSource: string
  extractDestination: string
  extractConflictPolicy: ConflictPolicy
  capabilities: ArchiveCapabilities
  feedback: ActionFeedback
  onSubmit: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onExtractSourceChange: (value: string) => void
  onExtractDestinationChange: (value: string) => void
  onExtractConflictPolicyChange: (value: ConflictPolicy) => void
  onPickExtractSource: () => void
  onPickExtractDestination: () => void
}

export function ExtractForm({
  desktopShell,
  extractSource,
  extractDestination,
  extractConflictPolicy,
  capabilities,
  feedback,
  onSubmit,
  onExtractSourceChange,
  onExtractDestinationChange,
  onExtractConflictPolicyChange,
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

      <div className="tool-card__footer">
        <button
          className="primary-button primary-button--cool"
          disabled={feedback.status === 'running'}
          type="submit"
        >
          {feedback.status === 'running' ? 'Extracting...' : 'Extract archive'}
        </button>
        <ActionBanner feedback={feedback} />
      </div>
    </form>
  )
}
