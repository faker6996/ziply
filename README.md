# Ziply

Ziply is a cross-platform desktop archive utility for macOS, Windows, and Linux. It focuses on fast compress and extract workflows, archive preview, selective extract, live job tracking, and release packaging from one Tauri + React codebase.

![Version](https://img.shields.io/github/v/tag/faker6996/ziply?label=version)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![CI](https://img.shields.io/github/actions/workflow/status/faker6996/ziply/ci.yml?branch=main&label=ci)
![Installers](https://img.shields.io/github/actions/workflow/status/faker6996/ziply/build-installers.yml?branch=main&label=installers)

## Native-Only Rule

Ziply only claims formats that it handles itself.

- no external archive helper apps
- no runtime shell-out to archive tools
- no hidden dependency on another archive utility

If a format is not implemented natively inside this repository, it stays out of active support.

## What Ziply Does Today

- Compress and extract archives from one desktop workspace
- Support `zip`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, `xz`, `bz2`, `gz`, `7z`, and native `rar` extraction
- Preview archive contents before extraction
- Search preview results and progressively load more entries
- Extract everything or only selected entries for supported formats
- Queue multiple jobs and retry failed jobs
- Track live job state and persist recent operations locally
- Handle destination conflicts with `keep both`, `overwrite`, or `stop`
- Accept drag and drop for files, folders, and archives
- Support password flows for encrypted `7z` and password-protected `zip` creation, plus password-protected `zip` / `7z` / `rar5` extraction

## Product Direction

Ziply is not treating `all formats` as “every archive format ever invented”.

The rule is:

1. Every format listed as supported in the app must be native.
2. Every format listed as supported in this README must be backed by native code in Ziply.
3. Formats that are still under investigation stay in `planned`, not `supported`.

## Supported Formats

| Format | Compress | Extract | Preview | Selective Extract | Notes |
| ------ | -------- | ------- | ------- | ----------------- | ----- |
| ZIP | ✅ | ✅ | ✅ | ✅ | Deflate compression. Password-protected ZIP creation is supported. AES extraction is supported when reading ZIP archives |
| TAR | ✅ | ✅ | ✅ | ✅ | Pure TAR |
| TAR.GZ | ✅ | ✅ | ✅ | ✅ | Gzip-compressed tar |
| TAR.BZ2 | ✅ | ✅ | ✅ | ✅ | Bzip2-compressed tar |
| TAR.XZ | ✅ | ✅ | ✅ | ✅ | XZ-compressed tar |
| XZ | ✅ | ✅ | ✅ | ❌ | Compression supports exactly one file |
| BZ2 | ✅ | ✅ | ✅ | ❌ | Compression supports exactly one file |
| GZ | ✅ | ✅ | ✅ | ❌ | Compression supports exactly one file |
| 7Z | ✅ | ✅ | ✅ | ✅ | Powered by `sevenz-rust2`. Supports encrypted archive creation and extraction |
| RAR | ❌ | ✅ | ✅ | ✅ | Native extract, preview, and selective extract support. Password-protected and multipart RAR5 archives are covered, and Ziply auto-resolves later volume entries back to the first archive part. `RAR` creation is not shipped because the current native stack is read-side only. Older RAR4 variants still need work |

## Planned Native Formats

No extra format claims are queued right now. Existing roadmap work is focused on hardening the native set already shipped.

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

### Native quality coverage

The current native archive test suite covers:

- round-trip archive creation and extraction for the supported native formats
- fixture-based compatibility checks for `zip`, `tar`, `tar.gz`, `tar.bz2`, `tar.xz`, raw stream formats, and `7z` archives produced outside Ziply
- self-contained RAR5 preview and extraction fixture coverage for the native `rar` path
- tracked RAR5 password-protected and multipart fixture coverage for the native `rar` path
- clean failure coverage for older RAR4 fixtures that the current native stack does not extract reliably yet
- unicode filenames
- empty files and empty directories
- large binary payloads for raw stream formats
- preview limits and hidden-entry counts
- selective extract behavior
- password success and wrong-password failure paths for `7z`
- unsafe ZIP path rejection during extraction

Compatibility fixtures are tracked in the repository and can be regenerated locally by script:

- `bash scripts/generate-compat-fixtures.sh`

### Password support

- Create encrypted `7z` archives
- Create password-protected `zip` archives
- Extract password-protected `7z` archives
- Extract password-protected `zip` archives, including AES-backed ZIPs
- Extract password-protected `rar5` archives on the native read-side path

### Conflict handling

- `Keep both`
- `Overwrite`
- `Stop on conflict`

For extraction, conflict handling currently applies at the destination level, not as a per-entry rename strategy inside the archive.

## Platform Notes

### Shell integration

- Windows: Explorer context commands are implemented for extract, extract-here, and compress flows
- Linux: desktop action integration is implemented for compatible launchers and file managers
- macOS: Finder Quick Actions are available for `Extract with Ziply` and `Extract here with Ziply`; Homebrew installs them immediately, while manual app installs repair or create them on first launch. `Open With Ziply` remains available through the bundle

### Packaging

- macOS: DMG
- Windows: NSIS installer
- Linux: DEB package

## Known Limits

- ZIP creation supports password-protected output. Use `7z` when you want the stronger encryption option already shipped in Ziply
- `rar` currently ships as native extract, preview, and selective extract support
- password-protected and multipart `rar5` archives are covered; broader `rar` variant coverage still needs work
- selecting `.part2.rar` or `.r00` routes back to the first archive volume automatically when the matching first volume exists
- older `rar4` archives are not broadly supported yet
- `rar` archive creation is not shipped because the current native Rust stack in Ziply is read-side only
- `gz`, `xz`, and `bz2` preview are single-stream oriented and selective extract is not applicable
- Batch jobs currently run one at a time
- Finder Quick Actions on macOS cover extract and extract-here flows; Homebrew installs them directly into `~/Library/Services`, while manual installs still rely on Ziply to repair them on first launch if needed

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

# Regenerate local compatibility fixtures
bash scripts/generate-compat-fixtures.sh
```

## Package Installation

### Homebrew

Current install flow:

```bash
brew tap faker6996/tap
brew install --cask faker6996/tap/ziply
```

### APT

Current repository feed:

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
- Archive libraries: `zip`, `tar`, `flate2`, `bzip2`, `xz2`, `sevenz-rust2`, `rar`

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
│   ├── src/commands/         # Tauri command layer
│   ├── tests/fixtures/compat # tracked compatibility archives generated from external tools
│   └── fixtures/rar          # tracked native RAR fixtures for extract, preview, multipart, and password coverage
├── packaging/homebrew/       # Homebrew cask templates and tap skeleton
├── scripts/                  # release, version, and fixture-generation helper scripts
└── .github/workflows/        # CI and release automation
```

## GitHub Actions

Ziply ships with two workflows:

- `ci.yml`
  - Checks version consistency across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`
  - Runs `npm run lint`
  - Runs `npm run build:web`
  - Runs `cargo fmt --all --check`
  - Runs `cargo test --manifest-path src-tauri/Cargo.toml`
  - Runs `cargo check --manifest-path src-tauri/Cargo.toml`
  - Runs `npm run build -- --ci --no-bundle` on macOS, Windows, and Linux as a cross-platform smoke build
- `build-installers.yml`
  - Checks version consistency before building installers
  - Builds DMG, NSIS, and DEB installers
  - Signs and notarizes the macOS DMG when Apple release secrets are configured, otherwise falls back to an unsigned DMG for internal testing
  - Verifies installer artifacts exist, are non-empty, and have the expected file type before upload
  - Uploads installer artifacts
  - Auto-tags the current app version from `main`
  - Publishes a GitHub Release
  - Verifies downloaded release assets again before publishing the release
  - Updates the Homebrew tap when Homebrew credentials are configured
  - Publishes the APT repository to GitHub Pages when APT signing secrets are configured

### macOS signing and notarization

To produce a macOS build that opens without Gatekeeper warnings, configure these GitHub Actions secrets:

- `APPLE_CERTIFICATE`: base64-encoded `Developer ID Application` `.p12`
- `APPLE_CERTIFICATE_PASSWORD`: password used to export the `.p12`
- `APPLE_API_KEY`: App Store Connect API key ID
- `APPLE_API_ISSUER`: App Store Connect issuer ID
- `APPLE_API_KEY_BASE64`: base64-encoded `AuthKey_<KEYID>.p8`
- `APPLE_SIGNING_IDENTITY`: optional explicit signing identity override

With those secrets present, `build-installers.yml` builds a signed universal app, notarizes it with Apple, staples both the `.app` and `.dmg`, and only then uploads the release artifact.

Without them, the workflow still builds a macOS DMG, but it is intentionally unsigned and will trigger Gatekeeper warnings after download or Homebrew install.

### Release Configuration

Repository secrets used by packaging:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `APPLE_API_KEY`
- `APPLE_API_ISSUER`
- `APPLE_API_KEY_BASE64`
- `APPLE_SIGNING_IDENTITY` (optional)
- `HOMEBREW_TAP_TOKEN`
- `APT_GPG_PRIVATE_KEY`
- `APT_GPG_PASSPHRASE`

Repository variables used by packaging:

- `HOMEBREW_TAP_REPOSITORY`
- `HOMEBREW_TAP_BRANCH`

GitHub Pages must also be enabled for the repository if you want the APT publish step to deploy.

If the Apple secrets are missing, the macOS release artifact is still built but remains unsigned.
If the Homebrew or APT secrets are missing, installer builds and the GitHub Release can still run, but feed publication steps are skipped.

## Validation Status

At the current state of the repository, the core local validation path is:

```bash
npm run lint
npm run build:web
cargo test --manifest-path src-tauri/Cargo.toml
```

Current local status:

- `npm run lint` passes
- `npm run build:web` passes
- `cargo test --manifest-path src-tauri/Cargo.toml` passes with `49` tests
- `bash scripts/check-version-consistency.sh` passes
- compatibility fixtures can be regenerated locally with `bash scripts/generate-compat-fixtures.sh`

For the full native-only roadmap and promotion rules for new formats, see [PLAN.md](/Users/tran_van_bach/Desktop/project/ziply/PLAN.md).

## License

MIT. See [LICENSE](/Users/tran_van_bach/Desktop/project/ziply/LICENSE).
