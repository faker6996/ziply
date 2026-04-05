import type { DragDropState } from '../app/types'

interface DropZonePanelProps {
  desktopShell: boolean
  dragDropState: DragDropState
}

export function DropZonePanel({ desktopShell, dragDropState }: DropZonePanelProps) {
  return (
    <article
      className={`panel-card panel-card--wide panel-card--dropzone ${
        dragDropState.active ? 'panel-card--dropzone-active' : ''
      }`}
    >
      <p className="card-label">Drop workspace</p>
      <div className="dropzone-copy">
        <strong>
          {desktopShell
            ? dragDropState.active
              ? dragDropState.intent === 'extract'
                ? 'Release to prepare Extract'
                : 'Release to prepare Compress'
              : 'Drag files, folders, or archives anywhere into Ziply'
            : 'Drag and drop is available inside the desktop shell'}
        </strong>
        <p>
          {desktopShell
            ? dragDropState.active
              ? dragDropState.message
              : 'Drop one archive to route it into Extract, or drop files and folders to route them into Compress.'
            : 'Launch the packaged desktop app to use native drag and drop.'}
        </p>
      </div>
    </article>
  )
}
