import type { ArchiveHistoryEntry } from '../app/types'
import { formatHistoryTimestamp } from '../app/utils'

interface RecentOperationsPanelProps {
  desktopShell: boolean
  history: ArchiveHistoryEntry[]
  onRefresh: () => void
  onClear: () => void
}

export function RecentOperationsPanel({
  desktopShell,
  history,
  onRefresh,
  onClear,
}: RecentOperationsPanelProps) {
  return (
    <article className="panel-card panel-card--wide">
      <div className="history-header">
        <div>
          <p className="card-label">Recent operations</p>
          <h2>Latest archive jobs on this machine.</h2>
        </div>
        <div className="button-row">
          <button className="ghost-button" disabled={!desktopShell} onClick={onRefresh} type="button">
            Refresh
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell || history.length === 0}
            onClick={onClear}
            type="button"
          >
            Clear
          </button>
        </div>
      </div>
      {history.length === 0 ? (
        <p className="history-empty">No archive jobs have been recorded yet.</p>
      ) : (
        <div className="history-list">
          {history.map((entry) => (
            <article className="history-item" key={entry.id}>
              <div className="history-item__meta">
                <span className={`chip ${entry.operation === 'compress' ? '' : 'chip--soft'}`}>
                  {entry.operation}
                </span>
                <span className="chip chip--muted">{entry.format}</span>
                <span className="history-time">{formatHistoryTimestamp(entry.timestampMs)}</span>
              </div>
              <strong>{entry.sourceSummary}</strong>
              <p>{entry.outputPath}</p>
            </article>
          ))}
        </div>
      )}
    </article>
  )
}
