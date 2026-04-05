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
import { useArchiveHistory } from './useArchiveHistory'
import { useDesktopDragDrop } from './useDesktopDragDrop'
import { useLiveJobs } from './useLiveJobs'
import { useShellIntegration } from './useShellIntegration'

export function useZiplyRuntime() {
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
    compressFeedback,
    extractSource,
    extractDestination,
    extractConflictPolicy,
    extractFeedback,
    normalizedCompressSources,
    gzipSourceCount,
    setCompressSources,
    setCompressDestination,
    setCompressFormat,
    setCompressConflictPolicy,
    setExtractSource,
    setExtractDestination,
    setExtractConflictPolicy,
    handleShellIntent,
    handleDroppedPaths,
    pickCompressFiles,
    pickCompressFolders,
    pickCompressDestination,
    pickExtractSource,
    pickExtractDestination,
    runCompress,
    runExtract,
  } = useArchiveActions({
    desktopShell,
    refreshHistory,
  })
  const handleShellIntentEvent = useEffectEvent(async (intent: ShellIntent) => {
    await handleShellIntent(intent)
  })
  const handleDroppedPathsEvent = useEffectEvent(async (paths: string[]) => {
    await handleDroppedPaths(paths)
  })
  const { dragDropState } = useDesktopDragDrop({
    desktopShell,
    onDropPaths: handleDroppedPathsEvent,
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

  return {
    overview,
    capabilities,
    history,
    liveJobs,
    shellIntegration,
    runtimeStatus,
    compressSources,
    compressDestination,
    compressFormat,
    compressConflictPolicy,
    compressFeedback,
    extractSource,
    extractDestination,
    extractConflictPolicy,
    extractFeedback,
    shellIntegrationFeedback,
    dragDropState,
    desktopShell,
    normalizedCompressSources,
    gzipSourceCount,
    setCompressSources,
    setCompressDestination,
    setCompressFormat,
    setCompressConflictPolicy,
    setExtractSource,
    setExtractDestination,
    setExtractConflictPolicy,
    refreshHistory,
    refreshShellIntegration,
    clearHistory,
    installShellIntegration,
    pickCompressFiles,
    pickCompressFolders,
    pickCompressDestination,
    pickExtractSource,
    pickExtractDestination,
    runCompress,
    runExtract,
  }
}
