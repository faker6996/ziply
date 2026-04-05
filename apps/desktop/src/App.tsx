import { useEffect, useState } from 'react'
import { AppSidebar, type AppSection } from './components/AppSidebar'
import { HistoryScreen } from './components/HistoryScreen'
import { SettingsScreen } from './components/SettingsScreen'
import { WorkspaceScreen } from './components/WorkspaceScreen'
import { useAppPreferences } from './hooks/useAppPreferences'
import { useZiplyRuntime } from './hooks/useZiplyRuntime'

function App() {
  const [activeSection, setActiveSection] = useState<AppSection>('workspace')
  const { preferences, setPreferences } = useAppPreferences()
  const rememberExtractDestination = (path: string) => {
    setPreferences((currentPreferences) => ({
      ...currentPreferences,
      lastExtractDestination: path,
    }))
  }

  const {
    history,
    liveJobs,
    queueItems,
    shellIntegration,
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
  } = useZiplyRuntime({
    preferences,
    onRememberExtractDestination: rememberExtractDestination,
  })

  useEffect(() => {
    setCompressFormat(preferences.defaultCompressFormat)
  }, [preferences.defaultCompressFormat, setCompressFormat])

  useEffect(() => {
    setExtractConflictPolicy(preferences.extractConflictPolicy)
  }, [preferences.extractConflictPolicy, setExtractConflictPolicy])

  return (
    <main className="app-shell">
      <AppSidebar activeSection={activeSection} onSelectSection={setActiveSection} />

      <section className="app-content">
        {activeSection === 'workspace' ? (
          <WorkspaceScreen
            compressConflictPolicy={compressConflictPolicy}
            compressDestination={compressDestination}
            compressFeedback={compressFeedback}
            compressFormat={compressFormat}
            compressPassword={compressPassword}
            compressSources={compressSources}
            desktopShell={desktopShell}
            dragDropState={dragDropState}
            extractConflictPolicy={extractConflictPolicy}
            extractDestination={extractDestination}
            extractFeedback={extractFeedback}
            extractPassword={extractPassword}
            extractPreview={extractPreview}
            extractPreviewError={extractPreviewError}
            extractPreviewLimit={extractPreviewLimit}
            extractPreviewStatus={extractPreviewStatus}
            extractSelectedEntries={extractSelectedEntries}
            extractSource={extractSource}
            gzipSourceCount={gzipSourceCount}
            liveJobs={liveJobs}
            normalizedCompressSources={normalizedCompressSources}
            onClearFinishedQueue={() => {
              void clearFinishedQueue()
            }}
            onClearSelection={clearExtractSelection}
            onCompressConflictPolicyChange={setCompressConflictPolicy}
            onCompressDestinationChange={setCompressDestination}
            onCompressFormatChange={setCompressFormat}
            onCompressPasswordChange={setCompressPassword}
            onCompressSourcesChange={setCompressSources}
            onExtractConflictPolicyChange={setExtractConflictPolicy}
            onExtractDestinationChange={(value) => {
              setExtractDestination(value)
              if (value.trim()) {
                rememberExtractDestination(value)
              }
            }}
            onExtractPasswordChange={setExtractPassword}
            onExtractSourceChange={setExtractSource}
            onLoadMoreEntries={loadMoreExtractPreview}
            onPickCompressDestination={() => {
              void pickCompressDestination()
            }}
            onPickCompressFiles={() => {
              void pickCompressFiles()
            }}
            onPickCompressFolders={() => {
              void pickCompressFolders()
            }}
            onPickExtractDestination={() => {
              void pickExtractDestination()
            }}
            onPickExtractSource={() => {
              void pickExtractSource()
            }}
            onQueueAllExtract={queueAllExtract}
            onQueueCompress={queueCurrentCompress}
            onQueueExtract={queueCurrentExtract}
            onRemoveQueuedJob={removeQueuedJob}
            onRetryQueueJob={retryQueueJob}
            onRunCompress={runCompress}
            onRunExtract={runExtract}
            onRunExtractAll={runExtractAll}
            onRunExtractSelected={runExtractSelected}
            onSelectAllVisibleEntries={selectAllVisibleExtractEntries}
            onToggleEntry={toggleExtractEntry}
            queueItems={queueItems}
            supportsPasswordOnCompress={supportsArchivePasswordOnCompress}
            supportsPasswordOnExtract={supportsArchivePasswordOnExtract}
            supportsSelectiveExtract={supportsSelectiveExtract}
          />
        ) : null}

        {activeSection === 'history' ? (
          <HistoryScreen
            desktopShell={desktopShell}
            history={history}
            liveJobs={liveJobs}
            onClear={() => {
              void clearHistory()
            }}
          />
        ) : null}

        {activeSection === 'settings' ? (
          <SettingsScreen
            desktopShell={desktopShell}
            onInstallShellIntegration={() => {
              void installShellIntegration()
            }}
            onPreferencesChange={setPreferences}
            onRefreshShellIntegration={() => {
              void refreshShellIntegration()
            }}
            preferences={preferences}
            shellIntegration={shellIntegration}
            shellIntegrationFeedback={shellIntegrationFeedback}
          />
        ) : null}
      </section>
    </main>
  )
}

export default App
