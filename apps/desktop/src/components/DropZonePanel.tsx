import type { DragDropState } from '../app/types'
import { DropIcon } from './AppIcons'

interface DropZonePanelProps {
  desktopShell: boolean
  dragDropState: DragDropState
  onPickArchive?: () => void
  onPickFiles?: () => void
}

export function DropZonePanel({
  desktopShell,
  dragDropState,
  onPickArchive,
  onPickFiles,
}: DropZonePanelProps) {
  return (
    <article
      className={`panel-card panel-card--wide panel-card--dropzone ${
        dragDropState.active ? 'panel-card--dropzone-active' : ''
      }`}
    >
      <div className="dropzone-copy">
        <div className="dropzone-copy__icon">
          <DropIcon />
        </div>
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
              : 'Drop an archive to extract and preview, or drop files and folders to compress them.'
            : 'Launch the packaged desktop app to use native drag and drop.'}
        </p>
        <div className="button-row">
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickArchive}
            type="button"
          >
            Browse Archive
          </button>
          <button
            className="ghost-button"
            disabled={!desktopShell}
            onClick={onPickFiles}
            type="button"
          >
            Browse Files
          </button>
        </div>
      </div>
    </article>
  )
}
