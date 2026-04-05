import { getCurrentWindow } from '@tauri-apps/api/window'
import { startTransition, useEffect, useEffectEvent, useState } from 'react'
import type { DragDropState } from '../app/types'
import { isArchivePath } from '../app/utils'

const idleState: DragDropState = {
  active: false,
  intent: 'idle',
  message: 'Drag files or folders into Ziply to load them instantly.',
}

interface UseDesktopDragDropOptions {
  desktopShell: boolean
  onDropPaths: (paths: string[]) => void | Promise<void>
}

function describeDrop(paths: string[]): DragDropState {
  const normalizedPaths = Array.from(new Set(paths.map((path) => path.trim()).filter(Boolean)))

  if (normalizedPaths.length === 1 && isArchivePath(normalizedPaths[0])) {
    return {
      active: true,
      intent: 'extract',
      message: 'Drop to load this archive into Extract.',
    }
  }

  if (normalizedPaths.length > 0) {
    return {
      active: true,
      intent: 'compress',
      message:
        normalizedPaths.length === 1
          ? 'Drop to add this item to Compress.'
          : `Drop to add ${normalizedPaths.length} items to Compress.`,
    }
  }

  return {
    active: true,
    intent: 'idle',
    message: 'Drop files or folders into Ziply to load them instantly.',
  }
}

export function useDesktopDragDrop({
  desktopShell,
  onDropPaths,
}: UseDesktopDragDropOptions) {
  const [dragDropState, setDragDropState] = useState<DragDropState>(idleState)
  const handleDropPaths = useEffectEvent(async (paths: string[]) => {
    await onDropPaths(paths)
  })

  useEffect(() => {
    if (!desktopShell) {
      startTransition(() => {
        setDragDropState(idleState)
      })
      return
    }

    let unlisten: (() => void) | undefined

    void getCurrentWindow()
      .onDragDropEvent((event) => {
        const payload = event.payload

        switch (payload.type) {
          case 'enter':
            startTransition(() => {
              setDragDropState(describeDrop(payload.paths))
            })
            return
          case 'over':
            startTransition(() => {
              setDragDropState((currentState) => ({ ...currentState, active: true }))
            })
            return
          case 'leave':
            startTransition(() => {
              setDragDropState(idleState)
            })
            return
          case 'drop':
            startTransition(() => {
              setDragDropState(idleState)
            })
            void handleDropPaths(payload.paths)
            return
        }
      })
      .then((dispose) => {
        unlisten = dispose
      })

    return () => {
      unlisten?.()
    }
  }, [desktopShell])

  return {
    dragDropState,
  }
}
