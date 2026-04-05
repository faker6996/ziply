import type { ArchiveJobEvent } from '../app/types'
import {
  formatHistoryTimestamp,
  formatLiveJobStatus,
  liveJobProgressClass,
  liveJobStatusChipClass,
} from '../app/utils'

export function LiveJobsPanel({ liveJobs }: { liveJobs: ArchiveJobEvent[] }) {
  return (
    <article className="panel-card panel-card--wide">
      <div className="history-header">
        <div>
          <p className="card-label">Live jobs</p>
          <h2>Real-time backend status for active archive work.</h2>
        </div>
      </div>
      {liveJobs.length === 0 ? (
        <p className="jobs-empty">
          No live job events yet. Start compressing or extracting to watch progress here.
        </p>
      ) : (
        <div className="jobs-list">
          {liveJobs.map((job) => (
            <article className="job-card" key={job.jobId}>
              <div className="job-card__header">
                <div className="job-card__meta">
                  <span className={liveJobStatusChipClass(job.status)}>
                    {formatLiveJobStatus(job.status)}
                  </span>
                  <span className={`chip ${job.operation === 'compress' ? '' : 'chip--soft'}`}>
                    {job.operation}
                  </span>
                  <span className="chip chip--muted">{job.format}</span>
                  <span className="history-time">{formatHistoryTimestamp(job.timestampMs)}</span>
                </div>
                <strong>{job.progress}%</strong>
              </div>
              <div className="progress-track" aria-hidden="true">
                <div
                  className={liveJobProgressClass(job.status)}
                  style={{ width: `${job.progress}%` }}
                />
              </div>
              <strong>{job.message}</strong>
              <p>{job.sourceSummary}</p>
              {job.outputPath ? <p className="job-path">{job.outputPath}</p> : null}
            </article>
          ))}
        </div>
      )}
    </article>
  )
}
