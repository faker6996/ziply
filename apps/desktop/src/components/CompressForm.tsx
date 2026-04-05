import type { FormEvent } from 'react'
import { compressFormatOptions, conflictPolicyOptions } from '../app/defaults'
import type { ActionFeedback, CompressFormat, ConflictPolicy } from '../app/types'
import { suggestArchiveName } from '../app/utils'
import { ActionBanner } from './ActionBanner'

interface CompressFormProps {
  desktopShell: boolean
  compressSources: string
  compressDestination: string
  compressFormat: CompressFormat
  compressConflictPolicy: ConflictPolicy
  normalizedCompressSources: string[]
  gzipSourceCount: number
  feedback: ActionFeedback
  onSubmit: (event: FormEvent<HTMLFormElement>) => void | Promise<void>
  onCompressSourcesChange: (value: string) => void
  onCompressDestinationChange: (value: string) => void
  onCompressFormatChange: (value: CompressFormat) => void
  onCompressConflictPolicyChange: (value: ConflictPolicy) => void
  onPickCompressFiles: () => void
  onPickCompressFolders: () => void
  onPickCompressDestination: () => void
}

export function CompressForm({
  desktopShell,
  compressSources,
  compressDestination,
  compressFormat,
  compressConflictPolicy,
  normalizedCompressSources,
  gzipSourceCount,
  feedback,
  onSubmit,
  onCompressSourcesChange,
  onCompressDestinationChange,
  onCompressFormatChange,
  onCompressConflictPolicyChange,
  onPickCompressFiles,
  onPickCompressFolders,
  onPickCompressDestination,
}: CompressFormProps) {
  return (
    <form className="feature-card feature-card--compress tool-card" onSubmit={onSubmit}>
      <div className="tool-card__header">
        <div>
          <p className="card-label">Compress</p>
          <h2>Build an archive from files and folders.</h2>
        </div>
        <div className="button-row">
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickCompressFiles}
            type="button"
          >
            Add files
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickCompressFolders}
            type="button"
          >
            Add folder
          </button>
        </div>
      </div>

      <label className="field">
        <span>Sources</span>
        <textarea
          className="text-area"
          onChange={(event) => {
            onCompressSourcesChange(event.target.value)
          }}
          placeholder="/path/to/file.txt&#10;/path/to/folder"
          rows={6}
          value={compressSources}
        />
        <small>Enter one path per line. Mix files and folders as needed.</small>
      </label>

      <div className="field-grid">
        <label className="field">
          <span>Format</span>
          <select
            className="text-input"
            onChange={(event) => {
              onCompressFormatChange(event.target.value as CompressFormat)
            }}
            value={compressFormat}
          >
            {compressFormatOptions.map((format) => (
              <option key={format} value={format}>
                {format}
              </option>
            ))}
          </select>
        </label>

        <label className="field">
          <span>Output archive</span>
          <div className="input-action">
            <input
              className="text-input"
              onChange={(event) => {
                onCompressDestinationChange(event.target.value)
              }}
              placeholder={`/Users/you/Desktop/${suggestArchiveName(compressFormat, normalizedCompressSources)}`}
              type="text"
              value={compressDestination}
            />
            <button
              className="ghost-button"
              disabled={!desktopShell}
              onClick={onPickCompressDestination}
              type="button"
            >
              Browse
            </button>
          </div>
        </label>
      </div>

      <label className="field">
        <span>Conflict handling</span>
        <select
          className="text-input"
          onChange={(event) => {
            onCompressConflictPolicyChange(event.target.value as ConflictPolicy)
          }}
          value={compressConflictPolicy}
        >
          {conflictPolicyOptions.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <small>
          {
            conflictPolicyOptions.find((option) => option.value === compressConflictPolicy)
              ?.description
          }
        </small>
      </label>

      {compressFormat === 'gz' ? (
        <p className={`inline-note ${gzipSourceCount === 1 ? '' : 'inline-note--warning'}`}>
          `gz` currently works with exactly one file and does not accept directories.
        </p>
      ) : null}

      <div className="tool-card__footer">
        <button className="primary-button" disabled={feedback.status === 'running'} type="submit">
          {feedback.status === 'running' ? 'Compressing...' : 'Create archive'}
        </button>
        <ActionBanner feedback={feedback} />
      </div>
    </form>
  )
}
