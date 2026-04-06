import type { ActionFeedback, ShellIntegrationStatus } from '../app/types'
import { ActionBanner } from './ActionBanner'

interface ShellIntegrationPanelProps {
  desktopShell: boolean
  shellIntegration: ShellIntegrationStatus
  feedback: ActionFeedback
  onRefresh: () => void
  onInstall: () => void
}

export function ShellIntegrationPanel({
  desktopShell,
  shellIntegration,
  feedback,
  onRefresh,
  onInstall,
}: ShellIntegrationPanelProps) {
  return (
    <article className="panel-card panel-card--wide">
      <div className="history-header">
        <div>
          <p className="card-label">Shell integration</p>
          <h2>Open or extract archives from the operating system.</h2>
        </div>
        <div className="button-row">
          <button className="ghost-button" disabled={!desktopShell} onClick={onRefresh} type="button">
            Re-check
          </button>
          <button
            className="primary-button"
            disabled={!desktopShell || !shellIntegration.canInstall}
            onClick={onInstall}
            type="button"
          >
            {shellIntegration.installed ? 'Reinstall integration' : 'Install integration'}
          </button>
        </div>
      </div>
      <div className="support-note">
        <strong>
          {shellIntegration.installed
            ? 'Shell integration is active.'
            : shellIntegration.supported
              ? 'Shell integration is available but not installed yet.'
              : 'Shell integration is limited on this platform.'}
        </strong>
        <p>{shellIntegration.note}</p>
        <p>
          Windows installs Explorer commands for `Extract with Ziply`, `Extract here with Ziply`,
          and `Compress with Ziply`. Linux installs desktop actions for supporting file managers.
          macOS installs Finder Quick Actions for `Extract with Ziply` and `Extract here with Ziply`
          during Homebrew installs and automatically on first launch for manual installs. Use the
          button here to repair or reinstall them.
        </p>
      </div>
      <ActionBanner feedback={feedback} />
    </article>
  )
}
