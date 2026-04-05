# Ziply

> A lightweight, fast, and intuitive cross-platform desktop application for compressing and extracting archives. Ship one app for macOS, Windows, and Linux.

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)

## ✨ Features

- **🗜️ Multi-format Support**: Native support for ZIP, TAR, TAR.GZ, TAR.XZ, GZ, and 7Z
- **📦 RAR Extraction**: Extract RAR files when compatible external tools are available
- **⚡ Fast & Efficient**: Native Rust backend ensures high performance
- **🖥️ Cross-platform**: Single codebase for macOS, Windows, and Linux
- **📋 History & Persistence**: Track recent operations locally
- **🔄 Live Status**: Real-time progress updates during archive operations
- **🐚 Shell Integration**: Native file association and context menu support
- **💾 Conflict Handling**: Smart overwrite rules and conflict prompts

## 📋 Supported Archive Formats

| Format | Compress | Extract | Notes                                   |
| ------ | -------- | ------- | --------------------------------------- |
| ZIP    | ✅       | ✅      | With deflate and AES encryption support |
| TAR    | ✅       | ✅      | Pure TAR format                         |
| TAR.GZ | ✅       | ✅      | Gzip compressed                         |
| TAR.XZ | ✅       | ✅      | XZ compressed                           |
| GZ     | ✅       | ✅      | Basic Gzip                              |
| 7Z     | ✅       | ✅      | Via sevenz-rust                         |
| RAR    | ❌       | ✅      | Requires external tool                  |

## 🚀 Quick Start

### Installation (From Source)

```bash
# Clone the repository
git clone https://github.com/yourusername/ziply.git
cd ziply

# Install dependencies
npm install

# Run development build
npm run dev
```

### Build for Production

```bash
npm run build
```

## 🛠️ Development

### Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust + Tauri 2
- **Archive Libraries**: zip, tar, flate2, xz2, sevenz-rust
- **Dialogs**: Tauri native file/folder dialogs

### Project Structure

```
ziply/
├── apps/
│   └── desktop/           # React Vite frontend
│       ├── src/
│       │   ├── app/       # Main application logic
│       │   ├── components/
│       │   ├── hooks/
│       │   └── styles/
│       └── package.json
├── src-tauri/             # Tauri Rust backend
│   ├── src/
│   │   ├── commands/      # Archive operations (compress/extract)
│   │   ├── models.rs      # Data structures
│   │   ├── shell.rs       # Shell integration
│   │   ├── archive.rs     # Archive engine
│   │   ├── history.rs     # Operation history
│   │   └── main.rs
│   └── Cargo.toml
└── package.json           # Workspace manifest
```

### Available Commands

```bash
# Development
npm run dev              # Start development server with Tauri
npm run dev:web         # Start web server only (port 1420)

# Production
npm run build           # Build for all platforms
npm run build:web       # Build web assets only

# Code Quality
npm run lint            # Run ESLint on frontend
```

## 📊 Development Status

### ✅ Completed

- Clean Tauri + React foundation
- Native archive format support (ZIP, TAR, TAR.GZ, TAR.XZ, GZ, 7Z)
- Basic RAR extraction with external tool bridge
- Archive form UI for compress and extract workflows
- Recent operations history (persisted locally)
- Live job status tracking with backend events
- Shell integration and file associations
- Installer configurations for all three platforms

### 🔄 In Progress

- Progress reporting for long-running jobs
- Cross-platform shell integration polish
- Finder-specific macOS integration improvements

### 🗓️ Planned

- Password-protected archive support
- Enhanced error classification and recovery guidance
- Broader format coverage across platforms
- Drag-and-drop interface improvements
- Extended conflict handling scenarios

## 📦 Dependencies

### Frontend

- `@tauri-apps/api` - Tauri API client
- `@tauri-apps/plugin-dialog` - Native dialogs
- `react` & `react-dom` - UI framework
- `typescript` - Type safety

### Backend

- `tauri` - Desktop framework
- `zip` - ZIP format handling
- `tar` - TAR format handling
- `flate2` - Gzip compression
- `xz2` - XZ compression
- `sevenz-rust2` - 7Z format handling
- `walkdir` - Directory traversal
- `serde` & `serde_json` - Serialization

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - see LICENSE file for details

## 🎯 Next Steps

For detailed implementation plans and milestones, see [PLAN.md](PLAN.md).

---

Made with ❤️ for seamless file compression and extraction.
