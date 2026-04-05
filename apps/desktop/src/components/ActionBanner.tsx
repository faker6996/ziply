import type { ActionFeedback } from '../app/types'

export function ActionBanner({ feedback }: { feedback: ActionFeedback }) {
  if (feedback.status === 'idle') {
    return (
      <p className="action-banner action-banner--idle">
        Results will appear here after the command finishes.
      </p>
    )
  }

  return (
    <div className={`action-banner action-banner--${feedback.status}`}>
      <strong>{feedback.message}</strong>
      {feedback.recoveryHint ? <p>{feedback.recoveryHint}</p> : null}
      {feedback.outputPath ? <span>{feedback.outputPath}</span> : null}
    </div>
  )
}
