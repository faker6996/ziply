import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { startTransition, useState, type FormEvent } from 'react'
import { emptyFeedback } from '../app/defaults'
import type {
  ActionFeedback,
  ArchiveActionResult,
  CompressFormat,
  ShellIntent,
} from '../app/types'
import {
  normalizePaths,
  pathsToText,
  suggestArchiveName,
  suggestExtractDestination,
  toDialogPaths,
} from '../app/utils'

interface UseArchiveActionsOptions {
  desktopShell: boolean
  refreshHistory: () => Promise<void>
}

export function useArchiveActions({
  desktopShell,
  refreshHistory,
}: UseArchiveActionsOptions) {
  const [compressSources, setCompressSources] = useState('')
  const [compressDestination, setCompressDestination] = useState('')
  const [compressFormat, setCompressFormat] = useState<CompressFormat>('zip')
  const [compressFeedback, setCompressFeedback] = useState<ActionFeedback>(emptyFeedback)
  const [extractSource, setExtractSource] = useState('')
  const [extractDestination, setExtractDestination] = useState('')
  const [extractFeedback, setExtractFeedback] = useState<ActionFeedback>(emptyFeedback)

  const normalizedCompressSources = normalizePaths(compressSources)
  const gzipSourceCount = compressFormat === 'gz' ? normalizedCompressSources.length : 0

  async function executeExtract(archivePath: string, destinationDirectory: string) {
    if (!desktopShell) {
      setExtractFeedback({
        status: 'error',
        message: 'Archive operations run inside the Tauri desktop shell.',
      })
      return
    }

    setExtractFeedback({
      status: 'running',
      message: 'Extracting archive...',
    })

    try {
      const result = await invoke<ArchiveActionResult>('extract_archive', {
        request: {
          archivePath,
          destinationDirectory,
        },
      })

      startTransition(() => {
        setExtractFeedback({
          status: 'success',
          message: result.message,
          outputPath: result.outputPath,
        })
      })
      void refreshHistory()
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      startTransition(() => {
        setExtractFeedback({
          status: 'error',
          message: detail,
        })
      })
    }
  }

  async function executeCompress(
    sourcePaths: string[],
    destinationPath: string,
    format: CompressFormat,
  ) {
    if (!desktopShell) {
      setCompressFeedback({
        status: 'error',
        message: 'Archive operations run inside the Tauri desktop shell.',
      })
      return
    }

    setCompressFeedback({
      status: 'running',
      message: 'Creating archive...',
    })

    try {
      const result = await invoke<ArchiveActionResult>('compress_archive', {
        request: {
          sourcePaths,
          destinationPath,
          format,
        },
      })

      startTransition(() => {
        setCompressFeedback({
          status: 'success',
          message: result.message,
          outputPath: result.outputPath,
        })
      })
      void refreshHistory()
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      startTransition(() => {
        setCompressFeedback({
          status: 'error',
          message: detail,
        })
      })
    }
  }

  async function handleShellIntent(intent: ShellIntent) {
    if (intent.action === 'compress') {
      startTransition(() => {
        setCompressSources(pathsToText(intent.paths))
        setCompressFeedback({
          status: 'success',
          message: 'Items were loaded from the OS shell. Choose a format and create the archive.',
        })
      })
      return
    }

    const archivePath = intent.paths[0]?.trim()
    if (!archivePath) {
      return
    }

    const destinationDirectory =
      intent.destinationPath?.trim() || suggestExtractDestination(archivePath, intent.autoRun)

    startTransition(() => {
      setExtractSource(archivePath)
      setExtractDestination(destinationDirectory)
      if (!intent.autoRun) {
        setExtractFeedback({
          status: 'success',
          message:
            'Archive loaded from shell integration. Review the destination and extract when ready.',
          outputPath: destinationDirectory,
        })
      }
    })

    if (intent.autoRun) {
      await executeExtract(archivePath, destinationDirectory)
    }
  }

  function appendCompressSources(nextPaths: string[]) {
    const merged = Array.from(new Set([...normalizePaths(compressSources), ...nextPaths]))
    setCompressSources(pathsToText(merged))
  }

  async function pickCompressFiles() {
    if (!desktopShell) {
      return
    }

    const selection = toDialogPaths(
      await open({
        multiple: true,
        directory: false,
      }),
    )
    appendCompressSources(selection)
  }

  async function pickCompressFolders() {
    if (!desktopShell) {
      return
    }

    const selection = toDialogPaths(
      await open({
        multiple: true,
        directory: true,
      }),
    )
    appendCompressSources(selection)
  }

  async function pickCompressDestination() {
    if (!desktopShell) {
      return
    }

    const selection = await save({
      defaultPath: suggestArchiveName(compressFormat, normalizedCompressSources),
    })

    if (selection) {
      setCompressDestination(selection)
    }
  }

  async function pickExtractSource() {
    if (!desktopShell) {
      return
    }

    const selection = await open({
      multiple: false,
      directory: false,
    })

    if (typeof selection === 'string') {
      setExtractSource(selection)
    }
  }

  async function pickExtractDestination() {
    if (!desktopShell) {
      return
    }

    const selection = await open({
      multiple: false,
      directory: true,
    })

    if (typeof selection === 'string') {
      setExtractDestination(selection)
    }
  }

  async function runCompress(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    await executeCompress(normalizedCompressSources, compressDestination.trim(), compressFormat)
  }

  async function runExtract(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    await executeExtract(extractSource.trim(), extractDestination.trim())
  }

  return {
    compressSources,
    compressDestination,
    compressFormat,
    compressFeedback,
    extractSource,
    extractDestination,
    extractFeedback,
    normalizedCompressSources,
    gzipSourceCount,
    setCompressSources,
    setCompressDestination,
    setCompressFormat,
    setExtractSource,
    setExtractDestination,
    handleShellIntent,
    pickCompressFiles,
    pickCompressFolders,
    pickCompressDestination,
    pickExtractSource,
    pickExtractDestination,
    runCompress,
    runExtract,
  }
}
