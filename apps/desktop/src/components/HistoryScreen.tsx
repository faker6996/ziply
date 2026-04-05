import type { ArchiveHistoryEntry, ArchiveJobEvent } from '../app/types'
import { formatRelativeTimestamp } from '../app/utils'
import {
  ArchiveIcon,
  CheckIcon,
  ClockIcon,
  CompressIcon,
  ErrorIcon,
  ExtractIcon,
  TrashIcon,
} from './AppIcons'

interface HistoryScreenProps {
  desktopShell: boolean
  history: ArchiveHistoryEntry[]
  liveJobs: ArchiveJobEvent[]
  onClear: () => void
}

interface HistoryActivityItem {
  id: string
  operation: 'compress' | 'extract'
  format: string
  title: string
  detail: string
  timestampMs: number
  status: 'running' | 'success' | 'error'
}

interface HistoryActivityGroup {
  label: string
  items: HistoryActivityItem[]
}

function buildHistoryActivities(
  history: ArchiveHistoryEntry[],
  liveJobs: ArchiveJobEvent[],
): HistoryActivityItem[] {
  const liveActivities = liveJobs.map<HistoryActivityItem>((job) => ({
    id: `live:${job.jobId}`,
    operation: job.operation,
    format: job.format,
    title: job.sourceSummary,
    detail: job.message,
    timestampMs: job.timestampMs,
    status: job.status === 'queued' || job.status === 'running' ? 'running' : job.status,
  }))

  const liveKeys = new Set(
    liveJobs
      .filter((job) => job.status === 'success')
      .map((job) => `${job.operation}:${job.sourceSummary}:${job.outputPath ?? ''}`),
  )

  const persistedActivities = history
    .filter((entry) => !liveKeys.has(`${entry.operation}:${entry.sourceSummary}:${entry.outputPath}`))
    .map<HistoryActivityItem>((entry) => ({
      id: `history:${entry.id}`,
      operation: entry.operation === 'compress' ? 'compress' : 'extract',
      format: entry.format,
      title: entry.sourceSummary,
      detail:
        entry.operation === 'compress'
          ? `Compressed to ${entry.outputPath}`
          : `Extracted to ${entry.outputPath}`,
      timestampMs: entry.timestampMs,
      status: 'success',
    }))

  return [...liveActivities, ...persistedActivities].sort(
    (left, right) => right.timestampMs - left.timestampMs,
  )
}

function statusIcon(status: HistoryActivityItem['status']) {
  if (status === 'success') {
    return <CheckIcon />
  }

  if (status === 'error') {
    return <ErrorIcon />
  }

  return <ClockIcon />
}

function statusClassName(status: HistoryActivityItem['status']) {
  if (status === 'success') {
    return 'history-screen__status history-screen__status--success'
  }

  if (status === 'error') {
    return 'history-screen__status history-screen__status--error'
  }

  return 'history-screen__status history-screen__status--running'
}

function statusLabel(status: HistoryActivityItem['status']) {
  if (status === 'success') {
    return 'Completed'
  }

  if (status === 'error') {
    return 'Failed'
  }

  return 'Running'
}

function groupActivitiesByDay(activities: HistoryActivityItem[]) {
  const formatter = new Intl.DateTimeFormat(undefined, {
    weekday: 'long',
    month: 'short',
    day: 'numeric',
  })
  const now = new Date()
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime()
  const dayMs = 24 * 60 * 60 * 1000
  const yesterdayStart = todayStart - dayMs
  const groups = new Map<string, HistoryActivityItem[]>()

  for (const activity of activities) {
    const activityDate = new Date(activity.timestampMs)
    const activityStart = new Date(
      activityDate.getFullYear(),
      activityDate.getMonth(),
      activityDate.getDate(),
    ).getTime()

    let label = formatter.format(activityDate)
    if (activityStart === todayStart) {
      label = 'Today'
    } else if (activityStart === yesterdayStart) {
      label = 'Yesterday'
    }

    const currentItems = groups.get(label) ?? []
    currentItems.push(activity)
    groups.set(label, currentItems)
  }

  return Array.from(groups.entries()).map<HistoryActivityGroup>(([label, items]) => ({
    label,
    items,
  }))
}

export function HistoryScreen({ desktopShell, history, liveJobs, onClear }: HistoryScreenProps) {
  const activities = buildHistoryActivities(history, liveJobs)
  const groupedActivities = groupActivitiesByDay(activities)

  return (
    <section className="page-section page-section--history">
      <header className="page-header page-header--inline">
        <div>
          <h1>History</h1>
          <p>Recent compression and extraction jobs</p>
        </div>
        <button
          className="toolbar-button toolbar-button--danger"
          disabled={!desktopShell || activities.length === 0}
          onClick={onClear}
          type="button"
        >
          <TrashIcon />
          <span>Clear History</span>
        </button>
      </header>

      {activities.length === 0 ? (
        <article className="content-card content-card--empty">
          <p>No archive jobs have been recorded yet.</p>
        </article>
      ) : (
        <div className="history-screen__list">
          {groupedActivities.map((group) => (
            <section className="history-screen__group" key={group.label}>
              <div className="history-screen__group-label">{group.label}</div>
              <div className="history-screen__group-list">
                {group.items.map((entry) => {
                  const isCompress = entry.operation === 'compress'
                  return (
                    <article className="history-screen__item" key={entry.id}>
                      <div className={`history-screen__item-icon ${isCompress ? 'history-screen__item-icon--compress' : 'history-screen__item-icon--extract'}`}>
                        {isCompress ? <CompressIcon /> : <ExtractIcon />}
                      </div>

                      <div className="history-screen__item-body">
                        <div className="history-screen__item-topline">
                          <strong>{entry.title}</strong>
                          <span className="history-screen__timestamp">
                            <ClockIcon />
                            {formatRelativeTimestamp(entry.timestampMs)}
                          </span>
                        </div>

                        <div className="history-screen__item-meta">
                          <span className={statusClassName(entry.status)}>
                            {statusIcon(entry.status)}
                            {statusLabel(entry.status)}
                          </span>
                          <span className="history-screen__dot" />
                          <span>{entry.detail}</span>
                        </div>

                        <div className="history-screen__item-format">
                          <ArchiveIcon />
                          <span>{entry.format}</span>
                        </div>
                      </div>
                    </article>
                  )
                })}
              </div>
            </section>
          ))}
        </div>
      )}

      {!desktopShell ? (
        <p className="page-footnote">
          History controls are active inside the Tauri desktop shell.
        </p>
      ) : null}
    </section>
  )
}
