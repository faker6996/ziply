import { BatchQueuePanel } from './components/BatchQueuePanel'
import { CompressForm } from './components/CompressForm'
import { DropZonePanel } from './components/DropZonePanel'
import { ExtractForm } from './components/ExtractForm'
import { LiveJobsPanel } from './components/LiveJobsPanel'
import { OverviewPanels } from './components/OverviewPanels'
import { RecentOperationsPanel } from './components/RecentOperationsPanel'
import { ShellIntegrationPanel } from './components/ShellIntegrationPanel'
import { useZiplyRuntime } from './hooks/useZiplyRuntime'

function App() {
  const {
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
  } = useZiplyRuntime()

  return (
    <main className="app-shell">
      <section className="hero-card">
        <div className="hero-copy">
          <p className="eyebrow">Archive workspace</p>
          <h1>{overview.name}</h1>
          <p className="tagline">{overview.tagline}</p>
        </div>

        <div className="status-panel">
          <span className={`status-dot status-dot--${runtimeStatus}`} />
          <strong>
            {runtimeStatus === 'loading'
              ? 'Loading desktop runtime'
              : runtimeStatus === 'error'
                ? 'Using preview data'
                : 'Desktop runtime connected'}
          </strong>
          <p>
            Real archive commands are live for zip, tar, tar.gz, tgz, tar.xz, txz, gz, and 7z.
          </p>
        </div>
      </section>

      <section className="detail-grid">
        <DropZonePanel desktopShell={desktopShell} dragDropState={dragDropState} />
      </section>

      <section className="grid grid--tools">
        <CompressForm
          compressDestination={compressDestination}
          compressFormat={compressFormat}
          compressConflictPolicy={compressConflictPolicy}
          compressPassword={compressPassword}
          compressSources={compressSources}
          desktopShell={desktopShell}
          feedback={compressFeedback}
          gzipSourceCount={gzipSourceCount}
          normalizedCompressSources={normalizedCompressSources}
          onCompressDestinationChange={setCompressDestination}
          onCompressFormatChange={setCompressFormat}
          onCompressConflictPolicyChange={setCompressConflictPolicy}
          onCompressPasswordChange={setCompressPassword}
          onCompressSourcesChange={setCompressSources}
          onPickCompressDestination={() => {
            void pickCompressDestination()
          }}
          onPickCompressFiles={() => {
            void pickCompressFiles()
          }}
          onPickCompressFolders={() => {
            void pickCompressFolders()
          }}
          supportsPasswordOnCompress={supportsArchivePasswordOnCompress}
          onQueue={queueCurrentCompress}
          onSubmit={runCompress}
        />

        <ExtractForm
          capabilities={capabilities}
          desktopShell={desktopShell}
          extractDestination={extractDestination}
          extractConflictPolicy={extractConflictPolicy}
          extractPassword={extractPassword}
          extractSource={extractSource}
          feedback={extractFeedback}
          preview={extractPreview}
          previewError={extractPreviewError}
          previewLimit={extractPreviewLimit}
          previewStatus={extractPreviewStatus}
          selectedEntries={extractSelectedEntries}
          onExtractConflictPolicyChange={setExtractConflictPolicy}
          onExtractPasswordChange={setExtractPassword}
          onExtractDestinationChange={setExtractDestination}
          onExtractSourceChange={setExtractSource}
          onPickExtractDestination={() => {
            void pickExtractDestination()
          }}
          onPickExtractSource={() => {
            void pickExtractSource()
          }}
          onToggleEntry={toggleExtractEntry}
          onSelectAllVisibleEntries={selectAllVisibleExtractEntries}
          onClearSelection={clearExtractSelection}
          onLoadMoreEntries={loadMoreExtractPreview}
          supportsPasswordOnExtract={supportsArchivePasswordOnExtract}
          supportsSelectiveExtract={supportsSelectiveExtract}
          onQueue={queueCurrentExtract}
          onQueueAll={queueAllExtract}
          onSubmit={runExtract}
          onSubmitAll={runExtractAll}
          onSubmitSelected={runExtractSelected}
        />
      </section>

      <section className="detail-grid">
        <OverviewPanels capabilities={capabilities} overview={overview} />

        <ShellIntegrationPanel
          desktopShell={desktopShell}
          feedback={shellIntegrationFeedback}
          onInstall={() => {
            void installShellIntegration()
          }}
          onRefresh={() => {
            void refreshShellIntegration()
          }}
          shellIntegration={shellIntegration}
        />

        <LiveJobsPanel liveJobs={liveJobs} />

        <BatchQueuePanel
          onClearFinished={() => {
            void clearFinishedQueue()
          }}
          onRemoveQueuedJob={removeQueuedJob}
          onRetryJob={retryQueueJob}
          queueItems={queueItems}
        />

        <RecentOperationsPanel
          desktopShell={desktopShell}
          history={history}
          onClear={() => {
            void clearHistory()
          }}
          onRefresh={() => {
            void refreshHistory()
          }}
        />
      </section>
    </main>
  )
}

export default App
