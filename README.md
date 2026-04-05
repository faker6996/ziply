# Ziply

Ziply is a new cross-platform desktop app for compressing and extracting files and folders on macOS, Windows, and Linux.

## Current State

The old recorder codebase has been removed. This repository now contains a clean Tauri + React foundation for the next phase of Ziply.

## Product Direction

- Compress files and folders into archive formats.
- Extract archives into a destination folder.
- Support a broad set of file types through archive workflows.
- Ship one desktop app across macOS, Windows, and Linux.

## Local Development

```bash
npm install
npm run dev
```

## Structure

- `apps/desktop`: React frontend
- `src-tauri`: Tauri desktop shell and Rust backend
