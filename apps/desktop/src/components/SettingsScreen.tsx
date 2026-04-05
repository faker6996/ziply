import { compressFormatOptions, conflictPolicyOptions } from '../app/defaults'
import type { CompressFormat, ConflictPolicy, ShellIntegrationStatus, ActionFeedback } from '../app/types'
import { ActionBanner } from './ActionBanner'
import { CompressIcon, FolderIcon, SettingsIcon, ShieldIcon } from './AppIcons'

export interface AppPreferences {
  extractDestinationMode: 'askEveryTime' | 'archiveFolder' | 'rememberLast'
  extractConflictPolicy: ConflictPolicy
  defaultCompressFormat: CompressFormat
  compressionLevel: 'fastest' | 'balanced' | 'smallest'
  deleteArchiveAfterExtraction: boolean
  lastExtractDestination: string
}

interface SettingsScreenProps {
  desktopShell: boolean
  preferences: AppPreferences
  onPreferencesChange: (nextPreferences: AppPreferences) => void
  shellIntegration: ShellIntegrationStatus
  shellIntegrationFeedback: ActionFeedback
  onRefreshShellIntegration: () => void
  onInstallShellIntegration: () => void
}

const destinationModeOptions: Array<{
  value: AppPreferences['extractDestinationMode']
  label: string
}> = [
  { value: 'askEveryTime', label: 'Ask every time' },
  { value: 'archiveFolder', label: 'Same folder as archive' },
  { value: 'rememberLast', label: 'Reuse last destination' },
]

const compressionLevelOptions: Array<{
  value: AppPreferences['compressionLevel']
  label: string
}> = [
  { value: 'fastest', label: 'Fastest' },
  { value: 'balanced', label: 'Balanced' },
  { value: 'smallest', label: 'Smallest size' },
]

export function SettingsScreen({
  desktopShell,
  preferences,
  onPreferencesChange,
  shellIntegration,
  shellIntegrationFeedback,
  onRefreshShellIntegration,
  onInstallShellIntegration,
}: SettingsScreenProps) {
  return (
    <section className="page-section page-section--settings">
      <header className="page-header">
        <div>
          <h1>Settings</h1>
          <p>Configure extraction, compression, and app preferences</p>
        </div>
      </header>

      <div className="settings-stack">
        <section className="settings-group">
          <div className="settings-group__label">
            <FolderIcon />
            <span>Extraction</span>
          </div>

          <article className="settings-card">
            <div className="settings-row">
              <div>
                <strong>Default Destination</strong>
                <p>Where to extract files by default</p>
              </div>
              <select
                className="settings-select"
                onChange={(event) => {
                  onPreferencesChange({
                    ...preferences,
                    extractDestinationMode: event.target.value as AppPreferences['extractDestinationMode'],
                  })
                }}
                value={preferences.extractDestinationMode}
              >
                {destinationModeOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>

            {preferences.extractDestinationMode === 'rememberLast' ? (
              <div className="settings-inline-note">
                {preferences.lastExtractDestination
                  ? `Currently reusing: ${preferences.lastExtractDestination}`
                  : 'No destination has been remembered yet. Choose a folder once in Extract to seed this preference.'}
              </div>
            ) : null}

            <div className="settings-row">
              <div>
                <strong>Conflict Resolution</strong>
                <p>When a file already exists</p>
              </div>
              <select
                className="settings-select"
                onChange={(event) => {
                  onPreferencesChange({
                    ...preferences,
                    extractConflictPolicy: event.target.value as ConflictPolicy,
                  })
                }}
                value={preferences.extractConflictPolicy}
              >
                {conflictPolicyOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </article>
        </section>

        <section className="settings-group">
          <div className="settings-group__label settings-group__label--mint">
            <CompressIcon />
            <span>Compression</span>
          </div>

          <article className="settings-card">
            <div className="settings-row">
              <div>
                <strong>Default Format</strong>
                <p>Preferred archive format</p>
              </div>
              <select
                className="settings-select"
                onChange={(event) => {
                  onPreferencesChange({
                    ...preferences,
                    defaultCompressFormat: event.target.value as CompressFormat,
                  })
                }}
                value={preferences.defaultCompressFormat}
              >
                {compressFormatOptions.map((format) => (
                  <option key={format} value={format}>
                    {format}
                  </option>
                ))}
              </select>
            </div>

            <div className="settings-row">
              <div>
                <strong>Compression Level</strong>
                <p>Balance between speed and size</p>
              </div>
              <select
                className="settings-select"
                onChange={(event) => {
                  onPreferencesChange({
                    ...preferences,
                    compressionLevel: event.target.value as AppPreferences['compressionLevel'],
                  })
                }}
                value={preferences.compressionLevel}
              >
                {compressionLevelOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </div>
          </article>
        </section>

        <section className="settings-group">
          <div className="settings-group__label settings-group__label--amber">
            <ShieldIcon />
            <span>Advanced</span>
          </div>

          <article className="settings-card">
            <label className="settings-toggle">
              <div>
                <strong>Delete archive after extraction</strong>
                <p>Remove the source archive after a successful extraction.</p>
              </div>
              <input
                checked={preferences.deleteArchiveAfterExtraction}
                onChange={(event) => {
                  onPreferencesChange({
                    ...preferences,
                    deleteArchiveAfterExtraction: event.target.checked,
                  })
                }}
                type="checkbox"
              />
            </label>
          </article>
        </section>

        <section className="settings-group">
          <div className="settings-group__label settings-group__label--muted">
            <SettingsIcon />
            <span>System Integration</span>
          </div>

          <article className="settings-card settings-card--stacked">
            <div className="settings-row">
              <div>
                <strong>{shellIntegration.installed ? 'Shell integration active' : 'Shell integration available'}</strong>
                <p>{shellIntegration.note}</p>
              </div>
              <div className="settings-row__actions">
                <button
                  className="ghost-button"
                  disabled={!desktopShell}
                  onClick={onRefreshShellIntegration}
                  type="button"
                >
                  Re-check
                </button>
                <button
                  className="ghost-button"
                  disabled={!desktopShell || !shellIntegration.canInstall}
                  onClick={onInstallShellIntegration}
                  type="button"
                >
                  {shellIntegration.installed ? 'Reinstall' : 'Install'}
                </button>
              </div>
            </div>
            <ActionBanner feedback={shellIntegrationFeedback} />
          </article>
        </section>
      </div>
    </section>
  )
}
