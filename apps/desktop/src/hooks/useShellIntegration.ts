import { invoke } from '@tauri-apps/api/core'
import { startTransition, useState } from 'react'
import { emptyFeedback, fallbackShellIntegration } from '../app/defaults'
import type { ActionFeedback, ShellIntegrationStatus } from '../app/types'

export function useShellIntegration(desktopShell: boolean) {
  const [shellIntegration, setShellIntegration] =
    useState<ShellIntegrationStatus>(fallbackShellIntegration)
  const [shellIntegrationFeedback, setShellIntegrationFeedback] =
    useState<ActionFeedback>(emptyFeedback)

  async function refreshShellIntegration() {
    if (!desktopShell) {
      setShellIntegration(fallbackShellIntegration)
      return
    }

    try {
      const nextStatus = await invoke<ShellIntegrationStatus>('shell_integration_status')
      startTransition(() => {
        setShellIntegration(nextStatus)
      })
    } catch {
      startTransition(() => {
        setShellIntegration(fallbackShellIntegration)
      })
    }
  }

  async function installShellIntegration() {
    if (!desktopShell || !shellIntegration.canInstall) {
      return
    }

    setShellIntegrationFeedback({
      status: 'running',
      message: 'Installing shell integration...',
    })

    try {
      const nextStatus = await invoke<ShellIntegrationStatus>('install_shell_integration')
      startTransition(() => {
        setShellIntegration(nextStatus)
        setShellIntegrationFeedback({
          status: 'success',
          message: nextStatus.installed
            ? 'Shell integration is installed.'
            : nextStatus.note,
        })
      })
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      startTransition(() => {
        setShellIntegrationFeedback({
          status: 'error',
          message: detail,
        })
      })
    }
  }

  return {
    shellIntegration,
    setShellIntegration,
    shellIntegrationFeedback,
    refreshShellIntegration,
    installShellIntegration,
  }
}
