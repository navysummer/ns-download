# ns-download

A cross-platform download manager built with Tauri v2, Vue 3, and Rust. Successor to FluxDown, supporting multi-protocol downloads with a modern desktop UI.

![Build](https://github.com/navysummer/ns-download/actions/workflows/build.yml/badge.svg)

## Features

- **Multi-protocol**: HTTP/HTTPS, FTP, BitTorrent, ed2k, HLS (m3u8), DASH (mpd)
- **Concurrent downloads**: Configurable concurrency and per-task segment count
- **BT features**: DHT, PEX, magnet links, tracker subscriptions, encryption
- **ed2k**: Server-based source discovery, Kad network, LowID callback, UPnP port mapping
- **HLS/DASH**: Segment-wise parallel downloading, TS→MP4 remux (via ffmpeg), variant selection
- **Proxy support**: SOCKS4/5, HTTP CONNECT per-task or global
- **Speed limiting**: Global and per-task bandwidth control
- **Plugin system**: JavaScript (QuickJS) extensions for URL resolution
- **Database**: SQLite via rusqlite for task persistence and resume
- **Cross-platform**: macOS, Linux, Windows, Android, iOS

## Download

Grab the latest build from the [Releases](https://github.com/navysummer/ns-download/releases) page.

| Platform | Arch | Format |
|----------|------|--------|
| macOS | x86_64 / ARM64 | .dmg |
| Windows | x86_64 / ARM64 | .msi / .exe |
| Linux | x86_64 / ARM64 | .deb / .AppImage / .rpm |
| Android | ARM64 | .apk |
| iOS | ARM64 | .ipa |

## Architecture

```
ns-download/
├── src/                    # Vue 3 frontend (Tauri webview)
│   ├── views/              # TasksView, SettingsView
│   ├── components/         # Sidebar, StatusBar, TaskList, NewDownloadDialog
│   └── lib/                # Pinia store, Tauri invoke wrappers
├── src-tauri/              # Tauri 2 desktop shell (Rust)
│   ├── src/
│   │   ├── commands.rs     # Tauri IPC commands
│   │   ├── api_server.rs   # REST / JSON-RPC / MCP API server
│   │   └── lib.rs          # App setup, plugins, state
│   └── tauri.conf.json     # Tauri config (version, bundle, icons)
├── native/
│   ├── engine/             # Core download engine
│   │   ├── src/
│   │   │   ├── download_manager.rs  # Task orchestration
│   │   │   ├── downloader.rs        # HTTP client, redirect handling
│   │   │   ├── db.rs               # SQLite persistence
│   │   │   ├── bt_downloader.rs    # BitTorrent via librqbit
│   │   │   ├── ftp_downloader.rs   # FTP via suppaftp
│   │   │   ├── hls_downloader.rs   # HLS streaming
│   │   │   ├── dash_downloader.rs  # DASH streaming
│   │   │   ├── ed2k/              # ed2k protocol (client, Kad, server)
│   │   │   ├── segment_coordinator.rs  # Segment scheduling
│   │   │   ├── proxy_config.rs    # SOCKS/HTTP proxy
│   │   │   └── plugin/            # QuickJS plugin runtime
│   │   └── build.rs
│   ├── api/                # Public API server (REST, WebSocket, MCP)
│   ├── cli/                # CLI interface
│   ├── server/             # Standalone HTTP server
│   └── hub/                # Mobile/desktop hub
├── .github/workflows/
│   └── build.yml           # CI/CD: builds all platforms on tag push
└── Cargo.toml              # Rust workspace root
```

## Prerequisites

- **Rust** 1.85+ (edition 2024)
- **Node.js** 18+ and npm
- **System deps** (macOS): Xcode Command Line Tools
- **System deps** (Linux): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`

## Development

```bash
# Install JS dependencies
npm install

# Run in dev mode (hot-reload)
npm run tauri dev

# Build for production
npm run tauri build

# Build for specific target
npx tauri build --target aarch64-apple-darwin
```

## Building for mobile

```bash
# Android APK
npm run tauri android build -- --apk

# iOS IPA (macOS only)
npm run tauri ios build
```

## Version management

The app version is defined in **three places** that must be kept in sync:

| File | Field | Example |
|------|-------|--------|
| `src-tauri/tauri.conf.json` | `version` | `"0.1.0"` |
| `src-tauri/Cargo.toml` | `[package] version` | `"0.1.0"` |
| `package.json` | `version` | `"0.1.0"` |

To change the version:

```bash
# Update all three files, then tag and push:
git tag v0.2.0
git push origin v0.2.0
```

This triggers the GitHub Actions workflow to build all platforms and upload to [Releases](https://github.com/navysummer/ns-download/releases). The frontend reads the version at runtime via `@tauri-apps/api/app` `getVersion()`.

## Configuration

The engine reads configuration from the database (`tasks`, `config` tables) and environment:

| Variable | Description |
|---|---|
| `NSDOWNLOAD_APP_VERSION` | Version string override for User-Agent |
| `DATABASE_URL` | Postgres URL (default: SQLite in data dir) |
| `XDG_DATA_HOME` / `HOME` | Data directory resolution |

Proxy, BT, and download limits are configurable at runtime via the settings UI.

## CI/CD

On every `v*` tag push, GitHub Actions builds and publishes:

- **Desktop**: macOS (x86_64 + ARM64), Windows (x86_64 + ARM64), Linux (x86_64 + ARM64)
- **Mobile**: Android APK, iOS IPA

Artifacts are automatically uploaded to the GitHub Release.