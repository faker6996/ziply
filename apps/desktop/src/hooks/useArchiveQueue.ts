import { startTransition, useEffect, useEffectEvent, useState } from 'react'
import type {
  ArchiveActionResult,
  ArchiveQueueItem,
  CompressArchiveRequest,
  ExtractArchiveRequest,
} from '../app/types'
import {
  inferArchiveFormatFromPath,
  recoveryHintForArchiveError,
  summarizePathList,
} from '../app/utils'

interface UseArchiveQueueOptions {
  desktopShell: boolean
  runCompressRequest: (
    request: CompressArchiveRequest,
    options?: { skipFeedback?: boolean },
  ) => Promise<ArchiveActionResult>
  runExtractRequest: (
    request: ExtractArchiveRequest,
    options?: { skipFeedback?: boolean },
  ) => Promise<ArchiveActionResult>
}

function createQueueId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID()
  }

  return `queue-${Date.now()}-${Math.random().toString(16).slice(2)}`
}

export function useArchiveQueue({
  desktopShell,
  runCompressRequest,
  runExtractRequest,
}: UseArchiveQueueOptions) {
  const [queueItems, setQueueItems] = useState<ArchiveQueueItem[]>([])
  const activeJob = queueItems.find((item) => item.status === 'running') ?? null

  const executeQueuedJob = useEffectEvent(async (job: ArchiveQueueItem) => {
    try {
      const result =
        job.operation === 'compress'
          ? await runCompressRequest(job.request, { skipFeedback: true })
          : await runExtractRequest(job.request, { skipFeedback: true })

      startTransition(() => {
        setQueueItems((currentItems) =>
          currentItems.map((item) =>
            item.id === job.id
              ? {
                  ...item,
                  status: 'success',
                  message: result.message,
                  recoveryHint: undefined,
                  outputPath: result.outputPath,
                  finishedAt: Date.now(),
                }
              : item,
          ),
        )
      })
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      startTransition(() => {
        setQueueItems((currentItems) =>
          currentItems.map((item) =>
            item.id === job.id
              ? {
                  ...item,
                  status: 'error',
                  message: detail,
                  recoveryHint: recoveryHintForArchiveError(detail),
                  finishedAt: Date.now(),
                }
              : item,
          ),
        )
      })
    }
  })

  useEffect(() => {
    if (!desktopShell || activeJob) {
      return
    }

    const nextJob = queueItems.find((item) => item.status === 'queued')
    if (!nextJob) {
      return
    }

    startTransition(() => {
      setQueueItems((currentItems) =>
        currentItems.map((item) =>
          item.id === nextJob.id
            ? {
                ...item,
                status: 'running',
                message: 'Running batch job...',
                startedAt: Date.now(),
              }
            : item,
        ),
      )
    })

    void executeQueuedJob(nextJob)
  }, [activeJob, desktopShell, queueItems])

  function enqueueCompressJob(request: CompressArchiveRequest) {
    const createdAt = Date.now()
    const queueItem: ArchiveQueueItem = {
      id: createQueueId(),
      status: 'queued',
      operation: 'compress',
      format: request.format,
      sourceSummary: summarizePathList(request.sourcePaths),
      outputPath: request.destinationPath,
      message: 'Waiting for an available queue slot.',
      passwordProtected: Boolean(request.password),
      createdAt,
      request,
    }

    let queuePosition = 1
    startTransition(() => {
      setQueueItems((currentItems) => {
        queuePosition = currentItems.filter((item) => item.status === 'queued').length + 1
        return [...currentItems, queueItem]
      })
    })

    return queuePosition
  }

  function enqueueExtractJob(request: ExtractArchiveRequest) {
    const createdAt = Date.now()
    const queueItem: ArchiveQueueItem = {
      id: createQueueId(),
      status: 'queued',
      operation: 'extract',
      format: inferArchiveFormatFromPath(request.archivePath),
      sourceSummary: summarizePathList([request.archivePath]),
      outputPath: request.destinationDirectory,
      message: 'Waiting for an available queue slot.',
      passwordProtected: Boolean(request.password),
      createdAt,
      request,
    }

    let queuePosition = 1
    startTransition(() => {
      setQueueItems((currentItems) => {
        queuePosition = currentItems.filter((item) => item.status === 'queued').length + 1
        return [...currentItems, queueItem]
      })
    })

    return queuePosition
  }

  function removeQueuedJob(jobId: string) {
    startTransition(() => {
      setQueueItems((currentItems) =>
        currentItems.filter((item) => item.id !== jobId || item.status !== 'queued'),
      )
    })
  }

  function clearFinishedQueue() {
    startTransition(() => {
      setQueueItems((currentItems) =>
        currentItems.filter((item) => item.status === 'queued' || item.status === 'running'),
      )
    })
  }

  function retryQueueJob(jobId: string) {
    startTransition(() => {
      setQueueItems((currentItems) =>
        currentItems.map((item) =>
          item.id === jobId && item.status === 'error'
            ? {
                ...item,
                status: 'queued',
                message: 'Waiting for an available queue slot.',
                recoveryHint: undefined,
                startedAt: undefined,
                finishedAt: undefined,
              }
            : item,
        ),
      )
    })
  }

  return {
    queueItems,
    activeQueueJobId: activeJob?.id ?? null,
    enqueueCompressJob,
    enqueueExtractJob,
    removeQueuedJob,
    clearFinishedQueue,
    retryQueueJob,
  }
}
