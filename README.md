# Ziply

Ziply is a cross-platform desktop archive utility for macOS, Windows, and Linux. It focuses on fast compress and extract workflows, archive preview, selective extract, live job tracking, and release packaging from one Tauri + React codebase.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![CI](https://img.shields.io/github/actions/workflow/status/faker6996/ziply/ci.yml?branch=main&label=ci)
![Installers](https://img.shields.io/github/actions/workflow/status/faker6996/ziply/build-installers.yml?branch=main&label=installers)

## What Ziply Does Today

- Compress and extract archives from one desktop workspace
- Support `zip`, `tar`, `tar.gz`, `tar.xz`, `gz`, and `7z` natively
- Extract `rar` archives through compatible external tools when they exist on the machine
- Preview archive contents before extraction
- Search preview results and progressively load more entries
- Extract everything or only selected entries for supported formats
- Queue multiple jobs and retry failed jobs
- Track live job state and persist recent operations locally
- Handle destination conflicts with `keep both`, `overwrite`, or `stop`
- Accept drag and drop for files, folders, and archives
- Support password flows for encrypted `7z` creation and password-protected `zip` / `7z` extraction

## Supported Formats

| Format | Compress | Extract | Preview | Selective Extract | Notes |
| ------ | -------- | ------- | ------- | ----------------- | ----- |
| ZIP | ✅ | ✅ | ✅ | ✅ | Deflate compression. Password and AES support are extract-only right now |
| TAR | ✅ | ✅ | ✅ | ✅ | Pure TAR |
| TAR.GZ | ✅ | ✅ | ✅ | ✅ | Gzip-compressed tar |
| TAR.XZ | ✅ | ✅ | ✅ | ✅ | XZ-compressed tar |
| GZ | ✅ | ✅ | ✅ | ❌ | Compression supports exactly one file |
| 7Z | ✅ | ✅ | ✅ | ✅ | Powered by `sevenz-rust2`. Supports encrypted archive creation and extraction |
| RAR | ❌ | ✅ | ❌ | ❌ | Requires `unar`, `7z`, `7zz`, or `unrar` on the host machine |

## Current Product Scope

### End-user features

- `Compress` and `Extract` forms with native Tauri file and folder pickers
- Archive preview with search and `Load more`
- `Extract all`, `Extract selected`, `Queue all`, and `Queue selected`
- Batch queue with sequential execution and retry for failed jobs
- Recent operations history stored locally
- Live job state panel driven by backend events
- Drag-and-drop workspace that routes archives to extract and regular files or folders to compress
- Shell-intent handling for open, extract, extract-here, and compress launch flows

### Password support

- Create encrypted `7z` archives
- Extract password-protected `7z` archives
- Extract password-protected `zip` archives, including AES-backed ZIPs

### Conflict handling

- `Keep both`
- `Overwrite`
- `Stop on conflict`

For extraction, conflict handling currently applies at the destination level, not as a per-entry rename strategy inside the archive.

## Platform Notes

### Shell integration

- Windows: Explorer context commands are implemented for extract, extract-here, and compress flows
- Linux: desktop action integration is implemented for compatible launchers and file managers
- macOS: file association and `Open With Ziply` are supported through the bundle; custom Finder right-click actions are not fully shipped yet

### Packaging

- macOS: DMG
- Windows: NSIS installer
- Linux: DEB package

## Known Limits

- Creating encrypted ZIP archives is not implemented yet
- `rar` support depends on external tools and does not currently support preview or selective extract
- `gz` preview is single-stream oriented and selective extract is not applicable
- Batch jobs currently run one at a time
- Finder-specific custom context-menu actions on macOS still need a dedicated extension or Quick Action path

## Quick Start

### Build From Source

Requirements:

- Node.js 22 or newer
- npm
- Rust stable
- Platform build dependencies required by Tauri

Clone and run:

```bash
git clone https://github.com/faker6996/ziply.git
cd ziply
npm install
npm run dev
```

### Useful Commands

```bash
# Desktop dev
npm run dev

# Frontend-only dev server
npm run dev:web

# Web asset build
npm run build:web

# Desktop build
npm run build

# Frontend lint
npm run lint

# Rust tests
cargo test --manifest-path src-tauri/Cargo.toml
```

## Package Installation

### Homebrew

Once the Homebrew tap release flow is configured and published:

```bash
brew tap faker6996/tap
brew install --cask faker6996/tap/ziply
```

### APT

Once the APT repository and signing flow are configured and published:

```bash
curl -fsSL https://faker6996.github.io/ziply/apt/ziply-archive-keyring.asc | sudo gpg --dearmor -o /usr/share/keyrings/ziply-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/ziply-archive-keyring.gpg] https://faker6996.github.io/ziply/apt stable main" | sudo tee /etc/apt/sources.list.d/ziply.list
sudo apt update
sudo apt install ziply
```

If release infrastructure is not configured yet, install from source instead.

## Development Stack

- Frontend: React 19, TypeScript, Vite
- Desktop shell: Tauri 2
- Backend language: Rust
- Native dialogs: `@tauri-apps/plugin-dialog`
- Archive libraries: `zip`, `tar`, `flate2`, `xz2`, `sevenz-rust2`

## Repository Layout

```text
ziply/
├── apps/
│   └── desktop/
│       ├── src/app/          # shared app types, defaults, utilities
│       ├── src/components/   # UI panels and forms
│       ├── src/hooks/        # runtime, queue, history, shell, drag-drop hooks
│       └── src/styles/       # split CSS layers
├── src-tauri/
│   ├── src/archive.rs        # archive engine
│   ├── src/history.rs        # persisted operation history
│   ├── src/models.rs         # request and response models
│   ├── src/shell.rs          # OS shell integration helpers
│   └── src/commands/         # Tauri command layer
├── packaging/homebrew/       # Homebrew cask templates and tap skeleton
├── scripts/                  # release and packaging helper scripts
└── .github/workflows/        # CI and release automation
```

## GitHub Actions

Ziply ships with two workflows:

- `ci.yml`
  - Runs `npm run lint`
  - Runs `npm run build:web`
  - Runs `cargo fmt --all --check`
  - Runs `cargo test --manifest-path src-tauri/Cargo.toml`
  - Runs `cargo check --manifest-path src-tauri/Cargo.toml`
- `build-installers.yml`
  - Builds DMG, NSIS, and DEB installers
  - Uploads installer artifacts
  - Auto-tags the current app version from `main`
  - Publishes a GitHub Release
  - Updates the Homebrew tap when Homebrew credentials are configured
  - Publishes the APT repository to GitHub Pages when APT signing secrets are configured

### Release Configuration

Repository secrets used by packaging:

- `HOMEBREW_TAP_TOKEN`
- `APT_GPG_PRIVATE_KEY`
- `APT_GPG_PASSPHRASE`

Repository variables used by packaging:

- `HOMEBREW_TAP_REPOSITORY`
- `HOMEBREW_TAP_BRANCH`

GitHub Pages must also be enabled for the repository if you want the APT publish step to deploy.

If those values are missing, installer builds and the GitHub Release can still run, but feed publication steps are skipped.

## Validation Status

At the current state of the repository, the core local validation path is:

```bash
npm run lint
npm run build:web
cargo test --manifest-path src-tauri/Cargo.toml
```

## License

MIT. See [LICENSE](/Users/tran_van_bach/Desktop/project/ziply/LICENSE).
