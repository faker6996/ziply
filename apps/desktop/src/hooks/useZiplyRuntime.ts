import { listen } from '@tauri-apps/api/event'
import { startTransition, useEffect, useEffectEvent, useState } from 'react'
import {
  fallbackCapabilities,
  fallbackHistory,
  fallbackOverview,
  fallbackShellIntegration,
} from '../app/defaults'
import type { ShellIntent } from '../app/types'
import { isDesktopShell, loadBootstrapData } from '../app/utils'
import { useArchiveActions } from './useArchiveActions'
import { useArchiveQueue } from './useArchiveQueue'
import { useArchiveHistory } from './useArchiveHistory'
import { useDesktopDragDrop } from './useDesktopDragDrop'
import { useLiveJobs } from './useLiveJobs'
import { useShellIntegration } from './useShellIntegration'
import type { AppPreferences } from '../components/SettingsScreen'

interface UseZiplyRuntimeOptions {
  preferences: AppPreferences
  onRememberExtractDestination: (path: string) => void
}

export function useZiplyRuntime({
  preferences,
  onRememberExtractDestination,
}: UseZiplyRuntimeOptions) {
  const [overview, setOverview] = useState(fallbackOverview)
  const [capabilities, setCapabilities] = useState(fallbackCapabilities)
  const [runtimeStatus, setRuntimeStatus] = useState<'loading' | 'ready' | 'error'>('loading')
  const desktopShell = isDesktopShell()
  const { history, setHistory, refreshHistory, clearHistory } = useArchiveHistory(desktopShell)
  const {
    shellIntegration,
    setShellIntegration,
    shellIntegrationFeedback,
    refreshShellIntegration,
    installShellIntegration,
  } = useShellIntegration(desktopShell)
  const { liveJobs } = useLiveJobs(desktopShell)
  const {
    compressSources,
    compressDestination,
    compressFormat,
    compressConflictPolicy,
    compressPassword,
    compressFeedback,
    extractSource,
    extractDestination,
    extractConflictPolicy,
    extractPassword,
    extractFeedback,
    extractPreview,
    extractPreviewStatus,
    extractPreviewError,
    extractPreviewLimit,
    extractSelectedEntries,
    normalizedCompressSources,
    gzipSourceCount,
    setCompressFeedback,
    setCompressSources,
    setCompressDestination,
    setCompressFormat,
    setCompressConflictPolicy,
    setCompressPassword,
    setExtractSource,
    setExtractDestination,
    setExtractConflictPolicy,
    setExtractFeedback,
    setExtractPassword,
    buildCompressRequest,
    queueExtract,
    handleShellIntent,
    handleDroppedPaths,
    pickCompressFiles,
    pickCompressFolders,
    pickCompressDestination,
    pickExtractSource,
    pickExtractDestination,
    runCompressRequest,
    runExtractRequest,
    runCompress,
    runExtract,
    runExtractAll,
    runExtractSelected,
    toggleExtractEntry,
    selectAllVisibleExtractEntries,
    clearExtractSelection,
    loadMoreExtractPreview,
    supportsArchivePasswordOnCompress,
    supportsArchivePasswordOnExtract,
    supportsSelectiveExtract,
  } = useArchiveActions({
    desktopShell,
    refreshHistory,
    extractDestinationMode: preferences.extractDestinationMode,
    deleteAfterExtraction: preferences.deleteArchiveAfterExtraction,
    lastExtractDestination: preferences.lastExtractDestination,
    onRememberExtractDestination,
  })
  const {
    queueItems,
    activeQueueJobId,
    enqueueCompressJob,
    enqueueExtractJob,
    removeQueuedJob,
    clearFinishedQueue,
    retryQueueJob,
  } = useArchiveQueue({
    desktopShell,
    runCompressRequest,
    runExtractRequest,
  })
  const handleShellIntentEvent = useEffectEvent(async (intent: ShellIntent) => {
    await handleShellIntent(intent)
  })
  const { dragDropState } = useDesktopDragDrop({
    desktopShell,
    onDropPaths: handleDroppedPaths,
  })

  useEffect(() => {
    let active = true

    void loadBootstrapData()
      .then(
        ({
          overview: nextOverview,
          capabilities: nextCapabilities,
          history: nextHistory,
          shellIntegration: nextShellIntegration,
          shellIntents,
        }) => {
          if (!active) {
            return
          }

          startTransition(() => {
            setOverview(nextOverview)
            setCapabilities(nextCapabilities)
            setHistory(nextHistory)
            setShellIntegration(nextShellIntegration)
            setRuntimeStatus('ready')
          })

          for (const intent of shellIntents) {
            void handleShellIntentEvent(intent)
          }
        },
      )
      .catch(() => {
        if (!active) {
          return
        }

        startTransition(() => {
          setOverview(fallbackOverview)
          setCapabilities(fallbackCapabilities)
          setHistory(fallbackHistory)
          setShellIntegration(fallbackShellIntegration)
          setRuntimeStatus('error')
        })
      })

    return () => {
      active = false
    }
  }, [setHistory, setShellIntegration])

  useEffect(() => {
    if (!desktopShell) {
      return
    }

    let unlisten: (() => void) | undefined

    void listen<ShellIntent>('shell-intent', (event) => {
      void handleShellIntentEvent(event.payload)
    }).then((dispose) => {
      unlisten = dispose
    })

    return () => {
      unlisten?.()
    }
  }, [desktopShell])

  function queueCurrentCompress() {
    if (!desktopShell) {
      startTransition(() => {
        setCompressFeedback({
          status: 'error',
          message: 'Archive operations run inside the Tauri desktop shell.',
        })
      })
      return
    }

    const request = buildCompressRequest()
    const queuePosition = enqueueCompressJob(request)

    startTransition(() => {
      setCompressFeedback({
        status: 'success',
        message:
          activeQueueJobId == null && queuePosition === 1
            ? 'Job added to the batch queue. It will start immediately.'
            : `Job added to the batch queue in position ${queuePosition}.`,
        outputPath: request.destinationPath,
      })
    })
  }

  function queueCurrentExtract() {
    if (!desktopShell) {
      startTransition(() => {
        setExtractFeedback({
          status: 'error',
          message: 'Archive operations run inside the Tauri desktop shell.',
        })
      })
      return
    }

    const request = queueExtract(true)
    const queuePosition = enqueueExtractJob(request)

    startTransition(() => {
      setExtractFeedback({
        status: 'success',
        message:
          activeQueueJobId == null && queuePosition === 1
            ? 'Job added to the batch queue. It will start immediately.'
            : `Job added to the batch queue in position ${queuePosition}.`,
        outputPath: request.destinationDirectory,
      })
    })
  }

  function queueAllExtract() {
    if (!desktopShell) {
      startTransition(() => {
        setExtractFeedback({
          status: 'error',
          message: 'Archive operations run inside the Tauri desktop shell.',
        })
      })
      return
    }

    const request = queueExtract(false)
    const queuePosition = enqueueExtractJob(request)

    startTransition(() => {
      setExtractFeedback({
        status: 'success',
        message:
          activeQueueJobId == null && queuePosition === 1
            ? 'Full extract job added to the batch queue. It will start immediately.'
            : `Full extract job added to the batch queue in position ${queuePosition}.`,
        outputPath: request.destinationDirectory,
      })
    })
  }

  return {
    overview,
    capabilities,
    history,
    liveJobs,
    queueItems,
    shellIntegration,
    runtimeStatus,
    compressSources,
    compressDestination,
    compressFormat,
    compressConflictPolicy,
    compressPassword,
    compressFeedback,
    extractSource,
    extractDestination,
    extractConflictPolicy,
    extractPassword,
    extractFeedback,
    extractPreview,
    extractPreviewStatus,
    extractPreviewError,
    extractPreviewLimit,
    extractSelectedEntries,
    shellIntegrationFeedback,
    dragDropState,
    desktopShell,
    normalizedCompressSources,
    gzipSourceCount,
    setCompressSources,
    setCompressDestination,
    setCompressFormat,
    setCompressConflictPolicy,
    setCompressPassword,
    setExtractSource,
    setExtractDestination,
    setExtractConflictPolicy,
    setExtractPassword,
    refreshHistory,
    refreshShellIntegration,
    clearHistory,
    clearFinishedQueue,
    installShellIntegration,
    pickCompressFiles,
    pickCompressFolders,
    pickCompressDestination,
    pickExtractSource,
    pickExtractDestination,
    runCompress,
    runExtract,
    runExtractAll,
    runExtractSelected,
    queueCurrentCompress,
    queueCurrentExtract,
    queueAllExtract,
    removeQueuedJob,
    retryQueueJob,
    toggleExtractEntry,
    selectAllVisibleExtractEntries,
    clearExtractSelection,
    loadMoreExtractPreview,
    supportsArchivePasswordOnCompress,
    supportsArchivePasswordOnExtract,
    supportsSelectiveExtract,
  }
}
