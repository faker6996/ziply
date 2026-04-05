# Ziply Plan

## Goal

Ship Ziply as a cross-platform desktop archive utility for macOS, Windows, and Linux with a clean workflow for compressing and extracting files and folders.

## Current Status

- Done: clean Tauri + React codebase reset
- Done: native support for `zip`, `tar`, `tar.gz`, `tar.xz`, `gz`, and `7z`
- Done: extract-only `.rar` bridge when a compatible external tool exists
- Done: dialog-based source and destination picking
- Done: beta UX, persistence, queueing, and preview-based extraction workflow

## Beta Milestones

### 1. Core archive engine

- Done: archive creation for supported native formats
- Done: archive extraction for supported native formats
- Done: backend tests for `zip`, `tar.gz`, `tar.xz`, `gz`, and `7z`

### 2. Beta workflow

- Done: archive form UI for compress and extract
- Done: runtime capability detection for optional `.rar` extraction
- Done: recent operations history persisted locally
- Done: live job status panel fed by backend archive events
- Done: shell-open flow for archives and shell-driven extract intents
- Done: installer-facing file associations plus Windows/Linux shell integration installers
- Done: overwrite rules and conflict handling
- Done: drag and drop entry flow
- Done: archive preview before extraction
- Done: selective extract from previewed entries for zip, tar, tar.gz, tar.xz, and 7z
- Done: batch queue for compress and extract jobs

### 3. Production hardening

- Done: phase-based progress reporting for live archive jobs
- Done: password-protected 7z creation and password-based zip/7z extraction
- Done: frontend recovery hints for common archive failures
- Done: retry flow for failed batch jobs
- Done: current cross-platform shell integration baseline with documented macOS `Open With Ziply` limitation
- Done: better error classification and recovery guidance in frontend feedback
- Done: broader format coverage review surfaced inside the app as a format support matrix

## Immediate Implementation Track

1. Keep archive operations stable and persisted.
2. Surface live backend job states while commands are running.
3. Keep polishing extraction ergonomics and platform integration as future product work, not as a blocker for v1.
