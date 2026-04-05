import { invoke } from '@tauri-apps/api/core'
import { startTransition, useState } from 'react'
import { fallbackHistory } from '../app/defaults'
import type { ArchiveHistoryEntry } from '../app/types'

export function useArchiveHistory(desktopShell: boolean) {
  const [history, setHistory] = useState<ArchiveHistoryEntry[]>(fallbackHistory)

  async function refreshHistory() {
    if (!desktopShell) {
      setHistory(fallbackHistory)
      return
    }

    try {
      const nextHistory = await invoke<ArchiveHistoryEntry[]>('get_archive_history')
      startTransition(() => {
        setHistory(nextHistory)
      })
    } catch {
      startTransition(() => {
        setHistory(fallbackHistory)
      })
    }
  }

  async function clearHistory() {
    if (!desktopShell) {
      setHistory(fallbackHistory)
      return
    }

    await invoke('clear_archive_history')
    startTransition(() => {
      setHistory([])
    })
  }

  return {
    history,
    setHistory,
    refreshHistory,
    clearHistory,
  }
}
