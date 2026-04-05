import type { ArchiveQueueItem } from '../app/types'
import {
  formatHistoryTimestamp,
  formatLiveJobStatus,
  liveJobStatusChipClass,
} from '../app/utils'

interface BatchQueuePanelProps {
  queueItems: ArchiveQueueItem[]
  onClearFinished: () => void
  onRemoveQueuedJob: (jobId: string) => void
}

export function BatchQueuePanel({
  queueItems,
  onClearFinished,
  onRemoveQueuedJob,
}: BatchQueuePanelProps) {
  const queuedJobs = queueItems.filter((item) => item.status === 'queued')
  const finishedJobCount = queueItems.filter(
    (item) => item.status === 'success' || item.status === 'error',
  ).length

  return (
    <article className="panel-card panel-card--wide">
      <div className="history-header">
        <div>
          <p className="card-label">Batch queue</p>
          <h2>Stack multiple archive jobs and let Ziply run them one by one.</h2>
        </div>
        <div className="queue-summary">
          <span className="chip chip--soft">{queuedJobs.length} waiting</span>
          <button
            className="ghost-button"
            disabled={finishedJobCount === 0}
            onClick={onClearFinished}
            type="button"
          >
            Clear finished
          </button>
        </div>
      </div>

      {queueItems.length === 0 ? (
        <p className="jobs-empty">
          No queued jobs yet. Add any compress or extract task to build a batch.
        </p>
      ) : (
        <div className="jobs-list">
          {queueItems.map((job) => {
            const queuePosition =
              job.status === 'queued'
                ? queuedJobs.findIndex((queuedJob) => queuedJob.id === job.id) + 1
                : null

            return (
              <article className="job-card" key={job.id}>
                <div className="job-card__header">
                  <div className="job-card__meta">
                    <span className={liveJobStatusChipClass(job.status)}>
                      {formatLiveJobStatus(job.status)}
                    </span>
                    <span className={`chip ${job.operation === 'compress' ? '' : 'chip--soft'}`}>
                      {job.operation}
                    </span>
                    <span className="chip chip--muted">{job.format}</span>
                    {job.passwordProtected ? <span className="chip chip--warm">password</span> : null}
                    <span className="history-time">{formatHistoryTimestamp(job.createdAt)}</span>
                  </div>
                  <div className="queue-card__actions">
                    {queuePosition ? <span className="chip chip--soft">#{queuePosition}</span> : null}
                    {job.status === 'queued' ? (
                      <button
                        className="ghost-button"
                        onClick={() => {
                          onRemoveQueuedJob(job.id)
                        }}
                        type="button"
                      >
                        Remove
                      </button>
                    ) : null}
                  </div>
                </div>
                <strong>{job.message}</strong>
                <p>{job.sourceSummary}</p>
                <p className="job-path">{job.outputPath}</p>
              </article>
            )
          })}
        </div>
      )}
    </article>
  )
}
