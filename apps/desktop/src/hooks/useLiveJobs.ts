import { listen } from '@tauri-apps/api/event'
import { startTransition, useEffect, useState } from 'react'
import { fallbackLiveJobs } from '../app/defaults'
import type { ArchiveJobEvent } from '../app/types'
import { upsertLiveJob } from '../app/utils'

export function useLiveJobs(desktopShell: boolean) {
  const [liveJobs, setLiveJobs] = useState<ArchiveJobEvent[]>(fallbackLiveJobs)

  useEffect(() => {
    if (!desktopShell) {
      startTransition(() => {
        setLiveJobs(fallbackLiveJobs)
      })
      return
    }

    let unlisten: (() => void) | undefined

    void listen<ArchiveJobEvent>('archive-job', (event) => {
      startTransition(() => {
        setLiveJobs((currentJobs) => upsertLiveJob(currentJobs, event.payload))
      })
    }).then((dispose) => {
      unlisten = dispose
    })

    return () => {
      unlisten?.()
    }
  }, [desktopShell])

  return {
    liveJobs,
  }
}
