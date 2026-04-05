import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { startTransition, useEffect, useEffectEvent, useState, type FormEvent } from 'react'
import { emptyFeedback } from '../app/defaults'
import type {
  ActionFeedback,
  ArchiveActionResult,
  ArchivePreviewRequest,
  ArchivePreviewResult,
  CompressArchiveRequest,
  CompressFormat,
  ConflictPolicy,
  ExtractArchiveRequest,
  ShellIntent,
} from '../app/types'
import {
  normalizePaths,
  pathsToText,
  isArchivePath,
  suggestArchiveName,
  suggestExtractDestination,
  supportsArchivePasswordOnCompress,
  supportsArchivePasswordOnExtract,
  supportsSelectiveExtract,
  toDialogPaths,
  recoveryHintForArchiveError,
} from '../app/utils'

interface UseArchiveActionsOptions {
  desktopShell: boolean
  refreshHistory: () => Promise<void>
  extractDestinationMode: 'askEveryTime' | 'archiveFolder' | 'rememberLast'
  lastExtractDestination: string
  onRememberExtractDestination: (path: string) => void
}

const EXTRACT_PREVIEW_PAGE_SIZE = 160

export function useArchiveActions({
  desktopShell,
  refreshHistory,
  extractDestinationMode,
  lastExtractDestination,
  onRememberExtractDestination,
}: UseArchiveActionsOptions) {
  const [compressSources, setCompressSources] = useState('')
  const [compressDestination, setCompressDestination] = useState('')
  const [compressFormat, setCompressFormat] = useState<CompressFormat>('zip')
  const [compressConflictPolicy, setCompressConflictPolicy] = useState<ConflictPolicy>('keepBoth')
  const [compressPassword, setCompressPassword] = useState('')
  const [compressFeedback, setCompressFeedback] = useState<ActionFeedback>(emptyFeedback)
  const [extractSource, setExtractSource] = useState('')
  const [extractDestination, setExtractDestination] = useState('')
  const [extractConflictPolicy, setExtractConflictPolicy] = useState<ConflictPolicy>('keepBoth')
  const [extractPassword, setExtractPassword] = useState('')
  const [extractFeedback, setExtractFeedback] = useState<ActionFeedback>(emptyFeedback)
  const [extractPreview, setExtractPreview] = useState<ArchivePreviewResult | null>(null)
  const [extractPreviewStatus, setExtractPreviewStatus] =
    useState<'idle' | 'loading' | 'ready' | 'error'>('idle')
  const [extractPreviewError, setExtractPreviewError] = useState('')
  const [extractSelectedEntries, setExtractSelectedEntries] = useState<string[]>([])
  const [extractPreviewLimit, setExtractPreviewLimit] = useState(EXTRACT_PREVIEW_PAGE_SIZE)

  const normalizedCompressSources = normalizePaths(compressSources)
  const gzipSourceCount = compressFormat === 'gz' ? normalizedCompressSources.length : 0

  function suggestPreferredExtractDestination(archivePath: string, extractHere: boolean) {
    if (extractHere) {
      return suggestExtractDestination(archivePath, true)
    }

    if (extractDestinationMode === 'rememberLast' && lastExtractDestination.trim()) {
      return lastExtractDestination.trim()
    }

    if (extractDestinationMode === 'archiveFolder') {
      return suggestExtractDestination(archivePath, true)
    }

    return suggestExtractDestination(archivePath, false)
  }

  const loadArchivePreview = useEffectEvent(async (archivePath: string) => {
    if (!desktopShell) {
      startTransition(() => {
        setExtractPreview(null)
        setExtractPreviewStatus('idle')
        setExtractPreviewError('')
      })
      return
    }

    const nextPath = archivePath.trim()
    if (!nextPath || !isArchivePath(nextPath)) {
      startTransition(() => {
        setExtractPreview(null)
        setExtractPreviewStatus('idle')
        setExtractPreviewError('')
      })
      return
    }

    startTransition(() => {
      setExtractPreviewStatus('loading')
      setExtractPreviewError('')
    })

    try {
      const request: ArchivePreviewRequest = {
        archivePath: nextPath,
        limit: extractPreviewLimit,
      }
      if (extractPassword) {
        request.password = extractPassword
      }
      const result = await invoke<ArchivePreviewResult>('preview_archive_contents', {
        request,
      })
      startTransition(() => {
        setExtractPreview(result)
        setExtractPreviewStatus('ready')
      })
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      startTransition(() => {
        setExtractPreview(null)
        setExtractPreviewStatus('error')
        setExtractPreviewError(detail)
      })
    }
  })

  useEffect(() => {
    void loadArchivePreview(extractSource)
  }, [extractSource, extractPassword, extractPreviewLimit])

  useEffect(() => {
    startTransition(() => {
      setExtractSelectedEntries([])
      setExtractPreviewLimit(EXTRACT_PREVIEW_PAGE_SIZE)
    })
  }, [extractSource])

  useEffect(() => {
    if (!extractPreview) {
      return
    }

    const visibleEntries = new Set(extractPreview.visibleEntries.map((entry) => entry.path))
    startTransition(() => {
      setExtractSelectedEntries((currentEntries) =>
        currentEntries.filter((entry) => visibleEntries.has(entry)),
      )
    })
  }, [extractPreview])

  function buildCompressRequest(): CompressArchiveRequest {
    const request: CompressArchiveRequest = {
      sourcePaths: normalizedCompressSources,
      destinationPath: compressDestination.trim(),
      format: compressFormat,
      conflictPolicy: compressConflictPolicy,
    }

    if (compressPassword.trim()) {
      request.password = compressPassword.trim()
    }

    return request
  }

  function buildExtractRequest(includeSelection = true): ExtractArchiveRequest {
    const request: ExtractArchiveRequest = {
      archivePath: extractSource.trim(),
      destinationDirectory: extractDestination.trim(),
      conflictPolicy: extractConflictPolicy,
    }

    if (extractPassword.trim()) {
      request.password = extractPassword.trim()
    }

    if (includeSelection && extractSelectedEntries.length > 0) {
      request.selectedEntries = extractSelectedEntries
    }

    return request
  }

  async function runExtractRequest(
    request: ExtractArchiveRequest,
    options?: { skipFeedback?: boolean },
  ) {
    if (!desktopShell) {
      const detail = 'Archive operations run inside the Tauri desktop shell.'
      if (!options?.skipFeedback) {
        setExtractFeedback({
          status: 'error',
          message: detail,
        })
      }
      throw new Error(detail)
    }

    if (!options?.skipFeedback) {
      setExtractFeedback({
        status: 'running',
        message: 'Extracting archive...',
      })
    }

    try {
      const result = await invoke<ArchiveActionResult>('extract_archive', {
        request,
      })

      if (!options?.skipFeedback) {
        startTransition(() => {
          setExtractFeedback({
            status: 'success',
            message: result.message,
            outputPath: result.outputPath,
          })
        })
      }
      onRememberExtractDestination(request.destinationDirectory)
      void refreshHistory()
      return result
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      if (!options?.skipFeedback) {
        startTransition(() => {
          setExtractFeedback({
            status: 'error',
            message: detail,
            recoveryHint: recoveryHintForArchiveError(detail),
          })
        })
      }
      throw error instanceof Error ? error : new Error(detail)
    }
  }

  async function runCompressRequest(
    request: CompressArchiveRequest,
    options?: { skipFeedback?: boolean },
  ) {
    if (!desktopShell) {
      const detail = 'Archive operations run inside the Tauri desktop shell.'
      if (!options?.skipFeedback) {
        setCompressFeedback({
          status: 'error',
          message: detail,
        })
      }
      throw new Error(detail)
    }

    if (!options?.skipFeedback) {
      setCompressFeedback({
        status: 'running',
        message: 'Creating archive...',
      })
    }

    try {
      const result = await invoke<ArchiveActionResult>('compress_archive', {
        request,
      })

      if (!options?.skipFeedback) {
        startTransition(() => {
          setCompressFeedback({
            status: 'success',
            message: result.message,
            outputPath: result.outputPath,
          })
        })
      }
      void refreshHistory()
      return result
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error)
      if (!options?.skipFeedback) {
        startTransition(() => {
          setCompressFeedback({
            status: 'error',
            message: detail,
            recoveryHint: recoveryHintForArchiveError(detail),
          })
        })
      }
      throw error instanceof Error ? error : new Error(detail)
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
      intent.destinationPath?.trim() || suggestPreferredExtractDestination(archivePath, intent.autoRun)

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
      await runExtractRequest({
        archivePath,
        destinationDirectory,
        conflictPolicy: extractConflictPolicy,
        ...(extractPassword.trim() ? { password: extractPassword.trim() } : {}),
      })
    }
  }

  function handleDroppedPaths(nextPaths: string[]) {
    const normalizedPaths = Array.from(
      new Set(nextPaths.map((path) => path.trim()).filter(Boolean)),
    )

    if (normalizedPaths.length === 0) {
      return
    }

    if (normalizedPaths.length === 1 && isArchivePath(normalizedPaths[0])) {
      const archivePath = normalizedPaths[0]
      const destinationDirectory = suggestPreferredExtractDestination(archivePath, false)

      startTransition(() => {
        setExtractSource(archivePath)
        setExtractDestination(destinationDirectory)
        setExtractFeedback({
          status: 'success',
          message: 'Archive loaded from drag and drop. Review the destination and extract when ready.',
          outputPath: destinationDirectory,
        })
      })
      return
    }

    appendCompressSources(normalizedPaths)
    startTransition(() => {
      setCompressFeedback({
        status: 'success',
        message:
          normalizedPaths.length === 1
            ? 'Item loaded from drag and drop. Choose a format and create the archive.'
            : `${normalizedPaths.length} items loaded from drag and drop. Choose a format and create the archive.`,
      })
    })
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
      setExtractDestination(suggestPreferredExtractDestination(selection, false))
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
      onRememberExtractDestination(selection)
    }
  }

  async function runCompress(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    await runCompressRequest(buildCompressRequest())
  }

  async function runExtract(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    await runExtractRequest(buildExtractRequest())
  }

  async function runExtractAll(event?: FormEvent<HTMLFormElement>) {
    event?.preventDefault()
    await runExtractRequest(buildExtractRequest(false))
  }

  async function runExtractSelected() {
    await runExtractRequest(buildExtractRequest(true))
  }

  function toggleExtractEntry(path: string) {
    startTransition(() => {
      setExtractSelectedEntries((currentEntries) =>
        currentEntries.includes(path)
          ? currentEntries.filter((entry) => entry !== path)
          : [...currentEntries, path],
      )
    })
  }

  function selectAllVisibleExtractEntries(paths?: string[]) {
    const nextEntries = paths ?? extractPreview?.visibleEntries.map((entry) => entry.path)
    if (!nextEntries) {
      return
    }

    startTransition(() => {
      setExtractSelectedEntries(nextEntries)
    })
  }

  function clearExtractSelection() {
    startTransition(() => {
      setExtractSelectedEntries([])
    })
  }

  function loadMoreExtractPreview() {
    startTransition(() => {
      setExtractPreviewLimit((currentLimit) => currentLimit + EXTRACT_PREVIEW_PAGE_SIZE)
    })
  }

  function queueExtract(includeSelection = true) {
    return buildExtractRequest(includeSelection)
  }

  return {
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
    buildExtractRequest,
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
  }
}
