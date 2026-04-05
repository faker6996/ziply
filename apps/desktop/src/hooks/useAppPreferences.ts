import { useEffect, useState } from 'react'
import type { CompressFormat, ConflictPolicy } from '../app/types'
import type { AppPreferences } from '../components/SettingsScreen'

const storageKey = 'ziply-ui-preferences'

const fallbackPreferences: AppPreferences = {
  extractDestinationMode: 'askEveryTime',
  extractConflictPolicy: 'keepBoth',
  defaultCompressFormat: 'zip',
  compressionLevel: 'fastest',
  deleteArchiveAfterExtraction: false,
  lastExtractDestination: '',
}

interface StoredPreferences {
  extractDestinationMode?: AppPreferences['extractDestinationMode']
  extractConflictPolicy?: ConflictPolicy
  defaultCompressFormat?: CompressFormat
  compressionLevel?: AppPreferences['compressionLevel']
  deleteArchiveAfterExtraction?: boolean
  lastExtractDestination?: string
}

function readStoredPreferences(): AppPreferences {
  if (typeof window === 'undefined') {
    return fallbackPreferences
  }

  try {
    const rawValue = window.localStorage.getItem(storageKey)
    if (!rawValue) {
      return fallbackPreferences
    }

    const parsedValue = JSON.parse(rawValue) as StoredPreferences

    return {
      extractDestinationMode:
        parsedValue.extractDestinationMode ?? fallbackPreferences.extractDestinationMode,
      extractConflictPolicy:
        parsedValue.extractConflictPolicy ?? fallbackPreferences.extractConflictPolicy,
      defaultCompressFormat:
        parsedValue.defaultCompressFormat ?? fallbackPreferences.defaultCompressFormat,
      compressionLevel: parsedValue.compressionLevel ?? fallbackPreferences.compressionLevel,
      deleteArchiveAfterExtraction:
        parsedValue.deleteArchiveAfterExtraction ?? fallbackPreferences.deleteArchiveAfterExtraction,
      lastExtractDestination:
        parsedValue.lastExtractDestination ?? fallbackPreferences.lastExtractDestination,
    }
  } catch {
    return fallbackPreferences
  }
}

export function useAppPreferences() {
  const [preferences, setPreferences] = useState<AppPreferences>(readStoredPreferences)

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    window.localStorage.setItem(storageKey, JSON.stringify(preferences))
  }, [preferences])

  return {
    preferences,
    setPreferences,
  }
}
