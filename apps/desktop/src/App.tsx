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
          compressSources={compressSources}
          desktopShell={desktopShell}
          feedback={compressFeedback}
          gzipSourceCount={gzipSourceCount}
          normalizedCompressSources={normalizedCompressSources}
          onCompressDestinationChange={setCompressDestination}
          onCompressFormatChange={setCompressFormat}
          onCompressConflictPolicyChange={setCompressConflictPolicy}
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
          onSubmit={runCompress}
        />

        <ExtractForm
          capabilities={capabilities}
          desktopShell={desktopShell}
          extractDestination={extractDestination}
          extractConflictPolicy={extractConflictPolicy}
          extractSource={extractSource}
          feedback={extractFeedback}
          onExtractConflictPolicyChange={setExtractConflictPolicy}
          onExtractDestinationChange={setExtractDestination}
          onExtractSourceChange={setExtractSource}
          onPickExtractDestination={() => {
            void pickExtractDestination()
          }}
          onPickExtractSource={() => {
            void pickExtractSource()
          }}
          onSubmit={runExtract}
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
